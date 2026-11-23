#ifndef LOBO_NATIVE_ABI_V1_H
#error "Native adapter requires the ABI header"
#endif
/* Versioned, closed native ABI. Inputs are borrowed; outputs belong to the invocation arena. */
#if defined(__clang__) || defined(__GNUC__)
#define LOBO_NATIVE_NOINLINE __attribute__((noinline))
#else
#define LOBO_NATIVE_NOINLINE
#endif
LOBO_NATIVE_NOINLINE int64_t lobo_native_abi_version_v1(void) { return 1; }
LOBO_NATIVE_NOINLINE int64_t lobo_native_roundtrip_i64_v1(int64_t value) { return value; }
LOBO_NATIVE_NOINLINE bool lobo_native_roundtrip_bool_v1(bool value) { return value; }
LOBO_NATIVE_NOINLINE lobo_buffer lobo_native_copy_bytes_v1(lobo_buffer value) {
    if(value.length && !value.data) lobo_fail("invocation.native.buffer");
    lobo_spend(value.length);
    lobo_buffer copy = lobo_buffer_new(value.length);
    if (value.length) memcpy((void *)copy.data, value.data, (size_t)value.length);
    return copy;
}

/* Linux adapter v1. These handles are internal process tokens, never wire values. */
#if defined(__linux__)
#include <unistd.h>
#include <fcntl.h>
#include <poll.h>
#include <time.h>
#include <sys/socket.h>
#include <sys/random.h>
#include <arpa/inet.h>

enum { LN_DENIED=1, LN_INPUT=2, LN_HANDLE=3, LN_KIND=4, LN_CLOSED=5,
       LN_TIMEOUT=6, LN_WOULD_BLOCK=7, LN_REFUSED=8, LN_ADDRESS=9, LN_OS=10, LN_LIMIT=11 };
