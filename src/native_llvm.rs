use super::*;

struct Lowering<'b, 'a> {
    c: &'b mut Emitter<'a>,
    globals: String,
    declarations: BTreeMap<String, String>,
    functions: String,
    code: String,
    slots: String,
    next: usize,
    helpers: BTreeMap<String, String>,
    sanitized: bool,
    output: Type,
    return_label: String,
}

impl Lowering<'_, '_> {
    fn fresh(&mut self) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        name
    }
    fn line(&mut self, text: impl AsRef<str>) {
        writeln!(self.code, "  {}", text.as_ref()).unwrap();
    }
    fn reg(&mut self, instruction: impl AsRef<str>) -> String {
        let name = format!("%{}", self.fresh());
        self.line(format!("{name} = {}", instruction.as_ref()));
        name
    }
    fn slot(&mut self, ty: &Type) -> String {
        let name = format!("%{}", self.fresh());
        writeln!(
            self.slots,
            "  {name} = alloca [{} x i64], align 8",
            self.c.inline_width(ty).max(1)
        )
        .unwrap();
        name
    }
    fn string(&mut self, text: &str) -> String {
        let name = format!("@lobo_ir_s_{}", self.fresh());
        let bytes = text
            .bytes()
            .chain(std::iter::once(0))
            .map(|b| format!("\\{b:02X}"))
            .collect::<String>();
        writeln!(
            self.globals,
            "{name} = private unnamed_addr constant [{} x i8] c\"{bytes}\", align 1",
            text.len() + 1
        )
        .unwrap();
        name
    }
    fn label(&mut self, label: &str) {
        writeln!(self.code, "{label}:").unwrap();
    }
    fn guard(&mut self, bad: &str, code: &str) {
        let fail = self.fresh();
        let pass = self.fresh();
        self.line(format!("br i1 {bad}, label %{fail}, label %{pass}"));
        self.label(&fail);
        let code = self.string(code);
        self.line(format!("call void @lobo_ir_fail(ptr {code})"));
        self.line("unreachable");
        self.label(&pass);
    }
    fn scalar(&mut self, value: &str, ty: &str) -> String {
        self.reg(format!(
            "load {ty}, ptr {value}, align {}",
            if ty == "i8" { 1 } else { 8 }
        ))
    }
    fn condition(&mut self, value: &str) -> String {
        let v = self.scalar(value, "i8");
        self.reg(format!("icmp ne i8 {v}, 0"))
    }
    fn store_bool(&mut self, out: &str, value: &str) {
        let byte = self.reg(format!("zext i1 {value} to i8"));
        self.line(format!("store i8 {byte}, ptr {out}, align 1"));
    }
    fn helper(
        &mut self,
        key: String,
        types: &[Type],
        output: &Type,
        expr: Option<&Expr>,
        custom: Option<&str>,
    ) -> String {
        if let Some(name) = self.helpers.get(&key) {
            return name.clone();
        }
        let name = format!("lobo_ir_op_{}", self.helpers.len());
        let signature = std::iter::once("ptr".to_string())
            .chain(types.iter().map(|_| "ptr".into()))
            .collect::<Vec<_>>()
            .join(", ");
        self.declarations
            .insert(name.clone(), format!("declare void @{name}({signature})"));
        let parameters = std::iter::once("void *out".to_string())
            .chain(types.iter().enumerate().map(|(i, _)| format!("const void *a{i}")))
            .collect::<Vec<_>>()
            .join(", ");
        self.c.line(format!("void {name}({parameters}) {{"));
        let mut locals = Locals::new();
        for (i, ty) in types.iter().enumerate() {
            self.c.line(format!(
                "typedef {} T{i}; T{i} x{i} = *(const T{i} *)a{i};",
                self.c.ctype(ty)
            ));
            locals.insert(format!("a{i}"), (format!("x{i}"), ty.clone()));
        }
        let value = if let Some(expr) = expr {
            self.c.expression_fuel = false;
            let (value, ty) = self.c.expr(expr, &locals);
            self.c.expression_fuel = true;
            assert_eq!(&ty, output, "checked primitive output");
            value
        } else {
            custom.expect("layout primitive expression").to_string()
        };
        self.c
            .line(format!("*({} *)out = {value}; }}", self.c.ctype(output)));
        self.helpers.insert(key, name.clone());
        name
    }
    fn primitive_to(&mut self, out: &str, expr: &Expr, args: &[(String, Type)], output: &Type) {
        let types = args.iter().map(|a| a.1.clone()).collect::<Vec<_>>();
        let key = format!(
            "expr:{}",
            serde_json::to_string(&(expr, &types)).expect("primitive serializes")
        );
        let name = self.helper(key, &types, output, Some(expr), None);
        let arguments = std::iter::once(format!("ptr {out}"))
            .chain(args.iter().map(|a| format!("ptr {}", a.0)))
            .collect::<Vec<_>>()
            .join(", ");
        self.line(format!("call void @{name}({arguments})"));
    }
    fn primitive(&mut self, expr: &Expr, args: &[(String, Type)], output: Type) -> (String, Type) {
        let out = self.slot(&output);
        self.primitive_to(&out, expr, args, &output);
        (out, output)
    }
    fn layout(&mut self, value: &(String, Type), output: Type, field: &str) -> (String, Type) {
        let out = self.slot(&output);
        let key = format!("layout:{:?}:{field}", value.1);
        let name = self.helper(
            key,
            std::slice::from_ref(&value.1),
            &output,
            None,
            Some(&format!("x0.{field}")),
        );
        self.line(format!("call void @{name}(ptr {out}, ptr {})", value.0));
        (out, output)
    }
    fn copy(&mut self, out: &str, value: &str, ty: &Type) {
        let scalar = match ty {
            Type::F64 => Some("double"),
            Type::I64 | Type::Function(_) => Some("i64"),
            Type::Bool => Some("i8"),
            Type::List(_) => Some("ptr"),
            _ => None,
        };
        if let Some(t) = scalar {
            let v = self.scalar(value, t);
            self.line(format!(
                "store {t} {v}, ptr {out}, align {}",
                if t == "i8" { 1 } else { 8 }
            ));
        } else {
            self.primitive_to(out, &name(0), &[(value.into(), ty.clone())], ty);
        }
    }
    fn block(&mut self, block: &Block, outer: &Locals) -> (String, Type) {
        let mut locals = outer.clone();
        for binding in &block.bindings {
            let value = self.expr(&binding.value, &locals);
            locals.insert(binding.name.clone(), value);
        }
        self.expr(&block.value, &locals)
    }
    fn checked_math(&mut self, op: &str, a: &str, b: &str) -> String {
        if matches!(op, "+" | "-" | "*") {
            let intrinsic = match op {
                "+" => "sadd",
                "-" => "ssub",
                _ => "smul",
            };
            let both = self.reg(format!(
                "call {{ i64, i1 }} @llvm.{intrinsic}.with.overflow.i64(i64 {a}, i64 {b})"
            ));
            let overflow = self.reg(format!("extractvalue {{ i64, i1 }} {both}, 1"));
            self.guard(&overflow, "runtime.integer.overflow");
            self.reg(format!("extractvalue {{ i64, i1 }} {both}, 0"))
        } else {
            let zero = self.reg(format!("icmp eq i64 {b}, 0"));
            self.guard(&zero, "runtime.integer.zero_divisor");
            let min = self.reg(format!("icmp eq i64 {a}, -9223372036854775808"));
            let negative_one = self.reg(format!("icmp eq i64 {b}, -1"));
            let overflow = self.reg(format!("and i1 {min}, {negative_one}"));
            self.guard(&overflow, "runtime.integer.overflow");
            self.reg(format!(
                "{} i64 {a}, {b}",
                if op == "/" { "sdiv" } else { "srem" }
            ))
        }
    }
    fn float_normalize(&mut self, value: &str) -> String {
        let nan = self.reg(format!("fcmp uno double {value}, {value}"));
        self.reg(format!(
            "select i1 {nan}, double 0x7FF8000000000000, double {value}"
        ))
    }
    fn expr(&mut self, expr: &Expr, locals: &Locals) -> (String, Type) {
        self.line("call void @lobo_ir_tick()");
        match expr {
            Expr::Literal {
                value: Value::F64(bits),
                ..
            } => {
                let out = self.slot(&Type::F64);
                self.line(format!("store i64 {bits}, ptr {out}, align 8"));
                (out, Type::F64)
            }
            Expr::Reference { function, .. } => {
                let (index, target) = self.c.functions[function.as_str()];
                let ty = Type::Function(target.signature_type());
                let out = self.slot(&ty);
                self.line(format!("store i64 {index}, ptr {out}, align 8"));
                (out, ty)
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                let (callee, Type::Function(signature)) = self.expr(callee, locals) else {
                    unreachable!("checked invocation")
                };
                let index = self.scalar(&callee, "i64");
                let mut args = Vec::new();
                for argument in arguments {
                    args.push(self.expr(argument, locals).0);
                }
                let out = self.slot(&signature.output);
                let args = std::iter::once(format!("ptr {out}"))
                    .chain(args.iter().map(|p| format!("ptr {p}")))
                    .collect::<Vec<_>>()
                    .join(", ");
                let done = self.fresh();
                let invalid = self.fresh();
                let targets: Vec<_> = self
                    .c
                    .functions
                    .values()
                    .copied()
                    .filter(|(_, f)| f.actions.is_empty() && f.signature_type() == signature)
                    .collect();
                let cases: Vec<_> = targets.iter().map(|(index, _)| (*index, self.fresh())).collect();
                self.line(format!(
                    "switch i64 {index}, label %{invalid} [ {} ]",
                    cases
                        .iter()
                        .map(|(i, label)| format!("i64 {i}, label %{label}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
                for (i, label) in cases {
                    self.label(&label);
                    self.line(format!("call void @lobo_llvm_fn_{i}({args})"));
                    self.line(format!("br label %{done}"));
                }
                self.label(&invalid);
                let code = self.string("invocation.arguments.invalid");
                self.line(format!("call void @lobo_ir_fail(ptr {code})"));
                self.line("unreachable");
                self.label(&done);
                (out, *signature.output)
            }
            Expr::Name { name, .. } => locals[name].clone(),
            Expr::Literal {
                value: Value::I64(value),
                ..
            } => {
                let out = self.slot(&Type::I64);
                self.line(format!("store i64 {value}, ptr {out}, align 8"));
                (out, Type::I64)
            }
            Expr::Literal {
                value: Value::Bool(value),
                ..
            } => {
                let out = self.slot(&Type::Bool);
                self.line(format!("store i8 {}, ptr {out}, align 1", u8::from(*value)));
                (out, Type::Bool)
            }
            Expr::Literal { .. } => self.primitive(expr, &[], Type::Text),
            Expr::List { element, .. } => {
                let ty = Type::List(Box::new(element.clone()));
                let out = self.slot(&ty);
                self.line(format!("store ptr null, ptr {out}, align 8"));
                (out, ty)
            }
            Expr::Unary { op, operand, .. } => {
                let (value, ty) = self.expr(operand, locals);
                if op == "try" {
                    let Type::Variant(id) = &ty else {
                        unreachable!("checked try")
                    };
                    let source = self.c.variants[id.as_str()].1;
                    let ok = source.cases.iter().position(|c| c.name == "Ok").unwrap();
                    let error = source.cases.iter().position(|c| c.name == "Err").unwrap();
                    let tag = self.layout(&(value.clone(), ty.clone()), Type::I64, "tag");
                    let tag = self.scalar(&tag.0, "i64");
                    let failed = self.reg(format!("icmp eq i64 {tag}, {error}"));
                    let fail = self.fresh();
                    let success = self.fresh();
                    self.line(format!("br i1 {failed}, label %{fail}, label %{success}"));
                    self.label(&fail);
                    let error_type = source.cases[error].payload.as_ref().unwrap().clone();
                    let payload = self.layout(
                        &(value.clone(), ty.clone()),
                        error_type,
                        &format!("payload.c{error}"),
                    );
                    let Type::Variant(target) = &self.output else {
                        unreachable!("checked try output")
                    };
                    let output = self.output.clone();
                    let construct = Expr::Variant {
                        variant: target.clone(),
                        type_arguments: vec![],
                        case: "Err".into(),
                        payload: Some(Box::new(name(0))),
                        span: None,
                    };
                    self.primitive_to("%out", &construct, &[payload], &output);
                    self.line(format!("br label %{}", self.return_label));
                    self.label(&success);
                    let ok_type = source.cases[ok].payload.as_ref().unwrap().clone();
                    return self.layout(&(value, ty), ok_type, &format!("payload.c{ok}"));
                }
                let out = self.slot(&ty);
                if op == "!" {
                    let v = self.condition(&value);
                    let v = self.reg(format!("xor i1 {v}, true"));
                    self.store_bool(&out, &v);
                } else if ty == Type::F64 {
                    let value = self.scalar(&value, "double");
                    let value = self.reg(format!("fneg double {value}"));
                    let value = self.float_normalize(&value);
                    self.line(format!("store double {value}, ptr {out}, align 8"));
                } else {
                    let v = self.scalar(&value, "i64");
                    let v = self.checked_math("-", "0", &v);
                    self.line(format!("store i64 {v}, ptr {out}, align 8"));
                }
                (out, ty)
            }
            Expr::Binary { op, left, right, .. } => {
                let a = self.expr(left, locals);
                if op == "&&" || op == "||" {
                    let out = self.slot(&Type::Bool);
                    self.copy(&out, &a.0, &Type::Bool);
                    let condition = self.condition(&a.0);
                    let rhs = self.fresh();
                    let done = self.fresh();
                    let (yes, no) = if op == "&&" { (&rhs, &done) } else { (&done, &rhs) };
                    self.line(format!("br i1 {condition}, label %{yes}, label %{no}"));
                    self.label(&rhs);
                    let b = self.expr(right, locals);
                    self.copy(&out, &b.0, &Type::Bool);
                    self.line(format!("br label %{done}"));
                    self.label(&done);
                    return (out, Type::Bool);
                }
                let b = self.expr(right, locals);
                if a.1 == Type::F64 {
                    let av = self.scalar(&a.0, "double");
                    let bv = self.scalar(&b.0, "double");
                    if matches!(op.as_str(), "+" | "-" | "*" | "/") {
                        let opcode = match op.as_str() {
                            "+" => "fadd",
                            "-" => "fsub",
                            "*" => "fmul",
                            _ => "fdiv",
                        };
                        let value = self.reg(format!("{opcode} double {av}, {bv}"));
                        let value = self.float_normalize(&value);
                        let out = self.slot(&Type::F64);
                        self.line(format!("store double {value}, ptr {out}, align 8"));
                        return (out, Type::F64);
                    }
                    let predicate = match op.as_str() {
                        "==" => "oeq",
                        "!=" => "une",
                        "<" => "olt",
                        "<=" => "ole",
                        ">" => "ogt",
                        ">=" => "oge",
                        _ => unreachable!("checked F64 comparison"),
                    };
                    let value = self.reg(format!("fcmp {predicate} double {av}, {bv}"));
                    let out = self.slot(&Type::Bool);
                    self.store_bool(&out, &value);
                    return (out, Type::Bool);
                }
                if matches!(op.as_str(), "==" | "!=") && !matches!(a.1, Type::I64 | Type::Bool) {
                    let primitive = Expr::Binary {
                        op: op.clone(),
                        left: Box::new(name(0)),
                        right: Box::new(name(1)),
                        span: None,
                    };
                    return self.primitive(&primitive, &[a, b], Type::Bool);
                }
                let scalar = if a.1 == Type::Bool { "i8" } else { "i64" };
                let av = self.scalar(&a.0, scalar);
                let bv = self.scalar(&b.0, scalar);
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                    let out = self.slot(&Type::I64);
                    let v = self.checked_math(op, &av, &bv);
                    self.line(format!("store i64 {v}, ptr {out}, align 8"));
                    (out, Type::I64)
                } else {
                    let predicate = match op.as_str() {
                        "==" => "eq",
                        "!=" => "ne",
                        "<" => "slt",
                        "<=" => "sle",
                        ">" => "sgt",
                        ">=" => "sge",
                        _ => unreachable!("checked comparison"),
                    };
                    let out = self.slot(&Type::Bool);
                    let v = self.reg(format!("icmp {predicate} {scalar} {av}, {bv}"));
                    self.store_bool(&out, &v);
                    (out, Type::Bool)
                }
            }
            Expr::Call {
                function, arguments, ..
            } => {
                let (index, target) = self.c.functions[function.as_str()];
                let ty = target.output.clone();
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.expr(arg, locals).0);
                }
                let out = self.slot(&ty);
                let args = std::iter::once(format!("ptr {out}"))
                    .chain(args.iter().map(|p| format!("ptr {p}")))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.line(format!("call void @lobo_llvm_fn_{index}({args})"));
                (out, ty)
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let condition = self.expr(condition, locals);
                let condition = self.condition(&condition.0);
                let ty = self.c.block_type(then_block, locals);
                let out = self.slot(&ty);
                let yes = self.fresh();
                let no = self.fresh();
                let done = self.fresh();
                self.line(format!("br i1 {condition}, label %{yes}, label %{no}"));
                self.label(&yes);
                let value = self.block(then_block, locals);
                self.copy(&out, &value.0, &ty);
                self.line(format!("br label %{done}"));
                self.label(&no);
                let value = self.block(else_block, locals);
                self.copy(&out, &value.0, &ty);
                self.line(format!("br label %{done}"));
                self.label(&done);
                (out, ty)
            }
            Expr::Fold {
                index,
                accumulator,
                count,
                initial,
                body,
                ..
            } => {
                let count = self.expr(count, locals);
                let count = self.scalar(&count.0, "i64");
                let bad = self.reg(format!("icmp slt i64 {count}, 0"));
                self.guard(&bad, "runtime.iteration.negative_count");
                let initial = self.expr(initial, locals);
                let out = self.slot(&accumulator.ty);
                self.copy(&out, &initial.0, &accumulator.ty);
                let counter = self.slot(&Type::I64);
                self.line(format!("store i64 0, ptr {counter}, align 8"));
                let test = self.fresh();
                let step = self.fresh();
                let done = self.fresh();
                self.line(format!("br label %{test}"));
                self.label(&test);
                let n = self.scalar(&counter, "i64");
                let condition = self.reg(format!("icmp slt i64 {n}, {count}"));
                self.line(format!("br i1 {condition}, label %{step}, label %{done}"));
                self.label(&step);
                self.line("call void @lobo_ir_tick()");
                let mut inner = locals.clone();
                inner.insert(index.clone(), (counter.clone(), Type::I64));
                inner.insert(accumulator.name.clone(), (out.clone(), accumulator.ty.clone()));
                let value = self.block(body, &inner);
                self.copy(&out, &value.0, &accumulator.ty);
                let n = self.scalar(&counter, "i64");
                let n = self.reg(format!("add i64 {n}, 1"));
                self.line(format!("store i64 {n}, ptr {counter}, align 8"));
                self.line(format!("br label %{test}"));
                self.label(&done);
                (out, accumulator.ty.clone())
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                let initial = self.expr(initial, locals);
                let out = self.slot(&state.ty);
                self.copy(&out, &initial.0, &state.ty);
                let mut inner = locals.clone();
                inner.insert(state.name.clone(), (out.clone(), state.ty.clone()));
                let test = self.fresh();
                let step = self.fresh();
                let done = self.fresh();
                self.line(format!("br label %{test}"));
                self.label(&test);
                self.line("call void @lobo_ir_tick()");
                let condition = self.expr(condition, &inner);
                let condition = self.condition(&condition.0);
                self.line(format!("br i1 {condition}, label %{step}, label %{done}"));
                self.label(&step);
                let value = self.block(body, &inner);
                self.copy(&out, &value.0, &state.ty);
                self.line(format!("br label %{test}"));
                self.label(&done);
                (out, state.ty.clone())
            }
            Expr::Match { subject, arms, .. } => {
                let value = self.expr(subject, locals);
                let Type::Variant(id) = &value.1 else {
                    unreachable!("checked match")
                };
                let definition = self.c.variants[id.as_str()].1;
                let ty = self.c.expr_type(expr, locals);
                let out = self.slot(&ty);
                let tag = self.layout(&value, Type::I64, "tag");
                let tag = self.scalar(&tag.0, "i64");
                let done = self.fresh();
                let invalid = self.fresh();
                let labels = arms.iter().map(|_| self.fresh()).collect::<Vec<_>>();
                self.line(format!("switch i64 {tag}, label %{invalid} ["));
                for (arm, label) in arms.iter().zip(&labels) {
                    let tag = definition.cases.iter().position(|c| c.name == arm.case).unwrap();
                    self.line(format!("i64 {tag}, label %{label}"));
                }
                self.line("]");
                for (arm, label) in arms.iter().zip(&labels) {
                    self.label(label);
                    let (tag, case) = definition
                        .cases
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.name == arm.case)
                        .unwrap();
                    let mut inner = locals.clone();
                    if let (Some(binding), Some(ty)) = (&arm.binding, &case.payload) {
                        let payload = self.layout(&value, ty.clone(), &format!("payload.c{tag}"));
                        inner.insert(binding.clone(), payload);
                    }
                    let result = self.block(&arm.body, &inner);
                    self.copy(&out, &result.0, &ty);
                    self.line(format!("br label %{done}"));
                }
                self.label(&invalid);
                let code = self.string("invocation.arguments.invalid");
                self.line(format!("call void @lobo_ir_fail(ptr {code})"));
                self.line("unreachable");
                self.label(&done);
                (out, ty)
            }
            Expr::NativeOp { arguments, .. }
            | Expr::BufferOp { arguments, .. }
            | Expr::ListOp { arguments, .. } => {
                let ty = self.c.expr_type(expr, locals);
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.expr(arg, locals));
                }
                let mut primitive = expr.clone();
                match &mut primitive {
                    Expr::NativeOp { arguments, .. }
                    | Expr::BufferOp { arguments, .. }
                    | Expr::ListOp { arguments, .. } => *arguments = (0..args.len()).map(name).collect(),
                    _ => unreachable!(),
                };
                self.primitive(&primitive, &args, ty)
            }
            Expr::Field { base, field, .. } => {
                let ty = self.c.expr_type(expr, locals);
                let value = self.expr(base, locals);
                self.primitive(
                    &Expr::Field {
                        base: Box::new(name(0)),
                        field: field.clone(),
                        span: None,
                    },
                    &[value],
                    ty,
                )
            }
            Expr::Record { record, fields, .. } => {
                let mut args = Vec::new();
                let mut primitive = expr.clone();
                for field in fields {
                    args.push(self.expr(&field.value, locals));
                }
                if let Expr::Record { fields, .. } = &mut primitive {
                    for (i, field) in fields.iter_mut().enumerate() {
                        field.value = name(i);
                    }
                }
                self.primitive(&primitive, &args, Type::Record(record.clone()))
            }
            Expr::Variant { variant, payload, .. } => {
                let args = payload
                    .as_ref()
                    .map(|v| vec![self.expr(v, locals)])
                    .unwrap_or_default();
                let mut primitive = expr.clone();
                if let Expr::Variant { payload, .. } = &mut primitive {
                    *payload = (!args.is_empty()).then(|| Box::new(name(0)));
                }
                self.primitive(&primitive, &args, Type::Variant(variant.clone()))
            }
        }
    }
    fn function(&mut self, index: usize, function: &Function) {
        self.code.clear();
        self.slots.clear();
        self.output = function.output.clone();
        self.return_label = self.fresh();
        let context = format!("%{}", self.fresh());
        writeln!(self.slots, "  {context} = alloca [3 x ptr], align 8").unwrap();
        let id = self.string(&function.id);
        self.line(format!("call void @lobo_ir_enter(ptr {context}, ptr {id})"));
        let locals = function
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), (format!("%arg{i}"), p.ty.clone())))
            .collect::<Locals>();
        for expr in &function.requires {
            self.line(format!("call void @lobo_ir_phase(ptr {context}, i32 1)"));
            let value = self.expr(expr, &locals);
            let good = self.condition(&value.0);
            let bad = self.reg(format!("xor i1 {good}, true"));
            self.guard(&bad, "runtime.contract.precondition");
        }
        self.line(format!("call void @lobo_ir_phase(ptr {context}, i32 0)"));
        let value = self.block(&function.body, &locals);
        self.copy("%out", &value.0, &function.output);
        self.line(format!("br label %{}", self.return_label));
        self.label(&self.return_label.clone());
        let mut post = locals;
        post.insert("result".into(), ("%out".into(), function.output.clone()));
        for expr in &function.ensures {
            self.line(format!("call void @lobo_ir_phase(ptr {context}, i32 2)"));
            let value = self.expr(expr, &post);
            let good = self.condition(&value.0);
            let bad = self.reg(format!("xor i1 {good}, true"));
            self.guard(&bad, "runtime.contract.postcondition");
        }
        self.line(format!("call void @lobo_ir_leave(ptr {context})"));
        self.line("ret void");
        let args = std::iter::once("ptr %out".into())
            .chain(
                function
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("ptr %arg{i}")),
            )
            .collect::<Vec<String>>()
            .join(", ");
        let attributes = if self.sanitized { " sanitize_address" } else { "" };
        writeln!(
            self.functions,
            "define void @lobo_llvm_fn_{index}({args}){attributes} {{\nentry:\n{}{}\n}}",
            self.slots, self.code
        )
        .unwrap();
    }
}

