use super::*;
use std::sync::Arc;

pub(super) struct InputBudget {
    pub nodes: usize,
    pub bytes: usize,
}
impl Default for InputBudget {
    fn default() -> Self {
        Self {
            nodes: 100_000,
            bytes: 8 * 1024 * 1024,
        }
    }
}
impl InputBudget {
    pub fn node(&mut self) -> Fallible<()> {
        self.nodes = self.nodes.checked_sub(1).ok_or_else(input_limit)?;
        Ok(())
    }
    pub fn buffer(&mut self, n: usize) -> Fallible<()> {
        self.bytes = self.bytes.checked_sub(n).ok_or_else(input_limit)?;
        Ok(())
    }
}
fn input_limit() -> Diagnostic {
    Diagnostic::new("resource.input.limit", "invoke", "input value limit exceeded")
}
pub(super) fn input_size(value: &Value, remaining: &mut usize) -> Fallible<()> {
    match value {
        Value::Bytes(value) | Value::Text(value) => {
            *remaining = remaining.checked_sub(value.len()).ok_or_else(input_limit)?;
        }
        Value::Record(value) => {
            for value in value.fields.values() {
                input_size(value, remaining)?;
            }
        }
        Value::Variant(value) => {
            if let Some(value) = &value.payload {
                input_size(value, remaining)?;
            }
        }
        Value::List(value) => {
            for value in value.iter() {
                input_size(value, remaining)?;
            }
        }
        _ => (),
    }
    Ok(())
}

