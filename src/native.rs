use super::{Artifact, Block, Diagnostic, Expr, Fallible, Function, Type, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

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
        for name in ["program.c", "program", "run"] {
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

fn ctype(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "int64_t",
        Type::Bool => "bool",
    }
}

struct Emitter<'a> {
    text: String,
    functions: BTreeMap<&'a str, (usize, &'a Function)>,
    next: usize,
}

type Locals = BTreeMap<String, (String, Type)>;

impl Emitter<'_> {
    fn temp(&mut self) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        name
    }
    fn line(&mut self, text: impl AsRef<str>) {
        writeln!(self.text, "{}", text.as_ref()).unwrap();
    }
    fn expr(&mut self, expr: &Expr, locals: &Locals) -> (String, Type) {
        self.line("lobo_tick();");
        let name = self.temp();
        let ty = match expr {
            Expr::Literal { value } => {
                let (ty, literal) = match value {
                    Value::I64(i64::MIN) => (Type::I64, "INT64_MIN".into()),
                    Value::I64(n) if *n < 0 => (Type::I64, format!("(-INT64_C({}))", -n)),
                    Value::I64(n) => (Type::I64, format!("INT64_C({n})")),
                    Value::Bool(b) => (Type::Bool, b.to_string()),
                };
                self.line(format!("{} {name} = {literal};", ctype(ty)));
                ty
            }
            Expr::Name { name: binding } => {
                let (value, ty) = &locals[binding];
                self.line(format!("{} {name} = {value};", ctype(*ty)));
                *ty
            }
            Expr::Unary { op, operand } => {
                let (value, ty) = self.expr(operand, locals);
                if op == "!" {
                    self.line(format!("bool {name} = !{value};"));
                } else {
                    self.line(format!("int64_t {name};"));
                    self.line(format!("if (__builtin_sub_overflow(INT64_C(0), {value}, &{name})) lobo_fail(\"runtime.integer.overflow\");"));
                }
                ty
            }
            Expr::Binary { op, left, right } => {
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
                    _ => {
                        self.line(format!("bool {name} = {left} {op} {right};"));
                        Type::Bool
                    }
                }
            }
            Expr::Call { function, arguments } => {
                let (index, target) = self.functions[function.as_str()];
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.expr(argument, locals).0);
                }
                self.line(format!(
                    "{} {name} = lobo_fn_{index}({});",
                    ctype(target.output),
                    values.join(", ")
                ));
                target.output
            }
            Expr::If {
                condition,
                then_block,
                else_block,
            } => {
                let (condition, _) = self.expr(condition, locals);
                // The checker has already established that both branches have equal types.
                let ty = self.block_type(then_block, locals);
                self.line(format!("{} {name};", ctype(ty)));
                self.line(format!("if ({condition}) {{"));
                let (then_value, _) = self.block(then_block, locals);
                self.line(format!("{name} = {then_value};"));
                self.line("} else {");
                let (else_value, _) = self.block(else_block, locals);
                self.line(format!("{name} = {else_value};"));
                self.line("}");
                ty
            }
        };
        (name, ty)
    }
    fn expr_type(&self, expr: &Expr, locals: &Locals) -> Type {
        match expr {
            Expr::Literal { value } => value.ty(),
            Expr::Name { name } => locals[name].1,
            Expr::Unary { operand, .. } => self.expr_type(operand, locals),
            Expr::Binary { op, .. } => {
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                    Type::I64
                } else {
                    Type::Bool
                }
            }
            Expr::Call { function, .. } => self.functions[function.as_str()].1.output,
            Expr::If { then_block, .. } => self.block_type(then_block, locals),
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
    fn signature(index: usize, function: &Function) -> String {
        let params = function
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{} a{i}", ctype(p.ty)))
            .collect::<Vec<_>>();
        format!(
            "static {} lobo_fn_{index}({})",
            ctype(function.output),
            if params.is_empty() {
                "void".into()
            } else {
                params.join(", ")
            }
        )
    }
    fn function(&mut self, index: usize, function: &Function) {
        self.next = 0;
        self.line(format!("{} {{", Self::signature(index, function)));
        self.line("const char *saved_subject = lobo_subject; const char *saved_phase = lobo_phase;");
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
            .map(|(i, p)| (p.name.clone(), (format!("a{i}"), p.ty)))
            .collect::<Locals>();
        for requirement in &function.requires {
            self.line("lobo_phase = \"precondition\";");
            let (value, _) = self.expr(requirement, &locals);
            self.line(format!(
                "if (!{value}) lobo_fail(\"runtime.contract.precondition\");"
            ));
        }
        self.line("lobo_phase = \"execute\";");
        let (value, _) = self.block(&function.body, &locals);
        let mut post_locals = locals;
        post_locals.insert("result".into(), (value.clone(), function.output));
        for assurance in &function.ensures {
            self.line("lobo_phase = \"postcondition\";");
            let (checked, _) = self.expr(assurance, &post_locals);
            self.line(format!(
                "if (!{checked}) lobo_fail(\"runtime.contract.postcondition\");"
            ));
        }
        self.line(format!(
            "--lobo_depth; lobo_subject = saved_subject; lobo_phase = saved_phase; return {value};"
        ));
        self.line("}");
    }
}

