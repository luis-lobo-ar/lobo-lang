use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod native;

pub const LANGUAGE: &str = "core/0.1-draft";
pub const FORMAT: &str = "lobo-checked/0.1";
const MAX_DEPTH: usize = 32;
const MAX_SOURCE: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub phase: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

impl Diagnostic {
    fn new(code: &str, phase: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            phase: phase.into(),
            message: message.into(),
            subject: None,
        }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    I64,
    Bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", deny_unknown_fields)]
pub enum Value {
    #[serde(with = "decimal")]
    I64(i64),
    Bool(bool),
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
            Self::I64(_) => Type::I64,
            Self::Bool(_) => Type::Bool,
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
    module: String,
    id: String,
    functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Function {
    name: String,
    id: String,
    parameters: Vec<Parameter>,
    output: Type,
    requires: Vec<Expr>,
    ensures: Vec<Expr>,
    body: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    name: String,
    ty: Type,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Expr {
    Literal {
        value: Value,
    },
    Name {
        name: String,
    },
    Unary {
        op: String,
        operand: Box<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        function: String,
        arguments: Vec<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_block: Block,
        else_block: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Number(String),
    Id(String),
    Symbol(String),
    End,
}

fn syntax(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("compile.syntax.invalid", "parse", message)
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
            | "Bool"
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

fn lex(source: &str) -> Fallible<Vec<Token>> {
    if source.len() > MAX_SOURCE {
        return Err(Diagnostic::new(
            "resource.frontend.limit",
            "parse",
            "source size limit exceeded",
        ));
    }
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
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
            tokens.push(Token::Number(source[start..i].into()));
        } else if c == b'"' {
            i += 1;
            let content = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i == bytes.len() {
                return Err(syntax("unterminated identity"));
            }
            let id = &source[content..i];
            if !valid_id(id) {
                return Err(syntax("invalid stable identity"));
            }
            tokens.push(Token::Id(id.into()));
            i += 1;
        } else {
            let two = bytes.get(i..i + 2);
            if matches!(two, Some(b"->" | b"==" | b"!=" | b"<=" | b">=" | b"&&" | b"||")) {
                tokens.push(Token::Symbol(source[i..i + 2].into()));
                i += 2;
            } else if b"@();,:{}+-*/%<>!".contains(&c) || c == b'=' {
                tokens.push(Token::Symbol((c as char).to_string()));
                i += 1;
            } else {
                return Err(syntax(format!("unexpected byte at offset {i}")));
            }
        }
        if tokens.len() > 8192 {
            return Err(Diagnostic::new(
                "resource.frontend.limit",
                "parse",
                "token limit exceeded",
            ));
        }
    }
    tokens.push(Token::End);
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    depth: usize,
}

impl Parser {
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
            Err(syntax(format!("expected {text}, got {:?}", self.current())))
        }
    }
    fn name(&mut self) -> Fallible<String> {
        if let Token::Word(name) = self.current().clone() {
            check_name(&name)?;
            self.position += 1;
            Ok(name)
        } else {
            Err(syntax("expected identifier"))
        }
    }
    fn identity(&mut self) -> Fallible<String> {
        self.expect("@")?;
        self.expect("id")?;
        self.expect("(")?;
        let Token::Id(id) = self.current().clone() else {
            return Err(syntax("expected identity string"));
        };
        self.position += 1;
        self.expect(")")?;
        Ok(id)
    }
    fn ty(&mut self) -> Fallible<Type> {
        if self.eat("I64") {
            Ok(Type::I64)
        } else if self.eat("Bool") {
            Ok(Type::Bool)
        } else {
            Err(syntax("expected I64 or Bool"))
        }
    }
    fn program(&mut self) -> Fallible<Program> {
        self.expect("module")?;
        let module = self.name()?;
        let id = self.identity()?;
        self.expect(";")?;
        let mut functions = Vec::new();
        while self.current() != &Token::End {
            functions.push(self.function()?);
        }
        if functions.is_empty() {
            return Err(syntax("module needs at least one function"));
        }
        Ok(Program {
            module,
            id,
            functions,
        })
    }
    fn function(&mut self) -> Fallible<Function> {
        self.expect("function")?;
        let name = self.name()?;
        let id = self.identity()?;
        self.expect("(")?;
        let mut parameters = Vec::new();
        if !self.is(")") {
            loop {
                let name = self.name()?;
                self.expect(":")?;
                parameters.push(Parameter { name, ty: self.ty()? });
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        self.expect("->")?;
        let output = self.ty()?;
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
            parameters,
            output,
            requires,
            ensures,
            body,
        })
    }
    fn block(&mut self) -> Fallible<Block> {
        self.expect("{")?;
        let mut bindings = Vec::new();
        while self.eat("let") {
            let name = self.name()?;
            let annotation = if self.eat(":") { Some(self.ty()?) } else { None };
            self.expect("=")?;
            let value = self.expr(0)?;
            self.expect(";")?;
            bindings.push(Binding {
                name,
                annotation,
                value,
            });
        }
        let value = Box::new(self.expr(0)?);
        self.expect("}")?;
        Ok(Block { bindings, value })
    }
    fn expr(&mut self, level: u8) -> Fallible<Expr> {
        if level == 6 {
            return self.unary();
        }
        let mut left = self.expr(level + 1)?;
        let mut count = 0;
        while let Token::Symbol(s) = self.current() {
            let op = s.clone();
            let matches = match level {
                0 => op == "||",
                1 => op == "&&",
                2 => matches!(op.as_str(), "==" | "!="),
                3 => matches!(op.as_str(), "<" | "<=" | ">" | ">="),
                4 => matches!(op.as_str(), "+" | "-"),
                _ => matches!(op.as_str(), "*" | "/" | "%"),
            };
            if !matches || (matches!(level, 2 | 3) && count > 0) {
                break;
            }
            self.position += 1;
            let right = self.expr(level + 1)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
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
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Diagnostic::new(
                "resource.frontend.limit",
                "parse",
                "nesting limit exceeded",
            ));
        }
        let result = self.unary_inner();
        self.depth -= 1;
        result
    }
    fn unary_inner(&mut self) -> Fallible<Expr> {
        if self.eat("-") {
            if let Token::Number(text) = self.current().clone() {
                self.position += 1;
                let value = format!("-{text}").parse::<i64>().map_err(|_| {
                    Diagnostic::new("compile.literal.range", "parse", "negative literal outside I64")
                })?;
                return Ok(Expr::Literal {
                    value: Value::I64(value),
                });
            }
            return Ok(Expr::Unary {
                op: "-".into(),
                operand: Box::new(self.unary()?),
            });
        }
        if self.eat("!") {
            return Ok(Expr::Unary {
                op: "!".into(),
                operand: Box::new(self.unary()?),
            });
        }
        if self.eat("(") {
            let value = self.expr(0)?;
            self.expect(")")?;
            return Ok(value);
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
            });
        }
        if self.eat("true") {
            return Ok(Expr::Literal {
                value: Value::Bool(true),
            });
        }
        if self.eat("false") {
            return Ok(Expr::Literal {
                value: Value::Bool(false),
            });
        }
        if let Token::Number(text) = self.current().clone() {
            self.position += 1;
            let value = text
                .parse::<i64>()
                .map_err(|_| Diagnostic::new("compile.literal.range", "parse", "literal outside I64"))?;
            return Ok(Expr::Literal {
                value: Value::I64(value),
            });
        }
        if let Token::Word(name) = self.current().clone() {
            if name != "result" {
                check_name(&name)?;
            }
            self.position += 1;
            if !self.eat("(") {
                return Ok(Expr::Name { name });
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
                arguments,
            });
        }
        Err(syntax("expected expression"))
    }
}

