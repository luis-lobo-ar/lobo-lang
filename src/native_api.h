#ifndef LOBO_NATIVE_ABI_V1_H
#define LOBO_NATIVE_ABI_V1_H
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#ifndef LOBO_BUFFER_DEFINED
#define LOBO_BUFFER_DEFINED
typedef struct lobo_buffer { const unsigned char *data; uint64_t length; } lobo_buffer;
#endif
/* Buffer arguments are borrowed only for the duration of the call. Returned
 * buffers belong to the invocation arena; handles belong to this process. */
typedef struct lobo_native_result_v1 {
    int64_t error;
    union { int64_t integer; double floating; bool boolean; uint64_t handle; lobo_buffer buffer; } value;
} lobo_native_result_v1;
#ifdef __cplusplus
extern "C" {
#endif
int64_t lobo_native_abi_version_v1(void);
int64_t lobo_native_roundtrip_i64_v1(int64_t value);
bool lobo_native_roundtrip_bool_v1(bool value);
lobo_buffer lobo_native_copy_bytes_v1(lobo_buffer value);
uint64_t lobo_native_capability_v1(void);
lobo_native_result_v1 lobo_native_restrict_capability_v1(uint64_t cap, int64_t mask);
lobo_native_result_v1 lobo_native_monotonic_ms_v1(uint64_t cap);
lobo_native_result_v1 lobo_native_random_bytes_v1(uint64_t cap, int64_t count);
lobo_native_result_v1 lobo_native_listen_v1(uint64_t cap, int64_t port);
lobo_native_result_v1 lobo_native_connect_v1(uint64_t cap, int64_t port, int64_t timeout);
lobo_native_result_v1 lobo_native_accept_v1(uint64_t listener, int64_t timeout);
lobo_native_result_v1 lobo_native_port_v1(uint64_t listener);
lobo_native_result_v1 lobo_native_read_v1(uint64_t stream, int64_t count, int64_t timeout);
lobo_native_result_v1 lobo_native_write_v1(uint64_t stream, lobo_buffer data, int64_t timeout);
lobo_native_result_v1 lobo_native_shutdown_write_v1(uint64_t stream);
lobo_native_result_v1 lobo_native_close_listener_v1(uint64_t listener);
lobo_native_result_v1 lobo_native_close_stream_v1(uint64_t stream);
#ifdef __cplusplus
}
#endif
#if defined(LOBO_NATIVE_ENABLED)
_Static_assert(sizeof(void*)==8 && sizeof(int64_t)==8 && sizeof(bool)==1, "native ABI v1 requires a 64-bit target");
_Static_assert(sizeof(lobo_buffer)==16 && offsetof(lobo_buffer,length)==8, "native buffer ABI v1");
_Static_assert(sizeof(lobo_native_result_v1)==24 && offsetof(lobo_native_result_v1,value)==8, "native result ABI v1");
#endif
#endif
