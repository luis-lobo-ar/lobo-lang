/* Windows TCP ABI profile. Policy adapted from the unchanged Linux provider.
   Selected explicitly by the native REST cross-build, never by the snapshot build. */
#if defined(_WIN32) && defined(_WIN64) && defined(LOBO_WINDOWS_TCP)
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0A00
#endif
#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <bcrypt.h>
enum { WT_DENIED=1, WT_INPUT=2, WT_HANDLE=3, WT_KIND=4, WT_CLOSED=5,
       WT_TIMEOUT=6, WT_WOULD_BLOCK=7, WT_REFUSED=8, WT_ADDRESS=9, WT_OS=10, WT_LIMIT=11 };
enum { WT_FREE=0, WT_CAPABILITY=1, WT_LISTENER=2, WT_STREAM=3, WT_RETIRED=4 };
typedef struct { uint64_t generation; SOCKET fd; unsigned kind; uint16_t port; uint64_t owner; unsigned rights; } lobo_windows_tcp_slot_v1;
static lobo_windows_tcp_slot_v1 lobo_native_slots[64];
static uint64_t lobo_native_root=0;
/* A synchronous connect owns this fd until registry publication or explicit close. */
static SOCKET lobo_native_pending_fd=INVALID_SOCKET;
static bool lobo_windows_wsa_started=false;
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
    switch(error) {
        case WSAEACCES: return WT_DENIED;
        case WSAEINVAL: return WT_INPUT;
        case WSAENOTSOCK: return WT_HANDLE;
        case WSAECONNREFUSED: return WT_REFUSED;
        case WSAECONNRESET: case WSAECONNABORTED: case WSAENOTCONN: case WSAESHUTDOWN: return WT_CLOSED;
        case WSAETIMEDOUT: return WT_TIMEOUT;
        case WSAEWOULDBLOCK: return WT_WOULD_BLOCK;
        case WSAEADDRINUSE: return WT_ADDRESS;
        case WSAEMFILE: case WSAENOBUFS: return WT_LIMIT;
        default: return WT_OS;
    }
}
static int64_t lobo_windows_start_v1(void) {
    if(lobo_windows_wsa_started) return 0;
    WSADATA data;int rc=WSAStartup(MAKEWORD(2,2),&data);
    if(rc) return lobo_native_errno_v1(rc);
    lobo_windows_wsa_started=true;return 0;
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
static int64_t lobo_native_lookup_v1(uint64_t token,unsigned kind,lobo_windows_tcp_slot_v1 **out) {
    unsigned index=(unsigned)(token & 255); uint64_t generation=token>>8;
    if(!index || index>64 || !generation) return WT_HANDLE;
    lobo_windows_tcp_slot_v1 *slot=&lobo_native_slots[index-1];
    if(slot->kind==WT_FREE && slot->generation==generation+1) return WT_CLOSED;
    if(slot->generation!=generation || slot->kind==WT_FREE || slot->kind==WT_RETIRED) return WT_HANDLE;
    if(slot->kind!=kind) return WT_KIND;
    *out=slot; return 0;
}
static int64_t lobo_native_begin_v1(uint64_t token,unsigned kind,lobo_windows_tcp_slot_v1 **out) {
    int64_t error=lobo_native_lookup_v1(token,kind,out); if(error) return error;
    if(!lobo_native_calls) return WT_LIMIT;
    --lobo_native_calls; return 0;
}
static int64_t lobo_native_insert_v1(unsigned kind,SOCKET fd,uint16_t port,uint64_t owner,uint64_t *token) {
    uint64_t live=0;
    for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind && lobo_native_slots[i].kind!=WT_RETIRED) ++live;
    if(live>=lobo_native_handles) return WT_LIMIT;
    for(unsigned i=0;i<64;++i) {
        lobo_windows_tcp_slot_v1 *slot=&lobo_native_slots[i]; if(slot->kind!=WT_FREE) continue;
        if(!slot->generation) slot->generation=1;
        slot->kind=kind;slot->fd=fd;slot->port=port;slot->owner=owner;slot->rights=0;
        *token=(slot->generation<<8)|(uint64_t)(i+1);return 0;
    }
    return WT_LIMIT;
}
static void lobo_native_release_v1(lobo_windows_tcp_slot_v1 *slot) {
    slot->fd=INVALID_SOCKET;slot->port=0;slot->owner=0;slot->rights=0;
    if(slot->generation==UINT64_C(0x00ffffffffffffff)) slot->kind=WT_RETIRED;
    else {++slot->generation;slot->kind=WT_FREE;}
}
static void lobo_native_discard_fd_v1(SOCKET fd) {
    if(lobo_native_pending_fd==fd) lobo_native_pending_fd=INVALID_SOCKET;
    (void)closesocket(fd);
}
static void lobo_native_cleanup(void) {
    if(lobo_native_pending_fd!=INVALID_SOCKET) lobo_native_discard_fd_v1(lobo_native_pending_fd);
    for(unsigned i=0;i<64;++i) {
        lobo_windows_tcp_slot_v1 *slot=&lobo_native_slots[i];
        if(slot->kind==WT_LISTENER || slot->kind==WT_STREAM) { (void)closesocket(slot->fd);lobo_native_release_v1(slot); }
    }
    if(lobo_windows_wsa_started) {WSACleanup();lobo_windows_wsa_started=false;}
}
LOBO_NATIVE_NOINLINE uint64_t lobo_native_capability_v1(void) {
    if(lobo_depth!=1) lobo_fail("invocation.native.authority");
    if(lobo_native_root) return lobo_native_root;
    if(lobo_native_insert_v1(WT_CAPABILITY,INVALID_SOCKET,0,0,&lobo_native_root)) lobo_fail("resource.native.exhausted");
    lobo_native_slots[(lobo_native_root&255)-1].rights=(lobo_native_clock?1:0)|(lobo_native_random?2:0)|(lobo_native_listen?4:0)|(lobo_native_connect?8:0);
    return lobo_native_root;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_restrict_capability_v1(uint64_t cap,int64_t mask) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,WT_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(mask<0 || mask>15) return lobo_native_error_v1(WT_INPUT);
    if(((unsigned)mask & slot->rights)!=(unsigned)mask) return lobo_native_error_v1(WT_DENIED);
    lobo_native_result_v1 result={0};
    error=lobo_native_insert_v1(WT_CAPABILITY,INVALID_SOCKET,0,cap,&result.value.handle);
    if(error) return lobo_native_error_v1(error);
    lobo_native_slots[(result.value.handle&255)-1].rights=(unsigned)mask;
    return result;
}
static int64_t lobo_native_now_v1(void) {
    uint64_t now=GetTickCount64();return now>INT64_MAX-2000?-1:(int64_t)now;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_monotonic_ms_v1(uint64_t cap) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,WT_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(!(slot->rights&1)) return lobo_native_error_v1(WT_DENIED);
    int64_t now=lobo_native_now_v1();return now<0?lobo_native_error_v1(WT_OS):lobo_native_integer_v1(now);
}
static int64_t lobo_native_reserve_v1(uint64_t bytes) {
    if(bytes>65536) return WT_INPUT;
    if(bytes>lobo_native_bytes) return WT_LIMIT;
    lobo_spend(bytes);lobo_native_bytes-=bytes;return 0;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_random_bytes_v1(uint64_t cap,int64_t count) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,WT_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(!(slot->rights&2)) return lobo_native_error_v1(WT_DENIED);
    if(count<0 || count>65536) return lobo_native_error_v1(WT_INPUT);
    if((error=lobo_native_reserve_v1((uint64_t)count))) return lobo_native_error_v1(error);
    lobo_native_result_v1 result={0};result.value.buffer=lobo_buffer_new((uint64_t)count);
    if(count && BCryptGenRandom(NULL,(PUCHAR)result.value.buffer.data,(ULONG)count,BCRYPT_USE_SYSTEM_PREFERRED_RNG)!=0)
        return lobo_native_error_v1(WT_OS);
    return result;
}
static int64_t lobo_native_fd_flags_v1(SOCKET fd) {
    u_long mode=1;
    if(ioctlsocket(fd,FIONBIO,&mode)) return lobo_native_errno_v1(WSAGetLastError());
    if(!SetHandleInformation((HANDLE)fd,HANDLE_FLAG_INHERIT,0)) return WT_OS;
    return 0;
}
static int64_t lobo_native_deadline_v1(int64_t timeout) {
    if(timeout<0 || timeout>2000) return -1;
    int64_t now=lobo_native_now_v1();return now<0?-1:now+timeout;
}
static int64_t lobo_native_wait_v1(SOCKET fd,short events,int64_t deadline) {
    for(;;) {
        lobo_spend(1);
        int64_t now=lobo_native_now_v1();if(now<0) return WT_OS;
        if(now>=deadline) return WT_TIMEOUT;
        int64_t remaining=deadline-now;
        WSAPOLLFD p={fd,events,0};int rc=WSAPoll(&p,1,(int)remaining);
        if(rc<0) {if(WSAGetLastError()==WSAEINTR) continue;return lobo_native_errno_v1(WSAGetLastError());}
        if(!rc) return WT_TIMEOUT;
        if(p.revents&POLLNVAL) return WT_HANDLE;
        if(p.revents&(events|POLLERR|POLLHUP)) return 0;
    }
}
static bool lobo_native_self_port_v1(uint64_t cap,uint16_t port) {
    for(unsigned i=0;i<64;++i) if(lobo_native_slots[i].kind==WT_LISTENER && lobo_native_slots[i].port==port && lobo_native_slots[i].owner==cap) return true;
    return false;
}
static lobo_native_result_v1 lobo_native_socket_result_v1(unsigned kind,SOCKET fd,uint16_t port,uint64_t owner) {
    lobo_native_result_v1 result={0};int64_t error=lobo_native_insert_v1(kind,fd,port,owner,&result.value.handle);
    if(error) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    if(lobo_native_pending_fd==fd) lobo_native_pending_fd=INVALID_SOCKET;
    return result;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_listen_v1(uint64_t cap,int64_t port) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,WT_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(port<0 || port>65535) return lobo_native_error_v1(WT_INPUT);
    if(!(slot->rights&4) || !lobo_native_listen || (uint16_t)port!=lobo_native_listen_port) return lobo_native_error_v1(WT_DENIED);
    if((error=lobo_windows_start_v1())) return lobo_native_error_v1(error);
    SOCKET fd=WSASocketW(AF_INET,SOCK_STREAM,IPPROTO_TCP,NULL,0,WSA_FLAG_NO_HANDLE_INHERIT);if(fd==INVALID_SOCKET) return lobo_native_error_v1(lobo_native_errno_v1(WSAGetLastError()));
    if((error=lobo_native_fd_flags_v1(fd))) {(void)closesocket(fd);return lobo_native_error_v1(error);}
    struct sockaddr_in addr={0};addr.sin_family=AF_INET;addr.sin_addr.s_addr=htonl(INADDR_LOOPBACK);addr.sin_port=htons((uint16_t)port);
    BOOL exclusive=TRUE;
    if(setsockopt(fd,SOL_SOCKET,SO_EXCLUSIVEADDRUSE,(const char *)&exclusive,sizeof(exclusive))) {error=lobo_native_errno_v1(WSAGetLastError());(void)closesocket(fd);return lobo_native_error_v1(error);}
    if(bind(fd,(struct sockaddr *)&addr,sizeof(addr)) || listen(fd,16)) {error=lobo_native_errno_v1(WSAGetLastError());(void)closesocket(fd);return lobo_native_error_v1(error);}
    int length=sizeof(addr);
    if(getsockname(fd,(struct sockaddr *)&addr,&length) || length!=sizeof(addr)) {(void)closesocket(fd);return lobo_native_error_v1(WT_OS);}
    return lobo_native_socket_result_v1(WT_LISTENER,fd,ntohs(addr.sin_port),cap);
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_connect_v1(uint64_t cap,int64_t port,int64_t timeout) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(cap,WT_CAPABILITY,&slot);
    if(error) return lobo_native_error_v1(error);
    if(port<1 || port>65535 || timeout<0 || timeout>2000) return lobo_native_error_v1(WT_INPUT);
    if(!(slot->rights&8) || !lobo_native_connect || (lobo_native_self ? !lobo_native_self_port_v1(cap,(uint16_t)port) : (uint16_t)port!=lobo_native_connect_port)) return lobo_native_error_v1(WT_DENIED);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(WT_OS);
    if((error=lobo_windows_start_v1())) return lobo_native_error_v1(error);
    SOCKET fd=WSASocketW(AF_INET,SOCK_STREAM,IPPROTO_TCP,NULL,0,WSA_FLAG_NO_HANDLE_INHERIT);if(fd==INVALID_SOCKET) return lobo_native_error_v1(lobo_native_errno_v1(WSAGetLastError()));
    lobo_native_pending_fd=fd;
    if((error=lobo_native_fd_flags_v1(fd))) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    struct sockaddr_in addr={0};addr.sin_family=AF_INET;addr.sin_addr.s_addr=htonl(INADDR_LOOPBACK);addr.sin_port=htons((uint16_t)port);
    int rc=connect(fd,(struct sockaddr *)&addr,sizeof(addr));
    if(rc<0 && WSAGetLastError()!=WSAEWOULDBLOCK && WSAGetLastError()!=WSAEINTR) {error=lobo_native_errno_v1(WSAGetLastError());lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    if(rc<0) {
        error=lobo_native_wait_v1(fd,POLLOUT,deadline);
        int status=0;int size=sizeof(status);
        if(!error && getsockopt(fd,SOL_SOCKET,SO_ERROR,(char *)&status,&size)<0) error=lobo_native_errno_v1(WSAGetLastError());
        if(!error && status) error=lobo_native_errno_v1(status);
        if(error) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    }
    struct sockaddr_in peer={0};int peer_size=sizeof(peer);
    if(getpeername(fd,(struct sockaddr *)&peer,&peer_size)<0) {error=lobo_native_errno_v1(WSAGetLastError());lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(error);}
    if(peer_size!=sizeof(peer) || peer.sin_family!=AF_INET || peer.sin_addr.s_addr!=htonl(INADDR_LOOPBACK) || peer.sin_port!=htons((uint16_t)port)) {lobo_native_discard_fd_v1(fd);return lobo_native_error_v1(WT_OS);}
    return lobo_native_socket_result_v1(WT_STREAM,fd,(uint16_t)port,cap);
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_accept_v1(uint64_t token,int64_t timeout) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,WT_LISTENER,&slot);
    if(error) return lobo_native_error_v1(error);
    if(timeout<0 || timeout>2000) return lobo_native_error_v1(WT_INPUT);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(WT_OS);
    for(;;) {
        lobo_spend(1);SOCKET fd=accept(slot->fd,NULL,NULL);
        if(fd!=INVALID_SOCKET) {
            if((error=lobo_native_fd_flags_v1(fd))) {(void)closesocket(fd);return lobo_native_error_v1(error);}
            return lobo_native_socket_result_v1(WT_STREAM,fd,slot->port,slot->owner);
        }
        if(WSAGetLastError()==WSAEINTR) {int64_t now=lobo_native_now_v1();if(now<0) return lobo_native_error_v1(WT_OS);if(now>=deadline) return lobo_native_error_v1(WT_TIMEOUT);continue;}
        if(WSAGetLastError()!=WSAEWOULDBLOCK) return lobo_native_error_v1(lobo_native_errno_v1(WSAGetLastError()));
        if((error=lobo_native_wait_v1(slot->fd,POLLIN,deadline))) return lobo_native_error_v1(error);
    }
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_port_v1(uint64_t token) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,WT_LISTENER,&slot);
    return error?lobo_native_error_v1(error):lobo_native_integer_v1(slot->port);
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_read_v1(uint64_t token,int64_t count,int64_t timeout) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,WT_STREAM,&slot);
    if(error) return lobo_native_error_v1(error);
    if(count<0 || count>65536 || timeout<0 || timeout>2000) return lobo_native_error_v1(WT_INPUT);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(WT_OS);
    if((error=lobo_native_reserve_v1((uint64_t)count))) return lobo_native_error_v1(error);
    lobo_native_result_v1 result={0};result.value.buffer=lobo_buffer_new((uint64_t)count);
    if(!count) return result;
    for(;;) {
        lobo_spend(1);int n=recv(slot->fd,(char *)result.value.buffer.data,(int)count,0);
        if(n>=0) {result.value.buffer.length=(uint64_t)n;return result;}
        if(WSAGetLastError()==WSAEINTR) {int64_t now=lobo_native_now_v1();if(now<0) return lobo_native_error_v1(WT_OS);if(now>=deadline) return lobo_native_error_v1(WT_TIMEOUT);continue;}
        if(WSAGetLastError()!=WSAEWOULDBLOCK) return lobo_native_error_v1(lobo_native_errno_v1(WSAGetLastError()));
        if((error=lobo_native_wait_v1(slot->fd,POLLIN,deadline))) return lobo_native_error_v1(error);
    }
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_write_v1(uint64_t token,lobo_buffer data,int64_t timeout) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,WT_STREAM,&slot);
    if(error) return lobo_native_error_v1(error);
    if(data.length>65536 || (data.length && !data.data) || timeout<0 || timeout>2000) return lobo_native_error_v1(WT_INPUT);
    int64_t deadline=lobo_native_deadline_v1(timeout);if(deadline<0) return lobo_native_error_v1(WT_OS);
    if((error=lobo_native_reserve_v1(data.length))) return lobo_native_error_v1(error);
    if(!data.length) return lobo_native_integer_v1(0);
    for(;;) {
        lobo_spend(1);int n=send(slot->fd,(const char *)data.data,(int)data.length,0);
        if(n>=0) return lobo_native_integer_v1(n);
        if(WSAGetLastError()==WSAEINTR) {int64_t now=lobo_native_now_v1();if(now<0) return lobo_native_error_v1(WT_OS);if(now>=deadline) return lobo_native_error_v1(WT_TIMEOUT);continue;}
        if(WSAGetLastError()!=WSAEWOULDBLOCK) return lobo_native_error_v1(lobo_native_errno_v1(WSAGetLastError()));
        if((error=lobo_native_wait_v1(slot->fd,POLLOUT,deadline))) return lobo_native_error_v1(error);
    }
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_shutdown_write_v1(uint64_t token) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,WT_STREAM,&slot);
    if(error) return lobo_native_error_v1(error);
    if(shutdown(slot->fd,SD_SEND)) return lobo_native_error_v1(lobo_native_errno_v1(WSAGetLastError()));
    lobo_native_result_v1 result={0};result.value.boolean=true;return result;
}
static lobo_native_result_v1 lobo_native_close_v1(uint64_t token,unsigned kind) {
    lobo_windows_tcp_slot_v1 *slot;int64_t error=lobo_native_begin_v1(token,kind,&slot);
    if(error) return lobo_native_error_v1(error);
    int rc=closesocket(slot->fd);int saved=WSAGetLastError();
    if(rc) return lobo_native_error_v1(lobo_native_errno_v1(saved));
    lobo_native_release_v1(slot);
    lobo_native_result_v1 result={0};result.value.boolean=true;return result;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_close_listener_v1(uint64_t token) {return lobo_native_close_v1(token,WT_LISTENER);}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_close_stream_v1(uint64_t token) {return lobo_native_close_v1(token,WT_STREAM);}
#endif