struct Checker<'a> {
    functions: BTreeMap<String, &'a Function>,
    names: BTreeSet<String>,
    depth: usize,
}

fn mismatch(message: &str) -> Diagnostic {
    Diagnostic::new("compile.type.mismatch", "check", message)
}
fn same(actual: Type, expected: Type) -> Fallible<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(mismatch("types must match exactly"))
    }
}

impl Checker<'_> {
    fn block(&mut self, block: &Block, outer: &BTreeMap<String, Type>) -> Fallible<Type> {
        let mut env = outer.clone();
        for binding in &block.bindings {
            check_name(&binding.name)?;
            if env.contains_key(&binding.name) || self.names.contains(&binding.name) {
                return Err(Diagnostic::new(
                    "compile.name.duplicate",
                    "resolve",
                    "binding shadows a visible name",
                ));
            }
            let ty = self.expr(&binding.value, &env)?;
            if let Some(annotation) = binding.annotation {
                same(ty, annotation)?;
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
        let result = self.expr_inner(expression, env);
        self.depth -= 1;
        result
    }
    fn expr_inner(&mut self, expression: &Expr, env: &BTreeMap<String, Type>) -> Fallible<Type> {
        match expression {
            Expr::Literal { value } => Ok(value.ty()),
            Expr::Name { name } => env.get(name).copied().ok_or_else(|| {
                Diagnostic::new(
                    "compile.name.unresolved",
                    "resolve",
                    format!("unknown value: {name}"),
                )
            }),
            Expr::Unary { op, operand } => {
                let expected = match op.as_str() {
                    "-" => Type::I64,
                    "!" => Type::Bool,
                    _ => return Err(syntax("unknown unary operation")),
                };
                same(self.expr(operand, env)?, expected)?;
                Ok(expected)
            }
            Expr::Binary { op, left, right } => {
                let a = self.expr(left, env)?;
                let b = self.expr(right, env)?;
                same(a, b)?;
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => {
                        same(a, Type::I64)?;
                        Ok(Type::I64)
                    }
                    "<" | "<=" | ">" | ">=" => {
                        same(a, Type::I64)?;
                        Ok(Type::Bool)
                    }
                    "==" | "!=" => Ok(Type::Bool),
                    "&&" | "||" => {
                        same(a, Type::Bool)?;
                        Ok(Type::Bool)
                    }
                    _ => Err(syntax("unknown binary operation")),
                }
            }
            Expr::Call { function, arguments } => {
                let target = *self.functions.get(function).ok_or_else(|| {
                    Diagnostic::new(
                        "compile.name.unresolved",
                        "resolve",
                        format!("unknown function: {function}"),
                    )
                })?;
                if target.parameters.len() != arguments.len() {
                    return Err(Diagnostic::new(
                        "compile.call.arity",
                        "check",
                        "wrong number of arguments",
                    ));
                }
                for (argument, parameter) in arguments.iter().zip(&target.parameters) {
                    same(self.expr(argument, env)?, parameter.ty)?;
                }
                Ok(target.output)
            }
            Expr::If {
                condition,
                then_block,
                else_block,
            } => {
                same(self.expr(condition, env)?, Type::Bool)?;
                let a = self.block(then_block, env)?;
                same(self.block(else_block, env)?, a)?;
                Ok(a)
            }
        }
    }
}

