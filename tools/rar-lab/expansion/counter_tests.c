/* Host test of pure C state only; native.h and entry.c are deliberately absent. */
#include "../../../apps/expansion/counter/model.h"
#include <assert.h>
int main(void){
    rar_counter c={0,0};rar_app_message m={0};uint8_t text[4];unsigned i;
    m.operation=RAR_APP_INPUT;m.length=1;m.sequence=1;m.payload[0]='+';
    assert(rar_counter_input(&c,&m));assert(c.value==1);
    assert(!rar_counter_input(&c,&m));assert(c.value==1);
    for(i=2;i<=10010;i++){m.sequence=i;assert(rar_counter_input(&c,&m));}
    assert(c.value==9999);rar_counter_text(&c,text);
    assert(text[0]=='9'&&text[1]=='9'&&text[2]=='9'&&text[3]=='9');
    m.sequence++;m.payload[0]='0';assert(rar_counter_input(&c,&m));assert(c.value==0);
    m.sequence++;m.payload[0]='-';assert(rar_counter_input(&c,&m));assert(c.value==0);
    m.sequence++;m.status=1;assert(!rar_counter_input(&c,&m));
    m.status=0;m.length=2;assert(!rar_counter_input(&c,&m));
    m.length=1;m.operation=RAR_APP_WRITE_DOCUMENT;assert(!rar_counter_input(&c,&m));

    {
        uint8_t raw[152]={0},message[128];rar_app_received received;size_t len;
        assert(rar_app_message_encode(RAR_APP_INPUT,0,1,(const uint8_t *)"+",1,message,128));
        raw[0]=3;raw[8]=3;raw[12]=1;raw[16]=128;
        for(len=0;len<128;len++)raw[24+len]=message[len];
        assert(rar_app_envelope_decode(raw,152,128,&received));
        assert(received.principal==3&&received.incarnation==UINT64_C(0x100000003));
        for(len=0;len<152;len++)assert(!rar_app_envelope_decode(raw,len,128,&received));
        assert(!rar_app_envelope_decode(raw,152,127,&received));
        assert(!rar_app_envelope_decode(raw,152,-5,&received));
        raw[4]=1;assert(!rar_app_envelope_decode(raw,152,128,&received));raw[4]=0;
        raw[8]=0;raw[12]=0;assert(!rar_app_envelope_decode(raw,152,128,&received));
        raw[8]=3;raw[16]=127;assert(!rar_app_envelope_decode(raw,152,128,&received));
        raw[16]=128;raw[24]='X';assert(!rar_app_envelope_decode(raw,152,128,&received));
    }
    return 0;
}