fn name(n: usize) -> Expr {
    Expr::Name {
        name: format!("a{n}"),
        span: None,
    }
}

pub(super) fn lower(c: &mut Emitter<'_>, functions: &[Function], sanitized: bool) -> String {
    let mut lower = Lowering {
        c,
        globals: String::new(),
        declarations: BTreeMap::new(),
        functions: String::new(),
        code: String::new(),
        slots: String::new(),
        next: 0,
        helpers: BTreeMap::new(),
        sanitized,
        output: Type::I64,
        return_label: String::new(),
    };
    for (i, f) in functions.iter().enumerate() {
        lower.function(i, f);
    }
    let declarations = lower
        .declarations
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    format!("; Direct Lobo application IR; runtime and layout primitives link separately.\n{}\n{declarations}\ndeclare void @lobo_ir_enter(ptr, ptr)\ndeclare void @lobo_ir_leave(ptr)\ndeclare void @lobo_ir_phase(ptr, i32)\ndeclare void @lobo_ir_tick()\ndeclare void @lobo_ir_fail(ptr) noreturn\ndeclare {{i64, i1}} @llvm.sadd.with.overflow.i64(i64, i64)\ndeclare {{i64, i1}} @llvm.ssub.with.overflow.i64(i64, i64)\ndeclare {{i64, i1}} @llvm.smul.with.overflow.i64(i64, i64)\n{}",lower.globals,lower.functions)
}