impl Artifact {
    pub fn run_native(&self, arguments: &[String]) -> Fallible<std::process::Output> {
        let temp = NativeTemp::new()?;
        let binary = temp.path.join("run");
        self.compile_native(&binary)?;
        Command::new(binary)
            .args(arguments)
            .output()
            .map_err(|e| native_error("compile.native.execution_failed", e.to_string()))
    }

    /// Lower checked Core into C that Clang compiles to native instructions.
    pub fn emit_c(&self) -> Fallible<String> {
        self.check()?;
        let functions = self
            .program
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id.as_str(), (i, f)))
            .collect();
        let mut emitter = Emitter {
            text: include_str!("native_runtime.c").into(),
            functions,
            next: 0,
        };
        for (i, function) in self.program.functions.iter().enumerate() {
            emitter.line(format!("{};", Emitter::signature(i, function)));
        }
        for (i, function) in self.program.functions.iter().enumerate() {
            emitter.function(i, function);
        }
        emitter.line("int main(int argc, char **argv) { struct lobo_args args = lobo_parse(argc, argv);");
        for (i, function) in self.program.functions.iter().enumerate() {
            emitter.line(format!("if (strcmp(args.entry, \"{}\") == 0) {{", function.id));
            emitter.line(format!(
                "if (args.count != {}) lobo_fail(\"invocation.arguments.invalid\");",
                function.parameters.len()
            ));
            for (n, parameter) in function.parameters.iter().enumerate() {
                emitter.line(format!(
                    "if (args.values[{n}].is_bool != {}) lobo_fail(\"invocation.arguments.invalid\");",
                    if parameter.ty == Type::Bool {
                        "true"
                    } else {
                        "false"
                    }
                ));
            }
            let arguments = function
                .parameters
                .iter()
                .enumerate()
                .map(|(n, p)| {
                    format!(
                        "args.values[{n}].{}",
                        if p.ty == Type::Bool { "boolean" } else { "integer" }
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            emitter.line(format!(
                "{} output = lobo_fn_{i}({arguments});",
                ctype(function.output)
            ));
            if function.output == Type::I64 {
                emitter.line(r#"printf("{\"status\":\"ok\",\"kind\":\"value\",\"value\":{\"type\":\"I64\",\"value\":\"%" PRId64 "\"}}\n", output);"#);
            } else {
                emitter.line(r#"printf("{\"status\":\"ok\",\"kind\":\"value\",\"value\":{\"type\":\"Bool\",\"value\":%s}}\n", output ? "true" : "false");"#);
            }
            emitter.line("free(args.values); return 0; }");
        }
        emitter.line("lobo_fail(\"invocation.entry.unknown\"); }");
        Ok(emitter.text)
    }

    /// Compile using Clang without invoking a shell; publish only after a successful native build.
    pub fn compile_native(&self, output: &Path) -> Fallible<()> {
        if output.exists() {
            return Err(native_error(
                "compile.native.output_exists",
                "output already exists",
            ));
        }
        let source = self.emit_c()?;
        let temp = NativeTemp::new()?;
        let source_path = temp.path.join("program.c");
        let binary_path = temp.path.join("program");
        fs::write(&source_path, source).map_err(|e| native_error("compile.native.io", e.to_string()))?;
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
        let mut input =
            File::open(&binary_path).map_err(|e| native_error("compile.native.io", e.to_string()))?;
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
}
