//! Persistent authoring metadata. Compiler entry points remain read-only.
use crate::*;
use serde::de::{MapAccess, Visitor};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub type Bindings = BTreeMap<String, String>;

struct UniqueBindings(Bindings);
impl<'de> Deserialize<'de> for UniqueBindings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueBindings;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("unique identity bindings")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut result = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, String>()? {
                    if result.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate identity key"));
                    }
                }
                Ok(UniqueBindings(result))
            }
        }
        deserializer.deserialize_map(UniqueVisitor)
    }
}

pub(crate) fn deserialize_optional<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<Bindings>, D::Error> {
    Option::<UniqueBindings>::deserialize(d).map(|value| value.map(|v| v.0))
}

fn deserialize_bindings<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Bindings, D::Error> {
    UniqueBindings::deserialize(d).map(|value| value.0)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    #[serde(deserialize_with = "deserialize_bindings")]
    pub identities: Bindings,
}

fn failure(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("invocation.identity.invalid", "load", message)
}

pub(crate) fn validate(bindings: &Bindings) -> Fallible<()> {
    if bindings.len() > 2048 {
        return Err(failure("identity binding limit exceeded"));
    }
    for (key, id) in bindings {
        let parts: Vec<_> = key.split('.').collect();
        if parts.len() > 2 || parts.iter().any(|part| !valid_name(part)) || !valid_id(id) {
            return Err(failure("invalid identity key or stable ID"));
        }
    }
    Ok(())
}

pub fn parse_manifest(text: &str) -> Fallible<Bindings> {
    if text.len() > MAX_SOURCE_BYTES {
        return Err(failure("identity manifest size limit exceeded"));
    }
    let value: Manifest = serde_json::from_str(text).map_err(|e| failure(e.to_string()))?;
    if value.format != "lobo-identities/0.1" {
        return Err(failure("unsupported identity manifest version"));
    }
    validate(&value.identities)?;
    Ok(value.identities)
}

pub fn sidecar(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".ids.json");
    name.into()
}

fn read_regular(path: &Path, budget: usize) -> Fallible<String> {
    if !fs::symlink_metadata(path)
        .map_err(|e| failure(format!("{}: {e}", path.display())))?
        .file_type()
        .is_file()
    {
        return Err(failure("identity input must be a regular file, not a symlink"));
    }
    let file = fs::File::open(path).map_err(|e| failure(format!("{}: {e}", path.display())))?;
    if !file.metadata().map_err(|e| failure(e.to_string()))?.is_file() {
        return Err(failure("identity input must be a regular file"));
    }
    let mut bytes = Vec::new();
    file.take(budget as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| failure(e.to_string()))?;
    if bytes.len() > budget {
        return Err(Diagnostic::new(
            "resource.frontend.limit",
            "parse",
            "source and identity metadata exceed their byte budget",
        ));
    }
    String::from_utf8(bytes).map_err(|e| failure(e.to_string()))
}

