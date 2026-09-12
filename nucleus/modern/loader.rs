//! Native sealed Settings trial construction. No disk or signature policy.
//! Runs only on the sole certified cloud guest CPU with IF=0. Source tests do
//! not execute this unsafe mapping/copy/TLB boundary; VM evidence is required.
use super::*;
impl Runtime{
    pub(super) fn construct_trial(&mut self,payload:&[u8],trial:model::Trial)->Result<(),Error>{
        let index=trial.endpoint().slot as usize;
        if !matches!(index,5|7)||self.policy.as_ref().unwrap().trial()!=Some(trial){
            return Err(Error::Denied);
        }
        let layout=pe::parse(payload)?;
        let handoff=support::trial_bootstrap(self.policy.as_ref().unwrap(),trial,layout.entry)?;
        self.construct_private(payload,index,trial.endpoint().incarnation,4,handoff)?;
        self.processes[index].state=State::Runnable;Ok(())
    }
    /// Only trial acceptance and all-or-nothing desktop preparation call this.
    /// A fully built context stays Blocked until its caller publishes authority.
    fn construct_private(&mut self,payload:&[u8],index:usize,generation:u64,stack_pages:u64,
        handoff:abi::Boot)->Result<(),Error>{
        let root=self.stage_context();
        if self.current!=8||index>7||!self.stage_readonly||self.stage_view||
            !matches!(stack_pages,4|16)||generation==0||!abi::valid_boot(&handoff){
            return Err(Error::Denied);
        }
        let victim=self.processes[index];let owner=self.processes[self.current];
        if victim.memory!=retirement::Memory::Clean||victim.state!=State::Dead||victim.root!=0||
            owner.root!=root||owner.memory!=retirement::Memory::Live||
            !(1..=256).contains(&owner.table_used){return Err(Error::Busy);}
        let layout=pe::parse(payload)?;
        if layout.entry!=handoff.entry||generation!=handoff.generation{return Err(Error::Invalid);}
        let physical=private_region(self.arena,index);
        let mut current=unsafe{Tables::resume(owner.root,owner.table_used)};
        if unsafe{current.modern_aperture()}!=Ok(owner.aperture){return Err(Error::Denied);}
        let leaf=owner.aperture as *mut u64;
        // SAFETY: current's validated owned table pool, preallocated leaf page.
        // Must be entirely absent before any writable construction mapping.
        unsafe{for i in 0..retirement::APERTURE_PAGES{
            if leaf.add(i).read_volatile()!=0{return Err(Error::Busy);}
        }}
        // Publish dirty/unschedulable ownership BEFORE the first mutation.
        // A construction error leaves Retiring; the next survivor trap removes
        // its root and erases the entire stride before Clean is possible again.
        self.processes[index]=Process{
            memory:retirement::Memory::Retiring,state:State::Dead,root:physical,
            generation,kernel_bottom:physical+KERNEL_BOTTOM,
            kernel_top:physical+KERNEL_TOP,frame:physical+KERNEL_TOP-720,
            entry:layout.entry,stack_end:STACK_VA+stack_pages*4096,..Process::EMPTY};
        // SAFETY: victim is Clean, no root/user/RX aliases exist, not current,
        // sole CPU/IF=0. The fixed supervisor-only RW/NX aperture exposes only
        // this kernel-owned 2MiB stride. Source payload is disjoint and sealed.
        unsafe{
            for i in 0..retirement::APERTURE_PAGES{
                leaf.add(i).write_volatile((physical+i as u64*4096)|3|(1<<63));
            }
            for i in 0..retirement::APERTURE_PAGES{
                let address=retirement::APERTURE+i as u64*4096;
                core::arch::asm!("invlpg [{}]",in(reg)address,options(nostack,preserves_flags));
            }
            ptr::copy_nonoverlapping(payload.as_ptr(),(retirement::APERTURE+IMAGE)as *mut u8,layout.header_size);
            for section in &layout.sections[..layout.count]{
                ptr::copy_nonoverlapping(payload.as_ptr().add(section.file_offset),
                    (retirement::APERTURE+IMAGE+section.virtual_offset as u64)as *mut u8,section.file_size);
            }
            ((retirement::APERTURE+BOOT)as *mut abi::Boot).write(handoff);
            let frame=retirement::APERTURE+KERNEL_TOP-720;
            ptr::write_bytes(frame as *mut u8,0,720);
            (frame as *mut u16).write(0x37f);((frame+24)as *mut u32).write(0x1f80);
            ((frame+512)as *mut arch::Trap).write(arch::Trap{
                rip:layout.entry,rsp:STACK_VA+stack_pages*4096-40,..arch::Trap::EMPTY});
            // Remove and flush EVERY writable construction alias before RX.
            for i in 0..retirement::APERTURE_PAGES{leaf.add(i).write_volatile(0);}
            for i in 0..retirement::APERTURE_PAGES{
                let address=retirement::APERTURE+i as u64*4096;
                core::arch::asm!("invlpg [{}]",in(reg)address,options(nostack,preserves_flags));
            }
            for i in 0..retirement::APERTURE_PAGES{
                if leaf.add(i).read_volatile()!=0{fatal("RAR-PANIC:CODE=TRIAL-ALIAS-RETIRE");}
            }
        }
        let region=staging::region(self.arena,boot::ARENA_PAGES).map_err(stage_error)?;
        // SAFETY: private table bytes were Clean and never exposed to users;
        // identity kernel aliases cover only the table/kernel-stack allocation.
        let mut tables=unsafe{Tables::new(physical)};
        let result=(||->Result<(),Error>{
            for page in 0..boot::ARENA_PAGES{
                let address=self.arena+page as u64*4096;
                if omit(self.arena,address){continue;}
                let writable=!(region.start..region.start+staging::BUFFER_BYTES as u64).contains(&address);
                unsafe{tables.map(mapping(address,address,1,writable,false),self.arena,
                    self.arena+boot::ARENA_PAGES as u64*4096)}.map_err(|_|Error::Invalid)?;
            }
            unsafe{boot::map_image(&mut tables,self.image_base,self.image_size);}
            let p=&mut self.processes[index];
            // Each validated layout has <=16 sections, plus header/stack/Boot,
            // so the fixed24 range entries cannot overflow.
            let mut user=|v:u64,pa:u64,pages:u64,w:bool,x:bool|->Result<(),Error>{
                unsafe{tables.map_user(mapping(v,pa,pages,w,x),pa,pa+pages*4096,false)}
                    .map_err(|_|Error::Invalid)?;
                p.range(v,v+pages*4096,w,x);Ok(())
            };
            user(pe::BASE,physical+IMAGE,1,false,false)?;
            for section in &layout.sections[..layout.count]{
                user(pe::BASE+section.virtual_offset as u64,
                    physical+IMAGE+section.virtual_offset as u64,
                    section.memory_size.div_ceil(4096)as u64,section.writable,section.executable)?;
            }
            user(STACK_VA,physical+USER_STACK,stack_pages,true,false)?;
            user(abi::BOOT_ADDRESS as u64,physical+BOOT,1,false,false)?;
            if handoff.role==3{
                let h=self.hardware;
                unsafe{tables.map_user(mapping(0x800000,h.framebuffer,h.framebuffer_bytes/4096,true,false),
                    h.framebuffer,h.framebuffer+h.framebuffer_bytes,true)}.map_err(|_|Error::Invalid)?;
                p.range(0x800000,0x800000+h.framebuffer_bytes,true,false);
            }
            p.aperture=unsafe{tables.reserve_modern_aperture()}.map_err(|_|Error::Invalid)?;
            unsafe{tables.check_modern_staging(region.start,false)}.map_err(|_|Error::Invalid)?;
            Ok(())
        })();
        self.processes[index].table_used=tables.used();
        result?;
        // Fully constructed but unpublished: the caller makes runnable last.
        self.processes[index].memory=retirement::Memory::Live;
        self.processes[index].state=State::Blocked;
        Ok(())
    }
}

