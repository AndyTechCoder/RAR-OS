#ifndef RAR_EXPANSION_NETWORK_H
#define RAR_EXPANSION_NETWORK_H
/* RAR-owned experimental wire SDK. No syscalls, allocation or device authority.
 * Caller supplies valid, non-overlapping client/output/length/response storage.
 * Payload may overlap the output buffer; it is copied before output clearing.
 * Reply payload is borrowed from raw for the lifetime of that caller buffer.
 */
#include <stddef.h>
#include <stdint.h>
#define RNET_HEADER 16u
#define RNET_MESSAGE 128u
#define RNET_PAYLOAD 112u
enum rnet_operation { RNET_SEND=1, RNET_RECEIVE=2, RNET_CLOSE=3 };
enum rnet_status { RNET_OK=0, RNET_INVALID=1, RNET_DENIED=2, RNET_CLOSED=3,
    RNET_FULL=4, RNET_EMPTY=5, RNET_BUDGET=6, RNET_IO=7, RNET_OVERSIZE=8 };
enum rnet_error { RNET_SUCCESS=0, RNET_E_INVALID, RNET_E_BUSY, RNET_E_CLOSED,
    RNET_E_EXHAUSTED, RNET_E_PEER, RNET_E_UNEXPECTED };
struct rnet_client {
    uint64_t peer, incarnation, next, pending_id;
    uint8_t pending_op, closed;
};
struct rnet_response {
    uint8_t operation, status;
    const uint8_t *payload;
    size_t length;
};
static inline void rnet_zero(uint8_t out[RNET_MESSAGE]) {
    size_t i; for (i=0;i<RNET_MESSAGE;i++) out[i]=0;
}
static inline int rnet_shape(uint8_t op, uint64_t id, size_t length) {
    return op>=RNET_SEND && op<=RNET_CLOSE && id!=0 &&
        length<=RNET_PAYLOAD && (op==RNET_SEND || length==0);
}
static inline enum rnet_error rnet_encode(uint8_t op, uint64_t id,
    const uint8_t *payload, size_t length, uint8_t out[RNET_MESSAGE], size_t *written) {
    uint8_t copy[RNET_PAYLOAD]; size_t i;
    if (out==NULL || written==NULL) return RNET_E_INVALID;
    *written=0;
    if (!rnet_shape(op,id,length) || (length!=0 && payload==NULL)) {
        rnet_zero(out);return RNET_E_INVALID;
    }
    for (i=0;i<length;i++) copy[i]=payload[i];
    rnet_zero(out);
    out[0]='R';out[1]='N';out[2]='E';out[3]='T';out[5]=op;
    for (i=0;i<8;i++) out[8+i]=(uint8_t)(id>>(8*i));
    for (i=0;i<length;i++) out[RNET_HEADER+i]=copy[i];
    *written=RNET_HEADER+length;return RNET_SUCCESS;
}
static inline enum rnet_error rnet_init(struct rnet_client *client,
    uint64_t peer, uint64_t incarnation) {
    if (client==NULL) return RNET_E_INVALID;
    client->peer=0;client->incarnation=0;client->next=0;client->pending_id=0;
    client->pending_op=0;client->closed=1;
    if (peer!=7 || incarnation==0) return RNET_E_INVALID;
    client->peer=peer;client->incarnation=incarnation;client->next=1;client->closed=0;
    return RNET_SUCCESS;
}
static inline void rnet_retire(struct rnet_client *client) {
    if (client!=NULL) {client->closed=1;client->pending_op=0;client->pending_id=0;}
}
static inline enum rnet_error rnet_begin(struct rnet_client *client,uint8_t op,
    const uint8_t *payload,size_t length,uint8_t out[RNET_MESSAGE],size_t *written) {
    enum rnet_error error;
    if (out==NULL || written==NULL) return RNET_E_INVALID;
    *written=0;
    if (client==NULL) {rnet_zero(out);return RNET_E_INVALID;}
    if (client->closed) {rnet_zero(out);return RNET_E_CLOSED;}
    if (client->pending_op!=0) {rnet_zero(out);return RNET_E_BUSY;}
    if (client->next==UINT64_MAX) {rnet_zero(out);return RNET_E_EXHAUSTED;}
    error=rnet_encode(op,client->next,payload,length,out,written);
    if (error!=RNET_SUCCESS) return error;
    client->pending_op=op;client->pending_id=client->next;client->next++;
    return RNET_SUCCESS;
}
static inline enum rnet_error rnet_accept(struct rnet_client *client,
    uint64_t peer,uint64_t incarnation,const uint8_t *raw,size_t length,
    struct rnet_response *response) {
    uint64_t id=0;size_t i;uint8_t op,status;
    if (client==NULL || response==NULL) return RNET_E_INVALID;
    response->operation=0;response->status=0;response->payload=NULL;response->length=0;
    if (peer!=client->peer || incarnation!=client->incarnation) return RNET_E_PEER;
    if (client->closed) return RNET_E_CLOSED;
    if (client->pending_op==0) return RNET_E_UNEXPECTED;
    if (raw==NULL || length<RNET_HEADER || length>RNET_MESSAGE) return RNET_E_INVALID;
    op=client->pending_op;status=raw[6];
    if (raw[0]!='R' || raw[1]!='N' || raw[2]!='E' || raw[3]!='T' ||
        raw[4]!=0 || raw[5]!=op || raw[7]!=0 || status>RNET_OVERSIZE)
        return RNET_E_INVALID;
    for (i=0;i<8;i++) id|=(uint64_t)raw[8+i]<<(8*i);
    if (id!=client->pending_id || ((op!=RNET_RECEIVE || status!=RNET_OK) && length!=RNET_HEADER))
        return RNET_E_INVALID;
    client->pending_op=0;client->pending_id=0;
    if (status==RNET_CLOSED || status==RNET_IO || (op==RNET_CLOSE && status==RNET_OK))
        client->closed=1;
    response->operation=op;response->status=status;response->payload=raw+RNET_HEADER;
    response->length=length-RNET_HEADER;return RNET_SUCCESS;
}
#endif
