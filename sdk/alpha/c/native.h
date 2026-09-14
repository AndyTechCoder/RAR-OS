/* Native x86-64 RAR SDK. This header is not a host-runtime API. */
#ifndef RAR_ALPHA_NATIVE_H
#define RAR_ALPHA_NATIVE_H
#if !defined(__x86_64__) || (!defined(RAR_APP_NATIVE) && !defined(RAR_APP_OBJECT_CHECK))
#error Native SDK requires explicit RAR target or cloud object-only compilation
#endif
#include "app.h"
typedef struct{uint32_t principal;uint64_t incarnation;rar_app_message message;}rar_app_received;
/* Kernel int80 preserves registers except RAX. Memory/flags clobbers prevent
 * compiler reordering around the kernel's checked copy of complete spans. */
static inline int64_t rar_native_call(uint64_t n,uint64_t a,uint64_t b,uint64_t c,uint64_t d){
    register uint64_t r10 __asm__("r10")=d;
    __asm__ __volatile__("int $0x80":"+a"(n):"D"(a),"S"(b),"d"(c),"r"(r10):"memory","cc");
    return (int64_t)n;
}
/* Precondition: only RAR app entry, after kernel maps initialized RO bootstrap.
 * This pointer is never supplied by an IPC message or dereferenced on a host. */
static inline int rar_native_bootstrap(rar_app_boot *out){
    uint8_t raw[256];size_t i;
    const volatile uint8_t *p=(const volatile uint8_t *)(uintptr_t)0x700000u;
    for(i=0;i<256;i++)raw[i]=p[i];
    return rar_app_boot_decode(raw,256,out);
}
static inline void rar_native_yield(void){(void)rar_native_call(0,0,0,0,0);}
static inline int64_t rar_native_ticks(void){return rar_native_call(6,0,0,0,0);}
static inline int64_t rar_native_send(const rar_app_boot *boot,size_t peer,const uint8_t message[128]){
    if(boot==NULL||message==NULL||peer>=4||boot->peer_incarnations[peer]==0)return -1;
    return rar_native_call(1,boot->caps[peer+1],(uint64_t)(uintptr_t)message,128,0);
}
/* Return1 for canonical frame,0 empty,-1 malformed/failed. Never truncate IDs. */
static inline int rar_native_receive(const rar_app_boot *boot,rar_app_received *out){
    uint8_t raw[152]={0};rar_app_received got={0};uint64_t principal;int64_t result;
    if(boot==NULL||out==NULL)return -1;
    result=rar_native_call(2,boot->caps[0],(uint64_t)(uintptr_t)raw,152,0);
    if(result==-5)return 0;
    principal=rar_app_u64(raw);
    if(result!=128||principal>UINT32_MAX||rar_app_u64(raw+8)==0||rar_app_u64(raw+16)!=128||
       !rar_app_message_decode(raw+24,128,&got.message))return -1;
    got.principal=(uint32_t)principal;got.incarnation=rar_app_u64(raw+8);*out=got;return 1;
}
static inline int rar_native_from_peer(const rar_app_boot *boot,const rar_app_received *got,size_t peer){
    return boot!=NULL&&got!=NULL&&peer<4&&boot->peer_incarnations[peer]!=0&&
        got->principal==boot->peer_principals[peer]&&got->incarnation==boot->peer_incarnations[peer];
}
static inline _Noreturn void rar_native_exit(void){
    (void)rar_native_call(5,0,0,0,0);for(;;)rar_native_yield();
}
#endif
