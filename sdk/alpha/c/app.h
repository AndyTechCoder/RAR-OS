/* Experimental RAR app v0. Explicit wire bytes, never C structure layout. */
#ifndef RAR_ALPHA_APP_H
#define RAR_ALPHA_APP_H
#include <stddef.h>
#include <stdint.h>
#include "constants.h"
typedef struct {
    uint8_t application[16]; uint64_t incarnation; uint32_t principal,rights;
    uint64_t entry,caps[5]; uint32_t peer_principals[4]; uint64_t peer_incarnations[4];
} rar_app_boot;
typedef struct {
    uint8_t operation,status; uint32_t sequence; uint16_t length; uint8_t payload[112];
} rar_app_message;
static inline uint32_t rar_app_u32(const uint8_t *p){
    return (uint32_t)p[0]|((uint32_t)p[1]<<8)|((uint32_t)p[2]<<16)|((uint32_t)p[3]<<24);
}
static inline uint64_t rar_app_u64(const uint8_t *p){
    return (uint64_t)rar_app_u32(p)|((uint64_t)rar_app_u32(p+4)<<32);
}
static inline int rar_app_zero(const uint8_t *p,size_t n){
    size_t i;for(i=0;i<n;i++){if(p[i]!=0)return 0;}return 1;
}
/* Input/output must designate their stated valid memory spans. Overlap is
 * supported: no output changes until the complete candidate is validated. */
static inline int rar_app_boot_decode(const uint8_t *p,size_t n,rar_app_boot *out){
    static const uint8_t magic[8]={'R','A','R','A','P','P','0','0'};
    static const uint32_t peers[4]={3,1,7,12};
    rar_app_boot b={0};size_t i;uint32_t enabled;
    if(p==NULL||out==NULL||n!=256)return 0;
    for(i=0;i<8;i++){if(p[i]!=magic[i])return 0;}
    if(rar_app_u32(p+8)!=0||rar_app_u32(p+12)!=256||!rar_app_zero(p+144,112))return 0;
    for(i=0;i<16;i++)b.application[i]=p[16+i];
    b.incarnation=rar_app_u64(p+32);b.principal=rar_app_u32(p+40);
    b.rights=rar_app_u32(p+44);b.entry=rar_app_u64(p+48);
    if(rar_app_zero(b.application,16)||b.incarnation==0||(b.principal!=10&&b.principal!=11)||
       (b.rights&1u)==0||(b.rights&~15u)!=0||b.entry<0x400000u||b.entry>=0x420000u)return 0;
    for(i=0;i<5;i++){
        b.caps[i]=rar_app_u64(p+56+i*8);
        enabled=i==0?1u:(b.rights&(1u<<(i-1)));
        if(enabled){if((b.caps[i]>>32)==0||(uint32_t)b.caps[i]!=(uint32_t)i+1)return 0;}
        else if(b.caps[i]!=0)return 0;
    }
    for(i=0;i<4;i++){
        b.peer_principals[i]=rar_app_u32(p+96+i*4);
        b.peer_incarnations[i]=rar_app_u64(p+112+i*8);
        if((b.rights&(1u<<i))!=0){
            if(b.peer_principals[i]!=peers[i]||b.peer_incarnations[i]==0)return 0;
        }else if(b.peer_principals[i]!=0||b.peer_incarnations[i]!=0)return 0;
    }
    *out=b;return 1;
}
static inline int rar_app_message_encode(uint8_t op,uint8_t status,uint32_t sequence,
    const uint8_t *payload,size_t length,uint8_t *out,size_t capacity){
    uint8_t b[128]={0};size_t i;
    if(out==NULL||capacity!=128||op<1||op>6||status>7||sequence==0||
       length>112||(length!=0&&payload==NULL))return 0;
    b[0]='R';b[1]='A';b[2]='P';b[3]='P';b[5]=op;b[6]=status;
    for(i=0;i<4;i++)b[8+i]=(uint8_t)(sequence>>(i*8));
    b[12]=(uint8_t)length;b[13]=(uint8_t)(length>>8);
    for(i=0;i<length;i++)b[16+i]=payload[i];
    for(i=0;i<128;i++)out[i]=b[i];
    return 1;
}
static inline int rar_app_message_decode(const uint8_t *p,size_t n,rar_app_message *out){
    rar_app_message m={0};size_t i;
    if(p==NULL||out==NULL||n!=128||p[0]!='R'||p[1]!='A'||p[2]!='P'||p[3]!='P'||
       p[4]!=0||p[7]!=0||p[14]!=0||p[15]!=0)return 0;
    m.operation=p[5];m.status=p[6];m.sequence=rar_app_u32(p+8);
    m.length=(uint16_t)((uint16_t)p[12]|((uint16_t)p[13]<<8));
    if(m.operation<1||m.operation>6||m.status>7||m.sequence==0||m.length>112||
       !rar_app_zero(p+16+m.length,112-m.length))return 0;
    for(i=0;i<m.length;i++)m.payload[i]=p[16+i];
    *out=m;return 1;
}
static inline int rar_app_reply_to(const rar_app_message *reply,const rar_app_message *request,
    uint32_t principal,uint64_t incarnation,uint32_t expected_principal,uint64_t expected_incarnation){
    return reply!=NULL&&request!=NULL&&expected_incarnation!=0&&principal==expected_principal&&
        incarnation==expected_incarnation&&reply->sequence==request->sequence&&
        reply->operation==request->operation;
}
#endif
