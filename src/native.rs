use super::{
    Artifact, Block, Diagnostic, Expr, Fallible, Function, FunctionValue, Record, Type, Value, Variant,
};
use crate::propagation;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
#[path = "native_lists.rs"]
mod list_support;
#[path = "native_llvm.rs"]
mod llvm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBackend {
    C,
    Llvm,
}

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

pub(crate) struct NativeTemp {
    pub(crate) path: PathBuf,
}

impl NativeTemp {
    pub(crate) fn new() -> Fallible<Self> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| native_error("compile.native.io", e.to_string()))?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "lobo-native-{}-{now}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).map_err(|e| native_error("compile.native.io", e.to_string()))?;
        Ok(Self { path })
    }
}

impl Drop for NativeTemp {
    fn drop(&mut self) {
        for name in [
            "program.c",
            "program",
            "run",
            "application.ll",
            "runtime.ll",
            "linked.ll",
        ] {
            let _ = fs::remove_file(self.path.join(name));
        }
        let _ = fs::remove_dir(&self.path);
    }
}

fn native_error(code: &str, message: impl Into<String>) -> Diagnostic {
    // A host/tool failure does not mean that the program violates a language rule.
    let code = code.replacen("compile.native.", "invocation.native.", 1);
    Diagnostic::new(&code, "native_codegen", message)
}

struct Emitter<'a> {
    text: String,
    expression_fuel: bool,
    functions: BTreeMap<&'a str, (usize, &'a Function)>,
    next: usize,
    output: Type,
    records: BTreeMap<&'a str, (usize, &'a Record)>,
    variants: BTreeMap<&'a str, (usize, &'a Variant)>,
    lists: BTreeMap<Type, usize>,
    layouts: std::collections::BTreeSet<Type>,
    widths: BTreeMap<String, (usize, usize)>,
}

type Locals = BTreeMap<String, (String, Type)>;

