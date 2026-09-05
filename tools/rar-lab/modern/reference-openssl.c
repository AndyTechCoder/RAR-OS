/* Host-only OpenSSL3.0.13 oracle. Never linked into RAR OS. */
#include <openssl/crypto.h>
#include <openssl/evp.h>
#define REFERENCE_ID 2
#include "reference-common.h"
static int reference_init(void) {
    return OPENSSL_init_crypto(OPENSSL_INIT_NO_LOAD_CONFIG,NULL)==1 &&
        OpenSSL_version_num()==0x300000d0UL?0:2;
}
static int digest(const char *name,const unsigned char *p,size_t n,unsigned char *out,size_t expected) {
    EVP_MD *md=EVP_MD_fetch(NULL,name,"provider=default");unsigned int got=0;
    int ok=md && EVP_Digest(p,n,out,&got,md,NULL)==1 && got==expected;
    EVP_MD_free(md);return ok?0:2;
}
static int reference_hash(const unsigned char *p,size_t n,unsigned char out[32]) {
    return digest("SHA256",p,n,out,32);
}
static int reference_compute(unsigned op,const unsigned char *p,size_t n,
                             unsigned char *out,size_t *length) {
    *length=0;
    if (op==1 || op==2) {
        size_t want=op==1?32:64;
        int status=digest(op==1?"SHA256":"SHA512",p,n,out,want);
        if (status==0) *length=want;
        return status;
    }
    if (op==3) {
        EVP_PKEY *key=EVP_PKEY_new_raw_public_key_ex(NULL,"ED25519","provider=default",p,32);
        EVP_MD_CTX *ctx=EVP_MD_CTX_new();int status=2;
        if (key && ctx && EVP_DigestVerifyInit_ex(ctx,NULL,NULL,NULL,"provider=default",key,NULL)==1) {
            int result=EVP_DigestVerify(ctx,p+32,64,p+96,n-96);
            status=result==1?0:result==0?1:2;
        }
        EVP_MD_CTX_free(ctx);EVP_PKEY_free(key);return status;
    }
    size_t an=(size_t)p[44]|(size_t)p[45]<<8;
    size_t dn=(size_t)p[46]|(size_t)p[47]<<8;
    const unsigned char *aad=p+(op==4?48:64),*data=aad+an;
    EVP_CIPHER *cipher=EVP_CIPHER_fetch(NULL,"CHACHA20-POLY1305","provider=default");
    EVP_CIPHER_CTX *ctx=EVP_CIPHER_CTX_new();
    int count=0,total=0,final=0,status=2;
    if (!cipher || !ctx) goto done;
    if (op==4) {
        if (EVP_EncryptInit_ex(ctx,cipher,NULL,p,p+32)!=1 ||
            EVP_EncryptUpdate(ctx,NULL,&count,aad,(int)an)!=1 ||
            EVP_EncryptUpdate(ctx,out,&count,data,(int)dn)!=1) goto done;
        total=count;
        if (EVP_EncryptFinal_ex(ctx,out+total,&final)!=1 || total+final!=(int)dn ||
            EVP_CIPHER_CTX_ctrl(ctx,EVP_CTRL_AEAD_GET_TAG,16,out+dn)!=1) goto done;
        *length=dn+16;status=0;
    } else {
        if (EVP_DecryptInit_ex(ctx,cipher,NULL,NULL,NULL)!=1 ||
            EVP_CIPHER_CTX_ctrl(ctx,EVP_CTRL_AEAD_SET_TAG,16,(void *)(p+48))!=1 ||
            EVP_DecryptInit_ex(ctx,NULL,NULL,p,p+32)!=1 ||
            EVP_DecryptUpdate(ctx,NULL,&count,aad,(int)an)!=1 ||
            EVP_DecryptUpdate(ctx,out,&count,data,(int)dn)!=1) goto done;
        total=count;final=EVP_DecryptFinal_ex(ctx,out+total,&count);
        if (final==0) {status=1;goto done;}
        if (final!=1 || total+count!=(int)dn) goto done;
        *length=dn;status=0;
    }
done:
    EVP_CIPHER_CTX_free(ctx);EVP_CIPHER_free(cipher);return status;
}
