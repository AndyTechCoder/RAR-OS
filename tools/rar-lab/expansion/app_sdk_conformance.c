/* Host-only cross-language corpus. Does not execute app code or syscalls. */
#include "../../../sdk/alpha/c/app.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
static void put(uint8_t *p,uint64_t value,size_t n){
    size_t i;for(i=0;i<n;i++)p[i]=(uint8_t)(value>>(i*8));
}
int main(void){
    uint8_t payload[112],frame[128],boot[256]={0},copy[256],record[259];
    rar_app_boot decoded;rar_app_message msg;size_t i,len,op;
    for(i=0;i<112;i++)payload[i]=(uint8_t)i;
    for(op=1;op<=6;op++){for(len=0;len<=112;len++){
        assert(rar_app_message_encode((uint8_t)op,0,0xffffffffu,payload,len,frame,128));
        assert(rar_app_message_decode(frame,128,&msg));assert(msg.length==len);
        assert(fwrite(frame,1,128,stdout)==128);
    }}
    memset(frame,0xa5,128);
    assert(!rar_app_message_encode(1,0,0,payload,1,frame,128));
    for(i=0;i<128;i++)assert(frame[i]==0xa5);
    assert(!rar_app_message_encode(1,0,1,NULL,1,frame,128));
    assert(!rar_app_message_decode(NULL,128,&msg));
    memcpy(boot,"RARAPP00",8);put(boot+12,256,4);memset(boot+16,7,16);
    put(boot+32,0x100000001ULL,8);put(boot+40,10,4);put(boot+44,3,4);put(boot+48,0x401000,8);
    for(i=0;i<3;i++)put(boot+56+i*8,0x100000000ULL+i+1,8);
    put(boot+96,3,4);put(boot+100,1,4);put(boot+112,9,8);put(boot+120,10,8);
    assert(rar_app_boot_decode(boot,256,&decoded));
    assert(decoded.incarnation==0x100000001ULL);
    /* 257 exact-length cases followed by 256 short-length cases. */
    for(i=0;i<513;i++){
        memcpy(copy,boot,256);len=256;
        if(i>0&&i<=256)copy[i-1]^=0x80;
        if(i>256)len=i-257;
        put(record,len,2);
        record[2]=(uint8_t)rar_app_boot_decode(copy,len,&decoded);
        memcpy(record+3,copy,256);
        assert(fwrite(record,1,259,stdout)==259);
    }
    return ferror(stdout)?1:0;
}