impl Emitter<'_> {
    fn ctype(&self, ty: &Type) -> String {
        match ty {
            Type::Applied { .. } | Type::Parameter(_) => {
                unreachable!("checked program contains only concrete types")
            }
            Type::Resource(_) => "uint64_t".into(),
            Type::I64 => "int64_t".into(),
            Type::F64 => "double".into(),
            Type::Bool => "bool".into(),
            Type::Bytes | Type::Text => "lobo_buffer".into(),
            Type::Record(id) => format!("lobo_record_{}", self.records[id.as_str()].0),
            Type::Variant(id) => format!("lobo_variant_{}", self.variants[id.as_str()].0),
            Type::List(_) => "const lobo_list *".into(),
            Type::Function(_) => "uint64_t".into(),
        }
    }
    fn equal(&self, ty: &Type, left: &str, right: &str) -> String {
        if matches!(ty, Type::Bytes | Type::Text) {
            return format!("lobo_buffers_equal({left}, {right})");
        }
        if let Type::Record(id) = ty {
            format!("lobo_equal_{}({left}, {right})", self.records[id.as_str()].0)
        } else if let Type::Variant(id) = ty {
            format!(
                "lobo_variant_equal_{}({left}, {right})",
                self.variants[id.as_str()].0
            )
        } else if matches!(ty, Type::List(_)) {
            format!("lobo_equal_list_{}({left}, {right})", self.lists[ty])
        } else {
            format!("({left} == {right})")
        }
    }
    fn record_definition(&mut self, index: usize, record: &Record) {
        self.line("typedef struct {");
        for (i, field) in record.fields.iter().enumerate() {
            self.line(format!("{} f{i};", self.ctype(&field.ty)));
        }
        self.line(format!("}} lobo_record_{index};"));
    }
    fn record_helpers(&mut self, index: usize, record: &Record) {
        let comparisons = record
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| self.equal(&f.ty, &format!("a.f{i}"), &format!("b.f{i}")))
            .collect::<Vec<_>>();
        self.line(format!(
            "static bool lobo_equal_{index}(lobo_record_{index} a, lobo_record_{index} b) {{ lobo_compare_enter(); bool result = {}; lobo_compare_leave(); return result; }}",
            comparisons.join(" && ")
        ));
        self.line(format!(
            "static void lobo_print_record_{index}(lobo_record_{index} value) {{"
        ));
        self.line(format!(
            r#"fputs("{{\"type\":\"Record\",\"value\":{{\"id\":\"{}\",\"fields\":{{", stdout);"#,
            record.id
        ));
        for (i, field) in record.fields.iter().enumerate() {
            if i != 0 {
                self.line("fputs(\",\", stdout);");
            }
            self.line(format!(r#"fputs("\"{}\":", stdout);"#, field.name));
            self.print_value(&field.ty, &format!("value.f{i}"));
        }
        self.line("fputs(\"}}}\", stdout); }");
        self.line(format!("static lobo_record_{index} lobo_parse_record_{index}(const char **cursor) {{ lobo_input_tick(); ++lobo_input_depth; lobo_record_{index} value;"));
        for (i, field) in record.fields.iter().enumerate() {
            self.parse_value(&field.ty, &format!("value.f{i}"), "cursor");
        }
        self.line("--lobo_input_depth; return value; }");
        self.line(format!(
            "static void lobo_count_record_{index}(lobo_record_{index} value) {{ lobo_output_tick(); ++lobo_output_depth;"
        ));
        let prefix = format!(
            "{{\"type\":\"Record\",\"value\":{{\"id\":\"{}\",\"fields\":{{",
            record.id
        );
        self.line(format!("lobo_output_bytes({});", prefix.len() + 3));
        for (i, field) in record.fields.iter().enumerate() {
            self.line(format!(
                "lobo_output_bytes({});",
                field.name.len() + 3 + usize::from(i > 0)
            ));
            self.count_value(&field.ty, &format!("value.f{i}"));
        }
        self.line("--lobo_output_depth; }");
    }
    fn variant_definition(&mut self, index: usize, variant: &Variant) {
        self.line("typedef struct { unsigned tag; union { unsigned char empty;");
        for (i, case) in variant.cases.iter().enumerate() {
            if let Some(ty) = &case.payload {
                self.line(format!("{} c{i};", self.ctype(ty)));
            }
        }
        self.line(format!("}} payload; }} lobo_variant_{index};"));
    }
    fn variant_helpers(&mut self, index: usize, variant: &Variant) {
        self.line(format!("static bool lobo_variant_equal_{index}(lobo_variant_{index} a, lobo_variant_{index} b) {{ lobo_compare_enter(); bool result = false; if (a.tag == b.tag) switch (a.tag) {{"));
        for (i, case) in variant.cases.iter().enumerate() {
            let equality = case
                .payload
                .as_ref()
                .map(|ty| self.equal(ty, &format!("a.payload.c{i}"), &format!("b.payload.c{i}")))
                .unwrap_or_else(|| "true".into());
            self.line(format!("case {i}: result = {equality}; break;"));
        }
        self.line("default: abort(); } lobo_compare_leave(); return result; }");
        self.line(format!(
            "static void lobo_print_variant_{index}(lobo_variant_{index} value) {{ switch (value.tag) {{"
        ));
        for (i, case) in variant.cases.iter().enumerate() {
            self.line(format!("case {i}:"));
            self.line(format!(r#"fputs("{{\"type\":\"Variant\",\"value\":{{\"id\":\"{}\",\"case\":\"{}\",\"payload\":", stdout);"#, variant.id, case.name));
            if let Some(ty) = &case.payload {
                self.print_value(ty, &format!("value.payload.c{i}"));
            } else {
                self.line("fputs(\"null\", stdout);");
            }
            self.line("fputs(\"}}\", stdout); break;");
        }
        self.line("default: abort(); } }");
        self.line(format!("static lobo_variant_{index} lobo_parse_variant_{index}(const char **cursor) {{ lobo_input_tick(); ++lobo_input_depth; lobo_variant_{index} value = {{0}}; size_t length; const char *token = lobo_next(cursor, &length);"));
        for (i, case) in variant.cases.iter().enumerate() {
            let token = format!("Case:{}", case.name);
            self.line(format!(
                "if (length == {} && strncmp(token, \"{token}\", length) == 0) {{ value.tag = {i};",
                token.len()
            ));
            if let Some(ty) = &case.payload {
                self.parse_value(ty, &format!("value.payload.c{i}"), "cursor");
            }
            self.line("--lobo_input_depth; return value; }");
        }
        self.line("lobo_fail(\"invocation.arguments.invalid\"); }");
        self.line(format!("static void lobo_count_variant_{index}(lobo_variant_{index} value) {{ lobo_output_tick(); ++lobo_output_depth; switch (value.tag) {{"));
        for (i, case) in variant.cases.iter().enumerate() {
            self.line(format!("case {i}:"));
            let prefix = format!(
                "{{\"type\":\"Variant\",\"value\":{{\"id\":\"{}\",\"case\":\"{}\",\"payload\":",
                variant.id, case.name
            );
            self.line(format!("lobo_output_bytes({});", prefix.len() + 2));
            if let Some(ty) = &case.payload {
                self.count_value(ty, &format!("value.payload.c{i}"));
            } else {
                self.line("lobo_output_bytes(4);");
            }
            self.line("break;");
        }
        self.line("default: abort(); } --lobo_output_depth; }");
    }
    fn print_value(&mut self, ty: &Type, value: &str) {
        match ty {
            Type::Resource(_) => self.line("lobo_fail(\"invocation.native.resource_value\");"),
            Type::F64 => self.line(format!(
                r#"printf("{{\"type\":\"F64\",\"value\":\"0x%016" PRIx64 "\"}}", lobo_f64_bits({value}));"#
            )),
            Type::Function(signature) => {
                self.line(format!("switch ({value}) {{"));
                for (index, target) in self.functions.values().copied().collect::<Vec<_>>() {
                    if target.signature_type() == *signature {
                        let wire = serde_json::to_string(&Value::Function(FunctionValue {
                            id: target.id.clone(),
                            signature: signature.clone(),
                        }))
                        .unwrap();
                        self.line(format!(
                            "case {index}: fputs({}, stdout); break;",
                            serde_json::to_string(&wire).unwrap()
                        ));
                    }
                }
                self.line("default: lobo_fail(\"invocation.arguments.invalid\"); }");
            }
            Type::Applied { .. } | Type::Parameter(_) => unreachable!("checked type"),
            Type::Bytes | Type::Text => {
                self.line(format!("lobo_print_buffer({value}, {});", *ty == Type::Text))
            }
            Type::List(_) => self.line(format!("lobo_print_list_{}({value});", self.lists[ty])),
            Type::I64 => self.line(format!(
                r#"printf("{{\"type\":\"I64\",\"value\":\"%" PRId64 "\"}}", {value});"#
            )),
            Type::Bool => self.line(format!(
                r#"printf("{{\"type\":\"Bool\",\"value\":%s}}", {value} ? "true" : "false");"#
            )),
            Type::Record(id) => {
                self.line(format!(
                    "lobo_print_record_{}({value});",
                    self.records[id.as_str()].0
                ));
            }
            Type::Variant(id) => {
                self.line(format!(
                    "lobo_print_variant_{}({value});",
                    self.variants[id.as_str()].0
                ));
            }
        }
    }
    fn parse_value(&mut self, ty: &Type, value: &str, cursor: &str) {
        match ty {
            Type::Resource(_) => self.line("lobo_fail(\"invocation.arguments.invalid\");"),
            Type::F64 => self.line(format!("{value} = lobo_parse_f64({cursor});")),
            Type::Function(signature) => {
                self.line("lobo_input_tick();");
                let atom = self.temp();
                self.line(format!(
                    "size_t {atom}_length; const char *{atom} = lobo_next({cursor}, &{atom}_length);"
                ));
                let mut first = true;
                for (index, target) in self.functions.values().copied().collect::<Vec<_>>() {
                    if target.signature_type() == *signature {
                        let token = format!("Function:{}", target.id);
                        self.line(format!("{}if ({atom}_length == {} && memcmp({atom}, \"{token}\", {}) == 0) {value} = {index};", if first { "" } else { "else " },token.len(),token.len()));
                        first = false;
                    }
                }
                self.line(format!(
                    "{}lobo_fail(\"invocation.arguments.invalid\");",
                    if first { "" } else { "else " }
                ));
            }
            Type::Bytes | Type::Text => self.line(format!(
                "{value} = lobo_parse_buffer({cursor}, {});",
                *ty == Type::Text
            )),
            Type::List(_) => self.line(format!("{value} = lobo_parse_list_{}({cursor});", self.lists[ty])),
            Type::Record(id) => {
                self.line(format!(
                    "{value} = lobo_parse_record_{}({cursor});",
                    self.records[id.as_str()].0
                ));
            }
            Type::Variant(id) => {
                self.line(format!(
                    "{value} = lobo_parse_variant_{}({cursor});",
                    self.variants[id.as_str()].0
                ));
            }
            _ => {
                self.line("lobo_input_tick();");
                let scalar = self.temp();
                self.line(format!("struct lobo_value {scalar} = lobo_leaf({cursor});"));
                self.line(format!("if ({scalar}.is_bool != {}) lobo_fail(\"invocation.arguments.invalid\"); {value} = {scalar}.{};", if *ty == Type::Bool { "true" } else { "false" }, if *ty == Type::Bool { "boolean" } else { "integer" }));
            }
        }
    }
    fn temp(&mut self) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        name
    }
    fn line(&mut self, text: impl AsRef<str>) {
        writeln!(self.text, "{}", text.as_ref()).unwrap();
    }
    fn expr(&mut self, expr: &Expr, locals: &Locals) -> (String, Type) {
        if self.expression_fuel {
            self.line("lobo_tick();");
        }
        let name = self.temp();
        let ty = match expr {
            Expr::Reference { function, .. } => {
                let (index, target) = self.functions[function.as_str()];
                self.line(format!("uint64_t {name} = {index};"));
                Type::Function(target.signature_type())
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                let (callee, Type::Function(signature)) = self.expr(callee, locals) else {
                    unreachable!("checked invocation")
                };
                let args: Vec<_> = arguments.iter().map(|arg| self.expr(arg, locals).0).collect();
                self.line(format!(
                    "{} {name}; switch ({callee}) {{",
                    self.ctype(&signature.output)
                ));
                for (index, target) in self.functions.values().copied().collect::<Vec<_>>() {
                    if target.signature_type() == signature {
                        self.line(format!(
                            "case {index}: {name} = lobo_fn_{index}({}); break;",
                            args.join(", ")
                        ));
                    }
                }
                self.line("default: lobo_fail(\"invocation.arguments.invalid\"); }");
                *signature.output
            }
            Expr::NativeOp {
                binding,
                abi,
                arguments,
                ..
            } => {
                let signature = super::native_api::signature(binding, *abi).unwrap();
                let args: Vec<_> = arguments.iter().map(|arg| self.expr(arg, locals).0).collect();
                if let Type::Variant(id) = &signature.output {
                    if super::native_api::is_result(id) {
                        let raw = self.temp();
                        self.line(format!(
                            "lobo_native_result_v1 {raw} = {}({});",
                            signature.symbol,
                            args.join(", ")
                        ));
                        self.line(format!("{} {name} = {{0}};", self.ctype(&signature.output)));
                        let payload = self.variants[id.as_str()].1.cases[0].payload.as_ref().unwrap();
                        let member = match payload {
                            Type::Resource(_) => "handle",
                            Type::Bytes | Type::Text => "buffer",
                            Type::F64 => "floating",
                            Type::Bool => "boolean",
                            _ => "integer",
                        };
                        self.line(format!("if ({raw}.error) {{ {name}.tag=1; {name}.payload.c1={raw}.error; }} else {{ {name}.tag=0; {name}.payload.c0={raw}.value.{member}; }}"));
                    } else {
                        unreachable!("closed native result layout");
                    }
                } else {
                    self.line(format!(
                        "{} {name} = {}({});",
                        self.ctype(&signature.output),
                        signature.symbol,
                        args.join(", ")
                    ));
                }
                signature.output
            }
            Expr::BufferOp {
                family,
                op,
                arguments,
                ..
            } => {
                let mut args = Vec::new();
                for argument in arguments {
                    args.push(self.expr(argument, locals).0);
                }
                let output = super::buffers::signature(family, op).unwrap().1;
                let call = match (family.as_str(), op.as_str()) {
                    ("Bytes", "empty") => "(lobo_buffer){NULL,0}".into(),
                    ("Bytes", "length") => format!("(int64_t){}.length", args[0]),
                    ("Text", "bytes") => args[0].clone(),
                    (_, "concat") => format!("lobo_buffers_concat({})", args.join(", ")),
                    _ => format!("lobo_{}_{}({})", family.to_lowercase(), op, args.join(", ")),
                };
                self.line(format!("{} {name} = {call};", self.ctype(&output)));
                output
            }
            Expr::List { element, .. } => {
                self.line(format!("const lobo_list *{name} = NULL;"));
                Type::List(Box::new(element.clone()))
            }
            Expr::ListOp { op, arguments, .. } => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, locals));
                }
                let (list, Type::List(element)) = values.last().unwrap() else {
                    unreachable!("checked list");
                };
                match op.as_str() {
                    "length" => {
                        self.line(format!("int64_t {name} = {list} ? (int64_t){list}->length : 0;"));
                        Type::I64
                    }
                    "head" | "tail" => {
                        self.line(format!("if (!{list}) lobo_fail(\"runtime.list.empty\");"));
                        if op == "head" {
                            self.line(format!(
                                "{} {name}; memcpy(&{name}, {list}->data, sizeof({name}));",
                                self.ctype(element)
                            ));
                            *element.clone()
                        } else {
                            self.line(format!("const lobo_list *{name} = {list}->tail;"));
                            Type::List(element.clone())
                        }
                    }
                    _ => {
                        let value = &values[0].0;
                        let node = self.temp();
                        self.line(format!(
                            "lobo_list *{node} = lobo_cons({}, {list});",
                            self.inline_width(element)
                        ));
                        self.line(format!(
                            "memcpy({node}->data, &{value}, sizeof({value})); const lobo_list *{name} = {node};"
                        ));
                        Type::List(element.clone())
                    }
                }
            }
            Expr::Variant {
                variant,
                case,
                payload,
                ..
            } => {
                let (index, definition) = self.variants[variant.as_str()];
                let tag = definition.cases.iter().position(|c| &c.name == case).unwrap();
                let payload = payload.as_ref().map(|v| self.expr(v, locals).0);
                let initializer = payload
                    .map(|v| format!(", .payload.c{tag} = {v}"))
                    .unwrap_or_default();
                self.line(format!(
                    "lobo_variant_{index} {name} = {{ .tag = {tag}{initializer} }};"
                ));
                Type::Variant(variant.clone())
            }
            Expr::Match { subject, arms, .. } => {
                let (value, Type::Variant(id)) = self.expr(subject, locals) else {
                    unreachable!("checked variant");
                };
                let ty = self.expr_type(expr, locals);
                self.line(format!("{} {name}; switch ({value}.tag) {{", self.ctype(&ty)));
                let definition = self.variants[id.as_str()].1;
                for arm in arms {
                    let (tag, case) = definition
                        .cases
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.name == arm.case)
                        .unwrap();
                    self.line(format!("case {tag}: {{"));
                    let mut inner = locals.clone();
                    if let (Some(binding), Some(ty)) = (&arm.binding, &case.payload) {
                        inner.insert(binding.clone(), (format!("{value}.payload.c{tag}"), ty.clone()));
                    }
                    let (result, _) = self.block(&arm.body, &inner);
                    self.line(format!("{name} = {result}; break; }}"));
                }
                self.line("default: abort(); }");
                ty
            }
            Expr::Record { record, fields, .. } => {
                let (_, definition) = self.records[record.as_str()];
                let mut initializers = Vec::new();
                for field in fields {
                    let index = definition
                        .fields
                        .iter()
                        .position(|f| f.name == field.name)
                        .unwrap();
                    let value = self.expr(&field.value, locals).0;
                    initializers.push(format!(".f{index} = {value}"));
                }
                let ty = Type::Record(record.clone());
                self.line(format!(
                    "{} {name} = {{ {} }};",
                    self.ctype(&ty),
                    initializers.join(", ")
                ));
                ty
            }
            Expr::Field { base, field, .. } => {
                let (value, Type::Record(id)) = self.expr(base, locals) else {
                    unreachable!("checked record base")
                };
                let (_, definition) = self.records[id.as_str()];
                let (index, target) = definition
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, f)| &f.name == field)
                    .unwrap();
                self.line(format!("{} {name} = {value}.f{index};", self.ctype(&target.ty)));
                target.ty.clone()
            }
            Expr::Literal { value, .. } => {
                if let Value::Text(text) = value {
                    let encoded = text
                        .bytes()
                        .iter()
                        .map(|byte| format!("\\{byte:03o}"))
                        .collect::<String>();
                    self.line(format!("static const unsigned char {name}_text[] = \"{encoded}\"; lobo_buffer {name} = {{ {name}_text, UINT64_C({}) }};",text.len()));
                    return (name, Type::Text);
                }
                let (ty, literal) = match value {
                    Value::F64(bits) => (Type::F64, format!("lobo_f64_from_u64(UINT64_C({bits}))")),
                    Value::I64(i64::MIN) => (Type::I64, "INT64_MIN".into()),
                    Value::I64(n) if *n < 0 => (Type::I64, format!("(-INT64_C({}))", -n)),
                    Value::I64(n) => (Type::I64, format!("INT64_C({n})")),
                    Value::Bool(b) => (Type::Bool, b.to_string()),
                    Value::Function(_)
                    | Value::Record(_)
                    | Value::Variant(_)
                    | Value::List(_)
                    | Value::Bytes(_)
                    | Value::Text(_) => {
                        unreachable!("checked scalar literal")
                    }
                };
                self.line(format!("{} {name} = {literal};", self.ctype(&ty)));
                ty
            }
            Expr::Name { name: binding, .. } => {
                let (value, ty) = &locals[binding];
                self.line(format!("{} {name} = {value};", self.ctype(ty)));
                ty.clone()
            }
            Expr::Unary { op, operand, .. } => {
                let (value, ty) = self.expr(operand, locals);
                if op == "try" {
                    let Type::Variant(id) = ty else {
                        unreachable!("checked try operand")
                    };
                    let source = self.variants[id.as_str()].1;
                    let ok = source.cases.iter().position(|c| c.name == "Ok").unwrap();
                    let error = source.cases.iter().position(|c| c.name == "Err").unwrap();
                    let output = self.output.clone();
                    let Type::Variant(target) = &output else {
                        unreachable!("checked try output")
                    };
                    let target_error = self.variants[target.as_str()]
                        .1
                        .cases
                        .iter()
                        .position(|c| c.name == "Err")
                        .unwrap();
                    self.line(format!("if ({value}.tag == {error}) {{ lobo_return_value = ({}) {{ .tag = {target_error}, .payload.c{target_error} = {value}.payload.c{error} }}; goto lobo_return_epilogue; }}", self.ctype(&output)));
                    let ok_type = source.cases[ok].payload.as_ref().unwrap().clone();
                    self.line(format!(
                        "{} {name} = {value}.payload.c{ok};",
                        self.ctype(&ok_type)
                    ));
                    return (name, ok_type);
                }
                if op == "!" {
                    self.line(format!("bool {name} = !{value};"));
                } else if ty == Type::F64 {
                    self.line(format!("double {name} = lobo_f64_normalize(-{value});"));
                } else {
                    self.line(format!("int64_t {name};"));
                    self.line(format!("if (__builtin_sub_overflow(INT64_C(0), {value}, &{name})) lobo_fail(\"runtime.integer.overflow\");"));
                }
                ty
            }
            Expr::Binary { op, left, right, .. } => {
                let (left, left_ty) = self.expr(left, locals);
                if matches!(op.as_str(), "&&" | "||") {
                    self.line(format!("bool {name} = {left};"));
                    let test = if op == "&&" { left } else { format!("!{left}") };
                    self.line(format!("if ({test}) {{"));
                    let (right, _) = self.expr(right, locals);
                    self.line(format!("{name} = {right};"));
                    self.line("}");
                    return (name, Type::Bool);
                }
                let (right, _) = self.expr(right, locals);
                if left_ty == Type::F64 && matches!(op.as_str(), "+" | "-" | "*" | "/") {
                    self.line(format!(
                        "double {name} = lobo_f64_normalize({left} {op} {right});"
                    ));
                    return (name, Type::F64);
                }
                match op.as_str() {
                    "+" | "-" | "*" => {
                        let builtin = match op.as_str() {
                            "+" => "add",
                            "-" => "sub",
                            _ => "mul",
                        };
                        self.line(format!("int64_t {name};"));
                        self.line(format!("if (__builtin_{builtin}_overflow({left}, {right}, &{name})) lobo_fail(\"runtime.integer.overflow\");"));
                        left_ty
                    }
                    "/" | "%" => {
                        self.line(format!(
                            "if ({right} == 0) lobo_fail(\"runtime.integer.zero_divisor\");"
                        ));
                        self.line(format!("if ({left} == INT64_MIN && {right} == -1) lobo_fail(\"runtime.integer.overflow\");"));
                        self.line(format!("int64_t {name} = {left} {op} {right};"));
                        Type::I64
                    }
                    "==" | "!=" => {
                        let equality = self.equal(&left_ty, &left, &right);
                        self.line(format!(
                            "bool {name} = {}{equality};",
                            if op == "!=" { "!" } else { "" }
                        ));
                        Type::Bool
                    }
                    _ => {
                        self.line(format!("bool {name} = {left} {op} {right};"));
                        Type::Bool
                    }
                }
            }
            Expr::Call {
                function, arguments, ..
            } => {
                let (index, target) = self.functions[function.as_str()];
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, locals).0);
                }
                self.line(format!(
                    "{} {name} = lobo_fn_{index}({});",
                    self.ctype(&target.output),
                    values.join(", ")
                ));
                target.output.clone()
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let (condition, _) = self.expr(condition, locals);
                // The checker has already established that both branches have equal types.
                let ty = self.block_type(then_block, locals);
                self.line(format!("{} {name};", self.ctype(&ty)));
                self.line(format!("if ({condition}) {{"));
                let (then_value, _) = self.block(then_block, locals);
                self.line(format!("{name} = {then_value};"));
                self.line("} else {");
                let (else_value, _) = self.block(else_block, locals);
                self.line(format!("{name} = {else_value};"));
                self.line("}");
                ty
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                let (initial, _) = self.expr(initial, locals);
                self.line(format!("{} {name} = {initial};", self.ctype(&state.ty)));
                let mut inner = locals.clone();
                inner.insert(state.name.clone(), (name.clone(), state.ty.clone()));
                self.line("for (;;) {");
                self.line("lobo_tick();");
                let (condition, _) = self.expr(condition, &inner);
                self.line(format!("if (!{condition}) break;"));
                let (next, _) = self.block(body, &inner);
                self.line(format!("{name} = {next};"));
                self.line("}");
                state.ty.clone()
            }
            Expr::Fold {
                index,
                accumulator,
                count,
                initial,
                body,
                ..
            } => {
                let (count, _) = self.expr(count, locals);
                self.line(format!(
                    "if ({count} < 0) lobo_fail(\"runtime.iteration.negative_count\");"
                ));
                let (initial, _) = self.expr(initial, locals);
                self.line(format!("{} {name} = {initial};", self.ctype(&accumulator.ty)));
                let counter = self.temp();
                // count <= I64_MAX; the final increment reaches count, never count + 1.
                self.line(format!(
                    "for (int64_t {counter} = 0; {counter} < {count}; ++{counter}) {{"
                ));
                self.line("lobo_tick();");
                let mut inner = locals.clone();
                inner.insert(index.clone(), (counter, Type::I64));
                inner.insert(accumulator.name.clone(), (name.clone(), accumulator.ty.clone()));
                let (next, _) = self.block(body, &inner);
                self.line(format!("{name} = {next};"));
                self.line("}");
                accumulator.ty.clone()
            }
        };
        (name, ty)
    }
    fn expr_type(&self, expr: &Expr, locals: &Locals) -> Type {
        match expr {
            Expr::Reference { function, .. } => {
                Type::Function(self.functions[function.as_str()].1.signature_type())
            }
            Expr::Invoke { callee, .. } => {
                let Type::Function(signature) = self.expr_type(callee, locals) else {
                    unreachable!("checked invocation")
                };
                *signature.output
            }
            Expr::NativeOp { binding, abi, .. } => {
                super::native_api::signature(binding, *abi).unwrap().output
            }
            Expr::BufferOp { family, op, .. } => super::buffers::signature(family, op).unwrap().1,
            Expr::List { element, .. } => Type::List(Box::new(element.clone())),
            Expr::ListOp { op, arguments, .. } => {
                let Type::List(element) = self.expr_type(arguments.last().unwrap(), locals) else {
                    unreachable!("checked list");
                };
                match op.as_str() {
                    "length" => Type::I64,
                    "head" => *element,
                    _ => Type::List(element),
                }
            }
            Expr::Variant { variant, .. } => Type::Variant(variant.clone()),
            Expr::Match { subject, arms, .. } => {
                let Type::Variant(id) = self.expr_type(subject, locals) else {
                    unreachable!("checked variant");
                };
                let arm = &arms[0];
                let case = self.variants[id.as_str()]
                    .1
                    .cases
                    .iter()
                    .find(|c| c.name == arm.case)
                    .unwrap();
                let mut inner = locals.clone();
                if let (Some(name), Some(ty)) = (&arm.binding, &case.payload) {
                    inner.insert(name.clone(), (String::new(), ty.clone()));
                }
                self.block_type(&arm.body, &inner)
            }
            Expr::Record { record, .. } => Type::Record(record.clone()),
            Expr::Field { base, field, .. } => {
                let Type::Record(id) = self.expr_type(base, locals) else {
                    unreachable!("checked record")
                };
                self.records[id.as_str()]
                    .1
                    .fields
                    .iter()
                    .find(|f| &f.name == field)
                    .unwrap()
                    .ty
                    .clone()
            }
            Expr::Literal { value, .. } => value.ty(),
            Expr::Name { name, .. } => locals[name].1.clone(),
            Expr::Unary { op, operand, .. } => {
                let ty = self.expr_type(operand, locals);
                if op == "try" {
                    let Type::Variant(id) = ty else {
                        unreachable!("checked try")
                    };
                    propagation::shape(self.variants[id.as_str()].1)
                        .unwrap()
                        .0
                        .clone()
                } else {
                    ty
                }
            }
            Expr::Binary { op, left, .. } => {
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                    self.expr_type(left, locals)
                } else {
                    Type::Bool
                }
            }
            Expr::Call { function, .. } => self.functions[function.as_str()].1.output.clone(),
            Expr::If { then_block, .. } => self.block_type(then_block, locals),
            Expr::Fold { accumulator, .. } => accumulator.ty.clone(),
            Expr::While { state, .. } => state.ty.clone(),
        }
    }
    fn block_type(&self, block: &Block, outer: &Locals) -> Type {
        let mut locals = outer.clone();
        for binding in &block.bindings {
            locals.insert(
                binding.name.clone(),
                (String::new(), self.expr_type(&binding.value, &locals)),
            );
        }
        self.expr_type(&block.value, &locals)
    }
    fn block(&mut self, block: &Block, outer: &Locals) -> (String, Type) {
        let mut locals = outer.clone();
        for binding in &block.bindings {
            let value = self.expr(&binding.value, &locals);
            locals.insert(binding.name.clone(), value);
        }
        self.expr(&block.value, &locals)
    }
    fn signature(&self, index: usize, function: &Function) -> String {
        let params = function
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{} a{i}", self.ctype(&p.ty)))
            .collect::<Vec<_>>();
        format!(
            "static {} lobo_fn_{index}({})",
            self.ctype(&function.output),
            if params.is_empty() {
                "void".into()
            } else {
                params.join(", ")
            }
        )
    }
    fn function(&mut self, index: usize, function: &Function) {
        self.next = 0;
        self.output = function.output.clone();
        self.line(format!("{} {{", self.signature(index, function)));
        self.line(format!("{} lobo_return_value;", self.ctype(&function.output)));
        self.line("const char *saved_subject = lobo_subject; const char *saved_phase = lobo_phase;");
        self.line("const char *saved_contract_phase = lobo_contract_phase;");
        self.line(format!(
            "lobo_subject = \"{}\"; lobo_phase = \"execute\";",
            function.id
        ));
        self.line(
            "lobo_tick(); if (lobo_depth >= 32) lobo_fail(\"resource.stack.exhausted\"); ++lobo_depth;",
        );
        let locals = function
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), (format!("a{i}"), p.ty.clone())))
            .collect::<Locals>();
        for requirement in &function.requires {
            self.line("lobo_phase = \"precondition\";");
            self.line("if (!lobo_contract_phase) lobo_contract_phase = \"precondition\";");
            let (value, _) = self.expr(requirement, &locals);
            self.line(format!(
                "if (!{value}) lobo_fail(\"runtime.contract.precondition\");"
            ));
        }
        self.line("lobo_contract_phase = saved_contract_phase;");
        self.line("lobo_phase = \"execute\";");
        let (value, _) = self.block(&function.body, &locals);
        self.line(format!("lobo_return_value = {value};"));
        self.line("lobo_return_epilogue:;");
        let value = "lobo_return_value".to_string();
        let mut post_locals = locals;
        post_locals.insert("result".into(), (value.clone(), function.output.clone()));
        for assurance in &function.ensures {
            self.line("lobo_phase = \"postcondition\";");
            self.line("if (!lobo_contract_phase) lobo_contract_phase = \"postcondition\";");
            let (checked, _) = self.expr(assurance, &post_locals);
            self.line(format!(
                "if (!{checked}) lobo_fail(\"runtime.contract.postcondition\");"
            ));
        }
        self.line(format!(
            "--lobo_depth; lobo_subject = saved_subject; lobo_phase = saved_phase; lobo_contract_phase = saved_contract_phase; return {value};"
        ));
        self.line("}");
    }
}

