use super::*;

impl Emitter<'_> {
    pub(super) fn inline_width(&self, ty: &Type) -> usize {
        match ty {
            Type::Bytes | Type::Text => 2,
            Type::Record(id) | Type::Variant(id) => self.widths[id].0,
            _ => 1,
        }
    }
    pub(super) fn ensure_type(&mut self, ty: &Type) {
        if !self.layouts.insert(ty.clone()) {
            return;
        }
        match ty {
            Type::Function(signature) => {
                for parameter in &signature.parameters {
                    self.ensure_type(parameter);
                }
                self.ensure_type(&signature.output);
            }
            Type::Record(id) => {
                let (_, record) = self.records[id.as_str()];
                for field in &record.fields {
                    self.ensure_type(&field.ty);
                }
            }
            Type::Variant(id) => {
                let (_, variant) = self.variants[id.as_str()];
                for case in &variant.cases {
                    if let Some(ty) = &case.payload {
                        self.ensure_type(ty);
                    }
                }
            }
            Type::List(element) => {
                let index = self.lists.len();
                self.lists.insert(ty.clone(), index);
                self.ensure_type(element);
            }
            _ => (),
        }
    }
    pub(super) fn emit_types(&mut self, order: &[String]) {
        for id in order {
            if let Some(&(index, record)) = self.records.get(id.as_str()) {
                self.record_definition(index, record);
            } else if let Some(&(index, variant)) = self.variants.get(id.as_str()) {
                self.variant_definition(index, variant);
            }
        }
        for ty in self.layouts.clone() {
            self.line(format!(
                "_Static_assert(sizeof({}) <= {} && _Alignof({}) <= 8, \"checked inline layout\");",
                self.ctype(&ty),
                self.inline_width(&ty) * 8,
                self.ctype(&ty)
            ));
            let (equal, print, count, parse) = match &ty {
                Type::Record(id) => {
                    let i = self.records[id.as_str()].0;
                    (
                        format!("lobo_equal_{i}"),
                        format!("lobo_print_record_{i}"),
                        format!("lobo_count_record_{i}"),
                        format!("lobo_parse_record_{i}"),
                    )
                }
                Type::Variant(id) => {
                    let i = self.variants[id.as_str()].0;
                    (
                        format!("lobo_variant_equal_{i}"),
                        format!("lobo_print_variant_{i}"),
                        format!("lobo_count_variant_{i}"),
                        format!("lobo_parse_variant_{i}"),
                    )
                }
                Type::List(_) => {
                    let i = self.lists[&ty];
                    (
                        format!("lobo_equal_list_{i}"),
                        format!("lobo_print_list_{i}"),
                        format!("lobo_count_list_{i}"),
                        format!("lobo_parse_list_{i}"),
                    )
                }
                _ => continue,
            };
            let c = self.ctype(&ty);
            self.line(format!("static bool {equal}({c} a, {c} b);"));
            self.line(format!("static void {print}({c} value);"));
            self.line(format!("static void {count}({c} value);"));
            self.line(format!("static {c} {parse}(const char **cursor);"));
        }
        for id in order {
            if let Some(&(index, record)) = self.records.get(id.as_str()) {
                self.record_helpers(index, record);
            } else if let Some(&(index, variant)) = self.variants.get(id.as_str()) {
                self.variant_helpers(index, variant);
            }
        }
        for (ty, index) in self.lists.clone() {
            if let Type::List(element) = ty {
                self.list_layout(index, &element);
            }
        }
    }
    pub(super) fn ensure_block_types(&mut self, block: &Block) {
        for binding in &block.bindings {
            if let Some(ty) = &binding.annotation {
                self.ensure_type(ty);
            }
            self.ensure_expr_types(&binding.value);
        }
        self.ensure_expr_types(&block.value);
    }
    pub(super) fn ensure_expr_types(&mut self, expr: &Expr) {
        match expr {
            Expr::Reference { function, .. } => self.ensure_type(&Type::Function(
                self.functions[function.as_str()].1.signature_type(),
            )),
            Expr::Invoke {
                callee, arguments, ..
            } => {
                self.ensure_expr_types(callee);
                for argument in arguments {
                    self.ensure_expr_types(argument);
                }
            }
            Expr::List { element, .. } => self.ensure_type(&Type::List(Box::new(element.clone()))),
            Expr::ListOp { arguments, .. }
            | Expr::Call { arguments, .. }
            | Expr::NativeOp { arguments, .. }
            | Expr::BufferOp { arguments, .. } => {
                for argument in arguments {
                    self.ensure_expr_types(argument);
                }
            }
            Expr::Variant { payload, .. } => {
                if let Some(value) = payload {
                    self.ensure_expr_types(value);
                }
            }
            Expr::Record { fields, .. } => {
                for field in fields {
                    self.ensure_expr_types(&field.value);
                }
            }
            Expr::Field { base, .. } | Expr::Unary { operand: base, .. } => self.ensure_expr_types(base),
            Expr::Binary { left, right, .. } => {
                self.ensure_expr_types(left);
                self.ensure_expr_types(right);
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.ensure_expr_types(condition);
                self.ensure_block_types(then_block);
                self.ensure_block_types(else_block);
            }
            Expr::Fold {
                accumulator,
                count,
                initial,
                body,
                ..
            } => {
                self.ensure_type(&accumulator.ty);
                self.ensure_expr_types(count);
                self.ensure_expr_types(initial);
                self.ensure_block_types(body);
            }
            Expr::Match { subject, arms, .. } => {
                self.ensure_expr_types(subject);
                for arm in arms {
                    self.ensure_block_types(&arm.body);
                }
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                self.ensure_type(&state.ty);
                self.ensure_expr_types(initial);
                self.ensure_expr_types(condition);
                self.ensure_block_types(body);
            }
            Expr::Literal { .. } | Expr::Name { .. } => (),
        }
    }
    pub(super) fn count_value(&mut self, ty: &Type, value: &str) {
        match ty {
            Type::Resource(_) => self.line("lobo_fail(\"invocation.native.resource_value\");"),
            Type::F64 => self.line(format!(
                "lobo_output_tick(); lobo_output_bytes({});",
                r#"{"type":"F64","value":"0x0000000000000000"}"#.len()
            )),
            Type::Function(signature) => {
                self.line(format!("lobo_output_tick(); switch ({value}) {{"));
                for (index, target) in self.functions.values().copied().collect::<Vec<_>>() {
                    if target.actions.is_empty() && target.signature_type() == *signature {
                        let wire = serde_json::to_string(&Value::Function(FunctionValue {
                            id: target.id.clone(),
                            signature: signature.clone(),
                        }))
                        .unwrap();
                        self.line(format!("case {index}: lobo_output_bytes({}); break;", wire.len()));
                    }
                }
                self.line("default: lobo_fail(\"invocation.arguments.invalid\"); }");
            }
            Type::Applied { .. } | Type::Parameter(_) => unreachable!("checked type"),
            Type::Bytes | Type::Text => {
                self.line(format!("lobo_count_buffer({value},{});", *ty == Type::Text))
            }
            Type::Record(id) => self.line(format!(
                "lobo_count_record_{}({value});",
                self.records[id.as_str()].0
            )),
            Type::Variant(id) => self.line(format!(
                "lobo_count_variant_{}({value});",
                self.variants[id.as_str()].0
            )),
            Type::List(_) => self.line(format!("lobo_count_list_{}({value});", self.lists[ty])),
            Type::I64 => self.line(format!("lobo_count_integer({value});")),
            Type::Bool => self.line(format!(
                "lobo_output_tick(); lobo_output_bytes({value} ? 28 : 29);"
            )),
        }
    }
    fn list_layout(&mut self, index: usize, element: &Type) {
        let ctype = self.ctype(element);
        self.line(format!(
            "static bool lobo_equal_list_{index}(const lobo_list *a, const lobo_list *b) {{ lobo_compare_enter();
            bool result = (a ? a->length : 0) == (b ? b->length : 0);
            if (result) for (; a; a = a->tail, b = b->tail) {{ lobo_tick(); {ctype} left; {ctype} right; memcpy(&left, a->data, sizeof(left)); memcpy(&right, b->data, sizeof(right)); if (!{}) {{ result = false; break; }} }} lobo_compare_leave(); return result; }}",
            self.equal(element, "left", "right")
        ));
        self.line(format!(
            "static void lobo_print_list_{index}(const lobo_list *value) {{"
        ));
        let prefix = format!(
            "{{\"type\":\"List\",\"value\":{{\"element\":{},\"elements\":[",
            serde_json::to_string(element).unwrap()
        );
        self.line(format!("fputs({prefix:?}, stdout); bool first = true; for (; value; value = value->tail) {{ if (!first) fputs(\",\", stdout); first = false; {ctype} item; memcpy(&item, value->data, sizeof(item));"));
        self.print_value(element, "item");
        self.line("} fputs(\"]}}\", stdout); }");
        self.line(format!("static void lobo_count_list_{index}(const lobo_list *value) {{ lobo_output_tick(); ++lobo_output_depth; lobo_output_bytes({}); bool first = true; for (; value; value = value->tail) {{ if (!first) lobo_output_bytes(1); first = false; {ctype} item; memcpy(&item, value->data, sizeof(item));", prefix.len() + 3));
        self.count_value(element, "item");
        self.line("} --lobo_output_depth; }");
        self.line(format!(
            "static const lobo_list *lobo_parse_list_{index}(const char **cursor) {{ lobo_input_tick(); ++lobo_input_depth;
            uint64_t count = lobo_count_token(cursor); lobo_list *head = NULL, *last = NULL;
            for (uint64_t i = 0; i < count; ++i) {{ lobo_list *node = lobo_cons({}, NULL); {ctype} item;",
            self.inline_width(element)
        ));
        self.parse_value(element, "item", "cursor");
        self.line("if (node) { memcpy(node->data, &item, sizeof(item)); node->length = count - i; if (last) last->tail = node; else head = node; last = node; } } --lobo_input_depth; return head; }");
    }
}
