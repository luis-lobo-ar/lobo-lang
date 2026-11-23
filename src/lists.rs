use super::*;
use serde::ser::{SerializeSeq, SerializeStruct};
use std::sync::Arc;

#[derive(Clone)]
pub struct ListValue {
    pub element: Type,
    head: Option<Arc<Node>>,
    length: usize,
}

struct Node {
    value: Value,
    tail: Option<Arc<Node>>,
}

impl ListValue {
    pub fn empty(element: Type) -> Self {
        Self {
            element,
            head: None,
            length: 0,
        }
    }
    pub fn from_values(element: Type, elements: Vec<Value>) -> Self {
        let mut list = Self::empty(element);
        for value in elements.into_iter().rev() {
            list = list.prepend(value);
        }
        list
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        let mut node = self.head.as_deref();
        std::iter::from_fn(move || {
            let current = node?;
            node = current.tail.as_deref();
            Some(&current.value)
        })
    }
    pub(super) fn prepend(&self, value: Value) -> Self {
        Self {
            element: self.element.clone(),
            head: Some(Arc::new(Node {
                value,
                tail: self.head.clone(),
            })),
            length: self.length + 1,
        }
    }
    pub(super) fn head(&self) -> Fallible<Value> {
        self.head
            .as_ref()
            .map(|n| n.value.clone())
            .ok_or_else(empty_fault)
    }
    pub(super) fn tail(&self) -> Fallible<Self> {
        let node = self.head.as_ref().ok_or_else(empty_fault)?;
        Ok(Self {
            element: self.element.clone(),
            head: node.tail.clone(),
            length: self.length - 1,
        })
    }
}

fn empty_fault() -> Diagnostic {
    Diagnostic::new("runtime.list.empty", "execute", "empty list")
}

impl Drop for ListValue {
    fn drop(&mut self) {
        let mut nodes: Vec<_> = self.head.take().into_iter().collect();
        let mut values = Vec::new();
        while let Some(current) = nodes.pop() {
            if let Some(owned) = Arc::into_inner(current) {
                if let Some(tail) = owned.tail {
                    nodes.push(tail);
                }
                values.push(owned.value);
                while let Some(value) = values.pop() {
                    match value {
                        Value::List(mut list) => {
                            if let Some(head) = list.head.take() {
                                nodes.push(head);
                            }
                        }
                        Value::Record(record) => values.extend(record.fields.into_values()),
                        Value::Variant(variant) => {
                            if let Some(payload) = variant.payload {
                                values.push(*payload);
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
impl std::fmt::Debug for ListValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl PartialEq for ListValue {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element && self.length == other.length && self.iter().eq(other.iter())
    }
}
impl Eq for ListValue {}

impl Serialize for ListValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Elements<'a>(&'a ListValue);
        impl Serialize for Elements<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for value in self.0.iter() {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
        }
        let mut object = serializer.serialize_struct("ListValue", 2)?;
        object.serialize_field("element", &self.element)?;
        object.serialize_field("elements", &Elements(self))?;
        object.end()
    }
}
impl<'de> Deserialize<'de> for ListValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            element: Type,
            elements: Vec<Value>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Ok(Self::from_values(wire.element, wire.elements))
    }
}

pub(super) fn value_limit(value: &Value, remaining: &mut usize, code: &str) -> Fallible<()> {
    value_limit_inner(value, remaining, code, 0)
}
fn value_limit_inner(value: &Value, remaining: &mut usize, code: &str, depth: usize) -> Fallible<()> {
    if depth > 8 {
        return Err(Diagnostic::new(code, "invoke", "value nesting limit exceeded"));
    }
    *remaining = remaining
        .checked_sub(1)
        .ok_or_else(|| Diagnostic::new(code, "invoke", "value node limit exceeded"))?;
    match value {
        Value::List(list) => {
            for item in list.iter() {
                value_limit_inner(item, remaining, code, depth + 1)?;
            }
        }
        Value::Record(record) => {
            for field in record.fields.values() {
                value_limit_inner(field, remaining, code, depth + 1)?;
            }
        }
        Value::Variant(variant) => {
            if let Some(payload) = &variant.payload {
                value_limit_inner(payload, remaining, code, depth + 1)?;
            }
        }
        _ => (),
    }
    Ok(())
}

pub(super) fn output_limit(value: &Value) -> Fallible<()> {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len())
                .ok_or_else(|| std::io::Error::other("output byte limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    value_limit(value, &mut 100_000, "resource.output.limit")?;
    serde_json::to_writer(Counter(8 * 1024 * 1024), value)
        .map_err(|_| Diagnostic::new("resource.output.limit", "invoke", "output byte limit exceeded"))
}

pub(super) fn charge(remaining: &mut u64, cells: usize) -> Fallible<()> {
    *remaining = remaining.checked_sub(cells as u64).ok_or_else(|| {
        Diagnostic::new(
            "resource.memory.exhausted",
            "execute",
            "allocation allowance exhausted",
        )
    })?;
    Ok(())
}