impl Runtime{
    fn handover_context(&self,trial:model::Trial)->Result<(),Error>{
        self.stage_context();
        let index=trial.endpoint().slot as usize;let p=self.processes[index];
        if self.current!=8||self.stage_view||!self.stage_readonly||
            p.memory!=retirement::Memory::Live||p.state!=State::Blocked||
            p.root!=private_region(self.arena,index)||p.generation!=trial.endpoint().incarnation||
            p.stack_end!=STACK_VA+16384||p.kernel_bottom!=p.root+KERNEL_BOTTOM||
            p.kernel_top!=p.root+KERNEL_TOP||p.frame%16!=0||p.frame<p.kernel_bottom||
            p.frame.checked_add(720).is_none_or(|end|end>p.kernel_top){return Err(Error::Stale);}
        let stage=self.staging.as_ref().ok_or(Error::Stale)?;
        stage.view(trial.image_seal()).map_err(stage_error)?;
        if stage.reserved().is_none_or(|id|id.slot()!=index){return Err(Error::Stale);}
        let owner=self.processes[8];
        let mut tables=unsafe{Tables::resume(owner.root,owner.table_used)};
        if unsafe{tables.modern_aperture()}!=Ok(owner.aperture){return Err(Error::Denied);}
        // SAFETY: owned manager leaf and candidate saved frame, sole CPU/IF=0.
        unsafe{
            for i in 0..retirement::APERTURE_PAGES{
                if (owner.aperture as *const u64).add(i).read_volatile()!=0{return Err(Error::Busy);}
            }
            let frame=&*((p.frame+512)as *const arch::Trap);
            if !user_return_valid(&p,frame){return Err(Error::Invalid);}
        }
        Ok(())
    }
    pub(super) fn prepare_handover(&mut self,handle:u64,token:u64,seal:u64)->Result<(),Error>{
        if self.handover.is_some(){return Err(Error::Busy);}
        let policy=self.policy.as_ref().ok_or(Error::Denied)?;
        let t=policy.trial().ok_or(Error::Stale)?;
        if t.token()!=token||t.image_seal()!=seal{return Err(Error::Stale);}
        self.handover_context(t)?;
        let h=policy.prepare_cutover(self.current,handle,token)?;
        let b=support::handover_bootstrap(policy,&h,self.processes[t.endpoint().slot as usize].entry)?;
        self.handover=Some((h,b,seal));Ok(())
    }
    /// Manager calls only after the exact System transaction's durable ACK.
    /// Failure here is reconcile-required, not permission to undo the selector.
    /// No allocation/capability grant or recoverable operation remains after
    /// logical publication. Never allow caller retries to resurrect a process.
    pub(super) fn commit_handover(&mut self,handle:u64,token:u64,seal:u64){
        let (h,mut b,saved_seal)=self.handover.unwrap_or_else(||fatal("RAR-PANIC:CODE=UPDATE-RECONCILE"));
        let t=self.policy.as_ref().unwrap().trial()
            .unwrap_or_else(||fatal("RAR-PANIC:CODE=UPDATE-RECONCILE"));
        if h.token()!=token||saved_seal!=seal||t.token()!=token||t.image_seal()!=seal||
            h.endpoint()!=t.endpoint()||self.handover_context(t).is_err(){
            fatal("RAR-PANIC:CODE=UPDATE-RECONCILE");
        }
        #[cfg(rar_signed_updates)]
        let stale_probe=self.policy.as_mut().unwrap().lab_stale_begin(self.current,handle)
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=UPDATE-RECONCILE"));
        let cut=self.policy.as_mut().unwrap().cutover_prepared(self.current,handle,h)
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=UPDATE-RECONCILE"));
        self.handover=None;
        self.synchronize_revocations();
        // Old Settings is never this manager stack. Revoke its root and scrub
        // before exposing any new production context; peer state remains live.
        self.retire_pending();
        #[cfg(rar_signed_updates)]
        if let Some(probe)=stale_probe{
            self.policy.as_mut().unwrap().lab_stale_finish(self.current,handle,probe,token)
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=UPDATE-RECONCILE"));
            record("RAR-MODERN:STALE-AUTHORITY-REVOKED");
        }
        b.peers=self.policy.as_ref().unwrap().binding_generations();
        if !abi::valid_boot(&b){fatal("RAR-PANIC:CODE=UPDATE-RECONCILE");}
        self.write_active_boot(cut.current.slot as usize,b);
        self.processes[cut.current.slot as usize].state=State::Runnable;
        record("RAR-MODERN:SETTINGS-CUTOVER");
    }
}