pub fn read_source(path: &Path, budget: usize) -> Fallible<SourceFile> {
    let text = read_regular(path, budget)?;
    let metadata = sidecar(path);
    let lock = metadata.with_extension("json.lock");
    if lock.exists() {
        return Err(failure(format!(
            "identity update in progress: {}",
            lock.display()
        )));
    }
    let identities = match fs::symlink_metadata(&metadata) {
        Ok(_) => Some(parse_manifest(&read_regular(
            &metadata,
            budget.saturating_sub(text.len()),
        )?)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(failure(e.to_string())),
    };
    Ok(SourceFile {
        name: path.to_string_lossy().into_owned(),
        text,
        identities,
    })
}

struct Header {
    module: String,
    declarations: Vec<String>,
    imports: Vec<(String, Vec<String>)>,
    inline: bool,
    modern: bool,
}

fn header(text: &str) -> Fallible<Header> {
    let (tokens, _) = lex(text)?;
    let word = |index: usize| match tokens.get(index) {
        Some(Token::Word(s)) => Ok(s.clone()),
        _ => Err(failure("expected declaration name")),
    };
    if tokens.first() != Some(&Token::Word("module".into())) {
        return Err(failure("expected module header"));
    }
    let mut result = Header {
        module: word(1)?,
        declarations: vec![],
        imports: vec![],
        inline: false,
        modern: tokens.get(2) == Some(&Token::Symbol(";".into())),
    };
    let mut braces = 0usize;
    let mut parentheses = 0usize;
    let mut angles = 0usize;
    let mut i = 2;
    while i < tokens.len() {
        if braces == 0 && parentheses == 0 && angles == 0 {
            if matches!(&tokens[i],Token::Word(s) if s=="identities") {
                result.inline = true;
                break;
            }
            if matches!(&tokens[i],Token::Word(s) if matches!(s.as_str(),"fn"|"function"|"record"|"variant"))
            {
                result.declarations.push(word(i + 1)?);
                i += 2;
                continue;
            }
            if tokens[i] == Token::Word("use".into()) {
                let group = word(i + 1)?;
                i += 2;
                if tokens.get(i) != Some(&Token::Symbol("{".into())) {
                    return Err(failure("expected import group"));
                }
                i += 1;
                let mut names = vec![];
                loop {
                    names.push(word(i)?);
                    i += 1;
                    if tokens.get(i) == Some(&Token::Symbol(",".into())) {
                        i += 1;
                    } else {
                        break;
                    }
                }
                if tokens.get(i) != Some(&Token::Symbol("}".into())) {
                    return Err(failure("invalid import group"));
                }
                result.imports.push((group, names));
                i += 1;
                continue;
            }
        }
        match &tokens[i] {
            Token::Symbol(s) if s == "{" => braces += 1,
            Token::Symbol(s) if s == "}" => braces = braces.saturating_sub(1),
            Token::Symbol(s) if s == "(" => parentheses += 1,
            Token::Symbol(s) if s == ")" => parentheses = parentheses.saturating_sub(1),
            Token::Symbol(s) if s == "<" && braces == 0 && parentheses == 0 => angles += 1,
            Token::Symbol(s) if s == ">" && braces == 0 && parentheses == 0 => {
                angles = angles.saturating_sub(1)
            }
            _ => {}
        }
        i += 1;
    }
    Ok(result)
}

fn existing(source: &SourceFile, scan: &Header) -> Fallible<Option<Bindings>> {
    if let Some(ids) = &source.identities {
        return Ok(Some(ids.clone()));
    }
    if scan.modern && !scan.inline {
        return Ok(None);
    }
    let (tokens, spans) = lex(&source.text)?;
    let program = Parser {
        tokens,
        spans,
        position: 0,
        depth: 0,
        compact: None,
        managed: None,
        statement_counter: 0,
    }
    .program()?;
    let mut ids = Bindings::from([("module".into(), program.id)]);
    for (name, id) in program
        .functions
        .iter()
        .map(|d| (&d.name, &d.id))
        .chain(program.records.iter().map(|d| (&d.name, &d.id)))
        .chain(program.variants.iter().map(|d| (&d.name, &d.id)))
    {
        ids.insert(name.clone(), id.clone());
    }
    for (group, names) in &scan.imports {
        for name in names {
            let import = program.scopes[0]
                .imports
                .iter()
                .find(|import| &import.name == name)
                .ok_or_else(|| failure("missing import identity"))?;
            ids.insert(format!("{group}.{name}"), import.id.clone());
        }
    }
    Ok(Some(ids))
}

fn fresh(used: &mut BTreeSet<String>) -> Fallible<String> {
    for _ in 0..32 {
        let mut bytes = [0u8; 16];
        fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(&mut bytes))
            .map_err(|e| failure(format!("OS random ID allocation unavailable: {e}")))?;
        let id = format!(
            "lobo.{}",
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
        if used.insert(id.clone()) {
            return Ok(id);
        }
    }
    Err(failure("could not allocate a unique identity"))
}

fn keys(scan: &Header) -> BTreeSet<String> {
    std::iter::once("module".into())
        .chain(scan.declarations.iter().cloned())
        .chain(
            scan.imports
                .iter()
                .flat_map(|(g, names)| names.iter().map(move |n| format!("{g}.{n}"))),
        )
        .collect()
}

fn persist(source: &SourceFile, old: Option<&Bindings>, ids: &Bindings) -> Fallible<()> {
    let path = Path::new(&source.name);
    let metadata = sidecar(path);
    if let Ok(text) = read_regular(&metadata, MAX_SOURCE_BYTES) {
        if &parse_manifest(&text)? == ids {
            return Ok(());
        }
    }
    let lock = metadata.with_extension("json.lock");
    let _owner = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .map_err(|e| failure(format!("identity metadata is locked: {e}")))?;
    let result = (|| {
        if fs::read_to_string(path).map_err(|e| failure(e.to_string()))? != source.text {
            return Err(failure("source changed while assigning identities"));
        }
        if metadata.exists() {
            let current = parse_manifest(&read_regular(&metadata, MAX_SOURCE_BYTES)?)?;
            if Some(&current) != old {
                return Err(failure("identity metadata changed while editing"));
            }
        }
        let bytes = serde_json::to_vec_pretty(&Manifest {
            format: "lobo-identities/0.1".into(),
            identities: ids.clone(),
        })
        .unwrap();
        let temporary = metadata.with_extension("json.tmp");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| failure(e.to_string()))?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| failure(e.to_string()))?;
        if metadata.exists() {
            fs::rename(&temporary, &metadata).map_err(|e| failure(e.to_string()))?;
        } else {
            fs::hard_link(&temporary, &metadata).map_err(|e| failure(e.to_string()))?;
            fs::remove_file(&temporary).map_err(|e| failure(e.to_string()))?;
        }
        Ok(())
    })();
    let _ = fs::remove_file(lock);
    result
}

