/* Concrete replays of TLC witnesses; mapping and abstractions: verification/native/README.md. */
#define main existing_adapter_main
#include "adapter_harness.c"
#undef main

static int stream_token(uint64_t *token) {
    int fd=socket(AF_INET,SOCK_STREAM,0);CHECK(fd>=0);
    CHECK(!lobo_native_insert_v1(LN_STREAM,fd,0,0,token));return 0;
}
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    if(!strcmp(argv[1],"untracked-fault")) {
        char *args[]={argv[0],"formal-fault-before-publication"};return existing_adapter_main(2,args);
    }
    if(!strcmp(argv[1],"publish") || !strcmp(argv[1],"deadline")) {
        char *args[]={argv[0],!strcmp(argv[1],"publish")?"connect-eintr-unstarted":"poll-eintr-deadline"};
        return existing_adapter_main(2,args);
    }
    if(atexit(lobo_cleanup)) return 2;
    lobo_depth=1;lobo_validating=false;lobo_fuel=1000000;lobo_heap_cells=1000000;
    CHECK(lobo_native_option("--native-listen","127.0.0.1:0"));
    CHECK(lobo_native_option("--native-clock","true"));
    if(!strcmp(argv[1],"quota")) CHECK(lobo_native_option("--native-handles","1"));
    int before=descriptors();CHECK(before>0);
    uint64_t root=lobo_native_capability_v1();
    if(!strcmp(argv[1],"root")) {
        lobo_depth=2;(void)lobo_native_capability_v1();fputs("root entry guard bypassed\n",stderr);return 3;
    } else if(!strcmp(argv[1],"amplify")) {
        lobo_native_result_v1 parent=lobo_native_restrict_capability_v1(root,0);CHECK(!parent.error);
        CHECK(lobo_native_restrict_capability_v1(parent.value.handle,1).error==LN_DENIED);
    } else if(!strcmp(argv[1],"stale")) {
        uint64_t first=0,second=0;CHECK(!stream_token(&first));
        CHECK(!lobo_native_close_stream_v1(first).error);CHECK(!stream_token(&second));
        CHECK((first&255)==(second&255) && first!=second);
        CHECK(lobo_native_read_v1(first,0,0).error==LN_HANDLE);
        CHECK(!lobo_native_close_stream_v1(second).error);
    } else if(!strcmp(argv[1],"kind")) {
        CHECK(lobo_native_read_v1(root,0,0).error==LN_KIND);
    } else if(!strcmp(argv[1],"denied")) {
        lobo_native_result_v1 child=lobo_native_restrict_capability_v1(root,0);CHECK(!child.error);
        unsigned count=socket_calls;CHECK(lobo_native_listen_v1(child.value.handle,0).error==LN_DENIED);
        CHECK(socket_calls==count);
    } else if(!strcmp(argv[1],"quota")) {
        CHECK(lobo_native_listen_v1(root,0).error==LN_LIMIT);CHECK(descriptors()==before);
    } else if(!strcmp(argv[1],"wrap")) {
        lobo_native_slots[1].generation=UINT64_C(0x00ffffffffffffff);
        uint64_t first=0,second=0;CHECK(!stream_token(&first));CHECK((first&255)==2);
        CHECK(!lobo_native_close_stream_v1(first).error);CHECK(lobo_native_slots[1].kind==LN_RETIRED);
        CHECK(!stream_token(&second));CHECK((second&255)!=2);
        CHECK(!lobo_native_close_stream_v1(second).error);
    } else if(!strcmp(argv[1],"cleanup")) {
        lobo_native_result_v1 listener=lobo_native_listen_v1(root,0);CHECK(!listener.error);
        lobo_native_calls=0;lobo_fuel=0;lobo_native_cleanup();CHECK(descriptors()==before);
        lobo_native_cleanup();CHECK(descriptors()==before);
    } else return 2;
    lobo_native_cleanup();CHECK(descriptors()==before);
    puts("formal-trace-passed");return 0;
}
