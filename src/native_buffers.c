/* Immutable byte views. UTF-8 validation is independent of the Rust reference path. */
#ifndef LOBO_BUFFER_DEFINED
#define LOBO_BUFFER_DEFINED
typedef struct lobo_buffer { const unsigned char *data; uint64_t length; } lobo_buffer;
#endif
static uint64_t lobo_input_buffer_bytes = 8 * 1024 * 1024;
static void lobo_spend(uint64_t amount) {
    if (lobo_fuel < amount) lobo_fail("resource.fuel.exhausted");
    lobo_fuel -= amount;
}
static int lobo_hex_digit(unsigned char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}
static unsigned lobo_octet(const unsigned char *data, uint64_t i, bool hex) {
    return hex ? (unsigned)(16 * lobo_hex_digit(data[2*i]) + lobo_hex_digit(data[2*i+1])) : data[i];
}
static bool lobo_decode(const unsigned char *data, uint64_t length, uint64_t *position, uint32_t *scalar, bool hex) {
    uint64_t i = *position;
    if (i >= length) return false;
    unsigned c = lobo_octet(data,i++,hex), tails; uint32_t value, minimum;
    if (c < 0x80) { *scalar = c; *position = i; return true; }
    if (c >= 0xc2 && c <= 0xdf) { tails = 1; value = c & 31; minimum = 0x80; }
    else if (c >= 0xe0 && c <= 0xef) { tails = 2; value = c & 15; minimum = 0x800; }
    else if (c >= 0xf0 && c <= 0xf4) { tails = 3; value = c & 7; minimum = 0x10000; }
    else return false;
    if (length - i < tails) return false;
    for (unsigned j = 0; j < tails; ++j) { c = lobo_octet(data,i++,hex); if (c < 0x80 || c > 0xbf) return false; value = (value << 6) | (c & 63); }
    if (value < minimum || value > 0x10ffff || (value >= 0xd800 && value <= 0xdfff)) return false;
    *scalar = value; *position = i; return true;
}
static bool lobo_utf8(const unsigned char *data, uint64_t length, bool hex) {
    uint64_t i = 0; uint32_t scalar;
    while (i < length) if (!lobo_decode(data,length,&i,&scalar,hex)) return false;
    return true;
}
static lobo_buffer lobo_buffer_new(uint64_t length) {
    if (!length || lobo_validating) return (lobo_buffer){NULL,length};
    if (length > INT64_MAX || length > SIZE_MAX - 32) lobo_fail("resource.memory.exhausted");
    lobo_list *storage = lobo_cons((size_t)(length / 8 + (length % 8 != 0)),NULL);
    return (lobo_buffer){(const unsigned char *)storage->data,length};
}
static lobo_buffer lobo_parse_buffer(const char **cursor, bool text) {
    lobo_input_tick(); size_t length; const char *token = lobo_next(cursor,&length);
    size_t prefix = text ? 5 : 6;
    if (length < prefix || strncmp(token,text ? "Text:" : "Bytes:",prefix) != 0 || (length - prefix) % 2) lobo_fail("invocation.arguments.invalid");
    const unsigned char *encoded = (const unsigned char *)token + prefix; uint64_t n = (length - prefix) / 2;
    for (uint64_t i = 0; i < 2*n; ++i) if (lobo_hex_digit(encoded[i]) < 0) lobo_fail("invocation.arguments.invalid");
    if (lobo_validating) { if (n > lobo_input_buffer_bytes) lobo_fail("resource.input.limit"); lobo_input_buffer_bytes -= n; }
    if (text && !lobo_utf8(encoded,n,true)) lobo_fail("invocation.arguments.invalid");
    lobo_buffer value = lobo_buffer_new(n);
    if (!lobo_validating) { unsigned char *out = (unsigned char *)value.data; for (uint64_t i = 0; i < n; ++i) out[i] = (unsigned char)lobo_octet(encoded,i,true); }
    return value;
}
static bool lobo_buffers_equal(lobo_buffer a, lobo_buffer b) {
    if (a.length != b.length) return false;
    for (uint64_t i = 0; i < a.length; ++i) { lobo_tick(); if (a.data[i] != b.data[i]) return false; }
    return true;
}
static int64_t lobo_bytes_at(lobo_buffer value, int64_t index) {
    if (index < 0 || (uint64_t)index >= value.length) lobo_fail("runtime.bytes.bounds"); return value.data[index];
}
static lobo_buffer lobo_bytes_slice(lobo_buffer value, int64_t start, int64_t count) {
    if (start < 0 || count < 0 || (uint64_t)start > value.length || (uint64_t)count > value.length - (uint64_t)start) lobo_fail("runtime.bytes.bounds");
    return count ? (lobo_buffer){value.data + start,(uint64_t)count} : (lobo_buffer){NULL,0};
}
static lobo_buffer lobo_buffers_concat(lobo_buffer a, lobo_buffer b) {
    if (b.length > INT64_MAX - a.length) lobo_fail("resource.memory.exhausted");
    uint64_t n = a.length + b.length; lobo_spend(n); lobo_buffer result = lobo_buffer_new(n);
    if (a.length) memcpy((void *)result.data,a.data,(size_t)a.length);
    if (b.length) memcpy((unsigned char *)result.data + a.length,b.data,(size_t)b.length);
    return result;
}
static lobo_buffer lobo_bytes_push(lobo_buffer value, int64_t byte) {
    if (byte < 0 || byte > 255) lobo_fail("runtime.bytes.range");
    if (value.length == INT64_MAX) lobo_fail("resource.memory.exhausted");
    lobo_spend(value.length + 1); lobo_buffer result = lobo_buffer_new(value.length + 1);
    if (value.length) memcpy((void *)result.data,value.data,(size_t)value.length);
    ((unsigned char *)result.data)[value.length] = (unsigned char)byte; return result;
}
static lobo_buffer lobo_bytes_from_list(const lobo_list *list) {
    uint64_t n = list ? list->length : 0; lobo_spend(n);
    for (const lobo_list *node = list; node; node = node->tail) { int64_t value; memcpy(&value,node->data,sizeof(value)); if (value < 0 || value > 255) lobo_fail("runtime.bytes.range"); }
    lobo_buffer result = lobo_buffer_new(n); unsigned char *out = (unsigned char *)result.data;
    for (; list; list = list->tail) { int64_t value; memcpy(&value,list->data,sizeof(value)); *out++ = (unsigned char)value; }
    return result;
}
static bool lobo_text_valid(lobo_buffer bytes) { lobo_spend(bytes.length); return lobo_utf8(bytes.data,bytes.length,false); }
static lobo_buffer lobo_text_from_bytes(lobo_buffer bytes) { if (!lobo_text_valid(bytes)) lobo_fail("runtime.text.invalid_utf8"); return bytes; }
static uint64_t lobo_text_count(lobo_buffer value) {
    uint64_t i = 0, count = 0; uint32_t scalar;
    while (i < value.length) { if (!lobo_decode(value.data,value.length,&i,&scalar,false)) abort(); ++count; } return count;
}
static int64_t lobo_text_length(lobo_buffer value) { lobo_spend(value.length); return (int64_t)lobo_text_count(value); }
static uint64_t lobo_text_offset(lobo_buffer value, uint64_t index) {
    uint64_t i = 0; uint32_t scalar;
    for (uint64_t j = 0; j < index; ++j) if (!lobo_decode(value.data,value.length,&i,&scalar,false)) abort(); return i;
}
static int64_t lobo_text_at(lobo_buffer value, int64_t index) {
    if (index < 0) lobo_fail("runtime.text.bounds"); lobo_spend(value.length);
    if ((uint64_t)index >= lobo_text_count(value)) lobo_fail("runtime.text.bounds");
    uint64_t i = lobo_text_offset(value,(uint64_t)index); uint32_t scalar;
    if (!lobo_decode(value.data,value.length,&i,&scalar,false)) abort(); return scalar;
}
static lobo_buffer lobo_text_slice(lobo_buffer value, int64_t start, int64_t count) {
    if (start < 0 || count < 0) lobo_fail("runtime.text.bounds"); lobo_spend(value.length);
    uint64_t length = lobo_text_count(value);
    if ((uint64_t)start > length || (uint64_t)count > length - (uint64_t)start) lobo_fail("runtime.text.bounds");
    if (!count) return (lobo_buffer){NULL,0};
    uint64_t begin = lobo_text_offset(value,(uint64_t)start), end = lobo_text_offset(value,(uint64_t)start + (uint64_t)count);
    return (lobo_buffer){value.data + begin,end - begin};
}
static unsigned lobo_json_byte_size(unsigned char byte) {
    if (byte == '"' || byte == '\\' || byte == '\b' || byte == '\f' || byte == '\n' || byte == '\r' || byte == '\t') return 2;
    return byte < 32 ? 6 : 1;
}
static void lobo_count_buffer(lobo_buffer value, bool text) {
    lobo_output_tick(); lobo_output_bytes(strlen(text ? "{\"type\":\"Text\",\"value\":\"\"}" : "{\"type\":\"Bytes\",\"value\":\"\"}"));
    if (!text) { if (value.length > lobo_output_remaining / 2) lobo_fail("resource.output.limit"); lobo_output_bytes((size_t)(2 * value.length)); return; }
    for (uint64_t i = 0; i < value.length; ++i) lobo_output_bytes(lobo_json_byte_size(value.data[i]));
}
static void lobo_print_buffer(lobo_buffer value, bool text) {
    static const char hex[] = "0123456789abcdef";
    fputs(text ? "{\"type\":\"Text\",\"value\":\"" : "{\"type\":\"Bytes\",\"value\":\"",stdout);
    for (uint64_t i = 0; i < value.length; ++i) {
        unsigned char c = value.data[i];
        if (!text) { fputc(hex[c >> 4],stdout); fputc(hex[c & 15],stdout); continue; }
        switch (c) {
            case '"': fputs("\\\"",stdout); break; case '\\': fputs("\\\\",stdout); break;
            case '\b': fputs("\\b",stdout); break; case '\f': fputs("\\f",stdout); break;
            case '\n': fputs("\\n",stdout); break; case '\r': fputs("\\r",stdout); break; case '\t': fputs("\\t",stdout); break;
            default: if (c < 32) { fputs("\\u00",stdout); fputc(hex[c >> 4],stdout); fputc(hex[c & 15],stdout); } else fputc(c,stdout);
        }
    }
    fputs("\"}",stdout);
}
