#define _POSIX_C_SOURCE 200809L
#define main lobo_generated_main
#include "core.c"
#undef main
#include <time.h>

int main(int argc, char **argv) {
    if (argc != 3) return 2;
    bool absolute = strcmp(argv[1], "absolute") == 0;
    if (!absolute && strcmp(argv[1], "increment") != 0) return 2;
    char *end;
    unsigned long long parsed = strtoull(argv[2], &end, 10);
    if (*end || parsed == 0 || parsed > 1000000000) return 2;
    uint64_t iterations = parsed;
    struct timespec start, finish;
    uint64_t checksum = 0;
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (uint64_t i = 0; i < iterations; ++i) {
        int64_t input = (int64_t)(i & 2047) - 1024;
        __asm__ volatile("" : "+r"(input));
        lobo_fuel = 100000;
        lobo_depth = 0;
        int64_t output = absolute ? lobo_fn_1(input) : lobo_fn_0(input);
        __asm__ volatile("" : "+r"(output));
        checksum += (uint64_t)output;
    }
    clock_gettime(CLOCK_MONOTONIC, &finish);
    int64_t nanos = (int64_t)(finish.tv_sec - start.tv_sec) * INT64_C(1000000000)
                  + (int64_t)(finish.tv_nsec - start.tv_nsec);
    printf("{\"ns_per_call\":%.6f,\"elapsed_ns\":\"%" PRId64 "\",\"checksum\":\"%" PRIu64 "\",\"iterations\":%" PRIu64 "}\n",
           (double)nanos / iterations, nanos, checksum, iterations);
    return 0;
}
