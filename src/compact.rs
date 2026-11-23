use super::*;

pub(super) struct Syntax {
    ids: BTreeMap<String, (String, Option<ByteSpan>)>,
    used: BTreeSet<String>,
    pub namespaces: BTreeMap<String, BTreeSet<String>>,
    groups: BTreeSet<String>,
    imports: BTreeMap<String, String>,
    pub generated: Vec<modules::Import>,
    pub native: bool,
    pub sqlite: bool,
    pub end: usize,
}

impl Syntax {
    fn external(bindings: BTreeMap<String, String>, end: usize) -> Fallible<Self> {
        let mut namespaces: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for key in bindings.keys() {
            if let Some((group, member)) = key.split_once('.') {
                check_name(group)?;
                namespaces.entry(group.into()).or_default().insert(member.into());
            }
        }
        Ok(Self {
            ids: bindings.into_iter().map(|(key, id)| (key, (id, None))).collect(),
            used: BTreeSet::new(),
            namespaces,
            groups: BTreeSet::new(),
            imports: BTreeMap::new(),
            generated: vec![],
            native: false,
            sqlite: false,
            end,
        })
    }
}

impl Parser {
    pub(super) fn compact_metadata(&mut self) -> Fallible<Syntax> {
        let saved = self.position;
        let mut depth = 0usize;
        let mut table = None;
        for index in saved..self.tokens.len() {
            match &self.tokens[index] {
                Token::Word(word) if word == "identities" && depth == 0 => {
                    table = Some(index);
                    break;
                }
                Token::Symbol(symbol) if symbol == "{" => depth += 1,
                Token::Symbol(symbol) if symbol == "}" => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        let external = self.managed.take();
        if table.is_none() {
            let external = external.ok_or_else(|| {
                syntax("identity metadata is missing; initialize a .lobo.ids.json manifest")
            })?;
            identities::validate(&external)?;
            return Syntax::external(external, self.tokens.len() - 1);
        }
        let end = table.unwrap();
        self.position = end + 1;
        self.expect("{")?;
        let mut ids = BTreeMap::new();
        loop {
            let span = Some(self.span());
            let Token::Word(mut key) = self.current().clone() else {
                return Err(syntax("expected identity key").at(span));
            };
            self.position += 1;
            if self.eat(".") {
                key.push('.');
                key.push_str(&self.name()?);
            }
            self.expect(":")?;
            let id_span = self.span();
            let Token::Id(id) = self.current().clone() else {
                return Err(syntax("expected unescaped stable identity").at(Some(id_span)));
            };
            if !valid_id(&id) {
                return Err(syntax("invalid stable identity").at(Some(id_span)));
            }
            self.position += 1;
            if ids.insert(key, (id, Some(id_span))).is_some() {
                return Err(syntax("repeated identity key").at(span));
            }
            if !self.eat(",") {
                break;
            }
        }
        if let Some(external) = external {
            identities::validate(&external)?;
            let inline: BTreeMap<_, _> = ids
                .iter()
                .map(|(key, (id, _))| (key.clone(), id.clone()))
                .collect();
            if inline != external {
                return Err(syntax("inline and managed identity metadata conflict"));
            }
        }
        self.expect("}")?;
        if self.current() != &Token::End {
            return Err(syntax("identities must be the final source declaration").at(Some(self.span())));
        }
        let mut namespaces: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for key in ids.keys() {
            if let Some((group, member)) = key.split_once('.') {
                check_name(group)?;
                namespaces.entry(group.into()).or_default().insert(member.into());
            }
        }
        self.position = saved;
        Ok(Syntax {
            ids,
            used: BTreeSet::new(),
            namespaces,
            groups: BTreeSet::new(),
            imports: BTreeMap::new(),
            generated: vec![],
            native: false,
            sqlite: false,
            end,
        })
    }

    pub(super) fn declaration_identity(&mut self, key: &str) -> Fallible<(String, Option<ByteSpan>)> {
        let span = Some(self.span());
        if let Some(compact) = self.compact.as_mut() {
            let (id, id_span) = compact
                .ids
                .get(key)
                .cloned()
                .ok_or_else(|| syntax(format!("missing persistent identity for {key}")).at(span))?;
            compact.used.insert(key.into());
            Ok((id, id_span))
        } else {
            let id_span = Some(self.spans.get(self.position + 3).copied().unwrap_or(self.span()));
            Ok((self.identity()?, id_span))
        }
    }

    pub(super) fn reference_name(&mut self) -> Fallible<(String, Option<ByteSpan>)> {
        let namespace = match self.current() {
            Token::Word(name)
                if self
                    .compact
                    .as_ref()
                    .is_some_and(|c| c.namespaces.contains_key(name)) =>
            {
                Some(name.clone())
            }
            _ => None,
        };
        if let Some(namespace) = namespace {
            self.position += 1;
            self.expect(".")?;
            let span = Some(self.span());
            let member = self.name()?;
            if !self.compact.as_ref().unwrap().namespaces[&namespace].contains(&member) {
                return Err(syntax("qualified reference must select an imported member").at(span));
            }
            Ok((member, span))
        } else {
            let span = Some(self.span());
            Ok((self.name()?, span))
        }
    }

    pub(super) fn import_group(&mut self) -> Fallible<Vec<modules::Import>> {
        let span = Some(self.span());
        let Token::Word(group) = self.current().clone() else {
            return Err(syntax("expected import group name").at(span));
        };
        check_name(&group).map_err(|e| e.at(span))?;
        self.position += 1;
        if !self.compact.as_mut().unwrap().groups.insert(group.clone()) {
            return Err(syntax("repeated import group").at(span));
        }
        self.expect("{")?;
        let mut imports = Vec::new();
        loop {
            let span = Some(self.span());
            let name = self.name()?;
            let (id, _) = self.declaration_identity(&format!("{group}.{name}"))?;
            if self
                .compact
                .as_mut()
                .unwrap()
                .imports
                .insert(name.clone(), id.clone())
                .is_some()
            {
                return Err(syntax("selected import names must be unique across groups").at(span));
            }
            imports.push(modules::Import { name, id, span });
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        self.expect(";")?;
        Ok(imports)
    }

    pub(super) fn finish_compact(&mut self) -> Fallible<()> {
        if let Some(compact) = &self.compact {
            for (key, (_, span)) in &compact.ids {
                if !compact.used.contains(key) {
                    return Err(syntax(format!(
                        "unbound identity key {key}; preserve its ID when renaming"
                    ))
                    .at(*span));
                }
            }
            // Metadata was parsed once before declarations, with its original source spans.
            self.position = self.tokens.len() - 1;
        }
        Ok(())
    }

    fn json_dependency(&mut self, name: &str, id: &str, span: Option<ByteSpan>) -> Fallible<String> {
        let compact = self.compact.as_mut().unwrap();
        if name == "Json" {
            if compact.imports.get(name).map(String::as_str) != Some(id) {
                return Err(
                    syntax(format!("json objects require selected {name} with identity {id}")).at(span),
                );
            }
            return Ok(name.into());
        }
        if let Some((alias, _)) = compact.imports.iter().find(|(_, target)| target.as_str() == id) {
            return Ok(alias.clone());
        }
        let alias = format!("__lobo_json_{name}");
        if !compact.generated.iter().any(|import| import.id == id) {
            compact.generated.push(modules::Import {
                name: alias.clone(),
                id: id.into(),
                span,
            });
        }
        Ok(alias)
    }

    pub(super) fn json_object(&mut self) -> Fallible<Expr> {
        let start = self.span().start;
        let span = Some(self.span());
        self.json_dependency("Json", "lobo.json.value", span)?;
        let object_name = self.json_dependency("object", "lobo.json.object", span)?;
        self.position += 1;
        self.expect("{")?;
        let mut object = Expr::Call {
            function: object_name,
            type_arguments: vec![],
            arguments: vec![],
            span,
            name_span: span,
        };
        let mut keys = BTreeSet::new();
        if !self.is("}") {
            let field_name = self.json_dependency("field", "lobo.json.field", span)?;
            loop {
                let key_span = Some(self.span());
                let (Token::Id(key) | Token::Text(key)) = self.current().clone() else {
                    return Err(syntax("json fields require a quoted key").at(key_span));
                };
                if !keys.insert(key.clone()) {
                    return Err(syntax("duplicate decoded JSON key").at(key_span));
                }
                self.position += 1;
                self.expect(":")?;
                let value = self.expr(0)?;
                let value_span = value.span();
                object = Expr::Call {
                    function: field_name.clone(),
                    type_arguments: vec![],
                    arguments: vec![
                        object,
                        Expr::Literal {
                            value: Value::Text(BufferValue::from_bytes(key.into_bytes())),
                            span: key_span,
                        },
                        Expr::Variant {
                            variant: "Json".into(),
                            type_arguments: vec![],
                            case: "String".into(),
                            payload: Some(Box::new(value)),
                            span: value_span,
                        },
                    ],
                    span: self.through(start),
                    name_span: span,
                };
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect("}")?;
        Ok(Expr::Variant {
            variant: "Json".into(),
            type_arguments: vec![],
            case: "Object".into(),
            payload: Some(Box::new(object)),
            span: self.through(start),
        })
    }
}