pub(super) fn bridge(c: &mut Emitter<'_>, index: usize, function: &Function) {
    let params = std::iter::once("void *out".into())
        .chain(
            function
                .parameters
                .iter()
                .enumerate()
                .map(|(i, _)| format!("const void *a{i}")),
        )
        .collect::<Vec<String>>()
        .join(", ");
    c.line(format!("extern void lobo_llvm_fn_{index}({params});"));
    c.line(format!(
        "{} {{ {} output;",
        c.signature(index, function),
        c.ctype(&function.output)
    ));
    let args = std::iter::once("&output".into())
        .chain(
            function
                .parameters
                .iter()
                .enumerate()
                .map(|(i, _)| format!("&a{i}")),
        )
        .collect::<Vec<String>>()
        .join(", ");
    c.line(format!("lobo_llvm_fn_{index}({args}); return output; }}"));
}

fn tool(command: &mut Command) -> Fallible<()> {
    let result = command
        .output()
        .map_err(|e| native_error("compile.native.tool_unavailable", format!("{command:?}: {e}")))?;
    if !result.status.success() {
        return Err(native_error(
            "compile.native.backend_failed",
            format!(
                "{command:?}: {}",
                String::from_utf8_lossy(&result.stderr)
                    .chars()
                    .take(4096)
                    .collect::<String>()
            ),
        ));
    }
    Ok(())
}