fn validate(program: &Program, resolved: bool) -> Fallible<()> {
    check_name(&program.module)?;
    if !valid_id(&program.id) || program.functions.is_empty() {
        return Err(syntax("invalid module identity or empty module"));
    }
    let mut ids = BTreeSet::from([program.id.clone()]);
    let mut names = BTreeSet::new();
    let mut functions = BTreeMap::new();
    for function in &program.functions {
        check_name(&function.name)?;
        if !valid_id(&function.id) {
            return Err(syntax("invalid function identity"));
        }
        if !ids.insert(function.id.clone()) {
            return Err(Diagnostic::new(
                "compile.identity.duplicate",
                "resolve",
                "identity is repeated",
            ));
        }
        if !names.insert(function.name.clone()) {
            return Err(Diagnostic::new(
                "compile.name.duplicate",
                "resolve",
                "function name is repeated",
            ));
        }
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
        names,
        depth: 0,
    };
    for function in &program.functions {
        let result = (|| {
            let mut env = BTreeMap::new();
            for parameter in &function.parameters {
                check_name(&parameter.name)?;
                if checker.names.contains(&parameter.name)
                    || env.insert(parameter.name.clone(), parameter.ty).is_some()
                {
                    return Err(Diagnostic::new(
                        "compile.name.duplicate",
                        "resolve",
                        "parameter name is repeated or shadows a function",
                    ));
                }
            }
            for requirement in &function.requires {
                same(checker.expr(requirement, &env)?, Type::Bool)?;
            }
            same(checker.block(&function.body, &env)?, function.output)?;
            env.insert("result".into(), function.output);
            for assurance in &function.ensures {
                same(checker.expr(assurance, &env)?, Type::Bool)?;
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

fn resolve_block(block: &mut Block, ids: &BTreeMap<String, String>) {
    for binding in &mut block.bindings {
        resolve_expr(&mut binding.value, ids);
    }
    resolve_expr(&mut block.value, ids);
}

fn resolve_expr(expression: &mut Expr, ids: &BTreeMap<String, String>) {
    match expression {
        Expr::Literal { .. } | Expr::Name { .. } => (),
        Expr::Unary { operand, .. } => resolve_expr(operand, ids),
        Expr::Binary { left, right, .. } => {
            resolve_expr(left, ids);
            resolve_expr(right, ids);
        }
        Expr::Call { function, arguments } => {
            *function = ids[function].clone();
            for argument in arguments {
                resolve_expr(argument, ids);
            }
        }
        Expr::If {
            condition,
            then_block,
            else_block,
        } => {
            resolve_expr(condition, ids);
            resolve_block(then_block, ids);
            resolve_block(else_block, ids);
        }
    }
}

pub fn compile(source: &str) -> Fallible<Artifact> {
    let tokens = lex(source)?;
    let mut program = Parser {
        tokens,
        position: 0,
        depth: 0,
    }
    .program()?;
    validate(&program, false)?;
    let ids = program
        .functions
        .iter()
        .map(|f| (f.name.clone(), f.id.clone()))
        .collect();
    for function in &mut program.functions {
        for expression in function.requires.iter_mut().chain(&mut function.ensures) {
            resolve_expr(expression, &ids);
        }
        resolve_block(&mut function.body, &ids);
    }
    Ok(Artifact {
        format: FORMAT.into(),
        language: LANGUAGE.into(),
        program,
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
        if self.format != FORMAT || self.language != LANGUAGE {
            return Err(Diagnostic::new(
                "invocation.artifact.version",
                "load",
                "unsupported artifact or language version",
            ));
        }
        validate(&self.program, true)
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    pub fn entries(&self) -> Vec<&str> {
        self.program.functions.iter().map(|f| f.id.as_str()).collect()
    }
    pub fn run(&self, entry: &str, arguments: Vec<Value>, fuel: u64) -> Fallible<Value> {
        self.check()?;
        let functions = self
            .program
            .functions
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();
        Runner {
            functions,
            fuel,
            depth: 0,
            expression_depth: 0,
        }
        .call(entry, arguments)
    }
}

struct Runner<'a> {
    functions: BTreeMap<&'a str, &'a Function>,
    fuel: u64,
    depth: usize,
    expression_depth: usize,
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
        if arguments.len() != function.parameters.len()
            || arguments
                .iter()
                .zip(&function.parameters)
                .any(|(v, p)| v.ty() != p.ty)
        {
            return Err(Diagnostic::new(
                "invocation.arguments.invalid",
                "invoke",
                "arguments must match parameter types and arity",
            ));
        }
        self.tick()?;
        if self.depth >= MAX_DEPTH {
            return Err(Diagnostic::new(
                "resource.stack.exhausted",
                "execute",
                "call depth limit exceeded",
            ));
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
                let value = self.expr(requirement, &env).map_err(|mut e| {
                    if e.code.starts_with("runtime.") {
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
            let value = self.block(&function.body, &env)?;
            env.insert("result".into(), value.clone());
            for assurance in &function.ensures {
                let checked = self.expr(assurance, &env).map_err(|mut e| {
                    if e.code.starts_with("runtime.") {
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
    fn block(&mut self, block: &Block, outer: &BTreeMap<String, Value>) -> Fallible<Value> {
        let mut env = outer.clone();
        for binding in &block.bindings {
            let value = self.expr(&binding.value, &env)?;
            env.insert(binding.name.clone(), value);
        }
        self.expr(&block.value, &env)
    }
    fn expr(&mut self, expression: &Expr, env: &BTreeMap<String, Value>) -> Fallible<Value> {
        self.tick()?;
        self.expression_depth += 1;
        if self.expression_depth > 256 {
            return Err(Diagnostic::new(
                "resource.stack.exhausted",
                "execute",
                "evaluation depth limit exceeded",
            ));
        }
        let result = self.expr_inner(expression, env);
        self.expression_depth -= 1;
        result
    }
    fn expr_inner(&mut self, expression: &Expr, env: &BTreeMap<String, Value>) -> Fallible<Value> {
        match expression {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Name { name } => Ok(env[name].clone()),
            Expr::Unary { op, operand } => {
                let value = self.expr(operand, env)?;
                match (op.as_str(), value) {
                    ("-", Value::I64(n)) => checked(n.checked_neg()),
                    ("!", Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err(Diagnostic::new(
                        "internal.invalid_ir",
                        "execute",
                        "invalid checked unary expression",
                    )),
                }
            }
            Expr::Binary { op, left, right } => {
                let a = self.expr(left, env)?;
                if op == "&&" && a == Value::Bool(false) {
                    return Ok(a);
                }
                if op == "||" && a == Value::Bool(true) {
                    return Ok(a);
                }
                let b = self.expr(right, env)?;
                match (op.as_str(), a, b) {
                    ("+", Value::I64(a), Value::I64(b)) => checked(a.checked_add(b)),
                    ("-", Value::I64(a), Value::I64(b)) => checked(a.checked_sub(b)),
                    ("*", Value::I64(a), Value::I64(b)) => checked(a.checked_mul(b)),
                    ("/" | "%", Value::I64(_), Value::I64(0)) => Err(Diagnostic::new(
                        "runtime.integer.zero_divisor",
                        "execute",
                        "zero divisor",
                    )),
                    ("/", Value::I64(a), Value::I64(b)) => checked(a.checked_div(b)),
                    ("%", Value::I64(a), Value::I64(b)) => checked(a.checked_rem(b)),
                    ("<", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a < b)),
                    ("<=", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a <= b)),
                    (">", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a > b)),
                    (">=", Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a >= b)),
                    ("==", a, b) => Ok(Value::Bool(a == b)),
                    ("!=", a, b) => Ok(Value::Bool(a != b)),
                    ("&&", Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    ("||", Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => Err(Diagnostic::new(
                        "internal.invalid_ir",
                        "execute",
                        "invalid checked binary expression",
                    )),
                }
            }
            Expr::Call { function, arguments } => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, env)?);
                }
                self.call(function, values)
            }
            Expr::If {
                condition,
                then_block,
                else_block,
            } => {
                if self.expr(condition, env)? == Value::Bool(true) {
                    self.block(then_block, env)
                } else {
                    self.block(else_block, env)
                }
            }
        }
    }
}

fn checked(value: Option<i64>) -> Fallible<Value> {
    value
        .map(Value::I64)
        .ok_or_else(|| Diagnostic::new("runtime.integer.overflow", "execute", "I64 arithmetic overflow"))
}
