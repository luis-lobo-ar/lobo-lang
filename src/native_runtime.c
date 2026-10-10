/* Host adapter and checks for generated Core functions. No expression tree or interpreter is included. */
#include <stdbool.h>
#include <stdint.h>
#include <inttypes.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>

static uint64_t lobo_fuel = 100000;
static unsigned lobo_depth = 0;
static const char *lobo_subject = NULL;
static const char *lobo_phase = "invoke";

_Noreturn static void lobo_fail(const char *code) {
    const char *kind = strncmp(code, "runtime.", 8) == 0 ? "runtime_fault" :
                       strncmp(code, "resource.", 9) == 0 ? "resource_exhausted" : "invocation_error";
    printf("{\"status\":\"error\",\"kind\":\"%s\",\"diagnostic\":{\"code\":\"%s\",\"phase\":\"%s\"",
           kind, code, lobo_phase);
    if (lobo_subject) printf(",\"subject\":\"%s\"", lobo_subject);
    printf("}}\n");
    exit(1);
}

static void lobo_tick(void) {
    if (lobo_fuel == 0) lobo_fail("resource.fuel.exhausted");
    --lobo_fuel;
}

struct lobo_value { bool is_bool; int64_t integer; bool boolean; };
struct lobo_args { const char *entry; size_t count; struct lobo_value *values; };

static bool lobo_decimal(const char *text, bool allow_negative) {
    if (*text == '+' || (allow_negative && *text == '-')) ++text;
    if (*text == '\0') return false;
    for (; *text; ++text) if (*text < '0' || *text > '9') return false;
    return true;
}

static struct lobo_args lobo_parse(int argc, char **argv) {
    struct lobo_args args = { NULL, 0, NULL };
    args.values = calloc((size_t)argc, sizeof(*args.values));
    if (!args.values) lobo_fail("resource.memory.exhausted");
    bool fuel_seen = false;
    for (int i = 1; i < argc; i += 2) {
        if (i + 1 >= argc) lobo_fail("invocation.cli.invalid");
        const char *value = argv[i + 1];
        if (strcmp(argv[i], "--entry") == 0 && !args.entry) args.entry = value;
        else if (strcmp(argv[i], "--fuel") == 0 && !fuel_seen) {
            if (!lobo_decimal(value, false)) lobo_fail("invocation.cli.invalid");
            errno = 0; char *end;
            unsigned long long parsed = strtoull(value, &end, 10);
            if (errno == ERANGE || *end || parsed > UINT64_MAX) lobo_fail("invocation.cli.invalid");
            lobo_fuel = (uint64_t)parsed;
            fuel_seen = true;
        } else if (strcmp(argv[i], "--arg") == 0) {
            struct lobo_value argument = {false, 0, false};
            if (strncmp(value, "I64:", 4) == 0) {
                value += 4;
                if (!lobo_decimal(value, true)) lobo_fail("invocation.arguments.invalid");
                errno = 0; char *end;
                long long parsed = strtoll(value, &end, 10);
                if (errno == ERANGE || *end || parsed < INT64_MIN || parsed > INT64_MAX)
                    lobo_fail("invocation.arguments.invalid");
                argument.integer = (int64_t)parsed;
            } else if (strcmp(value, "Bool:true") == 0 || strcmp(value, "Bool:false") == 0) {
                argument.is_bool = true;
                argument.boolean = strcmp(value, "Bool:true") == 0;
            } else lobo_fail("invocation.arguments.invalid");
            args.values[args.count++] = argument;
        } else lobo_fail("invocation.cli.invalid");
    }
    if (!args.entry) lobo_fail("invocation.cli.invalid");
    return args;
}
