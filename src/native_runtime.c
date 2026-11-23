#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif
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
/* Preserve the outermost contract phase across nested calls, as the evaluator does. */
static const char *lobo_contract_phase = NULL;
typedef struct lobo_list { struct lobo_list *allocation_next; const struct lobo_list *tail; uint64_t length; uint64_t data[]; } lobo_list;
static lobo_list *lobo_allocations = NULL;
static void *lobo_argument_storage = NULL;
static uint64_t lobo_heap_cells = 1000000;
static bool lobo_validating = true;
static uint64_t lobo_input_nodes = 100000, lobo_output_nodes = 100000;
static uint64_t lobo_output_remaining = 8 * 1024 * 1024;
static unsigned lobo_input_depth = 0, lobo_output_depth = 0, lobo_compare_depth = 0;

static void lobo_native_cleanup(void);
static bool lobo_native_option(const char *name,const char *value);

static void lobo_cleanup(void) {
    lobo_native_cleanup();
    while (lobo_allocations) { lobo_list *next = lobo_allocations->allocation_next; free(lobo_allocations); lobo_allocations = next; }
    free(lobo_argument_storage); lobo_argument_storage = NULL;
}

_Noreturn static void lobo_fail(const char *code) {
    const char *kind = strncmp(code, "runtime.", 8) == 0 ? "runtime_fault" :
                       strncmp(code, "resource.", 9) == 0 ? "resource_exhausted" : "invocation_error";
    printf("{\"status\":\"error\",\"kind\":\"%s\",\"diagnostic\":{\"code\":\"%s\",\"phase\":\"%s\"",
           kind, code, (strncmp(code, "runtime.", 8) == 0 || strncmp(code, "resource.", 9) == 0) && lobo_contract_phase
                       ? lobo_contract_phase : lobo_phase);
    if (lobo_subject) printf(",\"subject\":\"%s\"", lobo_subject);
    printf("}}\n");
    exit(1);
}

static void lobo_tick(void) {
    if (lobo_fuel == 0) lobo_fail("resource.fuel.exhausted");
    --lobo_fuel;
}
static void lobo_compare_enter(void) {
    if (lobo_compare_depth >= 64) lobo_fail("resource.stack.exhausted");
    ++lobo_compare_depth;
}
static void lobo_compare_leave(void) { --lobo_compare_depth; }

static void lobo_input_tick(void) {
    if (!lobo_validating) return;
    if (lobo_input_depth > 8) lobo_fail("resource.input.limit");
    if (!lobo_input_nodes) lobo_fail("resource.input.limit");
    --lobo_input_nodes;
}
static void lobo_output_tick(void) {
    if (lobo_output_depth > 8) lobo_fail("resource.output.limit");
    if (!lobo_output_nodes) lobo_fail("resource.output.limit");
    --lobo_output_nodes;
}
static void lobo_output_bytes(size_t amount) {
    if (lobo_output_remaining < amount) lobo_fail("resource.output.limit");
    lobo_output_remaining -= amount;
}
static void lobo_count_integer(int64_t value) {
    char text[64]; int length = snprintf(text, sizeof(text), "{\"type\":\"I64\",\"value\":\"%" PRId64 "\"}", value);
    lobo_output_tick(); lobo_output_bytes((size_t)length);
}
static lobo_list *lobo_cons(size_t width, const lobo_list *tail) {
    if (lobo_validating) return NULL;
    if (lobo_heap_cells < 3 + width || (tail && tail->length == INT64_MAX)) lobo_fail("resource.memory.exhausted");
    lobo_heap_cells -= 3 + width;
    lobo_list *node = malloc(sizeof(*node) + 8 * width);
    if (!node) lobo_fail("resource.memory.exhausted");
    node->allocation_next = lobo_allocations; lobo_allocations = node;
    node->tail = tail; node->length = tail ? tail->length + 1 : 1;
    return node;
}

struct lobo_value { bool is_bool; int64_t integer; bool boolean; const char *record; };
struct lobo_args { const char *entry; size_t count; struct lobo_value *values; };

