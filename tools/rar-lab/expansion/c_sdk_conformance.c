/* Host-only contract tests and a streaming Rust/C byte comparison producer. */
#include "../../../sdk/expansion/c/network.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
static void zeros(const uint8_t *bytes) {
    size_t i;for (i=0;i<RNET_MESSAGE;i++) assert(bytes[i]==0);
}
static void response(uint8_t *raw,uint8_t op,uint8_t status,uint64_t id,size_t length) {
    size_t i;rnet_zero(raw);raw[0]='R';raw[1]='N';raw[2]='E';raw[3]='T';
    raw[5]=op;raw[6]=status;
    for (i=0;i<8;i++) raw[8+i]=(uint8_t)(id>>(8*i));
    for (i=0;i<length;i++) raw[RNET_HEADER+i]=(uint8_t)i;
}
int main(void) {
    struct rnet_client c;struct rnet_response r;
    uint8_t out[RNET_MESSAGE],raw[RNET_MESSAGE],payload[RNET_PAYLOAD];
    size_t i,n,j;uint8_t op,status;
    for (i=0;i<RNET_PAYLOAD;i++) payload[i]=(uint8_t)i;
    assert(rnet_init(&c,7,UINT64_C(1)<<40)==RNET_SUCCESS);
    assert(rnet_begin(&c,RNET_SEND,(const uint8_t *)"abc",3,out,&n)==RNET_SUCCESS);
    assert(n==19 && memcmp(out,"RNET\0\1\0\0\1\0\0\0\0\0\0\0abc",19)==0);
    assert(rnet_begin(&c,RNET_SEND,payload,1,out,&n)==RNET_E_BUSY);assert(n==0);zeros(out);
    response(raw,RNET_SEND,RNET_OK,1,0);
    assert(rnet_accept(&c,7,1,raw,16,&r)==RNET_E_PEER);
    assert(rnet_accept(&c,7,UINT64_C(1)<<40,raw,16,&r)==RNET_SUCCESS);
    assert(rnet_accept(&c,7,UINT64_C(1)<<40,raw,16,&r)==RNET_E_UNEXPECTED);
    for (op=1;op<=3;op++) for (status=0;status<=8;status++) {
        assert(rnet_init(&c,7,UINT64_MAX)==RNET_SUCCESS);
        assert(rnet_begin(&c,op,NULL,0,out,&n)==RNET_SUCCESS);
        response(raw,op,status,1,1);
        assert((rnet_accept(&c,7,UINT64_MAX,raw,17,&r)==RNET_SUCCESS)==
            (op==RNET_RECEIVE && status==RNET_OK));
    }
    for (i=0;i<=RNET_PAYLOAD;i++) {
        assert(rnet_init(&c,7,UINT64_MAX)==RNET_SUCCESS);
        assert(rnet_begin(&c,RNET_RECEIVE,NULL,0,out,&n)==RNET_SUCCESS);
        response(raw,RNET_RECEIVE,RNET_OK,1,i);
        assert(rnet_accept(&c,7,UINT64_MAX,raw,16+i,&r)==RNET_SUCCESS);
        assert(r.length==i && (i==0 || memcmp(r.payload,payload,i)==0));
    }
    for (i=0;i<16;i++) {
        assert(rnet_init(&c,7,1)==RNET_SUCCESS);
        assert(rnet_begin(&c,RNET_RECEIVE,NULL,0,out,&n)==RNET_SUCCESS);
        response(raw,RNET_RECEIVE,RNET_OK,1,0);
        assert(rnet_accept(&c,7,1,raw,i,&r)==RNET_E_INVALID);
        raw[i]=255;
        assert(rnet_accept(&c,7,1,raw,16,&r)==RNET_E_INVALID);
        assert(c.pending_op==RNET_RECEIVE && c.pending_id==1);
    }
    assert(rnet_accept(&c,7,1,raw,129,&r)==RNET_E_INVALID);
    assert(rnet_accept(&c,7,1,NULL,16,&r)==RNET_E_INVALID);
    assert(rnet_init(&c,7,0)==RNET_E_INVALID);
    assert(rnet_init(&c,6,1)==RNET_E_INVALID);
    assert(rnet_init(&c,7,1)==RNET_SUCCESS);
    assert(rnet_begin(&c,RNET_RECEIVE,payload,1,out,&n)==RNET_E_INVALID);
    assert(c.next==1 && c.pending_op==0);zeros(out);
    assert(rnet_begin(&c,RNET_SEND,payload,113,out,&n)==RNET_E_INVALID);
    assert(rnet_begin(&c,RNET_SEND,NULL,1,out,&n)==RNET_E_INVALID);
    assert(rnet_begin(NULL,RNET_SEND,NULL,0,out,&n)==RNET_E_INVALID);
    for (status=0;status<=8;status++) {
        assert(rnet_init(&c,7,1)==RNET_SUCCESS);
        assert(rnet_begin(&c,RNET_CLOSE,NULL,0,out,&n)==RNET_SUCCESS);
        response(raw,RNET_CLOSE,status,1,0);
        assert(rnet_accept(&c,7,1,raw,16,&r)==RNET_SUCCESS);
        assert(c.closed==(status==0 || status==3 || status==7));
    }
    assert(rnet_init(&c,7,1)==RNET_SUCCESS);c.next=UINT64_MAX;
    assert(rnet_begin(&c,RNET_SEND,payload,1,out,&n)==RNET_E_EXHAUSTED);zeros(out);
    rnet_retire(&c);
    assert(rnet_begin(&c,RNET_SEND,payload,1,out,&n)==RNET_E_CLOSED);
    for (i=0;i<=112;i++) {
        for (j=0;j<i;j++) out[j]=(uint8_t)j;
        assert(rnet_encode(RNET_SEND,UINT64_MAX,out,i,out,&n)==RNET_SUCCESS);
        assert(n==16+i && (i==0 || memcmp(out+16,payload,i)==0));
    }
    /* 3 operations x payload lengths x 3 full-width IDs, exact length + all
       128 bytes including zero tail. The Rust consumer independently encodes. */
    for (op=1;op<=3;op++) for (i=0;i<=(op==1?112u:0u);i++) for (j=0;j<3;j++) {
        uint64_t id=j==0?1:j==1?(UINT64_C(1)<<40):UINT64_MAX;
        uint8_t record[RNET_MESSAGE+1];
        assert(rnet_encode(op,id,payload,i,out,&n)==RNET_SUCCESS);
        record[0]=(uint8_t)n;
        memcpy(record+1,out,RNET_MESSAGE);
        assert(fwrite(record,1,sizeof record,stdout)==sizeof record);
    }
    assert(fflush(stdout)==0);return 0;
}
