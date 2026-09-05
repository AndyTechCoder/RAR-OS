/* Host-only libsodium1.0.19 oracle. Never linked into RAR OS. */
#include <sodium.h>
#define REFERENCE_ID 1
#include "reference-common.h"
static int reference_init(void) {
    return sodium_init()>=0 && strcmp(sodium_version_string(),"1.0.19")==0?0:2;
}
static int reference_hash(const unsigned char *p,size_t n,unsigned char out[32]) {
    return crypto_hash_sha256(out,p,n)==0?0:2;
}
static int reference_compute(unsigned op,const unsigned char *p,size_t n,
                             unsigned char *out,size_t *length) {
    *length=0;
    if (op==1 || op==2) {
        int rc=op==1?crypto_hash_sha256(out,p,n):crypto_hash_sha512(out,p,n);
        if (rc!=0) return 2;
        *length=op==1?32:64;return 0;
    }
    if (op==3) return crypto_sign_verify_detached(p+32,p+96,n-96,p)==0?0:1;
    size_t an=(size_t)p[44]|(size_t)p[45]<<8;
    size_t dn=(size_t)p[46]|(size_t)p[47]<<8;
    const unsigned char *aad=p+(op==4?48:64),*data=aad+an;
    if (op==4) {
        unsigned long long got=0;
        if (crypto_aead_chacha20poly1305_ietf_encrypt(out,&got,data,dn,aad,an,NULL,p+32,p)!=0 ||
            got!=dn+16) return 2;
        *length=dn+16;return 0;
    }
    if (crypto_aead_chacha20poly1305_ietf_decrypt_detached(out,NULL,data,dn,p+48,aad,an,p+32,p)!=0) return 1;
    *length=dn;return 0;
}
