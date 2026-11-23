/* Adapted behavior checks: see tests/upstream/native/README.md for pinned sources and licenses. */
#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#include <poll.h>
#include <sys/socket.h>
#include <sys/random.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#include <dirent.h>
#include "native_runtime.c"
#define LOBO_NATIVE_ENABLED 1
#include "native_api.h"
#include "native_buffers.c"
#include "native_floats.c"

static unsigned injection=0, clock_calls=0, clock_total=0, socket_calls=0, random_calls=0;
static int fault_baseline=-1;
static int64_t fake_millis=100;
static int test_clock(clockid_t id,struct timespec *out) {
    ++clock_total;
    if(!injection) return clock_gettime(id,out);
    ++clock_calls;
    if(injection==4 && clock_calls>1) {errno=EIO;return -1;}
    out->tv_sec=(time_t)(fake_millis/1000);out->tv_nsec=(long)(fake_millis%1000)*1000000;
    ++fake_millis;return 0;
}
static int test_poll(struct pollfd *p,nfds_t count,int timeout) {
    if(injection==1) {errno=EINTR;return -1;}
    if(injection==2) {p[0].revents=POLLERR;return 1;}
    return poll(p,count,timeout);
}
static ssize_t test_recv(int fd,void *data,size_t count,int flags) {
    if(injection==4) {errno=EINTR;return -1;}
    return recv(fd,data,injection==3 && count>1?1:count,flags);
}
static ssize_t test_send(int fd,const void *data,size_t count,int flags) {
    return send(fd,data,injection==3 && count>1?1:count,flags);
}
static int test_socket(int domain,int type,int protocol) {
    ++socket_calls;return socket(domain,type,protocol);
}
static ssize_t test_random(void *data,size_t count,unsigned flags) {
    ++random_calls;
    if(injection==5) {
        if(random_calls%3==1) {errno=EINTR;return -1;}
        if(count>3) count=3;
        memset(data,0xa7,count);return (ssize_t)count;
    }
    return getrandom(data,count,flags);
}
static int test_connect(int fd,const struct sockaddr *address,socklen_t length) {
    if(injection==6) {errno=ECONNREFUSED;return -1;}
    if(injection==7) {errno=EINTR;return -1;}
    return connect(fd,address,length);
}
#define connect test_connect
#define clock_gettime test_clock
#define poll test_poll
#define recv test_recv
#define send test_send
#define socket test_socket
#define getrandom test_random
#include "native_api.c"
#undef clock_gettime
#undef poll
#undef recv
#undef send
#undef socket
#undef getrandom
#undef connect
#define CHECK(condition) do {if(!(condition)) {fprintf(stderr,"check failed line %d: %s\n",__LINE__,#condition);return 1;}} while(0)
static int descriptors(void) {
    DIR *directory=opendir("/proc/self/fd");if(!directory) return -1;
    int count=0;struct dirent *entry;
    while((entry=readdir(directory))) if(strcmp(entry->d_name,".") && strcmp(entry->d_name,"..")) ++count;
    closedir(directory);return count;
}
static void verify_fault_cleanup(void) {
    if(fault_baseline<0) return;
    if(descriptors()!=fault_baseline) {fputs("descriptor leak at fault exit\n",stderr);_Exit(2);}
    puts("native-cleanup-on-fault-passed");
}
static int pair(uint64_t *listener,uint64_t *client,uint64_t *server) {
    CHECK(lobo_native_option("--native-listen","127.0.0.1:0"));
    CHECK(lobo_native_option("--native-connect","self"));
    uint64_t cap=lobo_native_capability_v1();
    lobo_native_result_v1 l=lobo_native_listen_v1(cap,0);CHECK(!l.error);*listener=l.value.handle;
    lobo_native_result_v1 port=lobo_native_port_v1(*listener);CHECK(!port.error);
    lobo_native_result_v1 c=lobo_native_connect_v1(cap,port.value.integer,1000);CHECK(!c.error);*client=c.value.handle;
    lobo_native_result_v1 s=lobo_native_accept_v1(*listener,1000);CHECK(!s.error);*server=s.value.handle;
    return 0;
}
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    if(atexit(verify_fault_cleanup) || atexit(lobo_cleanup)) return 2;
    lobo_depth=1;lobo_validating=false;lobo_fuel=1000000;lobo_heap_cells=1000000;
    if(!strcmp(argv[1],"abi-owned-buffer")) {
        unsigned char input[17];for(unsigned i=0;i<17;++i) input[i]=(unsigned char)(i*17);
        lobo_buffer copy=lobo_native_copy_bytes_v1((lobo_buffer){input,17});
        CHECK(copy.length==17 && copy.data!=input);input[0]=99;
        for(unsigned i=0;i<17;++i) CHECK(copy.data[i]==(unsigned char)(i*17));
    } else if(!strcmp(argv[1],"null-buffer")) {
        (void)lobo_native_copy_bytes_v1((lobo_buffer){NULL,1});return 2;
    } else if(!strcmp(argv[1],"cap-rights-matrix")) {
        int before=descriptors();CHECK(before>0);
        for(unsigned parent=0;parent<16;++parent) {
            memset(lobo_native_slots,0,sizeof(lobo_native_slots));lobo_native_root=0;lobo_native_calls=65536;
            lobo_native_clock=true;lobo_native_random=true;lobo_native_listen=true;lobo_native_connect=true;lobo_native_self=true;lobo_native_listen_port=0;
            uint64_t root=lobo_native_capability_v1();
            lobo_native_result_v1 p=lobo_native_restrict_capability_v1(root,parent);CHECK(!p.error);
            for(unsigned child=0;child<16;++child) {
                lobo_native_result_v1 c=lobo_native_restrict_capability_v1(p.value.handle,child);
                if((parent&child)!=child) {CHECK(c.error==LN_DENIED);continue;}
                CHECK(!c.error);uint64_t cap=c.value.handle;
                unsigned clocks=clock_total; lobo_native_result_v1 clock=lobo_native_monotonic_ms_v1(cap);
                CHECK(clock.error==((child&1)?0:LN_DENIED));if(!(child&1)) CHECK(clock_total==clocks);
                unsigned randoms=random_calls;lobo_native_result_v1 random=lobo_native_random_bytes_v1(cap,0);
                CHECK(random.error==((child&2)?0:LN_DENIED));CHECK(random_calls==randoms);
                unsigned sockets=socket_calls;lobo_native_result_v1 listener=lobo_native_listen_v1(cap,0);
                CHECK(listener.error==((child&4)?0:LN_DENIED));
                if(child&4) {CHECK(socket_calls==sockets+1);CHECK(!lobo_native_close_listener_v1(listener.value.handle).error);} else CHECK(socket_calls==sockets);
                uint64_t shadow=0;CHECK(!lobo_native_insert_v1(LN_LISTENER,-1,31337,cap,&shadow));
                sockets=socket_calls;injection=6;lobo_native_result_v1 connected=lobo_native_connect_v1(cap,31337,1000);injection=0;
                CHECK(connected.error==((child&8)?LN_REFUSED:LN_DENIED));CHECK(socket_calls==sockets+((child&8)?1:0));
                lobo_native_slot_v1 *slot=NULL;CHECK(!lobo_native_lookup_v1(shadow,LN_LISTENER,&slot));lobo_native_release_v1(slot);
                CHECK(!lobo_native_lookup_v1(cap,LN_CAPABILITY,&slot));lobo_native_release_v1(slot);
                CHECK(descriptors()==before);
            }
            lobo_native_cleanup();CHECK(descriptors()==before);
        }
    } else if(!strcmp(argv[1],"registry")) {
        uint64_t tokens[64]={0},random=0x8a7219;unsigned live=0;
        for(unsigned step=0;step<50000;++step) {
            random^=random<<13;random^=random>>7;random^=random<<17;
            unsigned i=(unsigned)(random%64);
            if(tokens[i]) {
                lobo_native_slot_v1 *slot=NULL;
                CHECK(!lobo_native_lookup_v1(tokens[i],LN_STREAM,&slot));
                CHECK(lobo_native_lookup_v1(tokens[i],LN_LISTENER,&slot)==LN_KIND);
                lobo_native_release_v1(slot);
                CHECK(lobo_native_lookup_v1(tokens[i],LN_STREAM,&slot)==LN_CLOSED);
                tokens[i]=0;--live;
            } else {
                uint64_t token=0;CHECK(!lobo_native_insert_v1(LN_STREAM,-1,0,0,&token));
                tokens[i]=token;++live;
            }
            unsigned count=0;
            for(unsigned k=0;k<64;++k) if(lobo_native_slots[k].kind==LN_STREAM) ++count;
            CHECK(count==live);
            lobo_native_slot_v1 *unused=NULL;
            CHECK(lobo_native_lookup_v1(0,LN_STREAM,&unused)==LN_HANDLE);
            CHECK(lobo_native_lookup_v1(UINT64_MAX,LN_STREAM,&unused)==LN_HANDLE);
            uint64_t invalid=(random&~UINT64_C(255))|255;
            CHECK(lobo_native_lookup_v1(invalid,LN_STREAM,&unused)==LN_HANDLE);
        }
        for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind==LN_STREAM) lobo_native_release_v1(&lobo_native_slots[i]);
        lobo_native_slots[0].generation=UINT64_C(0x00ffffffffffffff);
        uint64_t token=0;CHECK(!lobo_native_insert_v1(LN_STREAM,-1,0,0,&token));
        CHECK((token&255)==1);lobo_native_release_v1(&lobo_native_slots[0]);
        CHECK(lobo_native_slots[0].kind==LN_RETIRED);
        CHECK(!lobo_native_insert_v1(LN_STREAM,-1,0,0,&token));CHECK((token&255)!=1);
        for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind==LN_STREAM) lobo_native_release_v1(&lobo_native_slots[i]);
    } else if(!strcmp(argv[1],"deny")) {
        uint64_t cap=lobo_native_capability_v1();
        CHECK(lobo_native_listen_v1(cap,0).error==LN_DENIED);CHECK(socket_calls==0);
        CHECK(lobo_native_random_bytes_v1(cap,4).error==LN_DENIED);CHECK(random_calls==0);
    } else if(!strcmp(argv[1],"random-short-eintr")) {
        CHECK(lobo_native_option("--native-random","true"));injection=5;
        lobo_native_result_v1 r=lobo_native_random_bytes_v1(lobo_native_capability_v1(),17);
        CHECK(!r.error && r.value.buffer.length==17);CHECK(random_calls>6);
        for(unsigned i=0;i<17;++i) CHECK(r.value.buffer.data[i]==0xa7);
    } else {
        int before=descriptors();CHECK(before>0);
        uint64_t listener,client,server;CHECK(!pair(&listener,&client,&server));
        lobo_native_slot_v1 *slot=NULL;
        CHECK(!lobo_native_lookup_v1(listener,LN_LISTENER,&slot));CHECK(fcntl(slot->fd,F_GETFD)&FD_CLOEXEC);
        CHECK(!lobo_native_lookup_v1(client,LN_STREAM,&slot));CHECK(fcntl(slot->fd,F_GETFD)&FD_CLOEXEC);
        CHECK(!lobo_native_lookup_v1(server,LN_STREAM,&slot));CHECK(fcntl(slot->fd,F_GETFD)&FD_CLOEXEC);
        if(!strcmp(argv[1],"formal-fault-before-publication")) {
            lobo_native_result_v1 port=lobo_native_port_v1(listener);CHECK(!port.error);
            fault_baseline=before;injection=7;lobo_fuel=0;lobo_native_calls=1;
            (void)lobo_native_connect_v1(lobo_native_root,port.value.integer,1000);return 2;
        } else if(!strcmp(argv[1],"fault-cleanup")) {
            fault_baseline=before;lobo_fuel=0;lobo_tick();return 2;
        } else if(!strcmp(argv[1],"connect-eintr-unstarted")) {
            lobo_native_result_v1 port=lobo_native_port_v1(listener);CHECK(!port.error);
            int count=descriptors();injection=7;
            lobo_native_result_v1 r=lobo_native_connect_v1(lobo_native_root,port.value.integer,1000);
            CHECK(r.error==LN_CLOSED);CHECK(descriptors()==count);
        } else if(!strcmp(argv[1],"bad-handles")) {
            CHECK(lobo_native_read_v1(listener,1,0).error==LN_KIND);
            CHECK(lobo_native_port_v1(client).error==LN_KIND);
            CHECK(lobo_native_close_stream_v1(listener).error==LN_KIND);
            CHECK(lobo_native_random_bytes_v1(server,0).error==LN_KIND);
            CHECK(lobo_native_read_v1(0,1,0).error==LN_HANDLE);
            CHECK(!lobo_native_port_v1(listener).error);
        } else if(!strcmp(argv[1],"poll-eintr-deadline") || !strcmp(argv[1],"poll-ready-deadline")) {
            injection=!strcmp(argv[1],"poll-eintr-deadline")?1:2;
            lobo_native_result_v1 r=lobo_native_read_v1(server,16,10);
            CHECK(r.error==LN_TIMEOUT);
            CHECK(clock_calls<30);
        } else if(!strcmp(argv[1],"recv-eintr-clock-failure")) {
            injection=4;lobo_native_result_v1 r=lobo_native_read_v1(server,16,10);
            CHECK(r.error==LN_OS);CHECK(clock_calls<5);
        } else if(!strcmp(argv[1],"short-io")) {
            injection=3;
            unsigned char data[17];for(unsigned i=0;i<17;++i) data[i]=(unsigned char)(i*17);
            uint64_t sent=0;
            while(sent<17) {
                lobo_native_result_v1 r=lobo_native_write_v1(client,(lobo_buffer){data+sent,17-sent},1000);
                CHECK(!r.error && r.value.integer==1);sent+=(uint64_t)r.value.integer;
            }
            CHECK(!lobo_native_shutdown_write_v1(client).error);
            for(unsigned i=0;i<17;++i) {
                lobo_native_result_v1 r=lobo_native_read_v1(server,17,1000);
                CHECK(!r.error && r.value.buffer.length==1 && r.value.buffer.data[0]==data[i]);
            }
            lobo_native_result_v1 eof=lobo_native_read_v1(server,17,1000);CHECK(!eof.error && !eof.value.buffer.length);
        } else if(!strcmp(argv[1],"quota-cleanup")) {
            /* Fail after opening an accepted fd; the failed insertion must close it. */
            lobo_native_handles=4;
            lobo_native_result_v1 port=lobo_native_port_v1(listener);CHECK(!port.error);
            lobo_native_handles=5;
            lobo_native_result_v1 pending=lobo_native_connect_v1(lobo_native_root,port.value.integer,1000);CHECK(!pending.error);
            lobo_native_handles=5;int count=descriptors();CHECK(count>before);
            CHECK(lobo_native_accept_v1(listener,1000).error==LN_LIMIT);CHECK(descriptors()==count);
        } else if(!strcmp(argv[1],"bind-conflict")) {
            lobo_native_result_v1 port=lobo_native_port_v1(listener);CHECK(!port.error);
            lobo_native_listen_port=(uint16_t)port.value.integer;
            CHECK(lobo_native_listen_v1(lobo_native_root,port.value.integer).error==LN_ADDRESS);
        } else return 2;
        injection=0;lobo_native_cleanup();CHECK(descriptors()==before);lobo_native_cleanup();CHECK(descriptors()==before);
    }
    puts("native-adapter-case-passed");return 0;
}
