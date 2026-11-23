use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod buffers;
mod callbacks;
mod compact;
mod floats;
mod generics;
pub mod identities;
mod inference;
mod lists;
pub use buffers::BufferValue;
mod modules;
mod native;
mod native_api;
pub use native_api::ResourceKind;
mod operators;
mod propagation;
pub use native::NativeBackend;
mod records;
pub use lists::ListValue;
pub use modules::{compile_bundle, compile_bundle_json, SourceFile};

pub const LANGUAGE: &str = "core/1.11";
pub const FORMAT: &str = "lobo-checked/1.0";
pub const DEFAULT_HEAP_CELLS: u64 = 1_000_000;
const MAX_DEPTH: usize = 32;
pub const MAX_SOURCE_BYTES: usize = 1_048_576;
pub const MAX_BUNDLE_TOKENS: usize = 16_384;
const MAX_SOURCE: usize = MAX_SOURCE_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ByteSpan {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct SourcePosition {
    pub byte: usize,
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

fn source_position(source: &str, byte: usize) -> SourcePosition {
    let mut line = 0;
    let mut character = 0;
    let mut previous_cr = false;
    for scalar in source[..byte].chars() {
        if scalar == '\r' || (scalar == '\n' && !previous_cr) {
            line += 1;
            character = 0;
        } else if scalar != '\n' {
            character += scalar.len_utf16();
        }
        previous_cr = scalar == '\r';
    }
    SourcePosition {
        byte,
        line,
        character,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub phase: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Box<SourceRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Box<String>>,
    #[serde(skip)]
    byte_span: Option<Box<ByteSpan>>,
}

impl Diagnostic {
    fn new(code: &str, phase: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            phase: phase.into(),
            message: message.into(),
            subject: None,
            range: None,
            source: None,
            byte_span: None,
        }
    }
    fn at(mut self, span: Option<ByteSpan>) -> Self {
        if self.byte_span.is_none() {
            self.byte_span = span.map(Box::new);
        }
        self
    }
    fn locate(mut self, source: &str) -> Self {
        if let Some(span) = self.byte_span.take() {
            if span.start > span.end
                || span.end > source.len()
                || !source.is_char_boundary(span.start)
                || !source.is_char_boundary(span.end)
            {
                self.range = None;
                return self;
            }
            self.range = Some(Box::new(SourceRange {
                start: source_position(source, span.start),
                end: source_position(source, span.end),
            }));
        }
        self
    }
    pub fn kind(&self) -> &'static str {
        if self.code.starts_with("compile.") {
            "compile_error"
        } else if self.code.starts_with("runtime.") {
            "runtime_fault"
        } else if self.code.starts_with("resource.") {
            "resource_exhausted"
        } else if self.code.starts_with("internal.") {
            "internal_error"
        } else {
            "invocation_error"
        }
    }
}

type Fallible<T> = Result<T, Diagnostic>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Resource(ResourceKind),
    I64,
    F64,
    Bool,
    Bytes,
    Text,
    Record(String),
    Variant(String),
    List(Box<Type>),
    Applied { name: String, arguments: Vec<Type> },
    Parameter(String),
    Function(FunctionType),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct FunctionType {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    pub parameters: Vec<Type>,
    pub output: Box<Type>,
}
impl Function {
    fn signature_type(&self) -> FunctionType {
        FunctionType {
            actions: self.actions.clone(),
            parameters: self.parameters.iter().map(|p| p.ty.clone()).collect(),
            output: Box::new(self.output.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FunctionValue {
    pub id: String,
    pub signature: FunctionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", deny_unknown_fields)]
pub enum Value {
    #[serde(with = "floats::wire")]
    F64(u64),
    Function(FunctionValue),
    #[serde(with = "decimal")]
    I64(i64),
    Bool(bool),
    #[serde(with = "buffers::hex_wire")]
    Bytes(BufferValue),
    #[serde(with = "buffers::text_wire")]
    Text(BufferValue),
    Record(RecordValue),
    Variant(VariantValue),
    List(ListValue),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VariantValue {
    pub id: String,
    pub case: String,
    pub payload: Option<Box<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RecordValue {
    pub id: String,
    pub fields: BTreeMap<String, Value>,
}

mod decimal {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        let text = String::deserialize(deserializer)?;
        let value: i64 = text.parse().map_err(serde::de::Error::custom)?;
        if value.to_string() != text {
            return Err(serde::de::Error::custom("expected canonical decimal I64 string"));
        }
        Ok(value)
    }
}

impl Value {
    fn ty(&self) -> Type {
        match self {
            Self::Function(value) => Type::Function(value.signature.clone()),
            Self::F64(_) => Type::F64,
            Self::I64(_) => Type::I64,
            Self::Bool(_) => Type::Bool,
            Self::Bytes(_) => Type::Bytes,
            Self::Text(_) => Type::Text,
            Self::Record(value) => Type::Record(value.id.clone()),
            Self::Variant(value) => Type::Variant(value.id.clone()),
            Self::List(value) => Type::List(Box::new(value.element.clone())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub format: String,
    pub language: String,
    program: Program,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Program {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_abi: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sqlite_abi: Option<u32>,
    module: String,
    id: String,
    functions: Vec<Function>,
    records: Vec<Record>,
    variants: Vec<Variant>,
    #[serde(skip)]
    operators: Vec<operators::Binding>,
    #[serde(default)]
    scopes: Vec<modules::Scope>,
    #[serde(skip)]
    module_span: Option<ByteSpan>,
    #[serde(skip)]
    id_span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    name: String,
    id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    type_parameters: Vec<String>,
    fields: Vec<Parameter>,
    #[serde(skip)]
    span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Variant {
    name: String,
    id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    type_parameters: Vec<String>,
    cases: Vec<Case>,
    #[serde(skip)]
    span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    payload: Option<Type>,
    #[serde(skip)]
    span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchArm {
    case: String,
    binding: Option<String>,
    body: Block,
    #[serde(skip)]
    span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldValue {
    name: String,
    value: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Function {
    name: String,
    id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    type_parameters: Vec<String>,
    parameters: Vec<Parameter>,
    output: Type,
    actions: Vec<String>,
    requires: Vec<Expr>,
    ensures: Vec<Expr>,
    body: Block,
    #[serde(skip)]
    span: Option<ByteSpan>,
    #[serde(skip)]
    id_span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    name: String,
    ty: Type,
    #[serde(skip)]
    span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    bindings: Vec<Binding>,
    value: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    name: String,
    annotation: Option<Type>,
    value: Expr,
    #[serde(skip)]
    span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Expr {
    NativeOp {
        binding: String,
        abi: u32,
        arguments: Vec<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Reference {
        function: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_arguments: Vec<Type>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Invoke {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    BufferOp {
        family: String,
        op: String,
        arguments: Vec<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    List {
        element: Type,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    ListOp {
        op: String,
        arguments: Vec<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Variant {
        variant: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_arguments: Vec<Type>,
        case: String,
        payload: Option<Box<Expr>>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Record {
        record: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_arguments: Vec<Type>,
        fields: Vec<FieldValue>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "reject_update_base"
        )]
        base: Option<Box<Expr>>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Field {
        base: Box<Expr>,
        field: String,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Literal {
        value: Value,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Name {
        name: String,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Unary {
        op: String,
        operand: Box<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Call {
        function: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_arguments: Vec<Type>,
        arguments: Vec<Expr>,
        #[serde(skip)]
        span: Option<ByteSpan>,
        #[serde(skip)]
        name_span: Option<ByteSpan>,
    },
    If {
        condition: Box<Expr>,
        then_block: Block,
        else_block: Block,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
    Fold {
        index: String,
        accumulator: Parameter,
        count: Box<Expr>,
        initial: Box<Expr>,
        body: Block,
        #[serde(skip)]
        span: Option<ByteSpan>,
        #[serde(skip)]
        index_span: Option<ByteSpan>,
    },
    While {
        state: Parameter,
        initial: Box<Expr>,
        condition: Box<Expr>,
        body: Block,
        #[serde(skip)]
        span: Option<ByteSpan>,
    },
}

fn reject_update_base<'de, D: serde::Deserializer<'de>>(_d: D) -> Result<Option<Box<Expr>>, D::Error> {
    Err(serde::de::Error::custom("record update bases are source-only"))
}

impl Expr {
    fn span(&self) -> Option<ByteSpan> {
        match self {
            Self::Reference { span, .. } | Self::Invoke { span, .. } => *span,
            Self::NativeOp { span, .. }
            | Self::BufferOp { span, .. }
            | Self::List { span, .. }
            | Self::ListOp { span, .. }
            | Self::Variant { span, .. }
            | Self::Match { span, .. }
            | Self::Record { span, .. }
            | Self::Field { span, .. }
            | Self::Literal { span, .. }
            | Self::Name { span, .. }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Call { span, .. }
            | Self::If { span, .. }
            | Self::Fold { span, .. }
            | Self::While { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Number(String),
    Id(String),
    Text(String),
    Symbol(String),
    End,
}

fn syntax(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("compile.syntax.invalid", "parse", message)
}

fn binary_precedence(op: &str) -> Option<u8> {
    Some(match op {
        "||" => 0,
        "&&" => 1,
        "==" | "!=" => 2,
        "<" | "<=" | ">" | ">=" => 3,
        "+" | "-" => 4,
        "*" | "/" | "%" => 5,
        _ => return None,
    })
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && (name.as_bytes()[0].is_ascii_alphabetic() || name.starts_with('_'))
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

fn reserved(name: &str) -> bool {
    matches!(
        name,
        "module"
            | "function"
            | "let"
            | "if"
            | "else"
            | "requires"
            | "ensures"
            | "true"
            | "false"
            | "I64"
            | "Fn"
            | "F64"
            | "Bool"
            | "List"
            | "Bytes"
            | "Text"
            | "result"
    )
}

fn check_name(name: &str) -> Fallible<()> {
    if !valid_name(name) {
        return Err(syntax("invalid identifier"));
    }
    if reserved(name) {
        return Err(Diagnostic::new(
            "compile.name.reserved",
            "resolve",
            format!("reserved name: {name}"),
        ));
    }
    Ok(())
}

fn valid_action(action: &str) -> bool {
    action.len() <= 128
        && action.contains('.')
        && action.split('.').all(|part| valid_name(part) && !reserved(part))
}

fn action_error(code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, "check", message)
}

fn validate_actions(actions: &[String]) -> Fallible<()> {
    if actions.len() > 64 {
        return Err(Diagnostic::new(
            "resource.frontend.limit",
            "check",
            "action limit exceeded",
        ));
    }
    let mut seen = BTreeSet::new();
    for action in actions {
        if !valid_action(action) {
            return Err(action_error(
                "compile.action.invalid",
                "invalid qualified action ID",
            ));
        }
        if !seen.insert(action) {
            return Err(action_error("compile.action.duplicate", "repeated action ID"));
        }
    }
    if actions.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(action_error(
            "compile.action.invalid",
            "action IDs must be sorted",
        ));
    }
    Ok(())
}

fn lex(source: &str) -> Fallible<(Vec<Token>, Vec<ByteSpan>)> {
    if source.len() > MAX_SOURCE {
        return Err(Diagnostic::new(
            "resource.frontend.limit",
            "parse",
            "source size limit exceeded",
        ));
    }
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if b" \t\r\n".contains(&c) {
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && !b"\r\n".contains(&bytes[i]) {
                i += 1;
            }
            continue;
        }
        let start = i;
        if c.is_ascii_alphabetic() || c == b'_' {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            tokens.push(Token::Word(source[start..i].into()));
        } else if c.is_ascii_digit() {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
                i += 1;
                while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                    i += 1;
                }
            }
            if matches!(bytes.get(i), Some(b'e' | b'E')) {
                i += 1;
                if matches!(bytes.get(i), Some(b'+' | b'-')) {
                    i += 1;
                }
                let exponent = i;
                while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                    i += 1;
                }
                if i == exponent {
                    return Err(syntax("F64 exponent needs digits").at(Some(ByteSpan { start, end: i })));
                }
            }
            tokens.push(Token::Number(source[start..i].into()));
        } else if c == b'"' {
            i += 1;
            let content = i;
            let mut escaped = false;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    escaped = true;
                    i += 1;
                }
                i = (i + 1).min(bytes.len());
            }
            if i == bytes.len() {
                return Err(syntax("unterminated quoted string").at(Some(ByteSpan { start, end: i })));
            }
            let text: String = serde_json::from_str(&source[start..i + 1])
                .map_err(|_| syntax("invalid quoted string").at(Some(ByteSpan { start, end: i + 1 })))?;
            tokens.push(if escaped {
                Token::Text(text)
            } else {
                Token::Id(source[content..i].into())
            });
            i += 1;
        } else {
            let two = bytes.get(i..i + 2);
            if bytes.get(i..i + 3) == Some(b"...") {
                tokens.push(Token::Symbol("...".into()));
                i += 3;
            } else if matches!(
                two,
                Some(b"::" | b"->" | b"=>" | b"==" | b"!=" | b"<=" | b">=" | b"&&" | b"||")
            ) {
                tokens.push(Token::Symbol(source[i..i + 2].into()));
                i += 2;
            } else if b"@();,:{}.+-*/%<>!&".contains(&c) || c == b'=' {
                tokens.push(Token::Symbol((c as char).to_string()));
                i += 1;
            } else {
                let end = i + source[i..].chars().next().unwrap().len_utf8();
                return Err(
                    syntax(format!("unexpected byte at offset {i}")).at(Some(ByteSpan { start: i, end }))
                );
            }
        }
        spans.push(ByteSpan { start, end: i });
        if tokens.len() > 8192 {
            return Err(Diagnostic::new(
                "resource.frontend.limit",
                "parse",
                "token limit exceeded",
            ));
        }
    }
    tokens.push(Token::End);
    spans.push(ByteSpan {
        start: source.len(),
        end: source.len(),
    });
    Ok((tokens, spans))
}

struct Parser {
    tokens: Vec<Token>,
    spans: Vec<ByteSpan>,
    position: usize,
    depth: usize,
    compact: Option<compact::Syntax>,
    managed: Option<BTreeMap<String, String>>,
    statement_counter: usize,
}

impl Parser {
    fn span(&self) -> ByteSpan {
        self.spans[self.position]
    }
    fn through(&self, start: usize) -> Option<ByteSpan> {
        Some(ByteSpan {
            start,
            end: self.spans[self.position.saturating_sub(1)].end,
        })
    }
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }
    fn is(&self, text: &str) -> bool {
        matches!(self.current(), Token::Word(s) | Token::Symbol(s) if s == text)
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.is(text) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Fallible<()> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(syntax(format!("expected {text}, got {:?}", self.current())).at(Some(self.span())))
        }
    }
    fn name(&mut self) -> Fallible<String> {
        if let Token::Word(name) = self.current().clone() {
            if self
                .compact
                .as_ref()
                .is_some_and(|c| c.namespaces.contains_key(&name))
            {
                return Err(
                    syntax("import group names are reserved throughout modern source").at(Some(self.span()))
                );
            }
            check_name(&name).map_err(|e| e.at(Some(self.span())))?;
            if name.starts_with("__lobo_") {
                return Err(action_error(
                    "compile.name.reserved",
                    "__lobo_ names are reserved for compiler lowering",
                )
                .at(Some(self.span())));
            }
            self.position += 1;
            Ok(name)
        } else {
            Err(syntax("expected identifier").at(Some(self.span())))
        }
    }
    fn identity(&mut self) -> Fallible<String> {
        self.expect("@")?;
        self.expect("id")?;
        self.expect("(")?;
        let Token::Id(id) = self.current().clone() else {
            return Err(syntax("expected unescaped identity string").at(Some(self.span())));
        };
        if !valid_id(&id) {
            return Err(syntax("invalid stable identity").at(Some(self.span())));
        }
        self.position += 1;
        self.expect(")")?;
        Ok(id)
    }
    fn ty(&mut self) -> Fallible<Type> {
        self.ty_inner(0)
    }
    fn ty_inner(&mut self, depth: usize) -> Fallible<Type> {
        if depth > 8 {
            return Err(
                Diagnostic::new("resource.frontend.limit", "parse", "type nesting limit exceeded")
                    .at(Some(self.span())),
            );
        }
        if self.compact.is_some()
            && self.is("Native")
            && self.tokens.get(self.position + 1) == Some(&Token::Symbol("::".into()))
            && self.eat("Native")
        {
            self.compact.as_mut().unwrap().native = true;
            self.expect("::")?;
            let name = self.name()?;
            self.compact.as_mut().unwrap().sqlite |= native_api::sqlite_type_name(&name);
            return native_api::named_type(&name);
        }
        if self.eat("Fn") {
            self.expect("(")?;
            let mut parameters = Vec::new();
            if !self.is(")") {
                loop {
                    if parameters.len() == 16 {
                        return Err(action_error(
                            "resource.frontend.limit",
                            "at most sixteen function parameters",
                        ));
                    }
                    parameters.push(self.ty_inner(depth + 1)?);
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.expect(")")?;
            let actions = self.action_clause()?;
            self.expect("->")?;
            return Ok(Type::Function(FunctionType {
                actions,
                parameters,
                output: Box::new(self.ty_inner(depth + 1)?),
            }));
        }
        if self.eat("List") {
            self.expect("<")?;
            let element = self.ty_inner(depth + 1)?;
            self.type_close()?;
            return Ok(Type::List(Box::new(element)));
        }
        if self.eat("I64") {
            Ok(Type::I64)
        } else if self.eat("F64") {
            Ok(Type::F64)
        } else if self.eat("Bool") {
            Ok(Type::Bool)
        } else if self.eat("Bytes") {
            Ok(Type::Bytes)
        } else if self.eat("Text") {
            Ok(Type::Text)
        } else {
            let (name, _) = self.reference_name()?;
            if self.eat("<") {
                let mut arguments = vec![self.ty_inner(depth + 1)?];
                while self.eat(",") {
                    if arguments.len() == 8 {
                        return Err(action_error(
                            "resource.frontend.limit",
                            "at most eight type arguments",
                        ));
                    }
                    arguments.push(self.ty_inner(depth + 1)?);
                }
                self.type_close()?;
                Ok(Type::Applied { name, arguments })
            } else {
                Ok(Type::Record(name))
            }
        }
    }
    fn type_parameters(&mut self) -> Fallible<Vec<String>> {
        let mut parameters = Vec::new();
        if self.eat("<") {
            loop {
                let parameter = self.name()?;
                if parameters.contains(&parameter) {
                    return Err(action_error("compile.name.duplicate", "repeated type parameter")
                        .at(Some(self.span())));
                }
                if parameters.len() == 8 {
                    return Err(
                        action_error("resource.frontend.limit", "at most eight type parameters")
                            .at(Some(self.span())),
                    );
                }
                parameters.push(parameter);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(">")?;
        }
        Ok(parameters)
    }
    fn type_close(&mut self) -> Fallible<()> {
        if self.is(">=") {
            self.tokens[self.position] = Token::Symbol("=".into());
            self.spans[self.position].start += 1;
            Ok(())
        } else {
            self.expect(">")
        }
    }
    fn program(&mut self) -> Fallible<Program> {
        self.expect("module")?;
        let module_span = Some(self.span());
        let module = self.name()?;
        if self.is(";") {
            self.compact = Some(self.compact_metadata()?);
        }
        let (id, id_span) = self.declaration_identity("module")?;
        self.expect(";")?;
        let mut functions = Vec::new();
        let mut records = Vec::new();
        let mut variants = Vec::new();
        let mut imports = Vec::new();
        let mut operators = Vec::new();
        while self.current() != &Token::End && self.compact.as_ref().is_none_or(|c| self.position != c.end) {
            if self.compact.is_some() && self.eat("use") {
                if !functions.is_empty()
                    || !records.is_empty()
                    || !variants.is_empty()
                    || !operators.is_empty()
                {
                    return Err(syntax("use groups must precede declarations").at(Some(self.span())));
                }
                imports.extend(self.import_group()?);
            } else if self.compact.is_none() && self.eat("import") {
                let span = Some(self.span());
                let name = self.name()?;
                let id = self.identity()?;
                self.expect(";")?;
                imports.push(modules::Import { name, id, span });
            } else if self.eat("record") {
                records.push(self.record()?);
            } else if self.eat("variant") {
                variants.push(self.variant()?);
            } else if self.eat("operator") {
                if operators.len() == 64 {
                    return Err(action_error(
                        "resource.frontend.limit",
                        "at most 64 operator bindings",
                    ));
                }
                operators.push(self.operator(&id)?);
            } else {
                functions.push(self.function()?);
            }
        }
        self.finish_compact()?;
        if let Some(compact) = &self.compact {
            imports.extend(compact.generated.iter().cloned());
        }
        let declarations = functions
            .iter()
            .map(|f: &Function| f.id.clone())
            .chain(records.iter().map(|r: &Record| r.id.clone()))
            .chain(variants.iter().map(|v: &Variant| v.id.clone()))
            .collect();
        let scopes = vec![modules::Scope {
            name: module.clone(),
            id: id.clone(),
            declarations,
            imports,
        }];
        Ok(Program {
            sqlite_abi: self
                .compact
                .as_ref()
                .and_then(|syntax| syntax.sqlite.then_some(1)),
            native_abi: self
                .compact
                .as_ref()
                .and_then(|syntax| syntax.native.then_some(native_api::ABI)),
            module,
            id,
            functions,
            records,
            variants,
            operators,
            scopes,
            module_span,
            id_span,
        })
    }
    fn operator(&mut self, module: &str) -> Fallible<operators::Binding> {
        let span = Some(self.span());
        let Token::Symbol(symbol) = self.current().clone() else {
            return Err(
                action_error("compile.operator.symbol", "expected binary arithmetic symbol").at(span),
            );
        };
        if !operators::eligible(&symbol) {
            return Err(action_error("compile.operator.symbol", "expected +, -, *, / or %").at(span));
        }
        self.position += 1;
        self.expect("(")?;
        let left = self.ty()?;
        self.expect(",")?;
        let right = self.ty()?;
        self.expect(")")?;
        self.expect("->")?;
        let output = self.ty()?;
        self.expect("=")?;
        let target_span = Some(self.span());
        let target = self.name()?;
        self.expect(";")?;
        Ok(operators::Binding {
            module: module.into(),
            symbol,
            left,
            right,
            output,
            target,
            span,
            target_span,
        })
    }
    fn variant(&mut self) -> Fallible<Variant> {
        let span = Some(self.span());
        let name = self.name()?;
        let type_parameters = self.type_parameters()?;
        let (id, _) = self.declaration_identity(&name)?;
        self.expect("{")?;
        let mut cases = Vec::new();
        loop {
            let span = Some(self.span());
            let name = self.name()?;
            let payload = if self.eat("(") {
                let ty = self.ty()?;
                self.expect(")")?;
                Some(ty)
            } else {
                None
            };
            cases.push(Case { name, payload, span });
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        Ok(Variant {
            name,
            id,
            type_parameters,
            cases,
            span,
        })
    }

    fn is_match(&self) -> bool {
        if self.compact.is_some() {
            return self.is("match");
        }
        if !self.is("match") || self.tokens.get(self.position + 1) != Some(&Token::Symbol("(".into())) {
            return false;
        }
        let mut parentheses = 0;
        for (offset, token) in self.tokens[self.position + 1..].iter().enumerate() {
            if token == &Token::Symbol("(".into()) {
                parentheses += 1;
            }
            if token == &Token::Symbol(")".into()) {
                parentheses -= 1;
                if parentheses == 0 {
                    return self.tokens.get(self.position + offset + 2) == Some(&Token::Symbol("{".into()));
                }
            }
        }
        false
    }
    fn record(&mut self) -> Fallible<Record> {
        let span = Some(self.span());
        let name = self.name()?;
        let type_parameters = self.type_parameters()?;
        let (id, _) = self.declaration_identity(&name)?;
        self.expect("{")?;
        let mut fields = Vec::new();
        loop {
            let span = Some(self.span());
            let name = self.name()?;
            self.expect(":")?;
            fields.push(Parameter {
                name,
                ty: self.ty()?,
                span,
            });
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        Ok(Record {
            name,
            id,
            type_parameters,
            fields,
            span,
        })
    }
    fn function(&mut self) -> Fallible<Function> {
        self.statement_counter = 0;
        self.expect(if self.compact.is_some() { "fn" } else { "function" })?;
        let span = Some(self.span());
        let name = self.name()?;
        let mut type_parameters = Vec::new();
        if self.eat("<") {
            loop {
                let parameter = self.name()?;
                if type_parameters.contains(&parameter) {
                    return Err(action_error("compile.name.duplicate", "repeated type parameter").at(span));
                }
                if type_parameters.len() == 8 {
                    return Err(
                        action_error("resource.frontend.limit", "at most eight type parameters").at(span),
                    );
                }
                type_parameters.push(parameter);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(">")?;
        }
        let (id, id_span) = self.declaration_identity(&name)?;
        self.expect("(")?;
        let mut parameters = Vec::new();
        if !self.is(")") {
            loop {
                let span = Some(self.span());
                let name = self.name()?;
                self.expect(":")?;
                parameters.push(Parameter {
                    name,
                    ty: self.ty()?,
                    span,
                });
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        self.expect("->")?;
        let output = self.ty()?;
        let actions = self.action_clause()?;
        let mut requires = Vec::new();
        let mut ensures = Vec::new();
        while self.eat("requires") {
            self.expect("{")?;
            requires.push(self.expr(0)?);
            self.expect("}")?;
        }
        while self.eat("ensures") {
            self.expect("{")?;
            ensures.push(self.expr(0)?);
            self.expect("}")?;
        }
        let body = self.block()?;
        Ok(Function {
            name,
            id,
            type_parameters,
            parameters,
            output,
            actions,
            requires,
            ensures,
            body,
            span,
            id_span,
        })
    }
    fn action_clause(&mut self) -> Fallible<Vec<String>> {
        if !self.eat("actions") {
            return Ok(Vec::new());
        }
        self.expect("{")?;
        let mut actions = BTreeSet::new();
        if !self.is("}") {
            loop {
                let start = self.span().start;
                let mut action = String::new();
                loop {
                    let Token::Word(part) = self.current().clone() else {
                        return Err(syntax("expected action ID segment").at(Some(self.span())));
                    };
                    action.push_str(&part);
                    self.position += 1;
                    if !self.eat(".") {
                        break;
                    }
                    action.push('.');
                }
                if !valid_action(&action) {
                    return Err(
                        action_error("compile.action.invalid", "expected qualified action ID")
                            .at(self.through(start)),
                    );
                }
                if !actions.insert(action) {
                    return Err(action_error("compile.action.duplicate", "repeated action ID")
                        .at(self.through(start)));
                }
                if actions.len() > 64 {
                    return Err(
                        Diagnostic::new("resource.frontend.limit", "parse", "action limit exceeded")
                            .at(self.through(start)),
                    );
                }
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect("}")?;
        Ok(actions.into_iter().collect())
    }
    fn let_binding(&mut self) -> Fallible<Binding> {
        let span = Some(self.span());
        let name = self.name()?;
        let annotation = if self.eat(":") { Some(self.ty()?) } else { None };
        self.expect("=")?;
        let value = self.expr(0)?;
        self.expect(";")?;
        Ok(Binding {
            name,
            annotation,
            value,
            span,
        })
    }
    fn block(&mut self) -> Fallible<Block> {
        self.expect("{")?;
        let mut bindings = Vec::new();
        loop {
            if self.eat("let") {
                bindings.push(self.let_binding()?);
                continue;
            }
            let value = Box::new(self.expr(0)?);
            if self.compact.is_some()
                && self.is(";")
                && matches!(value.as_ref(),Expr::Unary {op,..} if op=="try")
            {
                self.position += 1;
                let span = value.span();
                let name = format!("__lobo_statement_{}", self.statement_counter);
                self.statement_counter += 1;
                bindings.push(Binding {
                    name,
                    annotation: None,
                    value: *value,
                    span,
                });
                continue;
            }
            self.expect("}")?;
            return Ok(Block { bindings, value });
        }
    }
    fn expr(&mut self, level: u8) -> Fallible<Expr> {
        let start = self.span().start;
        // Precedence climbing avoids retaining seven large debug frames per nested expression.
        let mut left = self.unary()?;
        let mut count = 0;
        while let Token::Symbol(s) = self.current() {
            let op = s.clone();
            let Some(precedence) = binary_precedence(&op) else {
                break;
            };
            if precedence < level {
                break;
            }
            self.position += 1;
            let right = self.expr(precedence + 1)?;
            if matches!(precedence, 2 | 3)
                && matches!(self.current(), Token::Symbol(next) if binary_precedence(next) == Some(precedence))
            {
                return Err(syntax("comparison/equality chains require parentheses").at(Some(self.span())));
            }
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.through(start),
            };
            count += 1;
            if count > MAX_DEPTH {
                return Err(Diagnostic::new(
                    "resource.frontend.limit",
                    "parse",
                    "expression depth limit exceeded",
                ));
            }
        }
        Ok(left)
    }
    fn unary(&mut self) -> Fallible<Expr> {
        let start = self.span().start;
        let span = Some(self.span());
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Diagnostic::new(
                "resource.frontend.limit",
                "parse",
                "nesting limit exceeded",
            ));
        }
        let result = (|| {
            let mut value = self.unary_inner()?;
            let mut fields = 0;
            while self.is(".") || self.is("(") {
                fields += 1;
                if fields > MAX_DEPTH {
                    return Err(Diagnostic::new(
                        "resource.frontend.limit",
                        "parse",
                        "field chain limit exceeded",
                    )
                    .at(self.through(start)));
                }
                if self.eat(".") {
                    let field = self.name()?;
                    value = Expr::Field {
                        base: Box::new(value),
                        field,
                        span: self.through(start),
                    };
                } else {
                    self.expect("(")?;
                    let mut arguments = Vec::new();
                    if !self.is(")") {
                        loop {
                            arguments.push(self.expr(0)?);
                            if !self.eat(",") {
                                break;
                            }
                        }
                    }
                    self.expect(")")?;
                    value = Expr::Invoke {
                        callee: Box::new(value),
                        arguments,
                        span: self.through(start),
                    };
                }
            }
            Ok(value)
        })()
        .map_err(|e: Diagnostic| e.at(span));
        self.depth -= 1;
        result
    }
    fn while_expression(&mut self) -> Fallible<Expr> {
        let start = self.span().start;
        self.position += 2;
        let state_span = Some(self.span());
        let name = self.name()?;
        self.expect(":")?;
        let ty = self.ty()?;
        self.expect("=")?;
        let initial = Box::new(self.expr(0)?);
        self.expect(";")?;
        let condition = Box::new(self.expr(0)?);
        self.expect(")")?;
        let body = self.block()?;
        Ok(Expr::While {
            state: Parameter {
                name,
                ty,
                span: state_span,
            },
            initial,
            condition,
            body,
            span: self.through(start),
        })
    }
    fn record_value(&mut self, name: String, type_arguments: Vec<Type>, start: usize) -> Fallible<Expr> {
        self.position += 1;
        let base = if self.eat("...") {
            if self.compact.is_none() {
                return Err(syntax("record updates require modern source"));
            }
            Some(Box::new(self.expr(0)?))
        } else {
            None
        };
        let has_fields = if base.is_some() { self.eat(",") } else { true };
        let mut fields = Vec::new();
        if has_fields {
            loop {
                let name = self.name()?;
                self.expect(":")?;
                fields.push(FieldValue {
                    name,
                    value: self.expr(0)?,
                });
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect("}")?;
        Ok(Expr::Record {
            record: name,
            type_arguments,
            fields,
            base,
            span: self.through(start),
        })
    }
    fn unary_inner(&mut self) -> Fallible<Expr> {
        let start = self.span().start;
        let try_operand = matches!(
            self.tokens.get(self.position + 1),
            Some(Token::Word(_) | Token::Number(_) | Token::Text(_) | Token::Id(_))
        );
        if self.is("try") && try_operand {
            self.position += 1;
            return Ok(Expr::Unary {
                op: "try".into(),
                operand: Box::new(self.unary()?),
                span: self.through(start),
            });
        }
        if self.eat("&") {
            let (function, _) = self.reference_name()?;
            let mut type_arguments = Vec::new();
            if self.eat("::") {
                self.expect("<")?;
                loop {
                    if type_arguments.len() == 8 {
                        return Err(action_error(
                            "resource.frontend.limit",
                            "at most eight type arguments",
                        ));
                    }
                    type_arguments.push(self.ty()?);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(">")?;
            }
            return Ok(Expr::Reference {
                function,
                type_arguments,
                span: self.through(start),
            });
        }
        if self.is("Bytes")
            || self.is("Text")
            || self.is("F64")
            || (self.compact.is_some()
                && self.is("Native")
                && self.tokens.get(self.position + 1) == Some(&Token::Symbol("::".into())))
        {
            let Token::Word(family) = self.current().clone() else {
                unreachable!()
            };
            self.position += 1;
            self.expect("::")?;
            let op = self.name()?;
            self.expect("(")?;
            let mut arguments = Vec::new();
            if !self.is(")") {
                loop {
                    arguments.push(self.expr(0)?);
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.expect(")")?;
            if family == "Native" {
                self.compact.as_mut().unwrap().native = true;
                self.compact.as_mut().unwrap().sqlite |= op.starts_with("sqlite_");
                return Ok(Expr::NativeOp {
                    binding: format!("lobo.native.{op}"),
                    abi: native_api::ABI,
                    arguments,
                    span: self.through(start),
                });
            }
            return Ok(Expr::BufferOp {
                family,
                op,
                arguments,
                span: self.through(start),
            });
        }
        if let Token::Id(text) | Token::Text(text) = self.current().clone() {
            self.position += 1;
            return Ok(Expr::Literal {
                value: Value::Text(BufferValue::from_bytes(text.into_bytes())),
                span: self.through(start),
            });
        }
        if self.eat("List") {
            self.expect("::")?;
            let op = self.name()?;
            if op == "empty" {
                self.expect("<")?;
                let element = self.ty()?;
                self.expect(">")?;
                self.expect("(")?;
                self.expect(")")?;
                return Ok(Expr::List {
                    element,
                    span: self.through(start),
                });
            }
            self.expect("(")?;
            let mut arguments = Vec::new();
            if !self.is(")") {
                loop {
                    arguments.push(self.expr(0)?);
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.expect(")")?;
            return Ok(Expr::ListOp {
                op,
                arguments,
                span: self.through(start),
            });
        }
        if self.is_match() {
            let modern = self.compact.is_some();
            self.position += 1;
            let subject = if self.eat("(") {
                let subject = Box::new(self.expr(0)?);
                self.expect(")")?;
                subject
            } else {
                let subject = Box::new(self.unary()?);
                if !matches!(
                    *subject,
                    Expr::Name { .. } | Expr::Call { .. } | Expr::NativeOp { .. }
                ) {
                    return Err(syntax("complex match subjects require parentheses"));
                }
                subject
            };
            self.expect("{")?;
            let mut arms = Vec::new();
            if !self.is("}") {
                loop {
                    let span = Some(self.span());
                    let case = self.name()?;
                    let binding = if self.eat("(") {
                        let name = self.name()?;
                        self.expect(")")?;
                        Some(name)
                    } else {
                        None
                    };
                    let body = if modern {
                        self.expect("=>")?;
                        if self.is("{") {
                            self.block()?
                        } else {
                            Block {
                                bindings: vec![],
                                value: Box::new(self.expr(0)?),
                            }
                        }
                    } else {
                        self.block()?
                    };
                    arms.push(MatchArm {
                        case,
                        binding,
                        body,
                        span,
                    });
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.expect("}")?;
            return Ok(Expr::Match {
                subject,
                arms,
                span: self.through(start),
            });
        }
        if self.eat("-") {
            if let Token::Number(text) = self.current().clone() {
                self.position += 1;
                if text.contains(['.', 'e', 'E']) {
                    return Ok(Expr::Literal {
                        value: floats::literal(&format!("-{text}")).map_err(|e| e.at(self.through(start)))?,
                        span: self.through(start),
                    });
                }
                let value = format!("-{text}").parse::<i64>().map_err(|_| {
                    Diagnostic::new("compile.literal.range", "parse", "negative literal outside I64")
                        .at(self.through(start))
                })?;
                return Ok(Expr::Literal {
                    value: Value::I64(value),
                    span: self.through(start),
                });
            }
            return Ok(Expr::Unary {
                op: "-".into(),
                operand: Box::new(self.unary()?),
                span: self.through(start),
            });
        }
        if self.eat("!") {
            return Ok(Expr::Unary {
                op: "!".into(),
                operand: Box::new(self.unary()?),
                span: self.through(start),
            });
        }
        if self.eat("(") {
            let value = self.expr(0)?;
            self.expect(")")?;
            return Ok(value);
        }
        // Typed-state prefixes commit to loops; ordinary while(value) remains a call.
        if self.is("while")
            && self.tokens.get(self.position + 1) == Some(&Token::Symbol("(".into()))
            && matches!(self.tokens.get(self.position + 2), Some(Token::Word(_)))
            && self.tokens.get(self.position + 3) == Some(&Token::Symbol(":".into()))
        {
            return self.while_expression();
        }
        // Contextual syntax preserves ordinary calls/names such as fold(in).
        if self.is("fold")
            && self.tokens.get(self.position + 1) == Some(&Token::Symbol("(".into()))
            && matches!(self.tokens.get(self.position + 2), Some(Token::Word(_)))
            && self.tokens.get(self.position + 3) == Some(&Token::Word("in".into()))
        {
            self.position += 2;
            let index_span = Some(self.span());
            let index = self.name()?;
            self.expect("in")?;
            let count = Box::new(self.expr(0)?);
            self.expect(",")?;
            let accumulator_span = Some(self.span());
            let name = self.name()?;
            self.expect(":")?;
            let ty = self.ty()?;
            self.expect("=")?;
            let initial = Box::new(self.expr(0)?);
            self.expect(")")?;
            let body = self.block()?;
            return Ok(Expr::Fold {
                index,
                accumulator: Parameter {
                    name,
                    ty,
                    span: accumulator_span,
                },
                count,
                initial,
                body,
                span: self.through(start),
                index_span,
            });
        }
        if self.eat("if") {
            let condition = Box::new(self.expr(0)?);
            let then_block = self.block()?;
            self.expect("else")?;
            let else_block = self.block()?;
            return Ok(Expr::If {
                condition,
                then_block,
                else_block,
                span: self.through(start),
            });
        }
        if self.eat("true") {
            return Ok(Expr::Literal {
                value: Value::Bool(true),
                span: self.through(start),
            });
        }
        if self.eat("false") {
            return Ok(Expr::Literal {
                value: Value::Bool(false),
                span: self.through(start),
            });
        }
        if self.compact.is_some()
            && self.is("json")
            && self.tokens.get(self.position + 1) == Some(&Token::Symbol("{".into()))
        {
            return self.json_object();
        }
        if let Token::Number(text) = self.current().clone() {
            self.position += 1;
            if text.contains(['.', 'e', 'E']) {
                return Ok(Expr::Literal {
                    value: floats::literal(&text).map_err(|e| e.at(self.through(start)))?,
                    span: self.through(start),
                });
            }
            let value = text.parse::<i64>().map_err(|_| {
                Diagnostic::new("compile.literal.range", "parse", "literal outside I64")
                    .at(self.through(start))
            })?;
            return Ok(Expr::Literal {
                value: Value::I64(value),
                span: self.through(start),
            });
        }
        if let Token::Word(word) = self.current().clone() {
            let (name, name_span) = if word == "result" {
                let span = Some(self.span());
                self.position += 1;
                (word, span)
            } else {
                self.reference_name()?
            };
            let mut type_arguments = Vec::new();
            if self.is("::") && self.tokens.get(self.position + 1) == Some(&Token::Symbol("<".into())) {
                self.position += 2;
                loop {
                    if type_arguments.len() == 8 {
                        return Err(
                            action_error("resource.frontend.limit", "at most eight type arguments")
                                .at(self.through(start)),
                        );
                    }
                    type_arguments.push(self.ty()?);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(">")?;
                if !self.is("(") && !self.is("{") && !self.is("::") {
                    return Err(syntax("generic application needs a call or constructor"));
                }
            }
            if self.eat("::") {
                let case = self.name()?;
                let payload = if self.eat("(") {
                    let value = Box::new(self.expr(0)?);
                    self.expect(")")?;
                    Some(value)
                } else {
                    None
                };
                return Ok(Expr::Variant {
                    variant: name,
                    type_arguments,
                    case,
                    payload,
                    span: self.through(start),
                });
            }
            if self.is("{")
                && (self.tokens.get(self.position + 1) == Some(&Token::Symbol("...".into()))
                    || (matches!(self.tokens.get(self.position + 1), Some(Token::Word(_)))
                        && self.tokens.get(self.position + 2) == Some(&Token::Symbol(":".into()))))
            {
                return self.record_value(name, type_arguments, start);
            }
            if !self.eat("(") {
                return Ok(Expr::Name {
                    name,
                    span: self.through(start),
                });
            }
            let mut arguments = Vec::new();
            if !self.is(")") {
                loop {
                    arguments.push(self.expr(0)?);
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.expect(")")?;
            return Ok(Expr::Call {
                function: name,
                type_arguments,
                arguments,
                span: self.through(start),
                name_span,
            });
        }
        Err(syntax("expected expression"))
    }
}

struct Checker<'a> {
    functions: BTreeMap<String, &'a Function>,
    names: BTreeSet<String>,
    visible: BTreeSet<String>,
    depth: usize,
    actions: BTreeSet<String>,
    in_contract: bool,
    output: Type,
    records: records::RecordTable<'a>,
    record_cells: usize,
}

fn mismatch(message: &str) -> Diagnostic {
    Diagnostic::new("compile.type.mismatch", "check", message)
}
fn same(actual: Type, expected: Type) -> Fallible<()> {
    if matches!((&actual, &expected), (Type::Function(a), Type::Function(b)) if a.actions != b.actions) {
        return Err(action_error(
            "compile.action.reference",
            "callback action bounds must match exactly",
        ));
    }
    if actual == expected {
        Ok(())
    } else {
        Err(mismatch("types must match exactly"))
    }
}

impl Checker<'_> {
    fn check_type(&self, ty: &Type) -> Fallible<()> {
        self.records.check_type(ty)?;
        modules::check_type(ty, &self.visible)
    }
    fn block(&mut self, block: &Block, outer: &BTreeMap<String, Type>) -> Fallible<Type> {
        let mut env = outer.clone();
        for binding in &block.bindings {
            check_name(&binding.name).map_err(|e| e.at(binding.span))?;
            if env.contains_key(&binding.name) || self.names.contains(&binding.name) {
                return Err(Diagnostic::new(
                    "compile.name.duplicate",
                    "resolve",
                    "binding shadows a visible name",
                )
                .at(binding.span));
            }
            let ty = self.expr(&binding.value, &env)?;
            if let Some(annotation) = &binding.annotation {
                self.check_type(annotation)?;
                same(ty.clone(), annotation.clone()).map_err(|e| e.at(binding.value.span()))?;
            }
            env.insert(binding.name.clone(), ty);
        }
        self.expr(&block.value, &env)
    }
    fn expr(&mut self, expression: &Expr, env: &BTreeMap<String, Type>) -> Fallible<Type> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Diagnostic::new(
                "resource.frontend.limit",
                "check",
                "expression nesting limit exceeded",
            ));
        }
        let result = self
            .expr_inner(expression, env)
            .and_then(|ty| {
                if !matches!(ty, Type::I64 | Type::Bool) {
                    self.record_cells += self.records.width(&ty);
                }
                if self.record_cells > 2048 {
                    return Err(Diagnostic::new(
                        "resource.frontend.limit",
                        "check",
                        "record temporary budget exceeded",
                    ));
                }
                Ok(ty)
            })
            .map_err(|e| e.at(expression.span()));
        self.depth -= 1;
        result
    }
    fn expr_inner(&mut self, expression: &Expr, env: &BTreeMap<String, Type>) -> Fallible<Type> {
        match expression {
            Expr::Reference {
                function,
                type_arguments,
                ..
            } => {
                if !type_arguments.is_empty() {
                    return Err(action_error(
                        "compile.generic.unlowered",
                        "reference type arguments must be lowered",
                    ));
                }
                let target = self
                    .functions
                    .get(function)
                    .filter(|_| self.visible.contains(function))
                    .ok_or_else(|| action_error("compile.name.unresolved", "unknown function reference"))?;
                let ty = Type::Function(target.signature_type());
                self.check_type(&ty)?;
                Ok(ty)
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                let Type::Function(signature) = self.expr(callee, env)? else {
                    return Err(action_error(
                        "compile.type.mismatch",
                        "invocation requires a function value",
                    ));
                };
                if signature.parameters.len() != arguments.len() {
                    return Err(action_error("compile.call.arity", "wrong number of arguments"));
                }
                if self.in_contract && !signature.actions.is_empty() {
                    return Err(action_error(
                        "compile.action.contract",
                        "contracts cannot invoke actionful callbacks",
                    ));
                }
                if signature
                    .actions
                    .iter()
                    .any(|action| !self.actions.contains(action))
                {
                    return Err(action_error(
                        "compile.action.undeclared",
                        "callback invocation requires declared actions",
                    ));
                }
                for (argument, ty) in arguments.iter().zip(&signature.parameters) {
                    same(self.expr(argument, env)?, ty.clone()).map_err(|e| e.at(argument.span()))?;
                }
                Ok(*signature.output)
            }
            Expr::NativeOp {
                binding,
                abi,
                arguments,
                ..
            } => {
                let signature = native_api::signature(binding, *abi)?;
                if arguments.len() != signature.parameters.len() {
                    return Err(action_error("compile.call.arity", "wrong native operation arity"));
                }
                for (argument, ty) in arguments.iter().zip(signature.parameters) {
                    same(self.expr(argument, env)?, ty).map_err(|e| e.at(argument.span()))?;
                }
                if self.in_contract && !signature.actions.is_empty() {
                    return Err(action_error(
                        "compile.action.contract",
                        "contracts require pure native operations",
                    ));
                }
                if signature
                    .actions
                    .iter()
                    .any(|action| !self.actions.contains(*action))
                {
                    return Err(action_error(
                        "compile.action.undeclared",
                        "native operation requires declared actions",
                    ));
                }
                Ok(signature.output)
            }
            Expr::BufferOp {
                family,
                op,
                arguments,
                ..
            } => {
                let (types, output) = buffers::signature(family, op)?;
                if types.len() != arguments.len() {
                    return Err(action_error(
                        "compile.buffer.invalid",
                        "wrong buffer operation arity",
                    ));
                }
                for (argument, ty) in arguments.iter().zip(types) {
                    same(self.expr(argument, env)?, ty).map_err(|e| e.at(argument.span()))?;
                }
                Ok(output)
            }
            Expr::List { element, .. } => {
                let ty = Type::List(Box::new(element.clone()));
                self.check_type(&ty)?;
                Ok(ty)
            }
            Expr::ListOp { op, arguments, .. } => {
                let arity = match op.as_str() {
                    "prepend" => 2,
                    "head" | "tail" | "length" => 1,
                    _ => 0,
                };
                if arity == 0 || arguments.len() != arity {
                    return Err(action_error(
                        "compile.list.invalid",
                        "unknown list operation or wrong arity",
                    ));
                }
                let mut types = Vec::new();
                for argument in arguments {
                    types.push(self.expr(argument, env)?);
                }
                let Type::List(element) = &types[arity - 1] else {
                    return Err(action_error("compile.list.invalid", "operation requires a list"));
                };
                match op.as_str() {
                    "length" => Ok(Type::I64),
                    "head" => Ok(*element.clone()),
                    "tail" => Ok(types[0].clone()),
                    _ => {
                        same(types[0].clone(), *element.clone()).map_err(|e| e.at(arguments[0].span()))?;
                        Ok(types[1].clone())
                    }
                }
            }
            Expr::Variant {
                variant,
                type_arguments,
                case,
                payload,
                ..
            } => {
                if !type_arguments.is_empty() {
                    return Err(action_error(
                        "compile.generic.unlowered",
                        "constructor type arguments must be lowered",
                    ));
                }
                let definition = self.records.get_variant(variant)?;
                modules::check_type(&Type::Variant(variant.clone()), &self.visible)?;
                let target = definition
                    .cases
                    .iter()
                    .find(|c| &c.name == case)
                    .ok_or_else(|| action_error("compile.variant.case", "unknown case"))?;
                match (&target.payload, payload) {
                    (Some(ty), Some(value)) => {
                        same(self.expr(value, env)?, ty.clone()).map_err(|e| e.at(value.span()))?
                    }
                    (None, None) => (),
                    _ => {
                        return Err(action_error(
                            "compile.variant.case",
                            "case payload presence must match declaration",
                        ))
                    }
                }
                Ok(Type::Variant(variant.clone()))
            }
            Expr::Match { subject, arms, .. } => {
                let Type::Variant(id) = self.expr(subject, env)? else {
                    return Err(action_error("compile.match.invalid", "match requires a variant"));
                };
                let definition = self.records.get_variant(&id)?;
                if arms.len() != definition.cases.len() {
                    return Err(action_error(
                        "compile.match.invalid",
                        "match must cover each case exactly once",
                    ));
                }
                let mut seen = BTreeSet::new();
                let mut output: Option<Type> = None;
                for arm in arms {
                    let result = (|| {
                        let target = definition
                            .cases
                            .iter()
                            .find(|c| c.name == arm.case)
                            .ok_or_else(|| action_error("compile.match.invalid", "unknown match case"))?;
                        if !seen.insert(&arm.case) || target.payload.is_some() != arm.binding.is_some() {
                            return Err(action_error(
                                "compile.match.invalid",
                                "duplicate case or wrong binding presence",
                            ));
                        }
                        let mut inner = env.clone();
                        if let (Some(name), Some(ty)) = (&arm.binding, &target.payload) {
                            check_name(name)?;
                            if inner.contains_key(name) || self.names.contains(name) {
                                return Err(action_error(
                                    "compile.name.duplicate",
                                    "match binding shadows a visible name",
                                ));
                            }
                            inner.insert(name.clone(), ty.clone());
                        }
                        let ty = self.block(&arm.body, &inner)?;
                        if let Some(expected) = &output {
                            same(ty.clone(), expected.clone()).map_err(|e| e.at(arm.body.value.span()))?;
                        }
                        output = Some(ty);
                        Ok(())
                    })();
                    result.map_err(|e: Diagnostic| e.at(arm.span))?;
                }
                Ok(output.expect("nonempty checked variant"))
            }
            Expr::Record {
                record, fields, base, ..
            } => {
                if base.is_some() {
                    return Err(action_error(
                        "compile.record.update",
                        "record update must be elaborated before checking",
                    ));
                }
                let definition = self.records.get(record)?;
                if let Expr::Record { type_arguments, .. } = expression {
                    if !type_arguments.is_empty() {
                        return Err(action_error(
                            "compile.generic.unlowered",
                            "constructor type arguments must be lowered",
                        ));
                    }
                }
                modules::check_type(&Type::Record(record.clone()), &self.visible)?;
                let mut seen = BTreeSet::new();
                if fields.len() != definition.fields.len() {
                    return Err(records::field_error("constructor must provide every field"));
                }
                for field in fields {
                    let target = definition
                        .fields
                        .iter()
                        .find(|f| f.name == field.name)
                        .ok_or_else(|| records::field_error("unknown constructor field"))?;
                    if !seen.insert(&field.name) {
                        return Err(records::field_error("repeated constructor field"));
                    }
                    same(self.expr(&field.value, env)?, target.ty.clone())
                        .map_err(|e| e.at(field.value.span()))?;
                }
                Ok(Type::Record(record.clone()))
            }
            Expr::Field { base, field, .. } => {
                let Type::Record(id) = self.expr(base, env)? else {
                    return Err(action_error(
                        "compile.record.field",
                        "selection requires a record",
                    ));
                };
                self.records
                    .get(&id)?
                    .fields
                    .iter()
                    .find(|f| &f.name == field)
                    .map(|f| f.ty.clone())
                    .ok_or_else(|| action_error("compile.record.field", "unknown field"))
            }
            Expr::Literal {
                value:
                    Value::Function(_) | Value::Record(_) | Value::Variant(_) | Value::List(_) | Value::Bytes(_),
                ..
            } => Err(syntax("record values require constructors")),
            Expr::Literal { value, .. } => {
                if let Value::F64(bits) = value {
                    if !f64::from_bits(*bits).is_finite() {
                        return Err(action_error(
                            "compile.literal.range",
                            "F64 literals must be finite",
                        ));
                    }
                }
                if let Value::Text(text) = value {
                    if text.len() > MAX_SOURCE {
                        return Err(action_error(
                            "resource.frontend.limit",
                            "text literal limit exceeded",
                        ));
                    }
                }
                Ok(value.ty())
            }
            Expr::Name { name, .. } => env.get(name).cloned().ok_or_else(|| {
                Diagnostic::new(
                    "compile.name.unresolved",
                    "resolve",
                    format!("unknown value: {name}"),
                )
            }),
            Expr::Unary { op, operand, .. } => {
                if op == "try" && self.in_contract {
                    return Err(action_error(
                        "compile.try.contract",
                        "try is forbidden in contracts",
                    ));
                }
                let actual = self.expr(operand, env)?;
                if op == "try" {
                    let (ok, error) = propagation::types(&self.records, &actual, "compile.try.operand")?;
                    let (_, target_error) =
                        propagation::types(&self.records, &self.output, "compile.try.output")?;
                    if error != target_error {
                        return Err(action_error(
                            "compile.try.error",
                            "try error payload types must match exactly",
                        ));
                    }
                    return Ok(ok);
                }
                let expected = match op.as_str() {
                    "-" if actual == Type::F64 => Type::F64,
                    "-" => Type::I64,
                    "!" => Type::Bool,
                    _ => return Err(syntax("unknown unary operation")),
                };
                same(actual, expected.clone()).map_err(|e| e.at(operand.span()))?;
                Ok(expected)
            }
            Expr::Binary { op, left, right, .. } => {
                let a = self.expr(left, env)?;
                let b = self.expr(right, env)?;
                same(a.clone(), b).map_err(|e| e.at(right.span()))?;
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => {
                        if a == Type::F64 && op != "%" {
                            return Ok(Type::F64);
                        }
                        same(a, Type::I64).map_err(|e| e.at(left.span()))?;
                        Ok(Type::I64)
                    }
                    "<" | "<=" | ">" | ">=" => {
                        if a == Type::F64 {
                            return Ok(Type::Bool);
                        }
                        same(a, Type::I64).map_err(|e| e.at(left.span()))?;
                        Ok(Type::Bool)
                    }
                    "==" | "!=" => Ok(Type::Bool),
                    "&&" | "||" => {
                        same(a, Type::Bool).map_err(|e| e.at(left.span()))?;
                        Ok(Type::Bool)
                    }
                    _ => Err(syntax("unknown binary operation")),
                }
            }
            Expr::Call {
                function,
                type_arguments,
                arguments,
                name_span,
                ..
            } => {
                if !type_arguments.is_empty() {
                    return Err(action_error(
                        "compile.generic.unlowered",
                        "checked calls must be concrete",
                    ));
                }
                let target = *self
                    .functions
                    .get(function)
                    .filter(|_| self.visible.contains(function))
                    .ok_or_else(|| {
                        {
                            Diagnostic::new(
                                "compile.name.unresolved",
                                "resolve",
                                format!("unknown function: {function}"),
                            )
                        }
                        .at(*name_span)
                    })?;
                if target.parameters.len() != arguments.len() {
                    return Err(Diagnostic::new(
                        "compile.call.arity",
                        "check",
                        "wrong number of arguments",
                    ));
                }
                for (argument, parameter) in arguments.iter().zip(&target.parameters) {
                    same(self.expr(argument, env)?, parameter.ty.clone())
                        .map_err(|e| e.at(argument.span()))?;
                }
                if self.in_contract && !target.actions.is_empty() {
                    return Err(action_error(
                        "compile.action.contract",
                        "contracts require pure callees",
                    ));
                }
                if let Some(action) = target
                    .actions
                    .iter()
                    .find(|action| !self.actions.contains(*action))
                {
                    return Err(action_error(
                        "compile.action.undeclared",
                        format!("caller must declare {action}"),
                    ));
                }
                Ok(target.output.clone())
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                same(self.expr(condition, env)?, Type::Bool).map_err(|e| e.at(condition.span()))?;
                let a = self.block(then_block, env)?;
                same(self.block(else_block, env)?, a.clone()).map_err(|e| e.at(else_block.value.span()))?;
                Ok(a)
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                self.check_type(&state.ty)?;
                same(self.expr(initial, env)?, state.ty.clone()).map_err(|e| e.at(initial.span()))?;
                check_name(&state.name).map_err(|e| e.at(state.span))?;
                if env.contains_key(&state.name) || self.names.contains(&state.name) {
                    return Err(Diagnostic::new(
                        "compile.name.duplicate",
                        "resolve",
                        "while binding shadows a visible name",
                    )
                    .at(state.span));
                }
                let mut inner = env.clone();
                inner.insert(state.name.clone(), state.ty.clone());
                same(self.expr(condition, &inner)?, Type::Bool).map_err(|e| e.at(condition.span()))?;
                same(self.block(body, &inner)?, state.ty.clone()).map_err(|e| e.at(body.value.span()))?;
                Ok(state.ty.clone())
            }
            Expr::Fold {
                index,
                accumulator,
                count,
                initial,
                body,
                index_span,
                ..
            } => {
                same(self.expr(count, env)?, Type::I64).map_err(|e| e.at(count.span()))?;
                self.check_type(&accumulator.ty)?;
                same(self.expr(initial, env)?, accumulator.ty.clone()).map_err(|e| e.at(initial.span()))?;
                let mut inner = env.clone();
                for (name, ty, span) in [
                    (index, Type::I64, *index_span),
                    (&accumulator.name, accumulator.ty.clone(), accumulator.span),
                ] {
                    check_name(name).map_err(|e| e.at(span))?;
                    if inner.contains_key(name) || self.names.contains(name) {
                        return Err(Diagnostic::new(
                            "compile.name.duplicate",
                            "resolve",
                            "fold binding shadows a visible name",
                        )
                        .at(span));
                    }
                    inner.insert(name.clone(), ty);
                }
                same(self.block(body, &inner)?, accumulator.ty.clone())
                    .map_err(|e| e.at(body.value.span()))?;
                Ok(accumulator.ty.clone())
            }
        }
    }
}

fn validate(program: &Program, resolved: bool) -> Fallible<()> {
    validate_mode(program, resolved, false)
}

fn validate_mode(program: &Program, resolved: bool, abstract_types: bool) -> Fallible<()> {
    if program.functions.iter().any(|f| !f.type_parameters.is_empty())
        || program.records.iter().any(|r| !r.type_parameters.is_empty())
        || program.variants.iter().any(|v| !v.type_parameters.is_empty())
    {
        return Err(action_error(
            "compile.generic.unlowered",
            "checked programs must contain concrete functions only",
        ));
    }
    let contexts = modules::contexts(program)?;
    let records =
        records::RecordTable::new_mode(&program.records, &program.variants, resolved, abstract_types)?;
    for record in &program.records {
        for field in &record.fields {
            modules::check_type(&field.ty, &contexts[&record.id].visible).map_err(|mut e| {
                e.subject = Some(record.id.clone());
                e.at(field.span)
            })?;
        }
    }
    let mut functions = BTreeMap::new();
    for variant in &program.variants {
        for case in &variant.cases {
            if let Some(ty) = &case.payload {
                modules::check_type(ty, &contexts[&variant.id].visible).map_err(|mut e| {
                    e.subject = Some(variant.id.clone());
                    e.at(case.span)
                })?;
            }
        }
    }
    for function in &program.functions {
        records
            .check_type(&function.output)
            .map_err(|e| e.at(function.span))?;
        for parameter in &function.parameters {
            records
                .check_type(&parameter.ty)
                .map_err(|e| e.at(parameter.span))?;
        }
        validate_actions(&function.actions).map_err(|mut e| {
            e.subject = Some(function.id.clone());
            e.at(function.span)
        })?;
        functions.insert(
            if resolved {
                function.id.clone()
            } else {
                function.name.clone()
            },
            function,
        );
    }
    let mut checker = Checker {
        functions,
        names: BTreeSet::new(),
        visible: BTreeSet::new(),
        depth: 0,
        actions: BTreeSet::new(),
        in_contract: false,
        output: Type::I64,
        records,
        record_cells: 0,
    };
    for function in &program.functions {
        checker.output = function.output.clone();
        checker.names = contexts[&function.id].aliases.keys().cloned().collect();
        checker.visible = contexts[&function.id].visible.clone();
        if abstract_types && !function.id.starts_with("lobo.generic.") {
            continue;
        }
        let result = (|| {
            checker
                .check_type(&function.output)
                .map_err(|e| e.at(function.span))?;
            checker.record_cells = if matches!(function.output, Type::I64 | Type::Bool) {
                0
            } else {
                checker.records.width(&function.output)
            };
            let mut env = BTreeMap::new();
            for parameter in &function.parameters {
                checker
                    .check_type(&parameter.ty)
                    .map_err(|e| e.at(parameter.span))?;
                if !matches!(parameter.ty, Type::I64 | Type::Bool) {
                    checker.record_cells += checker.records.width(&parameter.ty);
                }
                check_name(&parameter.name).map_err(|e| e.at(parameter.span))?;
                if checker.names.contains(&parameter.name)
                    || env.insert(parameter.name.clone(), parameter.ty.clone()).is_some()
                {
                    return Err(Diagnostic::new(
                        "compile.name.duplicate",
                        "resolve",
                        "parameter name is repeated or shadows a function",
                    )
                    .at(parameter.span));
                }
            }
            for requirement in &function.requires {
                checker.in_contract = true;
                same(checker.expr(requirement, &env)?, Type::Bool).map_err(|e| e.at(requirement.span()))?;
            }
            checker.in_contract = false;
            checker.actions = function.actions.iter().cloned().collect();
            same(checker.block(&function.body, &env)?, function.output.clone())
                .map_err(|e| e.at(function.body.value.span()))?;
            env.insert("result".into(), function.output.clone());
            for assurance in &function.ensures {
                checker.in_contract = true;
                same(checker.expr(assurance, &env)?, Type::Bool).map_err(|e| e.at(assurance.span()))?;
            }
            Ok(())
        })();
        if let Err(mut error) = result {
            error.subject = Some(function.id.clone());
            return Err(error);
        }
    }
    Ok(())
}

fn resolve_block(
    block: &mut Block,
    ids: &BTreeMap<String, String>,
    variants: &BTreeSet<String>,
    depth: usize,
) -> Fallible<()> {
    for binding in &mut block.bindings {
        if let Some(ty) = &mut binding.annotation {
            resolve_type(ty, ids, variants)?;
        }
        resolve_expr(&mut binding.value, ids, variants, depth)?;
    }
    resolve_expr(&mut block.value, ids, variants, depth)
}

fn resolve_type(ty: &mut Type, ids: &BTreeMap<String, String>, variants: &BTreeSet<String>) -> Fallible<()> {
    if let Type::Function(signature) = ty {
        for parameter in &mut signature.parameters {
            resolve_type(parameter, ids, variants)?;
        }
        resolve_type(&mut signature.output, ids, variants)?;
    }
    if let Type::Applied { name, arguments } = ty {
        *name = ids
            .get(name)
            .cloned()
            .ok_or_else(|| action_error("compile.type.unknown", "unknown generic type"))?;
        for argument in arguments {
            resolve_type(argument, ids, variants)?;
        }
    }
    if let Type::List(element) = ty {
        resolve_type(element, ids, variants)?;
    }
    if let Type::Record(id) = ty {
        if variants.contains(id) {
            *ty = Type::Variant(id.clone());
        }
    }
    if matches!(ty,Type::Variant(id) if native_api::is_result(id)) {
        return Ok(());
    }
    if let Type::Record(id) | Type::Variant(id) = ty {
        *id = ids
            .get(id)
            .cloned()
            .ok_or_else(|| action_error("compile.type.unknown", format!("unknown type {id}")))?;
    }
    Ok(())
}

fn resolve_expr(
    expression: &mut Expr,
    ids: &BTreeMap<String, String>,
    variants: &BTreeSet<String>,
    depth: usize,
) -> Fallible<()> {
    if depth >= MAX_DEPTH {
        return Err(Diagnostic::new(
            "resource.frontend.limit",
            "resolve",
            "expression nesting limit exceeded",
        )
        .at(expression.span()));
    }
    let depth = depth + 1;
    let expression_span = expression.span();
    let result = (|| {
        if let Expr::Call {
            function,
            type_arguments,
            arguments,
            span,
            name_span,
        } = expression
        {
            if !ids.contains_key(function) && type_arguments.is_empty() {
                *expression = Expr::Invoke {
                    callee: Box::new(Expr::Name {
                        name: function.clone(),
                        span: *name_span,
                    }),
                    arguments: std::mem::take(arguments),
                    span: *span,
                };
            }
        }
        match expression {
            Expr::Reference {
                function,
                type_arguments,
                ..
            } => {
                *function = ids.get(function).cloned().ok_or_else(|| {
                    action_error("compile.name.unresolved", "unknown named function reference")
                })?;
                for ty in type_arguments {
                    resolve_type(ty, ids, variants)?;
                }
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                resolve_expr(callee, ids, variants, depth)?;
                for argument in arguments {
                    resolve_expr(argument, ids, variants, depth)?;
                }
            }
            Expr::List { element, .. } => resolve_type(element, ids, variants)?,
            Expr::NativeOp { arguments, .. }
            | Expr::ListOp { arguments, .. }
            | Expr::BufferOp { arguments, .. } => {
                for argument in arguments {
                    resolve_expr(argument, ids, variants, depth)?;
                }
            }
            Expr::Variant {
                variant,
                type_arguments,
                payload,
                ..
            } => {
                for ty in type_arguments {
                    resolve_type(ty, ids, variants)?;
                }
                *variant = ids
                    .get(variant)
                    .cloned()
                    .ok_or_else(|| action_error("compile.type.unknown", "unknown variant constructor"))?;
                if let Some(value) = payload {
                    resolve_expr(value, ids, variants, depth)?;
                }
            }
            Expr::Match { subject, arms, .. } => {
                resolve_expr(subject, ids, variants, depth)?;
                for arm in arms {
                    resolve_block(&mut arm.body, ids, variants, depth)?;
                }
            }
            Expr::Record {
                record,
                type_arguments,
                fields,
                base,
                ..
            } => {
                for ty in type_arguments {
                    resolve_type(ty, ids, variants)?;
                }
                *record = ids
                    .get(record)
                    .cloned()
                    .ok_or_else(|| action_error("compile.type.unknown", "unknown record constructor"))?;
                if let Some(value) = base {
                    resolve_expr(value, ids, variants, depth)?;
                }
                for field in fields {
                    resolve_expr(&mut field.value, ids, variants, depth)?;
                }
            }
            Expr::Field { base, .. } => resolve_expr(base, ids, variants, depth)?,
            Expr::Literal { .. } | Expr::Name { .. } => (),
            Expr::Unary { operand, .. } => resolve_expr(operand, ids, variants, depth)?,
            Expr::Binary { left, right, .. } => {
                resolve_expr(left, ids, variants, depth)?;
                resolve_expr(right, ids, variants, depth)?;
            }
            Expr::Call {
                function,
                type_arguments,
                arguments,
                ..
            } => {
                for ty in type_arguments {
                    resolve_type(ty, ids, variants)?;
                }
                let name_span = expression_span.map(|span| ByteSpan {
                    start: span.start,
                    end: span.start + function.len(),
                });
                *function = ids.get(function).cloned().ok_or_else(|| {
                    Diagnostic::new(
                        "compile.name.unresolved",
                        "resolve",
                        format!("unknown function: {function}"),
                    )
                    .at(name_span)
                })?;
                for argument in arguments {
                    resolve_expr(argument, ids, variants, depth)?;
                }
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                resolve_expr(condition, ids, variants, depth)?;
                resolve_block(then_block, ids, variants, depth)?;
                resolve_block(else_block, ids, variants, depth)?;
            }
            Expr::Fold {
                count,
                initial,
                body,
                accumulator,
                ..
            } => {
                resolve_type(&mut accumulator.ty, ids, variants)?;
                resolve_expr(count, ids, variants, depth)?;
                resolve_expr(initial, ids, variants, depth)?;
                resolve_block(body, ids, variants, depth)?;
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                resolve_type(&mut state.ty, ids, variants)?;
                resolve_expr(initial, ids, variants, depth)?;
                resolve_expr(condition, ids, variants, depth)?;
                resolve_block(body, ids, variants, depth)?;
            }
        }
        Ok(())
    })();
    result.map_err(|e: Diagnostic| e.at(expression_span))
}

fn type_parameter_ids(
    ids: &BTreeMap<String, String>,
    parameters: &[String],
) -> Fallible<BTreeMap<String, String>> {
    let mut result = ids.clone();
    for name in parameters {
        if result.insert(name.clone(), format!("@{name}")).is_some() {
            return Err(action_error(
                "compile.name.duplicate",
                "type parameter shadows a visible declaration",
            ));
        }
    }
    Ok(result)
}

fn resolve_program(
    program: &mut Program,
    ids: &BTreeMap<String, String>,
    variants: &BTreeSet<String>,
) -> Fallible<()> {
    for binding in &mut program.operators {
        let result = (|| {
            resolve_type(&mut binding.left, ids, variants)?;
            resolve_type(&mut binding.right, ids, variants)?;
            resolve_type(&mut binding.output, ids, variants)?;
            binding.target = ids.get(&binding.target).cloned().ok_or_else(|| {
                action_error("compile.name.unresolved", "unknown operator target").at(binding.target_span)
            })?;
            Ok(())
        })();
        result.map_err(|mut e: Diagnostic| {
            e.subject = Some(binding.module.clone());
            e.at(binding.span)
        })?;
    }
    for record in &mut program.records {
        let local_ids = type_parameter_ids(ids, &record.type_parameters).map_err(|e| e.at(record.span))?;
        for field in &mut record.fields {
            resolve_type(&mut field.ty, &local_ids, variants).map_err(|mut e| {
                e.subject = Some(record.id.clone());
                e.at(field.span)
            })?;
        }
    }
    for variant in &mut program.variants {
        let local_ids = type_parameter_ids(ids, &variant.type_parameters).map_err(|e| e.at(variant.span))?;
        for case in &mut variant.cases {
            if let Some(ty) = &mut case.payload {
                resolve_type(ty, &local_ids, variants).map_err(|mut e| {
                    e.subject = Some(variant.id.clone());
                    e.at(case.span)
                })?;
            }
        }
    }
    for function in &mut program.functions {
        let mut local_ids = ids.clone();
        for name in &function.type_parameters {
            if local_ids.insert(name.clone(), format!("@{name}")).is_some() {
                return Err(action_error(
                    "compile.name.duplicate",
                    "type parameter shadows a visible declaration",
                )
                .at(function.span));
            }
        }
        let result = (|| {
            resolve_type(&mut function.output, &local_ids, variants).map_err(|e| e.at(function.span))?;
            for parameter in &mut function.parameters {
                resolve_type(&mut parameter.ty, &local_ids, variants).map_err(|e| e.at(parameter.span))?;
            }
            for expression in function.requires.iter_mut().chain(&mut function.ensures) {
                resolve_expr(expression, &local_ids, variants, 0)?;
            }
            resolve_block(&mut function.body, &local_ids, variants, 0)?;
            Ok(())
        })();
        result.map_err(|mut error: Diagnostic| {
            error.subject = Some(function.id.clone());
            error
        })?;
    }
    Ok(())
}

pub fn compile(source: &str) -> Fallible<Artifact> {
    compile_bundle(
        "<source>",
        &[SourceFile {
            identities: None,
            name: "<source>".into(),
            text: source.into(),
        }],
    )
    .map_err(|mut error| {
        error.source = None;
        error
    })
}

impl Artifact {
    pub fn from_json(json: &str) -> Fallible<Self> {
        if json.len() > 4 * MAX_SOURCE {
            return Err(Diagnostic::new(
                "resource.artifact.limit",
                "load",
                "artifact size limit exceeded",
            ));
        }
        let artifact: Self = serde_json::from_str(json)
            .map_err(|e| Diagnostic::new("invocation.artifact.invalid", "load", e.to_string()))?;
        artifact.check()?;
        Ok(artifact)
    }
    fn check(&self) -> Fallible<()> {
        if self.format != FORMAT
            || !matches!(
                self.language.as_str(),
                "core/1.0"
                    | "core/1.1"
                    | "core/1.2"
                    | "core/1.3"
                    | "core/1.4"
                    | "core/1.5"
                    | "core/1.6"
                    | "core/1.7"
                    | "core/1.8"
                    | "core/1.9"
                    | "core/1.10"
                    | LANGUAGE
            )
        {
            return Err(Diagnostic::new(
                "invocation.artifact.version",
                "load",
                "unsupported artifact or language version",
            ));
        }
        if !matches!(
            self.language.as_str(),
            "core/1.7" | "core/1.8" | "core/1.9" | "core/1.10" | LANGUAGE
        ) && (self.program.native_abi.is_some() || native_api::has_new_nodes(&self.program)?)
        {
            return Err(Diagnostic::new(
                "invocation.artifact.version",
                "load",
                "native operations require core/1.7",
            ));
        }
        if !matches!(
            self.language.as_str(),
            "core/1.8" | "core/1.9" | "core/1.10" | LANGUAGE
        ) && propagation::present(&self.program)?
        {
            return Err(Diagnostic::new(
                "invocation.artifact.version",
                "load",
                "try requires core/1.8",
            ));
        }
        if !matches!(self.language.as_str(), "core/1.9" | "core/1.10" | LANGUAGE)
            && (self.program.sqlite_abi.is_some() || native_api::has_sqlite_nodes(&self.program)?)
        {
            return Err(Diagnostic::new(
                "invocation.artifact.version",
                "load",
                "SQLite operations require core/1.9",
            ));
        }
        if self.language != LANGUAGE && callbacks::present(&self.program)? {
            return Err(Diagnostic::new(
                "invocation.artifact.version",
                "load",
                "actionful callbacks require core/1.11",
            ));
        }
        native_api::validate_layouts(&self.program)?;
        validate(&self.program, true)
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    pub fn entries(&self) -> Vec<&str> {
        self.program.functions.iter().map(|f| f.id.as_str()).collect()
    }
    pub fn action_summary(&self) -> BTreeMap<&str, &[String]> {
        self.program
            .functions
            .iter()
            .map(|f| (f.id.as_str(), f.actions.as_slice()))
            .collect()
    }
    pub fn parse_argument(&self, entry: &str, index: usize, text: &str) -> Fallible<Value> {
        self.parse_argument_bounded(entry, index, text, &mut buffers::InputBudget::default())
    }
    pub fn parse_arguments(&self, entry: &str, texts: &[&str]) -> Fallible<Vec<Value>> {
        let mut remaining = buffers::InputBudget::default();
        texts
            .iter()
            .enumerate()
            .map(|(i, text)| self.parse_argument_bounded(entry, i, text, &mut remaining))
            .collect()
    }
    fn parse_argument_bounded(
        &self,
        entry: &str,
        index: usize,
        text: &str,
        remaining: &mut buffers::InputBudget,
    ) -> Fallible<Value> {
        self.check()?;
        if !text.starts_with("Record:")
            && !text.starts_with("Variant:")
            && !text.starts_with("List:")
            && !text.starts_with("Bytes:")
            && !text.starts_with("Text:")
            && !text.starts_with("Function:")
        {
            remaining.node()?;
            return records::scalar_argument(text);
        }
        let invalid = || {
            Diagnostic::new(
                "invocation.arguments.invalid",
                "invoke",
                "invalid compound argument",
            )
        };
        let function = self
            .program
            .functions
            .iter()
            .find(|f| f.id == entry)
            .ok_or_else(invalid)?;
        let parameter = function.parameters.get(index).ok_or_else(invalid)?;
        let prefix = match &parameter.ty {
            Type::Record(id) => format!("Record:{id}:"),
            Type::Variant(id) => format!("Variant:{id}:"),
            Type::List(_) => "List:".into(),
            Type::Bytes | Type::Text | Type::Function(_) => "".into(),
            _ => return Err(invalid()),
        };
        let contents = text.strip_prefix(&prefix).ok_or_else(invalid)?;
        let mut leaves = contents.split(',');
        let value = records::RecordTable::new(&self.program.records, &self.program.variants, true)?
            .with_functions(&self.program.functions)
            .parse_value(&parameter.ty, &mut leaves, remaining)?;
        if leaves.next().is_some() {
            return Err(invalid());
        }
        Ok(value)
    }
    pub fn run(&self, entry: &str, arguments: Vec<Value>, fuel: u64) -> Fallible<Value> {
        self.run_with_heap(entry, arguments, fuel, DEFAULT_HEAP_CELLS)
    }
    pub fn run_with_heap(
        &self,
        entry: &str,
        arguments: Vec<Value>,
        fuel: u64,
        mut heap: u64,
    ) -> Fallible<Value> {
        self.check()?;
        floats::environment()?;
        let records = records::RecordTable::new(&self.program.records, &self.program.variants, true)?
            .with_functions(&self.program.functions);
        let function = self
            .program
            .functions
            .iter()
            .find(|f| f.id == entry)
            .ok_or_else(|| {
                Diagnostic::new("invocation.entry.unknown", "invoke", "unknown function identity")
            })?;
        if arguments.len() != function.parameters.len() {
            return Err(Diagnostic::new(
                "invocation.arguments.invalid",
                "invoke",
                "wrong arity",
            ));
        }
        let mut nodes = 100_000;
        let mut input_bytes = 8 * 1024 * 1024;
        for (argument, parameter) in arguments.iter().zip(&function.parameters) {
            lists::value_limit(argument, &mut nodes, "resource.input.limit")?;
            buffers::input_size(argument, &mut input_bytes)?;
            if !records.valid_value(argument, &parameter.ty) {
                return Err(Diagnostic::new(
                    "invocation.arguments.invalid",
                    "invoke",
                    "invalid argument shape",
                ));
            }
        }
        for (argument, parameter) in arguments.iter().zip(&function.parameters) {
            records
                .charge_value(argument, &parameter.ty, &mut heap)
                .map_err(|mut e| {
                    e.phase = "invoke".into();
                    e
                })?;
        }
        let functions = self
            .program
            .functions
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();
        let value = Runner {
            functions,
            fuel,
            depth: 0,
            expression_depth: 0,
            records,
            heap,
        }
        .call(entry, arguments)?;
        lists::output_limit(&value)?;
        Ok(value)
    }
}

struct Runner<'a> {
    functions: BTreeMap<&'a str, &'a Function>,
    fuel: u64,
    depth: usize,
    expression_depth: usize,
    records: records::RecordTable<'a>,
    heap: u64,
}

impl Runner<'_> {
    fn tick(&mut self) -> Fallible<()> {
        self.fuel = self.fuel.checked_sub(1).ok_or_else(|| {
            Diagnostic::new("resource.fuel.exhausted", "execute", "execution fuel exhausted")
        })?;
        Ok(())
    }
    fn call(&mut self, id: &str, arguments: Vec<Value>) -> Fallible<Value> {
        let function = *self.functions.get(id).ok_or_else(|| {
            Diagnostic::new("invocation.entry.unknown", "invoke", "unknown function identity")
        })?;
        self.tick().map_err(|mut error| {
            error.subject = Some(id.into());
            error
        })?;
        if self.depth >= MAX_DEPTH {
            let mut error =
                Diagnostic::new("resource.stack.exhausted", "execute", "call depth limit exceeded");
            error.subject = Some(id.into());
            return Err(error);
        }
        self.depth += 1;
        let mut env = function
            .parameters
            .iter()
            .zip(arguments)
            .map(|(p, v)| (p.name.clone(), v))
            .collect::<BTreeMap<_, _>>();
        let result = (|| {
            for requirement in &function.requires {
                let value = self
                    .expr(requirement, &env)
                    .map_err(propagation::EvaluationError::contract_fault)
                    .map_err(|mut e| {
                        if e.code.starts_with("runtime.") || e.code.starts_with("resource.") {
                            e.phase = "precondition".into();
                        }
                        e
                    })?;
                if value != Value::Bool(true) {
                    return Err(Diagnostic::new(
                        "runtime.contract.precondition",
                        "precondition",
                        "precondition is false",
                    ));
                }
            }
            let value = match self.block(&function.body, &env) {
                Ok(value) => value,
                Err(propagation::EvaluationError::Return(error)) => {
                    let Type::Variant(id) = &function.output else {
                        unreachable!("checked try output")
                    };
                    Value::Variant(VariantValue {
                        id: id.clone(),
                        case: "Err".into(),
                        payload: Some(Box::new(error)),
                    })
                }
                Err(propagation::EvaluationError::Fault(error)) => return Err(error),
            };
            env.insert("result".into(), value.clone());
            for assurance in &function.ensures {
                let checked = self
                    .expr(assurance, &env)
                    .map_err(propagation::EvaluationError::contract_fault)
                    .map_err(|mut e| {
                        if e.code.starts_with("runtime.") || e.code.starts_with("resource.") {
                            e.phase = "postcondition".into();
                        }
                        e
                    })?;
                if checked != Value::Bool(true) {
                    return Err(Diagnostic::new(
                        "runtime.contract.postcondition",
                        "postcondition",
                        "postcondition is false",
                    ));
                }
            }
            Ok(value)
        })();
        self.depth -= 1;
        result.map_err(|mut e| {
            if e.subject.is_none() {
                e.subject = Some(id.into());
            }
            e
        })
    }
    fn block(
        &mut self,
        block: &Block,
        outer: &BTreeMap<String, Value>,
    ) -> Result<Value, propagation::EvaluationError> {
        let mut env = outer.clone();
        for binding in &block.bindings {
            let value = self.expr(&binding.value, &env)?;
            env.insert(binding.name.clone(), value);
        }
        self.expr(&block.value, &env)
    }
    fn expr(
        &mut self,
        expression: &Expr,
        env: &BTreeMap<String, Value>,
    ) -> Result<Value, propagation::EvaluationError> {
        self.tick()?;
        self.expression_depth += 1;
        if self.expression_depth > 256 {
            return Err(Diagnostic::new(
                "resource.stack.exhausted",
                "execute",
                "evaluation depth limit exceeded",
            )
            .into());
        }
        let result = self.expr_inner(expression, env);
        self.expression_depth -= 1;
        result
    }
    fn expr_inner(
        &mut self,
        expression: &Expr,
        env: &BTreeMap<String, Value>,
    ) -> Result<Value, propagation::EvaluationError> {
        match expression {
            Expr::Reference { function, .. } => {
                let target = self.functions[function.as_str()];
                Ok(Value::Function(FunctionValue {
                    id: function.clone(),
                    signature: target.signature_type(),
                }))
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                let Value::Function(reference) = self.expr(callee, env)? else {
                    return Err(mismatch("invocation requires a function value").into());
                };
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, env)?);
                }
                Ok(self.call(&reference.id, values)?)
            }
            Expr::NativeOp {
                binding,
                abi,
                arguments,
                ..
            } => {
                native_api::signature(binding, *abi)?;
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, env)?);
                }
                match binding.as_str() {
                    "lobo.native.abi_version" => Ok(Value::I64(1)),
                    "lobo.native.roundtrip_i64" | "lobo.native.roundtrip_bool" => Ok(values.remove(0)),
                    "lobo.native.copy_bytes" => {
                        let Value::Bytes(bytes) = &values[0] else {
                            unreachable!("checked native bytes")
                        };
                        self.fuel = self.fuel.checked_sub(bytes.len() as u64).ok_or_else(|| {
                            Diagnostic::new(
                                "resource.fuel.exhausted",
                                "execute",
                                "native copy fuel exhausted",
                            )
                        })?;
                        lists::charge(&mut self.heap, buffers::cells(bytes.len()))?;
                        Ok(Value::Bytes(BufferValue::from_bytes(bytes.bytes().to_vec())))
                    }
                    _ => Err(Diagnostic::new(
                        "invocation.native.required",
                        "execute",
                        "operation requires native execution",
                    )
                    .into()),
                }
            }
            Expr::BufferOp {
                family,
                op,
                arguments,
                ..
            } => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, env)?);
                }
                if family == "F64" {
                    Ok(floats::operation(op, &values)?)
                } else {
                    Ok(self.buffer_op(family, op, values)?)
                }
            }
            Expr::List { element, .. } => Ok(Value::List(ListValue::empty(element.clone()))),
            Expr::ListOp { op, arguments, .. } => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, env)?);
                }
                let Value::List(list) = values.pop().expect("checked list argument") else {
                    unreachable!("checked list");
                };
                match op.as_str() {
                    "length" => Ok(Value::I64(list.len() as i64)),
                    "head" => Ok(list.head()?),
                    "tail" => Ok(Value::List(list.tail()?)),
                    _ => {
                        lists::charge(&mut self.heap, 3 + self.records.width(&list.element))?;
                        Ok(Value::List(list.prepend(values.pop().unwrap())))
                    }
                }
            }
            Expr::Variant {
                variant,
                case,
                payload,
                ..
            } => {
                let payload = payload
                    .as_ref()
                    .map(|v| self.expr(v, env).map(Box::new))
                    .transpose()?;
                Ok(Value::Variant(VariantValue {
                    id: variant.clone(),
                    case: case.clone(),
                    payload,
                }))
            }
            Expr::Match { subject, arms, .. } => {
                let Value::Variant(value) = self.expr(subject, env)? else {
                    unreachable!("checked variant");
                };
                let arm = arms
                    .iter()
                    .find(|a| a.case == value.case)
                    .expect("exhaustive checked match");
                let mut inner = env.clone();
                if let (Some(name), Some(payload)) = (&arm.binding, value.payload) {
                    inner.insert(name.clone(), *payload);
                }
                self.block(&arm.body, &inner)
            }
            Expr::Record {
                record,
                type_arguments,
                fields,
                ..
            } => {
                if !type_arguments.is_empty() {
                    return Err(action_error(
                        "compile.generic.unlowered",
                        "constructor type arguments must be lowered",
                    )
                    .into());
                }
                let mut values = BTreeMap::new();
                for field in fields {
                    values.insert(field.name.clone(), self.expr(&field.value, env)?);
                }
                Ok(Value::Record(RecordValue {
                    id: record.clone(),
                    fields: values,
                }))
            }
            Expr::Field { base, field, .. } => {
                let Value::Record(value) = self.expr(base, env)? else {
                    return Err(mismatch("expected checked record").into());
                };
                Ok(value.fields[field].clone())
            }
            Expr::Literal { value, .. } => Ok(value.clone()),
            Expr::Name { name, .. } => Ok(env[name].clone()),
            Expr::Unary { op, operand, .. } => {
                let value = self.expr(operand, env)?;
                if op == "try" {
                    let Value::Variant(value) = value else {
                        unreachable!("checked try operand")
                    };
                    let payload = *value.payload.expect("checked try payload");
                    return if value.case == "Ok" {
                        Ok(payload)
                    } else {
                        Err(propagation::EvaluationError::Return(payload))
                    };
                }
                match (op.as_str(), value) {
                    ("-", Value::I64(n)) => Ok(checked(n.checked_neg())?),
                    ("-", Value::F64(bits)) => Ok(floats::normalized(-f64::from_bits(bits))),
                    ("!", Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err(Diagnostic::new(
                        "internal.invalid_ir",
                        "execute",
                        "invalid checked unary expression",
                    )
                    .into()),
                }
            }
            Expr::Binary { op, left, right, .. } => {
                let a = self.expr(left, env)?;
                if op == "&&" && a == Value::Bool(false) {
                    return Ok(a);
                }
                if op == "||" && a == Value::Bool(true) {
                    return Ok(a);
                }
                let b = self.expr(right, env)?;
                match (op.as_str(), a, b) {
                    (op, Value::F64(a), Value::F64(b)) => {
                        let (a, b) = (f64::from_bits(a), f64::from_bits(b));
                        Ok(match op {
                            "+" => floats::normalized(a + b),
                            "-" => floats::normalized(a - b),
                            "*" => floats::normalized(a * b),
                            "/" => floats::normalized(a / b),
                            "==" => Value::Bool(a == b),
                            "!=" => Value::Bool(a != b),
                            "<" => Value::Bool(a < b),
                            "<=" => Value::Bool(a <= b),
                            ">" => Value::Bool(a > b),
                            ">=" => Value::Bool(a >= b),
                            _ => unreachable!("checked F64 operation"),
                        })
                    }
                    ("+", Value::I64(a), Value::I64(b)) => Ok(checked(a.checked_add(b))?),
                    ("-", Value::I64(a), Value::I64(b)) => Ok(checked(a.checked_sub(b))?),
                    ("*", Value::I64(a), Value::I64(b)) => Ok(checked(a.checked_mul(b))?),
                    ("/" | "%", Value::I64(_), Value::I64(0)) => {
                        Err(Diagnostic::new("runtime.integer.zero_divisor", "execute", "zero divisor").into())
                    }
                    ("/", Value::I64(a), Value::I64(b)) => Ok(checked(a.checked_div(b))?),
                    ("%", Value::I64(a), Value::I64(b)) => Ok(checked(a.checked_rem(b))?),
                    ("<", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a < b)),
                    ("<=", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a <= b)),
                    (">", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a > b)),
                    (">=", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a >= b)),
                    ("==", a, b) => Ok(Value::Bool(self.equal(&a, &b)?)),
                    ("!=", a, b) => Ok(Value::Bool(!self.equal(&a, &b)?)),
                    ("&&", Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    ("||", Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => Err(Diagnostic::new(
                        "internal.invalid_ir",
                        "execute",
                        "invalid checked binary expression",
                    )
                    .into()),
                }
            }
            Expr::Call {
                function, arguments, ..
            } => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, env)?);
                }
                Ok(self.call(function, values)?)
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                if self.expr(condition, env)? == Value::Bool(true) {
                    self.block(then_block, env)
                } else {
                    self.block(else_block, env)
                }
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                let mut value = self.expr(initial, env)?;
                let mut inner = env.clone();
                loop {
                    self.tick()?;
                    inner.insert(state.name.clone(), value);
                    if self.expr(condition, &inner)? == Value::Bool(false) {
                        return Ok(inner.remove(&state.name).unwrap());
                    }
                    value = self.block(body, &inner)?;
                }
            }
            Expr::Fold {
                index,
                accumulator,
                count,
                initial,
                body,
                ..
            } => {
                let Value::I64(count) = self.expr(count, env)? else {
                    return Err(mismatch("fold count must be I64").into());
                };
                if count < 0 {
                    return Err(Diagnostic::new(
                        "runtime.iteration.negative_count",
                        "execute",
                        "fold count must be nonnegative",
                    )
                    .into());
                }
                let mut value = self.expr(initial, env)?;
                let mut inner = env.clone();
                for iteration in 0..count {
                    self.tick()?;
                    inner.insert(index.clone(), Value::I64(iteration));
                    inner.insert(accumulator.name.clone(), value);
                    value = self.block(body, &inner)?;
                }
                Ok(value)
            }
        }
    }
    fn equal(&mut self, a: &Value, b: &Value) -> Fallible<bool> {
        self.equal_inner(a, b, 0)
    }
    fn equal_inner(&mut self, a: &Value, b: &Value, depth: usize) -> Fallible<bool> {
        if depth >= 64 && matches!(a, Value::List(_) | Value::Record(_) | Value::Variant(_)) {
            return Err(Diagnostic::new(
                "resource.stack.exhausted",
                "execute",
                "comparison depth limit exceeded",
            ));
        }
        match (a, b) {
            (Value::F64(a), Value::F64(b)) => Ok(f64::from_bits(*a) == f64::from_bits(*b)),
            (Value::Bytes(a), Value::Bytes(b)) | (Value::Text(a), Value::Text(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (a, b) in a.bytes().iter().zip(b.bytes()) {
                    self.tick()?;
                    if a != b {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (Value::List(a), Value::List(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (a, b) in a.iter().zip(b.iter()) {
                    self.tick()?;
                    if !self.equal_inner(a, b, depth + 1)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (Value::Record(a), Value::Record(b)) => {
                let definition = self.records.get(&a.id)?;
                for field in &definition.fields {
                    if !self.equal_inner(&a.fields[&field.name], &b.fields[&field.name], depth + 1)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (Value::Variant(a), Value::Variant(b)) => {
                if a.case != b.case {
                    return Ok(false);
                }
                match (&a.payload, &b.payload) {
                    (Some(a), Some(b)) => self.equal_inner(a, b, depth + 1),
                    _ => Ok(true),
                }
            }
            _ => Ok(a == b),
        }
    }
}

fn checked(value: Option<i64>) -> Fallible<Value> {
    value
        .map(Value::I64)
        .ok_or_else(|| Diagnostic::new("runtime.integer.overflow", "execute", "I64 arithmetic overflow"))
}
