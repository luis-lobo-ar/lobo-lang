use lobo_core::{
    compile, compile_bundle, compile_bundle_json, identities, Artifact, NativeBackend, DEFAULT_HEAP_CELLS,
    LANGUAGE, MAX_SOURCE_BYTES,
};
use serde_json::{json, Value as Json};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};

fn error(code: &str, message: impl ToString) -> Json {
    json!({"status":"error", "kind":"invocation_error", "diagnostic":{"code":code,"message":message.to_string()}})
}

fn load(path: &str, dependencies: &[String]) -> Result<Artifact, Json> {
    if !path.ends_with(".loboc") || !dependencies.is_empty() {
        if path.ends_with(".loboc") {
            return Err(error(
                "invocation.cli.invalid",
                "checked artifacts already contain their source closure",
            ));
        }
        if dependencies.len() >= 16 {
            return Err(error(
                "invocation.cli.invalid",
                "at most 15 additional source files are allowed",
            ));
        }
        let mut sources = Vec::new();
        let mut remaining = MAX_SOURCE_BYTES;
        for name in std::iter::once(path).chain(dependencies.iter().map(String::as_str)) {
            let source = identities::read_source(std::path::Path::new(name), remaining)
                .map_err(|e| json!({"status":"error","kind":e.kind(),"diagnostic":e}))?;
            remaining = remaining.saturating_sub(
                source.text.len()
                    + source
                        .identities
                        .as_ref()
                        .map_or(0, |ids| serde_json::to_vec(ids).unwrap().len()),
            );
            sources.push(source);
        }
        return compile_bundle(path, &sources)
            .map_err(|e| json!({"status":"error","kind":e.kind(),"diagnostic":e}));
    }
    let file = fs::File::open(path).map_err(|e| error("invocation.file.read", e))?;
    let mut text = String::new();
    file.take(4_194_305)
        .read_to_string(&mut text)
        .map_err(|e| error("invocation.file.read", e))?;
    let result = Artifact::from_json(&text);
    result.map_err(|e| {
        let mut outcome = json!({"status":"error", "kind":e.kind(), "diagnostic":e});
        if !path.ends_with(".loboc") {
            outcome["diagnostic"]["source"] = json!(path);
        }
        outcome
    })
}

