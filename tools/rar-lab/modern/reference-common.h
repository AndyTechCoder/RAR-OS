/* RAR-owned host-only binary adapter. One request per isolated process. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define MAX_PAYLOAD 4416u
#define MAX_OUTPUT 4112u
static unsigned char request_bytes[16+MAX_PAYLOAD], output[MAX_OUTPUT];
static int reference_init(void);
static int reference_hash(const unsigned char *,size_t,unsigned char[32]);
static int reference_compute(unsigned,const unsigned char *,size_t,unsigned char *,size_t *);
static uint32_t get32(const unsigned char *p) {
    return (uint32_t)p[0]|(uint32_t)p[1]<<8|(uint32_t)p[2]<<16|(uint32_t)p[3]<<24;
}
static void put32(unsigned char *p,uint32_t v) {
    for (unsigned i=0;i<4;i++) p[i]=(unsigned char)(v>>(8*i));
}
int main(void) {
    unsigned char header[64]={0};uint32_t n;unsigned op;size_t length=0;
    if (fread(request_bytes,1,16,stdin)!=16 || memcmp(request_bytes,"RARMCR00",8)!=0 ||
        request_bytes[9]!=0 || request_bytes[10]!=0 || request_bytes[11]!=0) return 64;
    op=request_bytes[8];n=get32(request_bytes+12);
    if (op<1 || op>5 || n>MAX_PAYLOAD || fread(request_bytes+16,1,n,stdin)!=n ||
        fgetc(stdin)!=EOF || ferror(stdin)) return 64;
    const unsigned char *p=request_bytes+16;
    if ((op==1 || op==2) && n>4096) return 64;
    if (op==3 && (n<96 || n>4192)) return 64;
    if (op==4 || op==5) {
        size_t base=op==4?48:64;
        if (n<base) return 64;
        size_t an=(size_t)p[44]|(size_t)p[45]<<8;
        size_t dn=(size_t)p[46]|(size_t)p[47]<<8;
        if (an>256 || dn>4096 || n!=base+an+dn) return 64;
    }
    if (reference_init()!=0) return 70;
    int status=reference_compute(op,p,n,output,&length);
    if (status<0 || status>2 || length>MAX_OUTPUT) return 70;
    if (status!=0) {length=0;memset(output,0,sizeof output);}
    if (status==1 && op!=3 && op!=5) return 70;
    memcpy(header,"RARMCO00",8);header[8]=(unsigned char)op;
    header[9]=(unsigned char)status;header[10]=REFERENCE_ID;
    put32(header+16,(uint32_t)length);
    if (reference_hash(request_bytes,n+16,header+24)!=0) return 70;
    if (fwrite(header,1,sizeof header,stdout)!=sizeof header ||
        fwrite(output,1,length,stdout)!=length || fflush(stdout)!=0) return 74;
    return status==2?70:0;
}
