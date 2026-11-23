/* Pinned single-threaded embedded driver profile. No runtime extension loading or disk temporary stores. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE 1
#endif
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif
#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#endif
#define SQLITE_THREADSAFE 0
#define SQLITE_OMIT_LOAD_EXTENSION 1
#define SQLITE_DQS 0
#define SQLITE_TEMP_STORE 3
#define SQLITE_MAX_WORKER_THREADS 0
#define SQLITE_ENABLE_API_ARMOR 1