fn execute(args: &[String]) -> Result<Json, Json> {
    if args.is_empty() || args[0] == "--help" || args[0] == "help" {
        return Ok(json!({"status":"ok", "language":LANGUAGE,"commands":[
            "lobo check FILE [--source FILE ...] | lobo check --stdin | lobo check --bundle-stdin", "lobo compile FILE [--source FILE ...] -o NEW_EXECUTABLE [--emit native|checked|c|llvm] [--backend c|llvm]",
            "lobo run FILE [--source FILE ...] [--backend c|llvm] --entry FUNCTION_ID [--arg TYPED_VALUE] [--fuel 100000] [--heap-cells 1000000]",
            "lobo eval FILE [--source FILE ...] --entry FUNCTION_ID [--arg TYPED_VALUE] [--fuel 100000] [--heap-cells 1000000] (reference evaluator)"
            ,"lobo identities init|sync FILE [--source FILE ...]",
            "lobo identities rename FILE OLD NEW [--source FILE ...] (retarget metadata after renaming source)"
        ]}));
    }
    if args.len() < 2 {
        return Err(error("invocation.cli.invalid", "expected command and file"));
    }
    if args[0] == "identities" {
        let operation = args[1].as_str();
        let path = args
            .get(2)
            .ok_or_else(|| error("invocation.cli.invalid", "expected source file"))?;
        let (rename, mut position) = if operation == "rename" {
            let old = args
                .get(3)
                .ok_or_else(|| error("invocation.cli.invalid", "expected old binding"))?;
            let new = args
                .get(4)
                .ok_or_else(|| error("invocation.cli.invalid", "expected new binding"))?;
            (Some((old.as_str(), new.as_str())), 5)
        } else {
            (None, 3)
        };
        let mut paths = vec![path.clone()];
        while position < args.len() {
            if args[position] != "--source" {
                return Err(error("invocation.cli.invalid", "expected --source FILE"));
            }
            paths.push(
                args.get(position + 1)
                    .ok_or_else(|| error("invocation.cli.invalid", "expected dependency file"))?
                    .clone(),
            );
            position += 2;
        }
        return identities::author(operation, &paths, rename)
            .map(|manifests| json!({"status":"ok","operation":operation,"manifests":manifests}))
            .map_err(|e| json!({"status":"error","kind":e.kind(),"diagnostic":e}));
    }
    if !matches!(args[0].as_str(), "check" | "compile" | "run" | "eval") {
        return Err(error("invocation.cli.invalid", "unknown command"));
    }
    if args[0] == "check" && args[1] == "--bundle-stdin" {
        if args.len() != 2 {
            return Err(error(
                "invocation.cli.invalid",
                "check --bundle-stdin accepts no other arguments",
            ));
        }
        let mut bytes = Vec::new();
        std::io::stdin()
            .take((4 * MAX_SOURCE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| error("invocation.source.read", e))?;
        if bytes.len() > 4 * MAX_SOURCE_BYTES {
            return Err(
                json!({"status":"error","kind":"resource_exhausted","diagnostic":{"code":"resource.frontend.limit","phase":"load","message":"source bundle envelope size limit exceeded"}}),
            );
        }
        let text = String::from_utf8(bytes).map_err(|e| error("invocation.source.read", e))?;
        return compile_bundle_json(&text)
            .map(|artifact| json!({"status":"ok","language":LANGUAGE,"entries":artifact.entries(),"actions":artifact.action_summary()}))
            .map_err(|e| json!({"status":"error","kind":e.kind(),"diagnostic":e}));
    }
    if args[0] == "check" && args[1] == "--stdin" {
        if args.len() != 2 {
            return Err(error(
                "invocation.cli.invalid",
                "check --stdin accepts no other arguments",
            ));
        }
        let mut bytes = Vec::new();
        std::io::stdin()
            .take((MAX_SOURCE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| error("invocation.source.read", e))?;
        if bytes.len() > MAX_SOURCE_BYTES {
            return Err(
                json!({"status":"error", "kind":"resource_exhausted", "diagnostic":{
                    "code":"resource.frontend.limit", "phase":"parse", "message":"source size limit exceeded"
                }}),
            );
        }
        let source = String::from_utf8(bytes).map_err(|e| error("invocation.source.read", e))?;
        return compile(&source)
            .map(|artifact| json!({"status":"ok", "language":LANGUAGE,"entries":artifact.entries(),"actions":artifact.action_summary()}))
            .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}));
    }
    let mut filtered = args[..2].to_vec();
    let mut dependencies = Vec::new();
    let mut position = 2;
    while position < args.len() {
        if args[position] == "--source" {
            dependencies.push(
                args.get(position + 1)
                    .ok_or_else(|| error("invocation.cli.invalid", "--source needs a file"))?
                    .clone(),
            );
        } else {
            filtered.push(args[position].clone());
            if let Some(value) = args.get(position + 1) {
                filtered.push(value.clone());
            }
        }
        position += 2;
    }
    let args = filtered.as_slice();
    let artifact = load(&args[1], &dependencies)?;
    match args[0].as_str() {
        "check" => {
            if args.len() != 2 {
                return Err(error("invocation.cli.invalid", "check accepts exactly one file"));
            }
            Ok(
                json!({"status":"ok", "language":LANGUAGE,"entries":artifact.entries(),"actions":artifact.action_summary()}),
            )
        }
        "compile" => {
            let mut output_path = None;
            let mut emit = None;
            let mut backend = None;
            let mut index = 2;
            while index < args.len() {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| error("invocation.cli.invalid", "missing option value"))?;
                match args[index].as_str() {
                    "-o" if output_path.is_none() => output_path = Some(value.as_str()),
                    "--emit" if emit.is_none() => emit = Some(value.as_str()),
                    "--backend" if backend.is_none() => backend = Some(parse_backend(value)?),
                    _ => {
                        return Err(error(
                            "invocation.cli.invalid",
                            "unknown or repeated compile option",
                        ))
                    }
                }
                index += 2;
            }
            let output_path = output_path.ok_or_else(|| error("invocation.cli.invalid", "-o is required"))?;
            let emit = emit.unwrap_or("native");
            if backend.is_some() && emit != "native" {
                return Err(error(
                    "invocation.cli.invalid",
                    "--backend selects native compilation; use --emit llvm directly for inspection",
                ));
            }
            if emit == "native" {
                if output_path.ends_with(".loboc") {
                    return Err(error(
                        "invocation.cli.invalid",
                        ".loboc is reserved for --emit checked",
                    ));
                }
                artifact
                    .compile_native_with_backend(
                        std::path::Path::new(output_path),
                        backend.unwrap_or(NativeBackend::C),
                    )
                    .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))?;
                return Ok(
                    json!({"status":"ok", "format":"native", "artifact":output_path, "entries":artifact.entries()}),
                );
            }
            let encoded = match emit {
                "checked" if output_path.ends_with(".loboc") => artifact
                    .to_json()
                    .map_err(|e| error("invocation.artifact.encode", e))?,
                "c" => artifact
                    .emit_c()
                    .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))?,
                "llvm" => artifact
                    .emit_llvm()
                    .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))?,
                _ => {
                    return Err(error(
                        "invocation.cli.invalid",
                        "expected --emit native|checked|c|llvm; checked output needs .loboc extension",
                    ))
                }
            };
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output_path)
                .map_err(|e| error("invocation.file.create", e))?;
            output
                .write_all(encoded.as_bytes())
                .map_err(|e| error("invocation.file.write", e))?;
            Ok(json!({"status":"ok", "format":emit,"artifact":output_path,"entries":artifact.entries()}))
        }
        "run" => {
            let mut backend = None;
            let mut arguments = Vec::new();
            for pair in args[2..].chunks(2) {
                if pair[0] == "--backend" {
                    if backend.is_some() {
                        return Err(error("invocation.cli.invalid", "repeated backend selector"));
                    }
                    backend =
                        Some(parse_backend(pair.get(1).ok_or_else(|| {
                            error("invocation.cli.invalid", "missing backend value")
                        })?)?);
                } else {
                    arguments.extend_from_slice(pair);
                }
            }
            let result = artifact
                .run_native_with_backend(&arguments, backend.unwrap_or(NativeBackend::C))
                .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))?;
            let json: Json = serde_json::from_slice(&result.stdout)
                .map_err(|e| error("invocation.native.output_invalid", e))?;
            if result.status.success() {
                Ok(json)
            } else {
                Err(json)
            }
        }
        "eval" => {
            let mut entry = None;
            let mut fuel = 100_000;
            let mut heap = DEFAULT_HEAP_CELLS;
            let mut fuel_seen = false;
            let mut heap_seen = false;
            let mut arguments = Vec::new();
            let mut i = 2;
            while i < args.len() {
                let option = &args[i];
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| error("invocation.cli.invalid", "missing option value"))?;
                match option.as_str() {
                    "--entry" if entry.is_none() => entry = Some(value.as_str()),
                    "--fuel" if !fuel_seen => {
                        fuel = value
                            .parse::<u64>()
                            .map_err(|e| error("invocation.cli.invalid", e))?;
                        fuel_seen = true;
                    }
                    "--heap-cells" if !heap_seen => {
                        heap = value
                            .parse::<u64>()
                            .map_err(|e| error("invocation.cli.invalid", e))?;
                        heap_seen = true;
                    }
                    "--arg" => {
                        arguments.push(value);
                    }
                    _ => return Err(error("invocation.cli.invalid", "unknown or repeated option")),
                }
                i += 2;
            }
            let entry = entry.ok_or_else(|| error("invocation.cli.invalid", "--entry is required"))?;
            let arguments = artifact
                .parse_arguments(entry, &arguments.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))?;
            artifact
                .run_with_heap(entry, arguments, fuel, heap)
                .map(|value| json!({"status":"ok", "kind":"value", "value":value}))
                .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))
        }
        _ => unreachable!(),
    }
}

fn parse_backend(value: &str) -> Result<NativeBackend, Json> {
    match value {
        "c" => Ok(NativeBackend::C),
        "llvm" => Ok(NativeBackend::Llvm),
        _ => Err(error("invocation.cli.invalid", "expected backend c|llvm")),
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (result, status) = match execute(&args) {
        Ok(value) => (value, 0),
        Err(value) => (value, 1),
    };
    println!("{result}");
    std::process::exit(status);
}