pub fn author(op: &str, paths: &[String], rename: Option<(&str, &str)>) -> Fallible<Vec<String>> {
    if paths.is_empty() || paths.len() > 16 {
        return Err(failure("supply 1..16 explicit sources"));
    }
    let sources: Vec<_> = paths
        .iter()
        .map(|path| read_source(Path::new(path), MAX_SOURCE_BYTES))
        .collect::<Fallible<_>>()?;
    let scans: Vec<_> = sources.iter().map(|s| header(&s.text)).collect::<Fallible<_>>()?;
    if !scans[0].modern {
        return Err(failure("managed authoring requires modern source"));
    }
    let originals: Vec<_> = sources
        .iter()
        .zip(&scans)
        .map(|(s, h)| existing(s, h))
        .collect::<Fallible<_>>()?;
    let mut bindings: Vec<_> = originals
        .iter()
        .map(|ids| ids.clone().unwrap_or_default())
        .collect();
    let mut used: BTreeSet<_> = bindings.iter().flat_map(|ids| ids.values().cloned()).collect();
    if op == "rename" {
        let (old, new) = rename.ok_or_else(|| failure("rename requires old and new binding names"))?;
        if old == "module" || !valid_name(new) || bindings[0].contains_key(new) {
            return Err(failure("invalid identity rename"));
        }
        if keys(&scans[0]).contains(old) || !scans[0].declarations.contains(&new.to_owned()) {
            return Err(failure(
                "rename source references first, then retarget the persistent binding",
            ));
        }
        let id = bindings[0]
            .remove(old)
            .ok_or_else(|| failure("old identity binding not found"))?;
        bindings[0].insert(new.into(), id);
    } else if !matches!(op, "init" | "sync") {
        return Err(failure("unknown identity authoring operation"));
    }
    for (scan, ids) in scans.iter().zip(&mut bindings) {
        let valid = keys(scan);
        if scan.modern && ids.keys().any(|key| !valid.contains(key)) {
            return Err(failure(
                "stale identity binding; use explicit identities rename rather than guessing a new identity",
            ));
        }
        for key in std::iter::once("module".to_owned()).chain(scan.declarations.iter().cloned()) {
            if let std::collections::btree_map::Entry::Vacant(entry) = ids.entry(key) {
                entry.insert(fresh(&mut used)?);
            }
        }
    }
    for (index, scan) in scans.iter().enumerate() {
        for (group, names) in &scan.imports {
            for member in names {
                let key = format!("{group}.{member}");
                if bindings[index].contains_key(&key) {
                    continue;
                }
                let candidates: Vec<_> = scans
                    .iter()
                    .enumerate()
                    .filter(|(i, h)| {
                        *i != index
                            && (h.module == *group
                                || Path::new(&sources[*i].name).file_stem().and_then(|s| s.to_str())
                                    == Some(group.as_str()))
                    })
                    .filter_map(|(i, _)| bindings[i].get(member).cloned())
                    .collect();
                if candidates.len() != 1 {
                    return Err(failure(format!(
                        "import {key} needs one matching explicitly supplied module"
                    )));
                }
                bindings[index].insert(key, candidates[0].clone());
            }
        }
    }
    let mut proposed = sources.clone();
    for (index, source) in proposed.iter_mut().enumerate() {
        if scans[index].modern && (index == 0 || source.identities.is_some() || originals[index].is_none()) {
            source.identities = Some(bindings[index].clone());
        }
    }
    compile_bundle(&paths[0], &proposed)?;
    let mut written = vec![];
    for (index, source) in sources.iter().enumerate() {
        if proposed[index].identities.is_some() {
            persist(source, source.identities.as_ref(), &bindings[index])?;
            written.push(sidecar(Path::new(&source.name)).to_string_lossy().into_owned());
        }
    }
    Ok(written)
}
