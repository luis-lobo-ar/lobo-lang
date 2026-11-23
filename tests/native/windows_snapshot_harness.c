#define LOBO_NATIVE_ENABLED 1
#include "native_runtime.c"
#include "native_api.h"
#include "native_buffers.c"
#include "native_floats.c"
#include "native_api.c"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <bcrypt.h>
static unsigned clock_calls=0,random_calls=0;
static bool fail_random=false,overflow_clock=false;
static ULONGLONG test_clock(void) {++clock_calls;return overflow_clock?UINT64_MAX:1234567;}
static NTSTATUS test_random(BCRYPT_ALG_HANDLE handle,PUCHAR data,ULONG count,ULONG flags) {
    ++random_calls;
    if(handle!=NULL || flags!=BCRYPT_USE_SYSTEM_PREFERRED_RNG || fail_random) return (NTSTATUS)-1;
    memset(data,0xa7,count);return 0;
}
#define LOBO_WINDOWS_CLOCK test_clock
#define LOBO_WINDOWS_RANDOM test_random
#include "native_windows_snapshot.c"
#define CHECK(x) do {if(!(x)) {fprintf(stderr,"check failed line %d: %s\n",__LINE__,#x);return 1;}} while(0)
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    if(atexit(lobo_cleanup)) return 2;
    lobo_depth=1;lobo_validating=false;lobo_fuel=1000000;lobo_heap_cells=1000000;
    if(!strcmp(argv[1],"matrix")) {
        CHECK(lobo_native_option("--native-clock","true"));CHECK(lobo_native_option("--native-random","true"));
        for(unsigned parent=0;parent<4;++parent) {
            lobo_windows_root=0;lobo_windows_live=0;lobo_windows_calls=65536;
            uint64_t root=lobo_native_capability_v1();lobo_native_result_v1 p=lobo_native_restrict_capability_v1(root,parent);CHECK(!p.error);
            for(unsigned child=0;child<16;++child) {
                lobo_native_result_v1 c=lobo_native_restrict_capability_v1(p.value.handle,child);
                if((child&parent)!=child) {CHECK(c.error==LW_DENIED);continue;}
                CHECK(!c.error);unsigned clocks=clock_calls,randoms=random_calls;
                lobo_native_result_v1 clock=lobo_native_monotonic_ms_v1(c.value.handle);
                CHECK(clock.error==((child&1)?0:LW_DENIED));CHECK(clock_calls==clocks+((child&1)?1:0));
                lobo_native_result_v1 random=lobo_native_random_bytes_v1(c.value.handle,1);
                CHECK(random.error==((child&2)?0:LW_DENIED));CHECK(random_calls==randoms+((child&2)?1:0));
                if(!random.error) CHECK(random.value.buffer.length==1 && random.value.buffer.data[0]==0xa7);
            }
        }
    } else if(!strcmp(argv[1],"deny")) {
        uint64_t root=lobo_native_capability_v1();
        CHECK(lobo_native_monotonic_ms_v1(root).error==LW_DENIED);
        CHECK(lobo_native_random_bytes_v1(root,1).error==LW_DENIED);CHECK(!clock_calls && !random_calls);
    } else {
        CHECK(lobo_native_option("--native-clock","true"));CHECK(lobo_native_option("--native-random","true"));
        uint64_t root=lobo_native_capability_v1();
        if(!strcmp(argv[1],"root-depth")) {lobo_depth=2;(void)lobo_native_capability_v1();return 2;}
        else if(!strcmp(argv[1],"rng-failure")) {fail_random=true;CHECK(lobo_native_random_bytes_v1(root,16).error==LW_OS);CHECK(random_calls==1);}
        else if(!strcmp(argv[1],"clock-overflow")) {overflow_clock=true;CHECK(lobo_native_monotonic_ms_v1(root).error==LW_OS);}
        else if(!strcmp(argv[1],"bounds")) {
            CHECK(lobo_native_random_bytes_v1(root,-1).error==LW_INPUT);CHECK(lobo_native_random_bytes_v1(root,65537).error==LW_INPUT);
            CHECK(!lobo_native_random_bytes_v1(root,0).error);CHECK(!random_calls);
            CHECK(lobo_native_restrict_capability_v1(root,-1).error==LW_INPUT);CHECK(lobo_native_restrict_capability_v1(root,16).error==LW_INPUT);
            CHECK(lobo_native_monotonic_ms_v1(0).error==LW_HANDLE);CHECK(lobo_native_monotonic_ms_v1(513).error==LW_HANDLE);
            CHECK(lobo_native_monotonic_ms_v1(UINT64_MAX).error==LW_HANDLE);
            lobo_native_result_v1 r=lobo_native_random_bytes_v1(root,65536);CHECK(!r.error && r.value.buffer.length==65536);
            CHECK(r.value.buffer.data[0]==0xa7 && r.value.buffer.data[65535]==0xa7);
        } else if(!strcmp(argv[1],"quotas")) {
            lobo_windows_handles=1;CHECK(lobo_native_restrict_capability_v1(root,0).error==LW_LIMIT);
            lobo_windows_bytes=0;CHECK(lobo_native_random_bytes_v1(root,1).error==LW_LIMIT);CHECK(!random_calls);
            lobo_windows_calls=0;CHECK(lobo_native_monotonic_ms_v1(root).error==LW_LIMIT);CHECK(!clock_calls);
        } else if(!strcmp(argv[1],"unsupported")) {(void)lobo_native_listen_v1(root,0);return 2;}
        else if(!strcmp(argv[1],"fuel")) {lobo_fuel=0;(void)lobo_native_random_bytes_v1(root,1);return 2;}
        else if(!strcmp(argv[1],"heap")) {lobo_heap_cells=0;(void)lobo_native_random_bytes_v1(root,1);return 2;}
        else return 2;
    }
    puts("windows-provider-case-passed");return 0;
}
