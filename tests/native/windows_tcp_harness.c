#define LOBO_NATIVE_ENABLED 1
#define LOBO_WINDOWS_TCP 1
#include "native_runtime.c"
#include "native_api.h"
#include "native_buffers.c"
#include "native_floats.c"
#include "native_api.c"
#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
static bool fail_close=false;
static int checked_close(SOCKET socket) {
    if(fail_close) {WSASetLastError(WSAEWOULDBLOCK);return SOCKET_ERROR;}
    return closesocket(socket);
}
#define closesocket checked_close
#include "native_windows_tcp.c"
#define CHECK(x) do {if(!(x)) {fprintf(stderr,"check failed line %d: %s\n",__LINE__,#x);return 1;}} while(0)
static uint64_t cap(void) {lobo_depth=1;return lobo_native_capability_v1();}
static void after_fault(void) {
    if(lobo_native_pending_fd!=INVALID_SOCKET || lobo_windows_wsa_started) {fputs("fault cleanup failed\n",stderr);abort();}
    for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind==WT_LISTENER || lobo_native_slots[i].kind==WT_STREAM) abort();
}
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    if(!strcmp(argv[1],"fuel-cleanup") && atexit(after_fault)) return 2;
    if(atexit(lobo_cleanup)) return 2;
    lobo_validating=false;lobo_fuel=10000000;lobo_heap_cells=1000000;
    if(!strcmp(argv[1],"options")) {
        CHECK(!lobo_native_option("--native-listen","0.0.0.0:8080"));CHECK(!lobo_native_option("--native-listen","127.0.0.1:08080"));
        CHECK(!lobo_native_option("--native-connect","127.0.0.1:0"));CHECK(!lobo_native_option("--native-calls","0"));
        CHECK(lobo_native_option("--native-listen","127.0.0.1:8080"));CHECK(!lobo_native_option("--native-listen","127.0.0.1:8080"));
    } else if(!strcmp(argv[1],"deny")) {
        uint64_t c=cap();CHECK(lobo_native_listen_v1(c,0).error==WT_DENIED);CHECK(lobo_native_connect_v1(c,1,0).error==WT_DENIED);CHECK(!lobo_windows_wsa_started);
    } else {
        CHECK(lobo_native_option("--native-clock","true"));CHECK(lobo_native_option("--native-random","true"));CHECK(lobo_native_option("--native-listen","127.0.0.1:0"));CHECK(lobo_native_option("--native-connect","self"));
        uint64_t c=cap();
        if(!strcmp(argv[1],"authority")) {
            for(unsigned mask=0;mask<16;++mask) {
                memset(lobo_native_slots,0,sizeof(lobo_native_slots));lobo_native_root=0;lobo_native_calls=65536;c=cap();
                lobo_native_result_v1 p=lobo_native_restrict_capability_v1(c,mask);CHECK(!p.error);
                for(unsigned child=0;child<16;++child) {
                    lobo_native_result_v1 v=lobo_native_restrict_capability_v1(p.value.handle,child);
                    if((child&mask)!=child) CHECK(v.error==WT_DENIED);
                    else {
                        CHECK(!v.error);CHECK(lobo_native_monotonic_ms_v1(v.value.handle).error==((child&1)?0:WT_DENIED));
                        CHECK(lobo_native_random_bytes_v1(v.value.handle,1).error==((child&2)?0:WT_DENIED));
                        lobo_native_result_v1 l=lobo_native_listen_v1(v.value.handle,0);
                        CHECK(l.error==((child&4)?0:WT_DENIED));
                        if(!l.error) CHECK(!lobo_native_close_listener_v1(l.value.handle).error);
                    }
                }
            }
        } else if(!strcmp(argv[1],"bounds")) {
            CHECK(lobo_native_listen_v1(c,-1).error==WT_INPUT);CHECK(lobo_native_listen_v1(c,65536).error==WT_INPUT);
            CHECK(lobo_native_listen_v1(c,1).error==WT_DENIED);CHECK(lobo_native_connect_v1(c,1,-1).error==WT_INPUT);
            CHECK(lobo_native_connect_v1(c,1,2001).error==WT_INPUT);CHECK(lobo_native_connect_v1(c,1,1).error==WT_DENIED);
            CHECK(lobo_native_port_v1(c).error==WT_KIND);CHECK(lobo_native_port_v1(0).error==WT_HANDLE);
            CHECK(lobo_native_random_bytes_v1(c,-1).error==WT_INPUT);CHECK(lobo_native_random_bytes_v1(c,65537).error==WT_INPUT);
        } else if(!strcmp(argv[1],"quota")) {
            lobo_native_handles=1;CHECK(lobo_native_listen_v1(c,0).error==WT_LIMIT);CHECK(lobo_native_pending_fd==INVALID_SOCKET);
            lobo_native_bytes=0;CHECK(lobo_native_random_bytes_v1(c,1).error==WT_LIMIT);
            lobo_native_calls=0;CHECK(lobo_native_monotonic_ms_v1(c).error==WT_LIMIT);
        } else {
            lobo_native_result_v1 l=lobo_native_listen_v1(c,0);CHECK(!l.error);uint64_t listener=l.value.handle;
            lobo_native_result_v1 p=lobo_native_port_v1(listener);CHECK(!p.error && p.value.integer>0);
            CHECK(lobo_native_accept_v1(listener,0).error==WT_TIMEOUT);
            if(!strcmp(argv[1],"fuel-cleanup")) {
                lobo_native_pending_fd=WSASocketW(AF_INET,SOCK_STREAM,IPPROTO_TCP,NULL,0,0);CHECK(lobo_native_pending_fd!=INVALID_SOCKET);
                lobo_fuel=0;(void)lobo_native_wait_v1(lobo_native_pending_fd,POLLIN,lobo_native_deadline_v1(100));return 2;
            } else if(!strcmp(argv[1],"stale")) {
                CHECK(!lobo_native_close_listener_v1(listener).error);CHECK(lobo_native_port_v1(listener).error==WT_CLOSED);
                l=lobo_native_listen_v1(c,0);CHECK(!l.error && l.value.handle!=listener);CHECK(lobo_native_port_v1(listener).error==WT_HANDLE);
                CHECK(!lobo_native_close_listener_v1(l.value.handle).error);CHECK(lobo_native_close_listener_v1(l.value.handle).error==WT_CLOSED);
            } else if(!strcmp(argv[1],"close-failure")) {
                fail_close=true;CHECK(lobo_native_close_listener_v1(listener).error==WT_WOULD_BLOCK);
                CHECK(!lobo_native_port_v1(listener).error);
                fail_close=false;CHECK(!lobo_native_close_listener_v1(listener).error);
            } else if(!strcmp(argv[1],"cleanup")) {
                lobo_native_pending_fd=WSASocketW(AF_INET,SOCK_STREAM,IPPROTO_TCP,NULL,0,0);CHECK(lobo_native_pending_fd!=INVALID_SOCKET);
                SOCKET pending=lobo_native_pending_fd,registered=lobo_native_slots[(listener&255)-1].fd;
                lobo_native_cleanup();CHECK(lobo_native_pending_fd==INVALID_SOCKET);CHECK(!lobo_windows_wsa_started);
                CHECK(closesocket(pending)==SOCKET_ERROR);CHECK(closesocket(registered)==SOCKET_ERROR);
                lobo_native_cleanup();
            } else if(!strcmp(argv[1],"generation")) {
                lobo_windows_tcp_slot_v1 *slot=&lobo_native_slots[(listener&255)-1];slot->generation=UINT64_C(0x00ffffffffffffff);
                uint64_t token=(slot->generation<<8)|(listener&255);CHECK(!lobo_native_close_listener_v1(token).error);CHECK(slot->kind==WT_RETIRED);
                l=lobo_native_listen_v1(c,0);CHECK(!l.error && (l.value.handle&255)!=(listener&255));
            } else if(!strcmp(argv[1],"roundtrip")) {
                lobo_native_result_v1 client=lobo_native_connect_v1(c,p.value.integer,1000);CHECK(!client.error);
                lobo_native_result_v1 server=lobo_native_accept_v1(listener,1000);CHECK(!server.error);
                CHECK(lobo_native_read_v1(server.value.handle,1,0).error==WT_TIMEOUT);
                CHECK(lobo_native_read_v1(server.value.handle,-1,0).error==WT_INPUT);
                CHECK(lobo_native_port_v1(client.value.handle).error==WT_KIND);
                lobo_buffer data=lobo_buffer_new(4096);memset((void *)data.data,0xab,4096);
                lobo_native_result_v1 sent=lobo_native_write_v1(client.value.handle,data,1000);CHECK(!sent.error && sent.value.integer>0 && sent.value.integer<=4096);
                lobo_native_result_v1 got=lobo_native_read_v1(server.value.handle,sent.value.integer,1000);CHECK(!got.error && got.value.buffer.length>0 && got.value.buffer.data[0]==0xab);
                CHECK(!lobo_native_shutdown_write_v1(client.value.handle).error);
                CHECK(!lobo_native_close_stream_v1(client.value.handle).error);CHECK(!lobo_native_close_stream_v1(server.value.handle).error);
                CHECK(!lobo_native_close_listener_v1(listener).error);
            } else return 2;
        }
    }
    puts("windows-tcp-case-passed");return 0;
}
