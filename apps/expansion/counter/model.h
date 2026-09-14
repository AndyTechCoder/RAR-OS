/* Independent C counter logic, no native calls or ambient services. */
#ifndef RAR_COUNTER_MODEL_H
#define RAR_COUNTER_MODEL_H
#include "../../../sdk/alpha/c/app.h"
typedef struct{uint32_t value,last_input;}rar_counter;
static inline int rar_counter_input(rar_counter *counter,const rar_app_message *m){
    if(counter==NULL||m==NULL||m->operation!=RAR_APP_INPUT||m->status!=0||m->length!=1||
        m->sequence==0||m->sequence<=counter->last_input)return 0;
    if(m->payload[0]!='+'&&m->payload[0]!='-'&&m->payload[0]!='0'&&m->payload[0]!=' ')return 0;
    counter->last_input=m->sequence;
    if(m->payload[0]=='0')counter->value=0;
    else if(m->payload[0]=='-'){if(counter->value>0)counter->value--;}
    else if(counter->value<9999)counter->value++;
    return 1;
}
static inline void rar_counter_text(const rar_counter *counter,uint8_t out[4]){
    uint32_t value=counter->value;size_t i;
    for(i=4;i>0;i--){out[i-1]=(uint8_t)('0'+value%10);value/=10;}
}
#endif