pub(super) fn linked(artifact: &Artifact, temp: &NativeTemp, sanitized: bool) -> Fallible<PathBuf> {
    let (runtime, application) = artifact.native_sources(NativeBackend::Llvm, sanitized)?;
    let c = temp.path.join("program.c");
    let app = temp.path.join("application.ll");
    let rt = temp.path.join("runtime.ll");
    let linked = temp.path.join("linked.ll");
    fs::write(&c, runtime)
        .and_then(|_| fs::write(&app, application))
        .map_err(|e| native_error("compile.native.io", e.to_string()))?;
    let mut clang = Command::new("clang-18");
    clang.args([
        "-std=c11",
        "-O1",
        "-Xclang",
        "-disable-llvm-passes",
        "-S",
        "-emit-llvm",
    ]);
    if sanitized {
        clang.args(["-fsanitize=address,undefined", "-fno-sanitize-recover=all"]);
    }
    tool(clang.arg(&c).arg("-o").arg(&rt))?;
    tool(
        Command::new("llvm-link-18")
            .arg("-S")
            .arg(&app)
            .arg(&rt)
            .arg("-o")
            .arg(&linked),
    )?;
    Ok(linked)
}

pub(super) fn build(artifact: &Artifact, temp: &NativeTemp, output: &Path, sanitized: bool) -> Fallible<()> {
    let linked = linked(artifact, temp, sanitized)?;
    let mut clang = Command::new("clang-18");
    clang.args([
        if sanitized { "-O1" } else { "-O3" },
        "-g0",
        "-ffunction-sections",
        "-fdata-sections",
        "-Wl,--gc-sections",
        "-Wl,-s",
    ]);
    if sanitized {
        clang.args(["-fsanitize=address,undefined", "-fno-sanitize-recover=all"]);
    }
    tool(clang.arg(linked).arg("-o").arg(output))
}

impl Artifact {
    /// Emit a linked LLVM module with direct application bodies and the native runtime.
    pub fn emit_llvm(&self) -> Fallible<String> {
        self.llvm_module(false)
    }
    /// Inspect LLVM with ASan attributes and frontend UBSan checks for the C runtime.
    pub fn emit_llvm_sanitized(&self) -> Fallible<String> {
        self.llvm_module(true)
    }
    fn llvm_module(&self, sanitized: bool) -> Fallible<String> {
        let temp = NativeTemp::new()?;
        let linked = linked(self, &temp, sanitized)?;
        fs::read_to_string(linked).map_err(|e| native_error("compile.native.io", e.to_string()))
    }
    /// Build a development-only LLVM/ASan/UBSan executable, never an implicit fallback.
    pub fn compile_llvm_sanitized(&self, output: &Path) -> Fallible<()> {
        if output.exists() {
            return Err(native_error(
                "compile.native.output_exists",
                "output already exists",
            ));
        }
        let temp = NativeTemp::new()?;
        let binary = temp.path.join("program");
        build(self, &temp, &binary, true)?;
        super::publish_native(&binary, output)
    }
}
