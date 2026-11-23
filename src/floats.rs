use super::*;

pub(super) const NAN: u64 = 0x7ff8_0000_0000_0000;

pub(super) mod wire {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("0x{value:016x}"))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let text = String::deserialize(deserializer)?;
        bits(&text)
            .ok_or_else(|| serde::de::Error::custom("expected 0x and sixteen lowercase hexadecimal bits"))
    }
    pub(super) fn bits(text: &str) -> Option<u64> {
        let digits = text.strip_prefix("0x")?;
        if digits.len() != 16
            || !digits
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return None;
        }
        u64::from_str_radix(digits, 16).ok()
    }
}

pub(super) fn normalized(value: f64) -> Value {
    Value::F64(if value.is_nan() { NAN } else { value.to_bits() })
}

pub(super) fn literal(text: &str) -> Fallible<Value> {
    let value: f64 = text
        .parse()
        .map_err(|_| Diagnostic::new("compile.literal.range", "parse", "invalid F64 literal"))?;
    if !value.is_finite() {
        return Err(Diagnostic::new(
            "compile.literal.range",
            "parse",
            "F64 literals must be finite",
        ));
    }
    Ok(Value::F64(value.to_bits()))
}

fn decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let start = i;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let mut digits = i - start;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        digits += i - start;
    }
    if digits == 0 {
        return false;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    i == bytes.len()
}

pub(super) fn argument(text: &str) -> Fallible<Value> {
    let invalid = || Diagnostic::new("invocation.arguments.invalid", "invoke", "invalid F64 atom");
    let text = text.strip_prefix("F64:").ok_or_else(invalid)?;
    if let Some(bits) = wire::bits(text) {
        return Ok(Value::F64(bits));
    }
    if !decimal(text) {
        return Err(invalid());
    }
    let value: f64 = text.parse().map_err(|_| invalid())?;
    if !value.is_finite() {
        return Err(invalid());
    }
    Ok(Value::F64(value.to_bits()))
}

pub(super) fn operation(op: &str, values: &[Value]) -> Fallible<Value> {
    match (op, &values[0]) {
        ("from_i64", Value::I64(value)) => Ok(Value::F64((*value as f64).to_bits())),
        ("from_bits", Value::I64(value)) => Ok(Value::F64(*value as u64)),
        (_, Value::F64(bits)) => {
            let value = f64::from_bits(*bits);
            match op {
                "to_i64" => {
                    if !value.is_finite() || !(-9223372036854775808.0..9223372036854775808.0).contains(&value)
                    {
                        return Err(Diagnostic::new(
                            "runtime.float.conversion",
                            "execute",
                            "F64 cannot be converted to I64",
                        ));
                    }
                    Ok(Value::I64(value as i64))
                }
                "to_bits" => Ok(Value::I64(*bits as i64)),
                "is_finite" => Ok(Value::Bool(value.is_finite())),
                "is_nan" => Ok(Value::Bool(value.is_nan())),
                _ => unreachable!("checked F64 operation"),
            }
        }
        _ => unreachable!("checked F64 operation"),
    }
}

pub(super) fn environment() -> Fallible<()> {
    use std::hint::black_box as opaque;
    let one = opaque(1.0_f64);
    let half_ulp = opaque(f64::from_bits(0x3ca0_0000_0000_0000));
    let quarter_ulp = opaque(f64::from_bits(0x3c90_0000_0000_0000));
    let normal = opaque(f64::MIN_POSITIVE);
    let subnormal = opaque(f64::from_bits(0x0008_0000_0000_0000));
    if opaque(one + half_ulp) != one
        || opaque(-one - half_ulp) != -one
        || opaque(one - quarter_ulp) != one
        || opaque(normal / opaque(2.0)).to_bits() != subnormal.to_bits()
        || opaque(subnormal * opaque(2.0)).to_bits() != normal.to_bits()
    {
        return Err(Diagnostic::new(
            "invocation.float.environment",
            "invoke",
            "requires round-to-nearest ties-even and gradual underflow",
        ));
    }
    Ok(())
}