static bool lobo_decimal(const char *text, bool allow_negative) {
    if (*text == '+' || (allow_negative && *text == '-')) ++text;
    if (*text == '\0') return false;
    for (; *text; ++text) if (*text < '0' || *text > '9') return false;
    return true;
}

static const char *lobo_next(const char **cursor, size_t *length) {
    const char *text = *cursor;
    if (!text || !*text) lobo_fail("invocation.arguments.invalid");
    const char *end = strchr(text, ',');
    *length = end ? (size_t)(end - text) : strlen(text);
    *cursor = end ? end + 1 : NULL;
    return text;
}

static uint64_t lobo_count_token(const char **cursor) {
    size_t length; const char *token = lobo_next(cursor, &length);
    if (length < 7 || strncmp(token, "Count:", 6) != 0) lobo_fail("invocation.arguments.invalid");
    const char *digits = token + 6;
    if (*digits < '0' || *digits > '9' || (*digits == '0' && length != 7)) lobo_fail("invocation.arguments.invalid");
    const char *end = digits; while (*end >= '0' && *end <= '9') ++end;
    errno = 0; char *parsed_end; unsigned long long n = strtoull(digits, &parsed_end, 10);
    if (errno == ERANGE || parsed_end != end || (size_t)(end - token) != length || n > INT64_MAX) lobo_fail("invocation.arguments.invalid");
    return (uint64_t)n;
}

static struct lobo_value lobo_leaf(const char **cursor) {
    size_t length;
    const char *token = lobo_next(cursor, &length);
    const char *text = token, *end;
    struct lobo_value value = {false, 0, false, NULL};
    if (strncmp(text, "I64:", 4) == 0) {
        text += 4; const char *digits = text;
        if (*digits == '+' || *digits == '-') ++digits;
        end = digits;
        while (*end >= '0' && *end <= '9') ++end;
        if (end == digits) lobo_fail("invocation.arguments.invalid");
        errno = 0; char *parsed_end;
        long long parsed = strtoll(text, &parsed_end, 10);
        if (errno == ERANGE || parsed_end != end || parsed < INT64_MIN || parsed > INT64_MAX)
            lobo_fail("invocation.arguments.invalid");
        value.integer = (int64_t)parsed;
    } else if (strncmp(text, "Bool:true", 9) == 0) {
        value.is_bool = true; value.boolean = true; end = text + 9;
    } else if (strncmp(text, "Bool:false", 10) == 0) {
        value.is_bool = true; value.boolean = false; end = text + 10;
    } else { lobo_fail("invocation.arguments.invalid"); }
    if ((size_t)(end - token) != length) lobo_fail("invocation.arguments.invalid");
    return value;
}

static struct lobo_args lobo_parse(int argc, char **argv) {
    struct lobo_args args = { NULL, 0, NULL };
    args.values = calloc((size_t)argc, sizeof(*args.values));
    lobo_argument_storage = args.values;
    if (!args.values) lobo_fail("resource.memory.exhausted");
    bool fuel_seen = false, heap_seen = false;
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
            struct lobo_value argument = {false, 0, false, NULL};
            if (strncmp(value, "Record:", 7) == 0 || strncmp(value, "Variant:", 8) == 0 || strncmp(value, "List:", 5) == 0 || strncmp(value, "Bytes:", 6) == 0 || strncmp(value, "Text:", 5) == 0 || strncmp(value, "Function:", 9) == 0 || strncmp(value, "F64:", 4) == 0) argument.record = value;
            else { argument = lobo_leaf(&value); if (value) lobo_fail("invocation.arguments.invalid"); }
            args.values[args.count++] = argument;
        } else if (strcmp(argv[i], "--heap-cells") == 0 && !heap_seen) {
            if (!lobo_decimal(value, false)) lobo_fail("invocation.cli.invalid");
            errno = 0; char *end; unsigned long long n = strtoull(value, &end, 10);
            if (errno == ERANGE || *end || n > UINT64_MAX) lobo_fail("invocation.cli.invalid");
            lobo_heap_cells = (uint64_t)n; heap_seen = true;
        } else if (!lobo_native_option(argv[i],value)) lobo_fail("invocation.cli.invalid");
    }
    if (!args.entry) lobo_fail("invocation.cli.invalid");
    return args;
}
