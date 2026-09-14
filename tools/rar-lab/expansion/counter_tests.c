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
    return 0;
}