enum { LN_FREE=0, LN_CAPABILITY=1, LN_LISTENER=2, LN_STREAM=3, LN_RETIRED=4 };
typedef struct { uint64_t generation; int fd; unsigned kind; uint16_t port; uint64_t owner; unsigned rights; } lobo_native_slot_v1;
static lobo_native_slot_v1 lobo_native_slots[64];
static uint64_t lobo_native_root=0;
/* A synchronous connect owns this fd until registry publication or explicit close. */
static int lobo_native_pending_fd=-1;
static uint64_t lobo_native_calls=1024, lobo_native_bytes=1048576, lobo_native_handles=64;
static bool lobo_native_clock=false, lobo_native_random=false, lobo_native_listen=false, lobo_native_connect=false, lobo_native_self=false;
static uint16_t lobo_native_listen_port=0, lobo_native_connect_port=0;
static unsigned lobo_native_options=0;
static lobo_native_result_v1 lobo_native_error_v1(int64_t error) {
    lobo_native_result_v1 r={0}; r.error=error; return r;
}
static lobo_native_result_v1 lobo_native_integer_v1(int64_t value) {
    lobo_native_result_v1 r={0}; r.value.integer=value; return r;
}
static int64_t lobo_native_errno_v1(int error) {
    switch (error) {
        case EACCES: case EPERM: return LN_DENIED;
        case EINVAL: return LN_INPUT;
        case ECONNREFUSED: return LN_REFUSED;
        case EADDRINUSE: return LN_ADDRESS;
        case EPIPE: case ECONNRESET: case ENOTCONN: return LN_CLOSED;
        case ETIMEDOUT: return LN_TIMEOUT;
        case EAGAIN: return LN_WOULD_BLOCK;
        case EMFILE: case ENFILE: case ENOMEM: case ENOBUFS: return LN_LIMIT;
        default: return LN_OS;
    }
}
static bool lobo_native_number_v1(const char *text, uint64_t maximum, uint64_t *out) {
    if (!*text || (*text=='0' && text[1])) return false;
    uint64_t n=0;
    for (const char *p=text; *p; ++p) {
        if (*p<'0'||*p>'9'||n>maximum/10) return false;
        n=n*10+(unsigned)(*p-'0'); if(n>maximum) return false;
    }
    *out=n; return true;
}
static bool lobo_native_option(const char *name,const char *value) {
    unsigned bit=0; uint64_t n=0;
    if(!strcmp(name,"--native-clock")) bit=1;
    else if(!strcmp(name,"--native-random")) bit=2;
    else if(!strcmp(name,"--native-listen")) bit=4;
    else if(!strcmp(name,"--native-connect")) bit=8;
    else if(!strcmp(name,"--native-handles")) bit=16;
    else if(!strcmp(name,"--native-bytes")) bit=32;
    else if(!strcmp(name,"--native-calls")) bit=64;
    else return false;
    if(lobo_native_options & bit) return false;
    if(bit==1 || bit==2) {
        if(strcmp(value,"true")) return false;
        if(bit==1) lobo_native_clock=true; else lobo_native_random=true;
    } else if(bit==4 || bit==8) {
        if(bit==8 && !strcmp(value,"self")) lobo_native_self=true;
        else {
            if(strncmp(value,"127.0.0.1:",10) || !lobo_native_number_v1(value+10,65535,&n) || (bit==8 && !n)) return false;
            if(bit==4) lobo_native_listen_port=(uint16_t)n; else lobo_native_connect_port=(uint16_t)n;
        }
        if(bit==4) lobo_native_listen=true; else lobo_native_connect=true;
    } else {
        uint64_t maximum=bit==16?64:bit==32?1048576:65536;
        if(!lobo_native_number_v1(value,maximum,&n) || (bit!=32 && !n)) return false;
        if(bit==16) lobo_native_handles=n; else if(bit==32) lobo_native_bytes=n; else lobo_native_calls=n;
    }
    lobo_native_options|=bit; return true;
}
static int64_t lobo_native_lookup_v1(uint64_t token,unsigned kind,lobo_native_slot_v1 **out) {
    unsigned index=(unsigned)(token & 255); uint64_t generation=token>>8;
    if(!index || index>64 || !generation) return LN_HANDLE;
    lobo_native_slot_v1 *slot=&lobo_native_slots[index-1];
    if(slot->kind==LN_FREE && slot->generation==generation+1) return LN_CLOSED;
    if(slot->generation!=generation || slot->kind==LN_FREE || slot->kind==LN_RETIRED) return LN_HANDLE;
    if(slot->kind!=kind) return LN_KIND;
    *out=slot; return 0;
}
static int64_t lobo_native_begin_v1(uint64_t token,unsigned kind,lobo_native_slot_v1 **out) {
    int64_t error=lobo_native_lookup_v1(token,kind,out); if(error) return error;
    if(!lobo_native_calls) return LN_LIMIT;
    --lobo_native_calls; return 0;
}
static int64_t lobo_native_insert_v1(unsigned kind,int fd,uint16_t port,uint64_t owner,uint64_t *token) {
    uint64_t live=0;
    for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind && lobo_native_slots[i].kind!=LN_RETIRED) ++live;
    if(live>=lobo_native_handles) return LN_LIMIT;
    for(unsigned i=0;i<64;++i) {
        lobo_native_slot_v1 *slot=&lobo_native_slots[i]; if(slot->kind!=LN_FREE) continue;
        if(!slot->generation) slot->generation=1;
        slot->kind=kind;slot->fd=fd;slot->port=port;slot->owner=owner;slot->rights=0;
        *token=(slot->generation<<8)|(uint64_t)(i+1);return 0;
    }
    return LN_LIMIT;
}
static void lobo_native_release_v1(lobo_native_slot_v1 *slot) {
    slot->fd=-1;slot->port=0;slot->owner=0;slot->rights=0;
    if(slot->generation==UINT64_C(0x00ffffffffffffff)) slot->kind=LN_RETIRED;
    else {++slot->generation;slot->kind=LN_FREE;}
}
static void lobo_native_discard_fd_v1(int fd) {
    if(lobo_native_pending_fd==fd) lobo_native_pending_fd=-1;
    (void)close(fd);
}
static void lobo_native_cleanup(void) {
    if(lobo_native_pending_fd>=0) lobo_native_discard_fd_v1(lobo_native_pending_fd);
    for(unsigned i=0;i<64;++i) {
        lobo_native_slot_v1 *slot=&lobo_native_slots[i];
        if(slot->kind==LN_LISTENER || slot->kind==LN_STREAM) { (void)close(slot->fd);lobo_native_release_v1(slot); }
    }
}
LOBO_NATIVE_NOINLINE uint64_t lobo_native_capability_v1(void) {
    if(lobo_depth!=1) lobo_fail("invocation.native.authority");
    if(lobo_native_root) return lobo_native_root;
    if(lobo_native_insert_v1(LN_CAPABILITY,-1,0,0,&lobo_native_root)) lobo_fail("resource.native.exhausted");
    lobo_native_slots[(lobo_native_root&255)-1].rights=(lobo_native_clock?1:0)|(lobo_native_random?2:0)|(lobo_native_listen?4:0)|(lobo_native_connect?8:0);
    return lobo_native_root;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_restrict_capability_v1(uint64_t cap,int64_t mask) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,LN_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(mask<0 || mask>15) return lobo_native_error_v1(LN_INPUT);
    if(((unsigned)mask & slot->rights)!=(unsigned)mask) return lobo_native_error_v1(LN_DENIED);
    lobo_native_result_v1 result={0};
    error=lobo_native_insert_v1(LN_CAPABILITY,-1,0,cap,&result.value.handle);
    if(error) return lobo_native_error_v1(error);
    lobo_native_slots[(result.value.handle&255)-1].rights=(unsigned)mask;
    return result;
}
static int64_t lobo_native_now_v1(void) {
    struct timespec time;
    if(clock_gettime(CLOCK_MONOTONIC,&time) || time.tv_sec<0 || time.tv_sec>INT64_MAX/1000-1) return -1;
    return (int64_t)time.tv_sec*1000+time.tv_nsec/1000000;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_monotonic_ms_v1(uint64_t cap) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,LN_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(!(slot->rights&1)) return lobo_native_error_v1(LN_DENIED);
    int64_t now=lobo_native_now_v1();return now<0?lobo_native_error_v1(LN_OS):lobo_native_integer_v1(now);
}
static int64_t lobo_native_reserve_v1(uint64_t bytes) {
    if(bytes>65536) return LN_INPUT;
    if(bytes>lobo_native_bytes) return LN_LIMIT;
    lobo_spend(bytes);lobo_native_bytes-=bytes;return 0;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_random_bytes_v1(uint64_t cap,int64_t count) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,LN_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(!(slot->rights&2)) return lobo_native_error_v1(LN_DENIED);
    if(count<0 || count>65536) return lobo_native_error_v1(LN_INPUT);
    if((error=lobo_native_reserve_v1((uint64_t)count))) return lobo_native_error_v1(error);
    lobo_native_result_v1 result={0};result.value.buffer=lobo_buffer_new((uint64_t)count);
    uint64_t offset=0;
    while(offset<(uint64_t)count) {
        lobo_spend(1);
        ssize_t n=getrandom((void *)(result.value.buffer.data+offset),(size_t)((uint64_t)count-offset),GRND_NONBLOCK);
        if(n<0) {if(errno==EINTR) continue;return lobo_native_error_v1(lobo_native_errno_v1(errno));}
        if(!n) return lobo_native_error_v1(LN_OS);
        offset+=(uint64_t)n;
    }
    return result;
}
static int64_t lobo_native_fd_flags_v1(int fd) {
    int flags=fcntl(fd,F_GETFL,0);if(flags<0 || fcntl(fd,F_SETFL,flags|O_NONBLOCK)<0 || fcntl(fd,F_SETFD,FD_CLOEXEC)<0) return lobo_native_errno_v1(errno);
    return 0;
}
static int64_t lobo_native_deadline_v1(int64_t timeout) {
    if(timeout<0 || timeout>2000) return -1;
    int64_t now=lobo_native_now_v1();return now<0?-1:now+timeout;
}
static int64_t lobo_native_wait_v1(int fd,short events,int64_t deadline) {
    for(;;) {
        lobo_spend(1);
        int64_t now=lobo_native_now_v1();if(now<0) return LN_OS;
        if(now>=deadline) return LN_TIMEOUT;
        int64_t remaining=deadline-now;
        struct pollfd p={fd,events,0};int rc=poll(&p,1,(int)remaining);
        if(rc<0) {if(errno==EINTR) continue;return lobo_native_errno_v1(errno);}
        if(!rc) return LN_TIMEOUT;
        if(p.revents&POLLNVAL) return LN_HANDLE;
        if(p.revents&(events|POLLERR|POLLHUP)) return 0;
    }
}
static bool lobo_native_self_port_v1(uint64_t cap,uint16_t port) {
    for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind==LN_LISTENER && lobo_native_slots[i].port==port && lobo_native_slots[i].owner==cap) return true;
    return false;
}
static lobo_native_result_v1 lobo_native_socket_result_v1(unsigned kind,int fd,uint16_t port,uint64_t owner) {
    lobo_native_result_v1 result={0};int64_t error=lobo_native_insert_v1(kind,fd,port,owner,&result.value.handle);
    if(error) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    if(lobo_native_pending_fd==fd) lobo_native_pending_fd=-1;
    return result;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_listen_v1(uint64_t cap,int64_t port) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,LN_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(port<0 || port>65535) return lobo_native_error_v1(LN_INPUT);
    if(!(slot->rights&4) || !lobo_native_listen || (uint16_t)port!=lobo_native_listen_port) return lobo_native_error_v1(LN_DENIED);
    int fd=socket(AF_INET,SOCK_STREAM,0);if(fd<0) return lobo_native_error_v1(lobo_native_errno_v1(errno));
    if((error=lobo_native_fd_flags_v1(fd))) {(void)close(fd);return lobo_native_error_v1(error);}
    struct sockaddr_in addr={0};addr.sin_family=AF_INET;addr.sin_addr.s_addr=htonl(INADDR_LOOPBACK);addr.sin_port=htons((uint16_t)port);
    if(bind(fd,(struct sockaddr *)&addr,sizeof(addr)) || listen(fd,16)) {error=lobo_native_errno_v1(errno);(void)close(fd);return lobo_native_error_v1(error);}
    socklen_t length=sizeof(addr);
    if(getsockname(fd,(struct sockaddr *)&addr,&length) || length!=sizeof(addr)) {(void)close(fd);return lobo_native_error_v1(LN_OS);}
    return lobo_native_socket_result_v1(LN_LISTENER,fd,ntohs(addr.sin_port),cap);
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_connect_v1(uint64_t cap,int64_t port,int64_t timeout) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,LN_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(port<1 || port>65535 || timeout<0 || timeout>2000) return lobo_native_error_v1(LN_INPUT);
    if(!(slot->rights&8) || !lobo_native_connect || (lobo_native_self ? !lobo_native_self_port_v1(cap,(uint16_t)port) : (uint16_t)port!=lobo_native_connect_port)) return lobo_native_error_v1(LN_DENIED);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(LN_OS);
    int fd=socket(AF_INET,SOCK_STREAM,0);if(fd<0) return lobo_native_error_v1(lobo_native_errno_v1(errno));
    lobo_native_pending_fd=fd;
    if((error=lobo_native_fd_flags_v1(fd))) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    struct sockaddr_in addr={0};addr.sin_family=AF_INET;addr.sin_addr.s_addr=htonl(INADDR_LOOPBACK);addr.sin_port=htons((uint16_t)port);
    int rc=connect(fd,(struct sockaddr *)&addr,sizeof(addr));
    if(rc<0 && errno!=EINPROGRESS && errno!=EINTR) {error=lobo_native_errno_v1(errno);lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    if(rc<0) {
        error=lobo_native_wait_v1(fd,POLLOUT,deadline);
        int status=0;socklen_t size=sizeof(status);
        if(!error && getsockopt(fd,SOL_SOCKET,SO_ERROR,&status,&size)<0) error=lobo_native_errno_v1(errno);
        if(!error && status) error=lobo_native_errno_v1(status);
        if(error) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    }
    struct sockaddr_in peer={0};socklen_t peer_size=sizeof(peer);
    if(getpeername(fd,(struct sockaddr *)&peer,&peer_size)<0) {error=lobo_native_errno_v1(errno);lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    if(peer_size!=sizeof(peer) || peer.sin_family!=AF_INET || peer.sin_addr.s_addr!=htonl(INADDR_LOOPBACK) || peer.sin_port!=htons((uint16_t)port)) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(LN_OS);}
    return lobo_native_socket_result_v1(LN_STREAM,fd,(uint16_t)port,cap);
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_accept_v1(uint64_t token,int64_t timeout) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,LN_LISTENER,&slot);
    if(error) return lobo_native_error_v1(error);
    if(timeout<0 || timeout>2000) return lobo_native_error_v1(LN_INPUT);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(LN_OS);
    for(;;) {
        lobo_spend(1);int fd=accept(slot->fd,NULL,NULL);
        if(fd>=0) {
            if((error=lobo_native_fd_flags_v1(fd))) {(void)close(fd);return lobo_native_error_v1(error);}
            return lobo_native_socket_result_v1(LN_STREAM,fd,slot->port,slot->owner);
        }
        if(errno==EINTR) {int64_t now=lobo_native_now_v1();if(now<0) return lobo_native_error_v1(LN_OS);if(now>=deadline) return lobo_native_error_v1(LN_TIMEOUT);continue;}
        if(errno!=EAGAIN && errno!=EWOULDBLOCK) return lobo_native_error_v1(lobo_native_errno_v1(errno));
        if((error=lobo_native_wait_v1(slot->fd,POLLIN,deadline))) return lobo_native_error_v1(error);
    }
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_port_v1(uint64_t token) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,LN_LISTENER,&slot);
    return error?lobo_native_error_v1(error):lobo_native_integer_v1(slot->port);
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_read_v1(uint64_t token,int64_t count,int64_t timeout) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,LN_STREAM,&slot);
    if(error) return lobo_native_error_v1(error);
    if(count<0 || count>65536 || timeout<0 || timeout>2000) return lobo_native_error_v1(LN_INPUT);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(LN_OS);
    if((error=lobo_native_reserve_v1((uint64_t)count))) return lobo_native_error_v1(error);
    lobo_native_result_v1 result={0};result.value.buffer=lobo_buffer_new((uint64_t)count);
    if(!count) return result;
    for(;;) {
        lobo_spend(1);ssize_t n=recv(slot->fd,(void *)result.value.buffer.data,(size_t)count,0);
        if(n>=0) {result.value.buffer.length=(uint64_t)n;return result;}
        if(errno==EINTR) {int64_t now=lobo_native_now_v1();if(now<0) return lobo_native_error_v1(LN_OS);if(now>=deadline) return lobo_native_error_v1(LN_TIMEOUT);continue;}
        if(errno!=EAGAIN && errno!=EWOULDBLOCK) return lobo_native_error_v1(lobo_native_errno_v1(errno));
        if((error=lobo_native_wait_v1(slot->fd,POLLIN,deadline))) return lobo_native_error_v1(error);
    }
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_write_v1(uint64_t token,lobo_buffer data,int64_t timeout) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,LN_STREAM,&slot);
    if(error) return lobo_native_error_v1(error);
    if(data.length>65536 || (data.length && !data.data) || timeout<0 || timeout>2000) return lobo_native_error_v1(LN_INPUT);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(LN_OS);
    if((error=lobo_native_reserve_v1(data.length))) return lobo_native_error_v1(error);
    if(!data.length) return lobo_native_integer_v1(0);
    for(;;) {
        lobo_spend(1);ssize_t n=send(slot->fd,data.data,(size_t)data.length,MSG_NOSIGNAL);
        if(n>=0) return lobo_native_integer_v1(n);
        if(errno==EINTR) {int64_t now=lobo_native_now_v1();if(now<0) return lobo_native_error_v1(LN_OS);if(now>=deadline) return lobo_native_error_v1(LN_TIMEOUT);continue;}
        if(errno!=EAGAIN && errno!=EWOULDBLOCK) return lobo_native_error_v1(lobo_native_errno_v1(errno));
        if((error=lobo_native_wait_v1(slot->fd,POLLOUT,deadline))) return lobo_native_error_v1(error);
    }
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_shutdown_write_v1(uint64_t token) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,LN_STREAM,&slot);
    if(error) return lobo_native_error_v1(error);
    if(shutdown(slot->fd,SHUT_WR)) return lobo_native_error_v1(lobo_native_errno_v1(errno));
    lobo_native_result_v1 result={0};result.value.boolean=true;return result;
}
static lobo_native_result_v1 lobo_native_close_v1(uint64_t token,unsigned kind) {
    lobo_native_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,kind,&slot);
    if(error) return lobo_native_error_v1(error);
    int rc=close(slot->fd);int saved=errno;lobo_native_release_v1(slot);
    if(rc && saved!=EINTR) return lobo_native_error_v1(lobo_native_errno_v1(saved));
    lobo_native_result_v1 result={0};result.value.boolean=true;return result;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_close_listener_v1(uint64_t token) {return lobo_native_close_v1(token,LN_LISTENER);}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_close_stream_v1(uint64_t token) {return lobo_native_close_v1(token,LN_STREAM);}
#elif !defined(_WIN32) || !defined(_WIN64)
static bool lobo_native_option(const char *name,const char *value) {(void)name;(void)value;return false;}
static void lobo_native_cleanup(void) {}
#endif