impl Runtime{
    /// Prevalidated owner/candidate roots and empty aperture; sole CPU IF=0.
    /// The only writable alias is one non-executable private Boot page.
    fn write_active_boot(&mut self,index:usize,b:abi::Boot){
        let owner=self.processes[8];let target=self.processes[index];
        unsafe{
            let leaf=owner.aperture as *mut u64;
            leaf.write_volatile((target.root+BOOT)|3|(1<<63));
            core::arch::asm!("invlpg [{}]",in(reg)retirement::APERTURE,options(nostack,preserves_flags));
            (retirement::APERTURE as *mut abi::Boot).write_volatile(b);
            leaf.write_volatile(0);
            core::arch::asm!("invlpg [{}]",in(reg)retirement::APERTURE,options(nostack,preserves_flags));
            if leaf.read_volatile()!=0{fatal("RAR-PANIC:CODE=UPDATE-RECONCILE");}
        }
    }
    pub(super) fn prepare_boot_desktop(&mut self,handle:u64,token:u64,seal:u64)->Result<(),Error>{
        if self.handover.is_some()||self.desktop.is_some(){return Err(Error::Busy);}
        let policy=self.policy.as_ref().ok_or(Error::Denied)?;
        let t=policy.trial().ok_or(Error::Stale)?;
        if t.token()!=token||t.image_seal()!=seal{return Err(Error::Stale);}
        self.handover_context(t)?;
        let plan=policy.prepare_desktop(self.current,handle,token)?;
        let layout=pe::parse(SERVICE)?;
        let b=support::desktop_bootstrap(&plan,5,self.processes[t.endpoint().slot as usize].entry,
            self.hardware.pitch,self.hardware.format)?;
        let result=(||->Result<(),Error>{
            for role in [0usize,1,2,3,4,6]{
                let handoff=support::desktop_bootstrap(&plan,role,layout.entry,
                    self.hardware.pitch,self.hardware.format)?;
                self.construct_private(SERVICE,role,handoff.generation,16,handoff)?;
            }
            Ok(())
        })();
        if let Err(e)=result{
            // No plan is retained: all unbound construction contexts retire.
            // The healthy selected trial remains for explicit manager handling.
            self.synchronize_revocations();return Err(e);
        }
        // At most8192 bytes of model plan retained in static Runtime. It is
        // never passed through the constructor's mapping/copy stack frames.
        self.desktop=Some(NativeDesktop{plan,boot:b,seal,token});Ok(())
    }
    pub(super) fn commit_boot_desktop(&mut self,handle:u64,token:u64,seal:u64){
        let t=self.policy.as_ref().unwrap().trial()
            .unwrap_or_else(||fatal("RAR-PANIC:CODE=BOOT-RECONCILE"));
        if t.token()!=token||t.image_seal()!=seal||self.handover_context(t).is_err(){
            fatal("RAR-PANIC:CODE=BOOT-RECONCILE");
        }
        let d=self.desktop.as_ref().unwrap_or_else(||fatal("RAR-PANIC:CODE=BOOT-RECONCILE"));
        if d.token!=token||d.seal!=seal{fatal("RAR-PANIC:CODE=BOOT-RECONCILE");}
        for role in [0usize,1,2,3,4,6]{
            let e=d.plan.binding(role).unwrap_or_else(||fatal("RAR-PANIC:CODE=BOOT-RECONCILE"));
            let p=self.processes[role];
            let physical=private_region(self.arena,role);
            if e.slot as usize!=role||p.memory!=retirement::Memory::Live||p.state!=State::Blocked||
                p.generation!=e.incarnation||p.root!=physical||p.stack_end!=STACK_END||
                p.kernel_bottom!=physical+KERNEL_BOTTOM||p.kernel_top!=physical+KERNEL_TOP||
                p.frame%16!=0||p.frame<p.kernel_bottom||
                p.frame.checked_add(720).is_none_or(|end|end>p.kernel_top){
                fatal("RAR-PANIC:CODE=BOOT-RECONCILE");
            }
            // SAFETY: exact kernel-owned unscheduled frame checked above.
            if !user_return_valid(&p,unsafe{&*((p.frame+512)as *const arch::Trap)}){
                fatal("RAR-PANIC:CODE=BOOT-RECONCILE");
            }
        }
        let d=self.desktop.take().unwrap();let mut b=d.boot;
        self.policy.as_mut().unwrap().publish_desktop(self.current,handle,d.plan)
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=BOOT-RECONCILE"));
        b.peers=self.policy.as_ref().unwrap().binding_generations();
        if !abi::valid_boot(&b){fatal("RAR-PANIC:CODE=BOOT-RECONCILE");}
        self.write_active_boot(t.endpoint().slot as usize,b);
        for role in [0usize,1,2,3,4,6,t.endpoint().slot as usize]{
            self.processes[role].state=State::Runnable;
        }
        record("RAR-MODERN:DESKTOP-PUBLISHED");
    }
}