#[derive(Clone)]
pub struct BufferValue {
    data: Arc<[u8]>,
    start: usize,
    length: usize,
}
impl BufferValue {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let length = bytes.len();
        Self {
            data: bytes.into(),
            start: 0,
            length,
        }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data[self.start..self.start + self.length]
    }
    fn view(&self, start: usize, length: usize) -> Self {
        Self {
            data: self.data.clone(),
            start: self.start + start,
            length,
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}
impl PartialEq for BufferValue {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}
impl Eq for BufferValue {}
impl std::fmt::Debug for BufferValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("length", &self.length)
            .field("prefix", &&self.bytes()[..self.length.min(32)])
            .finish()
    }
}
pub(super) fn cells(length: usize) -> usize {
    if length == 0 {
        0
    } else {
        3 + length.div_ceil(8)
    }
}
pub(super) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(DIGITS[(byte >> 4) as usize] as char);
        text.push(DIGITS[(byte & 15) as usize] as char);
    }
    text
}
pub(super) fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            Some((hi * 16 + lo) as u8)
        })
        .collect()
}
pub(super) mod hex_wire {
    use super::*;
    pub fn serialize<S: serde::Serializer>(value: &BufferValue, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex(value.bytes()))
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<BufferValue, D::Error> {
        let text = String::deserialize(deserializer)?;
        let bytes = decode_hex(&text).ok_or_else(|| serde::de::Error::custom("invalid hex bytes"))?;
        if hex(&bytes) != text {
            return Err(serde::de::Error::custom("expected lowercase hex"));
        }
        Ok(BufferValue::from_bytes(bytes))
    }
}
pub(super) mod text_wire {
    use super::*;
    pub fn serialize<S: serde::Serializer>(value: &BufferValue, serializer: S) -> Result<S::Ok, S::Error> {
        let text = std::str::from_utf8(value.bytes()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(text)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<BufferValue, D::Error> {
        Ok(BufferValue::from_bytes(
            String::deserialize(deserializer)?.into_bytes(),
        ))
    }
}

pub(super) fn signature(family: &str, op: &str) -> Fallible<(Vec<Type>, Type)> {
    use Type::*;
    let (args, output) = match (family, op) {
        ("F64", "from_i64" | "from_bits") => (vec![I64], F64),
        ("F64", "to_i64" | "to_bits") => (vec![F64], I64),
        ("F64", "is_finite" | "is_nan") => (vec![F64], Bool),
        ("Bytes", "empty") => (vec![], Bytes),
        ("Bytes", "length") => (vec![Bytes], I64),
        ("Bytes", "at") => (vec![Bytes, I64], I64),
        ("Bytes", "slice") => (vec![Bytes, I64, I64], Bytes),
        ("Bytes", "push") => (vec![Bytes, I64], Bytes),
        ("Bytes", "concat") => (vec![Bytes, Bytes], Bytes),
        ("Bytes", "from_list") => (vec![List(Box::new(I64))], Bytes),
        ("Text", "bytes") => (vec![Text], Bytes),
        ("Text", "valid") => (vec![Bytes], Bool),
        ("Text", "from_bytes") => (vec![Bytes], Text),
        ("Text", "length") => (vec![Text], I64),
        ("Text", "at") => (vec![Text, I64], I64),
        ("Text", "slice") => (vec![Text, I64, I64], Text),
        ("Text", "concat") => (vec![Text, Text], Text),
        _ => return Err(action_error("compile.buffer.invalid", "unknown buffer operation")),
    };
    Ok((args, output))
}
fn fault(code: &str) -> Diagnostic {
    Diagnostic::new(code, "execute", "invalid buffer operation input")
}
fn buffer(value: &Value) -> &BufferValue {
    match value {
        Value::Bytes(v) | Value::Text(v) => v,
        _ => unreachable!("checked buffer"),
    }
}
fn integer(value: &Value) -> i64 {
    let Value::I64(n) = value else {
        unreachable!("checked integer")
    };
    *n
}
fn range(start: i64, count: i64, length: usize, code: &str) -> Fallible<(usize, usize)> {
    if start < 0
        || count < 0
        || start as u64 > length as u64
        || count as u64 > (length - start as usize) as u64
    {
        return Err(fault(code));
    }
    Ok((start as usize, count as usize))
}
impl Runner<'_> {
    pub(super) fn spend_bytes(&mut self, amount: usize) -> Fallible<()> {
        self.fuel = self
            .fuel
            .checked_sub(amount as u64)
            .ok_or_else(|| fault("resource.fuel.exhausted"))?;
        Ok(())
    }
    pub(super) fn buffer_op(&mut self, family: &str, op: &str, values: Vec<Value>) -> Fallible<Value> {
        if op == "empty" {
            return Ok(Value::Bytes(BufferValue::from_bytes(vec![])));
        }
        if op == "from_list" {
            let Value::List(xs) = &values[0] else {
                unreachable!("checked list")
            };
            self.spend_bytes(xs.len())?;
            let mut bytes = Vec::new();
            for value in xs.iter() {
                let n = integer(value);
                if !(0..=255).contains(&n) {
                    return Err(fault("runtime.bytes.range"));
                }
                bytes.push(n as u8);
            }
            lists::charge(&mut self.heap, cells(bytes.len()))?;
            return Ok(Value::Bytes(BufferValue::from_bytes(bytes)));
        }
        let first = buffer(&values[0]);
        let wrap = |v| {
            if family == "Text" {
                Value::Text(v)
            } else {
                Value::Bytes(v)
            }
        };
        match (family, op) {
            ("Text", "bytes") => Ok(Value::Bytes(first.clone())),
            ("Text", "valid" | "from_bytes") => {
                self.spend_bytes(first.len())?;
                let valid = std::str::from_utf8(first.bytes()).is_ok();
                if op == "valid" {
                    return Ok(Value::Bool(valid));
                }
                if !valid {
                    return Err(fault("runtime.text.invalid_utf8"));
                }
                Ok(Value::Text(first.clone()))
            }
            (_, "concat" | "push") => {
                let pushed = if op == "push" {
                    let n = integer(&values[1]);
                    if !(0..=255).contains(&n) {
                        return Err(fault("runtime.bytes.range"));
                    }
                    Some(n as u8)
                } else {
                    None
                };
                let second = if pushed.is_none() {
                    Some(buffer(&values[1]))
                } else {
                    None
                };
                let n = first
                    .len()
                    .checked_add(second.map_or(1, BufferValue::len))
                    .filter(|n| *n <= i64::MAX as usize)
                    .ok_or_else(|| fault("resource.memory.exhausted"))?;
                self.spend_bytes(n)?;
                lists::charge(&mut self.heap, cells(n))?;
                let mut bytes = Vec::with_capacity(n);
                bytes.extend_from_slice(first.bytes());
                if let Some(b) = pushed {
                    bytes.push(b);
                } else {
                    bytes.extend_from_slice(second.unwrap().bytes());
                }
                Ok(wrap(BufferValue::from_bytes(bytes)))
            }
            ("Bytes", "length") => Ok(Value::I64(first.len() as i64)),
            ("Bytes", "at" | "slice") => {
                let start = integer(&values[1]);
                let count = if op == "at" { 1 } else { integer(&values[2]) };
                let (start, count) = range(start, count, first.len(), "runtime.bytes.bounds")?;
                if op == "at" {
                    Ok(Value::I64(first.bytes()[start] as i64))
                } else {
                    Ok(wrap(first.view(start, count)))
                }
            }
            ("Text", _) => {
                if op != "length" && (integer(&values[1]) < 0 || (op == "slice" && integer(&values[2]) < 0)) {
                    return Err(fault("runtime.text.bounds"));
                }
                self.spend_bytes(first.len())?;
                let text = std::str::from_utf8(first.bytes()).expect("checked UTF-8 text");
                let length = text.chars().count();
                if op == "length" {
                    return Ok(Value::I64(length as i64));
                }
                let count = if op == "at" { 1 } else { integer(&values[2]) };
                let (start, count) = range(integer(&values[1]), count, length, "runtime.text.bounds")?;
                if op == "at" {
                    return Ok(Value::I64(text.chars().nth(start).unwrap() as i64));
                }
                let begin = text.char_indices().nth(start).map_or(text.len(), |(i, _)| i);
                let end = text
                    .char_indices()
                    .nth(start + count)
                    .map_or(text.len(), |(i, _)| i);
                Ok(wrap(first.view(begin, end - begin)))
            }
            _ => unreachable!("checked buffer operation"),
        }
    }
}
