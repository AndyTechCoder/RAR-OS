/* Provider-neutral bounded agent requests. No network, file or device API.
 * The UI/application host owns grant issuance; provider output is only bytes. */
#ifndef RAR_ALPHA_AGENT_H
#define RAR_ALPHA_AGENT_H
#include <stdint.h>
#include <stddef.h>
#define RAR_AGENT_INCREMENT 1u
#define RAR_AGENT_READ_DOCUMENT 2u
#define RAR_AGENT_COUNTER_SCOPE UINT64_C(0x524152434f554e54)
enum rar_agent_decision { RAR_AGENT_ALLOWED=0,RAR_AGENT_MALFORMED=1,
    RAR_AGENT_STALE=2,RAR_AGENT_DENIED=3,RAR_AGENT_EXPIRED=4,
    RAR_AGENT_REVOKED=5,RAR_AGENT_EXHAUSTED=6 };
typedef struct {
    uint64_t sequence,actor,scope,grant;
    uint8_t operation;
} rar_agent_request;
typedef struct { uint64_t sequence;uint8_t operation,decision; } rar_agent_audit;
typedef struct {
    uint64_t actor,scope,grant,deadline,last,tick;
    unsigned remaining,count;int active;
    rar_agent_audit audit[8];
} rar_agent_broker;
typedef int (*rar_agent_provider)(void *context,rar_agent_request *request);
typedef int (*rar_agent_action)(void *context,uint8_t operation);
static inline int rar_agent_issue(rar_agent_broker *b,uint64_t actor,uint64_t now){
    rar_agent_broker fresh={0};
    if(b==NULL||actor==0||now>UINT64_MAX-100)return 0;
    fresh.actor=actor;fresh.scope=RAR_AGENT_COUNTER_SCOPE;fresh.grant=1;
    fresh.deadline=now+100;fresh.tick=now;fresh.remaining=1;fresh.active=1;*b=fresh;return 1;
}
static inline void rar_agent_revoke(rar_agent_broker *b){if(b!=NULL)b->active=0;}
static inline enum rar_agent_decision rar_agent_decide(rar_agent_broker *b,const rar_agent_request *r,uint64_t now){
    enum rar_agent_decision d;
    if(b==NULL||r==NULL)return RAR_AGENT_MALFORMED;
    if(b->count==8)return RAR_AGENT_EXHAUSTED;
    if(now<b->tick||r->sequence==0||r->sequence<=b->last)d=RAR_AGENT_STALE;
    else{
        b->last=r->sequence;b->tick=now;
        if(!b->active)d=RAR_AGENT_REVOKED;
        else if(now>=b->deadline)d=RAR_AGENT_EXPIRED;
        else if(r->actor!=b->actor||r->scope!=b->scope||r->grant!=b->grant||
                r->operation!=RAR_AGENT_INCREMENT)d=RAR_AGENT_DENIED;
        else if(b->remaining==0)d=RAR_AGENT_EXHAUSTED;
        else{b->remaining--;d=RAR_AGENT_ALLOWED;}
    }
    b->audit[b->count].sequence=r->sequence;b->audit[b->count].operation=r->operation;
    b->audit[b->count].decision=(uint8_t)d;b->count++;return d;
}
/* Action runs only after a consumed grant. Failure is never retried/refunded.
 * Provider/action functions are trusted host integrations, not model code.
 * Any model/backend supplies only the bounded request value via provider. */
static inline enum rar_agent_decision rar_agent_step(rar_agent_broker *b,uint64_t now,
    rar_agent_provider provider,void *provider_context,rar_agent_action action,void *action_context){
    rar_agent_request request={0};enum rar_agent_decision decision;
    if(provider==NULL||action==NULL||!provider(provider_context,&request))return RAR_AGENT_MALFORMED;
    decision=rar_agent_decide(b,&request,now);
    if(decision==RAR_AGENT_ALLOWED&&!action(action_context,request.operation))return RAR_AGENT_EXHAUSTED;
    return decision;
}
#endif
