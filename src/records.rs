use super::*;

pub(super) struct RecordTable<'a> {
    functions: BTreeMap<String, FunctionType>,
    abstract_types: bool,
    pub definitions: BTreeMap<String, &'a Record>,
    pub variants: BTreeMap<String, &'a Variant>,
    pub widths: BTreeMap<String, (usize, usize)>,
    pub order: Vec<String>,
}

pub(super) fn field_error(message: &str) -> Diagnostic {
    action_error("compile.record.fields", message)
}

impl<'a> RecordTable<'a> {
    pub fn with_functions(mut self, functions: &[Function]) -> Self {
        self.functions = functions
            .iter()
            .filter(|f| f.actions.is_empty())
            .map(|f| (f.id.clone(), f.signature_type()))
            .collect();
        self
    }
    pub fn new(records: &'a [Record], variants: &'a [Variant], resolved: bool) -> Fallible<Self> {
        Self::new_mode(records, variants, resolved, false)
    }
    pub fn new_mode(
        records: &'a [Record],
        variants: &'a [Variant],
        resolved: bool,
        abstract_types: bool,
    ) -> Fallible<Self> {
        if records.len() + variants.len() > 160 {
            return Err(Diagnostic::new(
                "resource.frontend.limit",
                "check",
                "compound declaration limit exceeded",
            ));
        }
        let mut table = Self {
            functions: BTreeMap::new(),
            abstract_types,
            definitions: BTreeMap::new(),
            variants: BTreeMap::new(),
            widths: BTreeMap::new(),
            order: Vec::new(),
        };
        let mut ids = BTreeSet::new();
        let mut record_names = BTreeSet::new();
        for record in records {
            check_name(&record.name).map_err(|e| e.at(record.span))?;
            if !valid_id(&record.id) {
                return Err(syntax("invalid record identity").at(record.span));
            }
            if !ids.insert(&record.id) {
                return Err(
                    action_error("compile.identity.duplicate", "record identity is repeated").at(record.span),
                );
            }
            if !resolved && !record_names.insert(&record.name) {
                return Err(action_error("compile.name.duplicate", "record name is repeated").at(record.span));
            }
            if record.fields.is_empty() || record.fields.len() > 16 {
                return Err(Diagnostic::new(
                    "resource.frontend.limit",
                    "check",
                    "record needs 1 to 16 fields",
                )
                .at(record.span));
            }
            let mut names = BTreeSet::new();
            for field in &record.fields {
                check_name(&field.name).map_err(|e| e.at(field.span))?;
                if !names.insert(&field.name) {
                    return Err(field_error("duplicate declaration field").at(field.span));
                }
            }
            let key = if resolved { &record.id } else { &record.name };
            if table.definitions.insert(key.clone(), record).is_some() {
                return Err(
                    action_error("compile.name.duplicate", "record name or identity is repeated")
                        .at(record.span),
                );
            }
        }
        for variant in variants {
            check_name(&variant.name).map_err(|e| e.at(variant.span))?;
            if !valid_id(&variant.id) {
                return Err(syntax("invalid variant identity").at(variant.span));
            }
            if !ids.insert(&variant.id) {
                return Err(
                    action_error("compile.identity.duplicate", "repeated compound identity").at(variant.span),
                );
            }
            if !resolved && !record_names.insert(&variant.name) {
                return Err(action_error("compile.name.duplicate", "repeated compound name").at(variant.span));
            }
            if variant.cases.is_empty() || variant.cases.len() > 16 {
                return Err(
                    action_error("resource.frontend.limit", "variant needs 1 to 16 cases").at(variant.span),
                );
            }
            let mut names = BTreeSet::new();
            for case in &variant.cases {
                check_name(&case.name).map_err(|e| e.at(case.span))?;
                if !names.insert(&case.name) {
                    return Err(
                        action_error("compile.variant.case", "duplicate declaration case").at(case.span)
                    );
                }
            }
            let key = if resolved { &variant.id } else { &variant.name };
            table.variants.insert(key.clone(), variant);
        }
        for key in table
            .definitions
            .keys()
            .chain(table.variants.keys())
            .cloned()
            .collect::<Vec<_>>()
        {
            table.layout(&key, &mut BTreeSet::new())?;
        }
        for ty in records.iter().flat_map(|r| r.fields.iter().map(|f| &f.ty)).chain(
            variants
                .iter()
                .flat_map(|v| v.cases.iter().filter_map(|c| c.payload.as_ref())),
        ) {
            table.check_type(ty)?;
        }
        Ok(table)
    }

