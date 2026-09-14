/* Standalone CPL3 C application. Cloud CI compiles, never executes this entry. */
#include "../../../sdk/alpha/c/native.h"
#include "model.h"
#include "../../../sdk/alpha/c/agent.h"
static int agent_done=0;
static int send_frame(const rar_app_boot *boot,const uint8_t bytes[128]){
    unsigned i;int64_t start=rar_native_ticks(),now,result;
    if(start<0||start>INT64_MAX-100)return 0;
    for(i=0;i<256;i++){
        result=rar_native_send(boot,0,bytes);
        if(result==0)return 1;
        if(result!=-4)return 0; /* Full means not accepted; no success retry. */
        now=rar_native_ticks();if(now<start||now>=start+100)return 0;
        rar_native_yield();
    }return 0;
}
static int frame(const rar_app_boot *boot,uint32_t seq,const uint8_t *data,size_t n){
    uint8_t bytes[128];
    return rar_app_message_encode(RAR_APP_PAINT,0,seq,data,n,bytes,128)&&send_frame(boot,bytes);
}
static int paint_line(const rar_app_boot *boot,uint32_t seq,uint8_t row,const uint8_t *text,size_t n){
    uint8_t payload[50]={1,0};size_t i;if(n>48)return 0;payload[1]=row;
    for(i=0;i<n;i++){payload[2+i]=text[i];}
    return frame(boot,seq,payload,n+2);
}
static int paint(const rar_app_boot *boot,const rar_counter *counter,uint32_t *version){
    static const uint8_t title[]="RAR COUNTER / INDEPENDENT C APP";
    static const uint8_t help[]="PLUS/MINUS: CHANGE   0:RESET   Q:FAULT";
    static const uint8_t boundary[]="UI ONLY / DEVICE AND STORAGE ACCESS DENIED";
    const uint8_t begin[2]={0,6},commit[1]={2};uint8_t number[4];
    if(*version==UINT32_MAX){return 0;}
    (*version)++;rar_counter_text(counter,number);
    return frame(boot,*version,begin,2)&&paint_line(boot,*version,0,title,sizeof(title)-1)&&
        paint_line(boot,*version,1,number,4)&&paint_line(boot,*version,2,help,sizeof(help)-1)&&
        paint_line(boot,*version,3,boundary,sizeof(boundary)-1)&&
        paint_line(boot,*version,4,(const uint8_t *)(agent_done?"AGENT: 1 ALLOWED / 2 DENIED":"G: RUN SCOPED AGENT DEMONSTRATION"),agent_done?27:33)&&
        paint_line(boot,*version,5,(const uint8_t *)"DETERMINISTIC TEST PROVIDER / NOT PAL",37)&&
        frame(boot,*version,commit,1);
}
/* RAR-owned volatile memory helpers prevent libc/compiler helper imports.
 * All spans must be valid for n bytes; memcpy requires disjoint spans. */
void *memcpy(void *destination,const void *source,size_t n){
    volatile uint8_t *d=destination;const volatile uint8_t *s=source;size_t i;
    for(i=0;i<n;i++){d[i]=s[i];}
    return destination;
}
void *memset(void *destination,int value,size_t n){
    volatile uint8_t *d=destination;size_t i;
    for(i=0;i<n;i++){d[i]=(uint8_t)value;}
    return destination;
}
void *memmove(void *destination,const void *source,size_t n){
    volatile uint8_t *d=destination;const volatile uint8_t *s=source;size_t i;
    if((uintptr_t)d<(uintptr_t)s){for(i=0;i<n;i++)d[i]=s[i];}
    else{for(i=n;i>0;i--)d[i-1]=s[i-1];}return destination;
}
int memcmp(const void *left,const void *right,size_t n){
    const volatile uint8_t *a=left,*b=right;size_t i;
    for(i=0;i<n;i++){if(a[i]!=b[i])return (int)a[i]-(int)b[i];}return 0;
}
struct demo_provider {uint64_t actor;unsigned step;};
static int provide_demo(void *context,rar_agent_request *request){
    struct demo_provider *p=context;
    if(p->step>=3)return 0;
    p->step++;request->sequence=p->step;request->actor=p->actor;
    request->scope=RAR_AGENT_COUNTER_SCOPE;request->grant=1;
    request->operation=p->step==2?RAR_AGENT_READ_DOCUMENT:RAR_AGENT_INCREMENT;return 1;
}
static int increment_demo(void *context,uint8_t operation){
    rar_counter *counter=context;
    if(operation!=RAR_AGENT_INCREMENT||counter->value>=9999)return 0;
    counter->value++;return 1;
}
static int run_agent(const rar_app_boot *boot,rar_counter *counter){
    rar_agent_broker broker;struct demo_provider provider={boot->incarnation,0};
    int64_t now=rar_native_ticks();uint32_t before=counter->value;
    if(now<0||!rar_agent_issue(&broker,boot->incarnation,(uint64_t)now))return 0;
    if(rar_agent_step(&broker,(uint64_t)now,provide_demo,&provider,increment_demo,counter)!=RAR_AGENT_ALLOWED)return 0;
    now=rar_native_ticks();if(now<0)return 0;
    if(rar_agent_step(&broker,(uint64_t)now,provide_demo,&provider,increment_demo,counter)!=RAR_AGENT_DENIED)return 0;
    rar_agent_revoke(&broker);
    now=rar_native_ticks();if(now<0)return 0;
    if(rar_agent_step(&broker,(uint64_t)now,provide_demo,&provider,increment_demo,counter)!=RAR_AGENT_REVOKED)return 0;
    return counter->value==before+1&&broker.count==3&&broker.remaining==0;
}
__attribute__((ms_abi,noreturn)) void efi_main(void){
    rar_app_boot boot;rar_counter counter={0,0};uint32_t version=0;
    if(!rar_native_bootstrap(&boot)||boot.principal!=11||boot.rights!=RAR_APP_UI)rar_native_exit();
    /* Read-only device status, port and network probes only. No write opcode.
     * Every unrelated handle must be denied before the normal UI is published. */
    {
        size_t i;uint8_t probe[128]={0};
        for(i=0;i<5;i++){
            if(rar_native_call(7,boot.caps[i],0,0,0)!=-2||
               rar_native_call(3,boot.caps[i],0x64,0,0)!=-2||
               rar_native_call(13,boot.caps[i],0,0,0)!=-2)rar_native_exit();
        }
        if(rar_native_call(1,boot.caps[2],(uint64_t)(uintptr_t)probe,128,0)!=-1)rar_native_exit();
    }
    if(!paint(&boot,&counter,&version))rar_native_exit();
    for(;;){
        rar_app_received received;int result=rar_native_receive(&boot,&received);
        if(result==-1)rar_native_exit(); /* malformed dequeued frames are ignored */
        if(result==1&&rar_native_from_peer(&boot,&received,0)&&received.message.operation==RAR_APP_INPUT&&
            received.message.status==0&&received.message.length==1&&received.message.payload[0]=='g'&&!agent_done){
            if(!run_agent(&boot,&counter))rar_native_exit();
            agent_done=1;if(!paint(&boot,&counter,&version))rar_native_exit();continue;
        }
        if(result==1&&rar_native_from_peer(&boot,&received,0)&&received.message.operation==RAR_APP_INPUT&&
            received.message.status==0&&received.message.length==1&&received.message.payload[0]=='q'){
            __asm__ __volatile__("ud2");
        }
        if(result==1&&rar_native_from_peer(&boot,&received,0)&&
            rar_counter_input(&counter,&received.message)&&!paint(&boot,&counter,&version))rar_native_exit();
        rar_native_yield();
    }
}