impl Artifact {
    pub fn run_native(&self, arguments: &[String]) -> Fallible<std::process::Output> {
        self.run_native_with_backend(arguments, NativeBackend::C)
    }
    pub fn run_native_with_backend(
        &self,
        arguments: &[String],
        backend: NativeBackend,
    ) -> Fallible<std::process::Output> {
        let temp = NativeTemp::new()?;
        let binary = temp.path.join("run");
        self.compile_native_with_backend(&binary, backend)?;
        execute_native(&binary, arguments)
            .map_err(|e| native_error("compile.native.execution_failed", e.to_string()))
    }

    /// Lower checked Core into C that Clang compiles to native instructions.
    pub fn emit_c(&self) -> Fallible<String> {
        self.native_sources(NativeBackend::C, false)
            .map(|(runtime, _)| runtime)
    }
    fn native_sources(&self, backend: NativeBackend, sanitized: bool) -> Fallible<(String, String)> {
        self.check()?;
        let native_enabled = self.program.native_abi == Some(super::native_api::ABI);
        if native_enabled && !cfg!(all(target_os = "linux", target_pointer_width = "64")) {
            return Err(native_error(
                "compile.native.platform",
                "native ABI v1 supports 64-bit Linux",
            ));
        }
        let table = super::records::RecordTable::new(&self.program.records, &self.program.variants, true)?;
        let functions = self
            .program
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id.as_str(), (i, f)))
            .collect();
        let sqlite_enabled = self.program.sqlite_abi == Some(1);
        let prelude = if sqlite_enabled {
            format!(
                "{}\n{}\n#if defined(__linux__)\n#include <sys/stat.h>\n#endif",
                include_str!("sqlite_profile.h"),
                include_str!("../vendor/sqlite/sqlite3.c")
            )
        } else {
            String::new()
        };
        let provider = if native_enabled {
            let network = format!(
                "{}\n{}\n{}",
                include_str!("native_api.c"),
                include_str!("native_windows_snapshot.c"),
                include_str!("native_windows_tcp.c")
            );
            if sqlite_enabled {
                let sqlite_api = include_str!("sqlite_api.h");
                format!("{sqlite_api}\n#define lobo_native_cleanup lobo_native_network_cleanup\n#define lobo_native_option lobo_native_network_option\n{network}\n#undef lobo_native_cleanup\n#undef lobo_native_option\n{}",include_str!("native_sqlite.c"))
            } else {
                network
            }
        } else {
            "static void lobo_native_cleanup(void) {}\nstatic bool lobo_native_option(const char *name,const char *value) { (void)name;(void)value;return false; }".to_string()
        };
        let mut emitter = Emitter {
            expression_fuel: true,
            text: format!(
                "{prelude}\n{}\n{}\n{}\n{}\n{}\n{provider}",
                if native_enabled {
                    "#define LOBO_NATIVE_ENABLED 1"
                } else {
                    ""
                },
                include_str!("native_runtime.c"),
                include_str!("native_api.h"),
                include_str!("native_buffers.c"),
                include_str!("native_floats.c")
            ),
            functions,
            next: 0,
            output: Type::I64,
            lists: BTreeMap::new(),
            layouts: std::collections::BTreeSet::new(),
            widths: table.widths,
            records: self
                .program
                .records
                .iter()
                .enumerate()
                .map(|(i, r)| (r.id.as_str(), (i, r)))
                .collect(),
            variants: self
                .program
                .variants
                .iter()
                .enumerate()
                .map(|(i, v)| (v.id.as_str(), (i, v)))
                .collect(),
        };
        for record in &self.program.records {
            emitter.ensure_type(&Type::Record(record.id.clone()));
        }
        for variant in &self.program.variants {
            emitter.ensure_type(&Type::Variant(variant.id.clone()));
        }
        for function in &self.program.functions {
            emitter.ensure_type(&function.output);
            for parameter in &function.parameters {
                emitter.ensure_type(&parameter.ty);
            }
            for expr in function.requires.iter().chain(&function.ensures) {
                emitter.ensure_expr_types(expr);
            }
            emitter.ensure_block_types(&function.body);
        }
        emitter.emit_types(&table.order);
        let application = if backend == NativeBackend::Llvm {
            emitter.line(include_str!("native_llvm_runtime.c"));
            llvm::lower(&mut emitter, &self.program.functions, sanitized)
        } else {
            String::new()
        };
        for (i, function) in self.program.functions.iter().enumerate() {
            emitter.line(format!("{};", emitter.signature(i, function)));
        }
        for (i, function) in self.program.functions.iter().enumerate() {
            if backend == NativeBackend::Llvm {
                llvm::bridge(&mut emitter, i, function);
            } else {
                emitter.function(i, function);
            }
        }
        emitter.line("int main(int argc, char **argv) {");
        if native_enabled {
            emitter.line("if (lobo_native_abi_version_v1() != 1) lobo_fail(\"invocation.native.abi\");");
        }
        emitter.line("lobo_float_environment(); if (atexit(lobo_cleanup) != 0) lobo_fail(\"resource.memory.exhausted\"); struct lobo_args args = lobo_parse(argc, argv);");
        for (i, function) in self.program.functions.iter().enumerate() {
            emitter.line(format!("if (strcmp(args.entry, \"{}\") == 0) {{", function.id));
            emitter.line(format!(
                "if (args.count != {}) lobo_fail(\"invocation.arguments.invalid\");",
                function.parameters.len()
            ));
            for (n, parameter) in function.parameters.iter().enumerate() {
                emitter.line(format!("{} input{n};", emitter.ctype(&parameter.ty)));
            }
            emitter.line("for (unsigned pass = 0; pass < 2; ++pass) { lobo_validating = pass == 0;");
            for (n, parameter) in function.parameters.iter().enumerate() {
                if !matches!(parameter.ty, Type::I64 | Type::Bool) {
                    let prefix = match &parameter.ty {
                        Type::Record(id) => format!("Record:{id}:"),
                        Type::Variant(id) => format!("Variant:{id}:"),
                        Type::Bytes | Type::Text | Type::Function(_) | Type::F64 => "".into(),
                        _ => "List:".into(),
                    };
                    emitter.line(format!("const char *cursor{n} = args.values[{n}].record;"));
                    emitter.line(format!("if (!cursor{n} || strncmp(cursor{n}, \"{prefix}\", {}) != 0) lobo_fail(\"invocation.arguments.invalid\"); cursor{n} += {};", prefix.len(), prefix.len()));
                    emitter.parse_value(&parameter.ty, &format!("input{n}"), &format!("&cursor{n}"));
                    emitter.line(format!(
                        "if (cursor{n}) lobo_fail(\"invocation.arguments.invalid\");"
                    ));
                    continue;
                }
                emitter.line("lobo_input_tick();");
                emitter.line(format!(
                    "if (args.values[{n}].record) lobo_fail(\"invocation.arguments.invalid\");"
                ));
                emitter.line(format!(
                    "if (args.values[{n}].is_bool != {}) lobo_fail(\"invocation.arguments.invalid\");",
                    if parameter.ty == Type::Bool {
                        "true"
                    } else {
                        "false"
                    }
                ));
                emitter.line(format!(
                    "input{n} = args.values[{n}].{};",
                    if parameter.ty == Type::Bool {
                        "boolean"
                    } else {
                        "integer"
                    }
                ));
            }
            emitter.line("}");
            let arguments = function
                .parameters
                .iter()
                .enumerate()
                .map(|(n, _)| format!("input{n}"))
                .collect::<Vec<_>>()
                .join(", ");
            emitter.line(format!(
                "{} output = lobo_fn_{i}({arguments});",
                emitter.ctype(&function.output)
            ));
            emitter.count_value(&function.output, "output");
            emitter.line(r#"fputs("{\"status\":\"ok\",\"kind\":\"value\",\"value\":", stdout);"#);
            emitter.print_value(&function.output, "output");
            emitter.line("fputs(\"}\\n\", stdout);");
            emitter.line("return 0; }");
        }
        emitter.line("lobo_fail(\"invocation.entry.unknown\"); }");
        Ok((emitter.text, application))
    }

    /// Compile using Clang without invoking a shell; publish only after a successful native build.
    pub fn compile_native(&self, output: &Path) -> Fallible<()> {
        self.compile_native_with_backend(output, NativeBackend::C)
    }
    pub fn compile_native_with_backend(&self, output: &Path, backend: NativeBackend) -> Fallible<()> {
        if output.exists() {
            return Err(native_error(
                "compile.native.output_exists",
                "output already exists",
            ));
        }
        let temp = NativeTemp::new()?;
        let source_path = temp.path.join("program.c");
        let binary_path = temp.path.join("program");
        if backend == NativeBackend::Llvm {
            llvm::build(self, &temp, &binary_path, false)?;
        } else {
            fs::write(&source_path, self.emit_c()?)
                .map_err(|e| native_error("compile.native.io", e.to_string()))?;
            let result = Command::new("clang")
                .args([
                    "-std=c11",
                    "-O3",
                    "-g0",
                    "-ffunction-sections",
                    "-fdata-sections",
                    "-Wl,--gc-sections",
                    "-Wl,-s",
                ])
                .arg(&source_path)
                .arg("-o")
                .arg(&binary_path)
                .output()
                .map_err(|e| {
                    native_error(
                        "compile.native.tool_unavailable",
                        format!("Clang is required for native compilation: {e}"),
                    )
                })?;
            if !result.status.success() {
                let details = String::from_utf8_lossy(&result.stderr);
                return Err(native_error(
                    "compile.native.backend_failed",
                    details.chars().take(4096).collect::<String>(),
                ));
            }
        }
        publish_native(&binary_path, output)
    }
}

