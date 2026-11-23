/* Windows x64 clock/random profile. No socket provider is claimed by this profile. */
#if defined(_WIN32) && defined(_WIN64) && !defined(LOBO_WINDOWS_TCP)
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0A00
#endif
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <bcrypt.h>
#ifndef LOBO_WINDOWS_CLOCK
#define LOBO_WINDOWS_CLOCK GetTickCount64
#endif
#ifndef LOBO_WINDOWS_RANDOM
#define LOBO_WINDOWS_RANDOM BCryptGenRandom
#endif

enum { LW_DENIED=1, LW_INPUT=2, LW_HANDLE=3, LW_OS=10, LW_LIMIT=11 };
static unsigned lobo_windows_rights[64];
static uint64_t lobo_windows_live=0, lobo_windows_root=0;
static uint64_t lobo_windows_calls=1024, lobo_windows_bytes=1048576, lobo_windows_handles=64;
static unsigned lobo_windows_options=0, lobo_windows_grants=0;
static lobo_native_result_v1 lobo_windows_error(int64_t error) {
    lobo_native_result_v1 r={0};r.error=error;return r;
}
static bool lobo_windows_number(const char *text,uint64_t maximum,uint64_t *out) {
    if(!*text || (*text=='0' && text[1])) return false;
    uint64_t n=0;
    for(const char *p=text;*p;++p) {
        if(*p<'0'||*p>'9'||n>maximum/10) return false;
        n=n*10+(unsigned)(*p-'0');if(n>maximum) return false;
    }
    *out=n;return true;
}
static bool lobo_native_option(const char *name,const char *value) {
    unsigned bit=0;uint64_t n=0;
    if(!strcmp(name,"--native-clock")) bit=1;
    else if(!strcmp(name,"--native-random")) bit=2;
    else if(!strcmp(name,"--native-handles")) bit=4;
    else if(!strcmp(name,"--native-calls")) bit=8;
    else if(!strcmp(name,"--native-bytes")) bit=16;
    else return false;
    if(lobo_windows_options&bit) return false;
    if(bit==1||bit==2) {
        if(strcmp(value,"true")) return false;
        lobo_windows_grants|=bit;
    } else {
        uint64_t maximum=bit==4?64:bit==8?65536:1048576;
        if(!lobo_windows_number(value,maximum,&n)||(bit!=16&&!n)) return false;
        if(bit==4) lobo_windows_handles=n;else if(bit==8) lobo_windows_calls=n;else lobo_windows_bytes=n;
    }
    lobo_windows_options|=bit;return true;
}
static void lobo_native_cleanup(void) { /* No OS handles: system RNG and clock need no open provider. */ }
static int64_t lobo_windows_insert(unsigned rights,uint64_t *token) {
    if(lobo_windows_live>=lobo_windows_handles) return LW_LIMIT;
    lobo_windows_rights[lobo_windows_live]=rights;
    ++lobo_windows_live;*token=UINT64_C(256)|lobo_windows_live;return 0;
}
static int64_t lobo_windows_begin(uint64_t token,unsigned *rights) {
    uint64_t index=token&255;
    if((token>>8)!=1 || !index || index>lobo_windows_live) return LW_HANDLE;
    if(!lobo_windows_calls) return LW_LIMIT;
    --lobo_windows_calls;*rights=lobo_windows_rights[index-1];return 0;
}
LOBO_NATIVE_NOINLINE uint64_t lobo_native_capability_v1(void) {
    if(lobo_depth!=1) lobo_fail("invocation.native.authority");
    if(lobo_windows_root) return lobo_windows_root;
    if(lobo_windows_insert(lobo_windows_grants,&lobo_windows_root)) lobo_fail("resource.native.exhausted");
    return lobo_windows_root;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_restrict_capability_v1(uint64_t cap,int64_t mask) {
    unsigned rights;int64_t error=lobo_windows_begin(cap,&rights);
    if(error) return lobo_windows_error(error);
    if(mask<0 || mask>15) return lobo_windows_error(LW_INPUT);
    if(((unsigned)mask&rights)!=(unsigned)mask) return lobo_windows_error(LW_DENIED);
    lobo_native_result_v1 r={0};error=lobo_windows_insert((unsigned)mask,&r.value.handle);
    return error?lobo_windows_error(error):r;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_monotonic_ms_v1(uint64_t cap) {
    unsigned rights;int64_t error=lobo_windows_begin(cap,&rights);
    if(error) return lobo_windows_error(error);
    if(!(rights&1)) return lobo_windows_error(LW_DENIED);
    uint64_t now=LOBO_WINDOWS_CLOCK();if(now>INT64_MAX) return lobo_windows_error(LW_OS);
    lobo_native_result_v1 r={0};r.value.integer=(int64_t)now;return r;
}
LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_random_bytes_v1(uint64_t cap,int64_t count) {
    unsigned rights;int64_t error=lobo_windows_begin(cap,&rights);
    if(error) return lobo_windows_error(error);
    if(!(rights&2)) return lobo_windows_error(LW_DENIED);
    if(count<0 || count>65536) return lobo_windows_error(LW_INPUT);
    if((uint64_t)count>lobo_windows_bytes) return lobo_windows_error(LW_LIMIT);
    lobo_spend((uint64_t)count);lobo_windows_bytes-=(uint64_t)count;
    lobo_native_result_v1 r={0};r.value.buffer=lobo_buffer_new((uint64_t)count);
    if(count && LOBO_WINDOWS_RANDOM(NULL,(PUCHAR)r.value.buffer.data,(ULONG)count,BCRYPT_USE_SYSTEM_PREFERRED_RNG)!=0)
        return lobo_windows_error(LW_OS);
    return r;
}
/* Imported library functions may be present in checked code. Unsupported OS calls fail closed. */
#define LW_UNSUPPORTED() lobo_fail("invocation.native.operation_unsupported")
lobo_native_result_v1 lobo_native_listen_v1(uint64_t c,int64_t p) {(void)c;(void)p;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_connect_v1(uint64_t c,int64_t p,int64_t t) {(void)c;(void)p;(void)t;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_accept_v1(uint64_t l,int64_t t) {(void)l;(void)t;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_port_v1(uint64_t l) {(void)l;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_read_v1(uint64_t s,int64_t n,int64_t t) {(void)s;(void)n;(void)t;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_write_v1(uint64_t s,lobo_buffer b,int64_t t) {(void)s;(void)b;(void)t;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_shutdown_write_v1(uint64_t s) {(void)s;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_close_listener_v1(uint64_t l) {(void)l;LW_UNSUPPORTED();}
lobo_native_result_v1 lobo_native_close_stream_v1(uint64_t s) {(void)s;LW_UNSUPPORTED();}
#undef LW_UNSUPPORTED
#endif
