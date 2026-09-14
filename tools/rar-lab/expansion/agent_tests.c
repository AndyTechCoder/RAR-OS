/* Host-only broker decisions: no provider network, native trap or target app. */
#include <assert.h>
#include "../../../sdk/alpha/c/agent.h"
static int calls=0;
static int action(void *ctx,uint8_t op){(void)ctx;assert(op==RAR_AGENT_INCREMENT);calls++;return 1;}
static int provider(void *ctx,rar_agent_request *r){*r=*(rar_agent_request *)ctx;return 1;}
int main(void){
    rar_agent_broker b;rar_agent_request r={1,7,RAR_AGENT_COUNTER_SCOPE,1,RAR_AGENT_INCREMENT};
    assert(!rar_agent_issue(NULL,7,0));assert(!rar_agent_issue(&b,0,0));
    assert(!rar_agent_issue(&b,7,UINT64_MAX));
    assert(rar_agent_issue(&b,7,100));assert(rar_agent_step(&b,100,provider,&r,action,NULL)==RAR_AGENT_ALLOWED);
    assert(calls==1&&b.remaining==0&&b.count==1);
    assert(rar_agent_step(&b,100,provider,&r,action,NULL)==RAR_AGENT_STALE&&calls==1);
    r.sequence=2;assert(rar_agent_step(&b,101,provider,&r,action,NULL)==RAR_AGENT_EXHAUSTED&&calls==1);
    rar_agent_revoke(&b);r.sequence=3;
    assert(rar_agent_step(&b,101,provider,&r,action,NULL)==RAR_AGENT_REVOKED&&calls==1);
    assert(rar_agent_issue(&b,7,100));r.sequence=1;
    assert(rar_agent_step(&b,200,provider,&r,action,NULL)==RAR_AGENT_EXPIRED&&calls==1);
    assert(rar_agent_issue(&b,7,100));r.operation=RAR_AGENT_READ_DOCUMENT;
    assert(rar_agent_step(&b,101,provider,&r,action,NULL)==RAR_AGENT_DENIED&&calls==1);
    r.operation=RAR_AGENT_INCREMENT;r.sequence=2;r.actor=8;
    assert(rar_agent_step(&b,101,provider,&r,action,NULL)==RAR_AGENT_DENIED&&calls==1);
    r.sequence=3;r.actor=7;r.scope++;
    assert(rar_agent_step(&b,101,provider,&r,action,NULL)==RAR_AGENT_DENIED&&calls==1);
    r.sequence=4;r.scope=RAR_AGENT_COUNTER_SCOPE;r.grant=2;
    assert(rar_agent_step(&b,101,provider,&r,action,NULL)==RAR_AGENT_DENIED&&calls==1);
    r.sequence=5;r.grant=1;
    assert(rar_agent_step(&b,99,provider,&r,action,NULL)==RAR_AGENT_STALE&&calls==1);
    for(r.sequence=6;r.sequence<9;r.sequence++)assert(rar_agent_decide(&b,&r,102)!=RAR_AGENT_MALFORMED);
    assert(b.count==8);r.sequence=9;assert(rar_agent_decide(&b,&r,103)==RAR_AGENT_EXHAUSTED);
    assert(b.count==8&&calls==1);return 0;
}