fn publish_native(binary_path: &Path, output: &Path) -> Fallible<()> {
    let mut input = File::open(binary_path).map_err(|e| native_error("compile.native.io", e.to_string()))?;
    let mut destination = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| native_error("compile.native.io", e.to_string()))?;
    let publish = (|| -> io::Result<()> {
        io::copy(&mut input, &mut destination)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            destination.set_permissions(fs::Permissions::from_mode(0o755))?;
        }
        destination.sync_all()?;
        Ok(())
    })();
    if let Err(error) = publish {
        let _ = fs::remove_file(output);
        return Err(native_error("compile.native.io", error.to_string()));
    }
    Ok(())
}

fn execute_native(binary: &Path, arguments: &[String]) -> io::Result<std::process::Output> {
    let mut retries = 0;
    loop {
        match Command::new(binary).args(arguments).output() {
            // A concurrent process spawn can briefly inherit a publishing writer before exec closes it.
            // Retry only an unsuccessful launch, never a program result or a different host error.
            Err(error) if error.kind() == io::ErrorKind::ExecutableFileBusy && retries < 20 => {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            result => return result,
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod launch_tests {
    use super::*;

    #[test]
    fn native_launch_tolerates_a_brief_executable_writer() {
        let artifact = crate::compile("module m @id(\"m\"); function f @id(\"f\")() -> I64 { 42 }").unwrap();
        let temp = NativeTemp::new().unwrap();
        let binary = temp.path.join("run");
        artifact.compile_native(&binary).unwrap();
        let writer = OpenOptions::new().write(true).open(&binary).unwrap();
        std::thread::scope(|scope| {
            scope.spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(20));
                drop(writer);
            });
            let output = execute_native(&binary, &["--entry".into(), "f".into()]).unwrap();
            assert!(output.status.success(), "{output:?}");
            let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(result["value"]["value"], "42");
        });
        let _writer = OpenOptions::new().write(true).open(&binary).unwrap();
        assert_eq!(
            execute_native(&binary, &["--entry".into(), "f".into()])
                .unwrap_err()
                .kind(),
            io::ErrorKind::ExecutableFileBusy
        );
        assert_eq!(
            execute_native(&temp.path.join("missing"), &[])
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
    }
}
