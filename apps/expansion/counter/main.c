/* Standalone CPL3 C application. Cloud CI compiles, never executes this entry. */
#include "../../../sdk/alpha/c/native.h"
#include "model.h"
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
    static const uint8_t help[]="PLUS/SPACE: UP   MINUS: DOWN   0: RESET";
    static const uint8_t boundary[]="UI ONLY / NO DOCUMENT OR DEVICE GRANT";
    const uint8_t begin[2]={0,4},commit[1]={2};uint8_t number[4];
    if(*version==UINT32_MAX){return 0;}
    (*version)++;rar_counter_text(counter,number);
    return frame(boot,*version,begin,2)&&paint_line(boot,*version,0,title,sizeof(title)-1)&&
        paint_line(boot,*version,1,number,4)&&paint_line(boot,*version,2,help,sizeof(help)-1)&&
        paint_line(boot,*version,3,boundary,sizeof(boundary)-1)&&frame(boot,*version,commit,1);
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
__attribute__((ms_abi,noreturn)) void efi_main(void){
    rar_app_boot boot;rar_counter counter={0,0};uint32_t version=0;
    if(!rar_native_bootstrap(&boot)||boot.principal!=11||boot.rights!=RAR_APP_UI)rar_native_exit();
    if(!paint(&boot,&counter,&version))rar_native_exit();
    for(;;){
        rar_app_received received;int result=rar_native_receive(&boot,&received);
        if(result<0)rar_native_exit();
        if(result==1&&rar_native_from_peer(&boot,&received,0)&&
            rar_counter_input(&counter,&received.message)&&!paint(&boot,&counter,&version))rar_native_exit();
        rar_native_yield();
    }
}
