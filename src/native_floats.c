#include <float.h>
_Static_assert(sizeof(double) == 8 && DBL_MANT_DIG == 53 && DBL_MAX_EXP == 1024, "binary64 required");

static uint64_t lobo_f64_bits(double value) { uint64_t bits; memcpy(&bits,&value,8); return bits; }
static double lobo_f64_from_u64(uint64_t bits) { double out; memcpy(&out,&bits,8); return out; }
static double lobo_f64_from_bits(int64_t value) { return lobo_f64_from_u64((uint64_t)value); }
static int64_t lobo_f64_to_bits(double value) { uint64_t bits=lobo_f64_bits(value); int64_t out; memcpy(&out,&bits,8); return out; }
static bool lobo_f64_is_finite(double value) { return (lobo_f64_bits(value)&UINT64_C(0x7ff0000000000000)) != UINT64_C(0x7ff0000000000000); }
static bool lobo_f64_is_nan(double value) { return (lobo_f64_bits(value)&UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000); }
static double lobo_f64_normalize(double value) { return lobo_f64_is_nan(value) ? lobo_f64_from_bits(INT64_C(9221120237041090560)) : value; }
static double lobo_f64_from_i64(int64_t value) { return (double)value; }
static int64_t lobo_f64_to_i64(double value) {
    if (!lobo_f64_is_finite(value) || value < -0x1p63 || value >= 0x1p63) lobo_fail("runtime.float.conversion");
    return (int64_t)value;
}
static void lobo_float_environment(void) {
    volatile double one=1.0, half=0x1p-53, quarter=0x1p-54, normal=0x1p-1022, subnormal=0x1p-1023, two=2.0;
    if (one+half != one || -one-half != -one || one-quarter != one
        || lobo_f64_bits(normal/two) != UINT64_C(0x0008000000000000)
        || lobo_f64_bits(subnormal*two) != UINT64_C(0x0010000000000000)) {
        lobo_fail("invocation.float.environment");
    }
}

static double lobo_parse_f64(const char **cursor) {
    lobo_input_tick(); size_t length; const char *token=lobo_next(cursor,&length);
    if (length<5 || memcmp(token,"F64:",4)!=0) lobo_fail("invocation.arguments.invalid");
    const char *text=token+4; size_t size=length-4;
    if (size==18 && memcmp(text,"0x",2)==0) {
        uint64_t bits=0;
        for (size_t n=2;n<18;n++) {
            unsigned char b=(unsigned char)text[n];
            if (!((b>='0'&&b<='9')||(b>='a'&&b<='f'))) lobo_fail("invocation.arguments.invalid");
            bits=(bits<<4)|(uint64_t)(b<='9' ? b-'0' : b-'a'+10);
        }
        double out; memcpy(&out,&bits,8); return out;
    }
    size_t i=0, digits=0;
    if (i<size && (text[i]=='+'||text[i]=='-')) i++;
    while (i<size && text[i]>='0'&&text[i]<='9') { i++; digits++; }
    if (i<size && text[i]=='.') { i++; while (i<size && text[i]>='0'&&text[i]<='9') { i++; digits++; } }
    if (!digits) lobo_fail("invocation.arguments.invalid");
    if (i<size && (text[i]=='e'||text[i]=='E')) {
        i++; if (i<size && (text[i]=='+'||text[i]=='-')) i++;
        size_t start=i; while (i<size && text[i]>='0'&&text[i]<='9') i++;
        if (i==start) lobo_fail("invocation.arguments.invalid");
    }
    if (i!=size) lobo_fail("invocation.arguments.invalid");
    char *end; double out=strtod(text,&end);
    if (end!=text+size || !lobo_f64_is_finite(out)) lobo_fail("invocation.arguments.invalid");
    return out;
}