    pub fn get_variant(&self, id: &str) -> Fallible<&'a Variant> {
        self.variants
            .get(id)
            .copied()
            .ok_or_else(|| action_error("compile.type.unknown", format!("unknown variant type {id}")))
    }

    pub fn get(&self, id: &str) -> Fallible<&'a Record> {
        self.definitions
            .get(id)
            .copied()
            .ok_or_else(|| action_error("compile.type.unknown", format!("unknown record type {id}")))
    }

    pub fn check_type(&self, ty: &Type) -> Fallible<()> {
        self.check_type_names(ty, 0)?;
        if self.height(ty) > 8 {
            return Err(action_error(
                "resource.frontend.limit",
                "type layout height exceeded",
            ));
        }
        Ok(())
    }
    fn check_type_names(&self, ty: &Type, depth: usize) -> Fallible<()> {
        if matches!(ty, Type::Applied { .. }) || (matches!(ty, Type::Parameter(_)) && !self.abstract_types) {
            return Err(action_error(
                "compile.generic.unlowered",
                "source type application in checked program",
            ));
        }
        if depth > 8 {
            return Err(action_error(
                "resource.frontend.limit",
                "type nesting limit exceeded",
            ));
        }
        if let Type::Record(id) = ty {
            self.get(id)?;
        }
        if let Type::Variant(id) = ty {
            self.get_variant(id)?;
        }
        if let Type::List(element) = ty {
            self.check_type_names(element, depth + 1)?;
        }
        if let Type::Function(signature) = ty {
            if signature.parameters.len() > 16 {
                return Err(action_error(
                    "resource.frontend.limit",
                    "function signature arity exceeded",
                ));
            }
            for parameter in &signature.parameters {
                self.check_type_names(parameter, depth + 1)?;
            }
            self.check_type_names(&signature.output, depth + 1)?;
        }
        Ok(())
    }

    pub fn width(&self, ty: &Type) -> usize {
        match ty {
            Type::Bytes | Type::Text => 2,
            Type::Record(id) | Type::Variant(id) => self.widths[id].0,
            _ => 1,
        }
    }
    fn height(&self, ty: &Type) -> usize {
        match ty {
            Type::Record(id) | Type::Variant(id) => self.widths[id].1,
            Type::List(_) => 1,
            _ => 0,
        }
    }
    fn layout_type(&mut self, ty: &Type, active: &mut BTreeSet<String>) -> Fallible<(usize, usize)> {
        match ty {
            Type::Function(_) => {
                self.check_type_names(ty, 0)?;
                Ok((1, 0))
            }
            Type::Resource(_) | Type::I64 | Type::F64 | Type::Bool => Ok((1, 0)),
            Type::Parameter(_) if self.abstract_types => Ok((1, 0)),
            Type::Parameter(_) => Err(action_error(
                "compile.generic.unlowered",
                "abstract parameter in checked program",
            )),
            Type::Applied { .. } => Err(action_error(
                "compile.generic.unlowered",
                "source type application in checked program",
            )),
            Type::Bytes | Type::Text => Ok((2, 0)),
            Type::Record(id) => {
                self.get(id)?;
                self.layout(id, active)
            }
            Type::Variant(id) => {
                self.get_variant(id)?;
                self.layout(id, active)
            }
            Type::List(element) => {
                // List storage is indirection: validate names, but do not follow the
                // element's physical layout on this inline dependency path.
                self.check_type_names(element, 1)?;
                Ok((1, 1))
            }
        }
    }

    fn layout(&mut self, id: &str, active: &mut BTreeSet<String>) -> Fallible<(usize, usize)> {
        if let Some(layout) = self.widths.get(id) {
            return Ok(*layout);
        }
        let record = self.definitions.get(id).copied();
        let variant = self.variants.get(id).copied();
        let span = record
            .and_then(|r| r.span)
            .or_else(|| variant.and_then(|v| v.span));
        if record.is_none() && variant.is_none() {
            return Err(action_error("compile.type.unknown", "unknown compound layout"));
        }
        if !active.insert(id.into()) {
            return Err(action_error("compile.record.cycle", "recursive inline compound layout").at(span));
        }
        let mut width = usize::from(variant.is_some());
        let mut height = 1;
        let children: Vec<(&Type, Option<ByteSpan>)> = if let Some(record) = record {
            record.fields.iter().map(|f| (&f.ty, f.span)).collect()
        } else {
            variant
                .unwrap()
                .cases
                .iter()
                .filter_map(|c| c.payload.as_ref().map(|ty| (ty, c.span)))
                .collect()
        };
        for (ty, child_span) in children {
            let (leaves, depth) = self.layout_type(ty, active).map_err(|e| e.at(child_span))?;
            if variant.is_some() {
                width = width.max(1 + leaves);
            } else {
                width += leaves;
            }
            height = height.max(depth + 1);
            if width > 64 || height > 8 {
                return Err(Diagnostic::new(
                    "resource.frontend.limit",
                    "check",
                    "compound layout limit exceeded",
                )
                .at(span));
            }
        }
        active.remove(id);
        self.widths.insert(id.into(), (width, height));
        self.order.push(id.into());
        Ok((width, height))
    }

    pub fn valid_value(&self, value: &Value, ty: &Type) -> bool {
        match (value, ty) {
            (Value::Function(value), Type::Function(signature)) => {
                &value.signature == signature && self.functions.get(&value.id) == Some(signature)
            }
            (Value::Bytes(_), Type::Bytes) => true,
            (Value::Text(value), Type::Text) => std::str::from_utf8(value.bytes()).is_ok(),
            (Value::List(list), Type::List(element)) => {
                &list.element == element.as_ref() && list.iter().all(|v| self.valid_value(v, element))
            }
            (Value::I64(_), Type::I64) | (Value::F64(_), Type::F64) | (Value::Bool(_), Type::Bool) => true,
            (Value::Record(value), Type::Record(id)) if &value.id == id => {
                let Ok(definition) = self.get(id) else {
                    return false;
                };
                value.fields.len() == definition.fields.len()
                    && definition.fields.iter().all(|field| {
                        value
                            .fields
                            .get(&field.name)
                            .is_some_and(|v| self.valid_value(v, &field.ty))
                    })
            }
            (Value::Variant(value), Type::Variant(id)) if &value.id == id => self
                .get_variant(id)
                .ok()
                .and_then(|v| v.cases.iter().find(|c| c.name == value.case))
                .is_some_and(|case| match (&case.payload, &value.payload) {
                    (None, None) => true,
                    (Some(ty), Some(payload)) => self.valid_value(payload, ty),
                    _ => false,
                }),
            _ => false,
        }
    }

    pub fn parse_value(
        &self,
        ty: &Type,
        leaves: &mut std::str::Split<'_, char>,
        remaining: &mut buffers::InputBudget,
    ) -> Fallible<Value> {
        self.parse_value_inner(ty, leaves, remaining, 0)
    }
    fn parse_value_inner(
        &self,
        ty: &Type,
        leaves: &mut std::str::Split<'_, char>,
        remaining: &mut buffers::InputBudget,
        depth: usize,
    ) -> Fallible<Value> {
        if depth > 8 {
            return Err(Diagnostic::new(
                "resource.input.limit",
                "invoke",
                "value nesting limit exceeded",
            ));
        }
        remaining.node()?;
        let invalid = || {
            Diagnostic::new(
                "invocation.arguments.invalid",
                "invoke",
                "invalid compound argument",
            )
        };
        if let Type::Function(signature) = ty {
            let id = leaves
                .next()
                .and_then(|text| text.strip_prefix("Function:"))
                .ok_or_else(invalid)?;
            if self.functions.get(id) != Some(signature) {
                return Err(invalid());
            }
            return Ok(Value::Function(FunctionValue {
                id: id.into(),
                signature: signature.clone(),
            }));
        }
        if matches!(ty, Type::Bytes | Type::Text) {
            let token = leaves.next().ok_or_else(invalid)?;
            let prefix = if *ty == Type::Text { "Text:" } else { "Bytes:" };
            let encoded = token.strip_prefix(prefix).ok_or_else(invalid)?;
            if encoded.len() % 2 != 0 || !encoded.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid());
            }
            remaining.buffer(encoded.len() / 2)?;
            let bytes = buffers::decode_hex(encoded).ok_or_else(invalid)?;
            if *ty == Type::Text && std::str::from_utf8(&bytes).is_err() {
                return Err(invalid());
            }
            let buffer = BufferValue::from_bytes(bytes);
            return Ok(if *ty == Type::Text {
                Value::Text(buffer)
            } else {
                Value::Bytes(buffer)
            });
        }
        if let Type::List(element) = ty {
            let text = leaves
                .next()
                .and_then(|s| s.strip_prefix("Count:"))
                .ok_or_else(invalid)?;
            let count: u64 = text.parse().map_err(|_| invalid())?;
            if count > i64::MAX as u64 || count.to_string() != text {
                return Err(invalid());
            }
            let mut elements = Vec::new();
            for _ in 0..count {
                elements.push(self.parse_value_inner(element, leaves, remaining, depth + 1)?);
            }
            return Ok(Value::List(ListValue::from_values(*element.clone(), elements)));
        }
        if let Type::Variant(id) = ty {
            let name = leaves
                .next()
                .and_then(|s| s.strip_prefix("Case:"))
                .ok_or_else(invalid)?;
            let case = self
                .get_variant(id)?
                .cases
                .iter()
                .find(|c| c.name == name)
                .ok_or_else(invalid)?;
            let payload = case
                .payload
                .as_ref()
                .map(|ty| {
                    self.parse_value_inner(ty, leaves, remaining, depth + 1)
                        .map(Box::new)
                })
                .transpose()?;
            return Ok(Value::Variant(VariantValue {
                id: id.clone(),
                case: name.into(),
                payload,
            }));
        }
        if let Type::Record(id) = ty {
            let definition = self.get(id)?;
            let mut fields = BTreeMap::new();
            for field in &definition.fields {
                fields.insert(
                    field.name.clone(),
                    self.parse_value_inner(&field.ty, leaves, remaining, depth + 1)?,
                );
            }
            return Ok(Value::Record(RecordValue {
                id: id.clone(),
                fields,
            }));
        }
        let text = leaves.next().ok_or_else(invalid)?;
        let value = scalar_argument(text)?;
        if value.ty() != *ty {
            return Err(invalid());
        }
        Ok(value)
    }

    pub fn charge_value(&self, value: &Value, ty: &Type, remaining: &mut u64) -> Fallible<()> {
        match (value, ty) {
            (Value::Bytes(value), Type::Bytes) | (Value::Text(value), Type::Text) => {
                lists::charge(remaining, buffers::cells(value.len()))?
            }
            (Value::List(list), Type::List(element)) => {
                for item in list.iter() {
                    lists::charge(remaining, 3 + self.width(element))?;
                    self.charge_value(item, element, remaining)?;
                }
            }
            (Value::Record(value), Type::Record(id)) => {
                for field in &self.get(id)?.fields {
                    self.charge_value(&value.fields[&field.name], &field.ty, remaining)?;
                }
            }
            (Value::Variant(value), Type::Variant(id)) => {
                if let Some(payload) = &value.payload {
                    let case = self
                        .get_variant(id)?
                        .cases
                        .iter()
                        .find(|c| c.name == value.case)
                        .unwrap();
                    self.charge_value(payload, case.payload.as_ref().unwrap(), remaining)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

pub(super) fn scalar_argument(text: &str) -> Fallible<Value> {
    if text.starts_with("F64:") {
        return floats::argument(text);
    }
    let invalid = || {
        Diagnostic::new(
            "invocation.arguments.invalid",
            "invoke",
            "expected a typed scalar argument",
        )
    };
    let (ty, value) = text.split_once(':').ok_or_else(invalid)?;
    match ty {
        "I64" => value.parse().map(Value::I64).map_err(|_| invalid()),
        "Bool" => value.parse().map(Value::Bool).map_err(|_| invalid()),
        _ => Err(invalid()),
    }
}
