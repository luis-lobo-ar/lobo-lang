use lobo_core::{compile, Artifact, Value, LANGUAGE};
use serde_json::{json, Value as Json};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};

fn error(code: &str, message: impl ToString) -> Json {
    json!({"status":"error", "kind":"invocation_error", "diagnostic":{"code":code,"message":message.to_string()}})
}

fn load(path: &str) -> Result<Artifact, Json> {
    let file = fs::File::open(path).map_err(|e| error("invocation.file.read", e))?;
    let mut text = String::new();
    file.take(4_194_305)
        .read_to_string(&mut text)
        .map_err(|e| error("invocation.file.read", e))?;
    let result = if path.ends_with(".loboc") {
        Artifact::from_json(&text)
    } else {
        compile(&text)
    };
    result.map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))
}

fn execute(args: &[String]) -> Result<Json, Json> {
    if args.is_empty() || args[0] == "--help" || args[0] == "help" {
        return Ok(json!({"status":"ok", "language":LANGUAGE,"commands":[
            "lobo check FILE", "lobo compile FILE -o NEW_EXECUTABLE [--emit native|checked|c]",
            "lobo run FILE --entry FUNCTION_ID [--arg I64:42 | --arg Bool:true] [--fuel 100000]",
            "lobo eval FILE --entry FUNCTION_ID [--arg I64:42 | --arg Bool:true] [--fuel 100000] (reference evaluator)"
        ]}));
    }
    if args.len() < 2 {
        return Err(error("invocation.cli.invalid", "expected command and file"));
    }
    if !matches!(args[0].as_str(), "check" | "compile" | "run" | "eval") {
        return Err(error("invocation.cli.invalid", "unknown command"));
    }
    let artifact = load(&args[1])?;
    match args[0].as_str() {
        "check" => {
            if args.len() != 2 {
                return Err(error("invocation.cli.invalid", "check accepts exactly one file"));
            }
            Ok(json!({"status":"ok", "language":LANGUAGE,"entries":artifact.entries()}))
        }
        "compile" => {
            let mut output_path = None;
            let mut emit = None;
            let mut index = 2;
            while index < args.len() {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| error("invocation.cli.invalid", "missing option value"))?;
                match args[index].as_str() {
                    "-o" if output_path.is_none() => output_path = Some(value.as_str()),
                    "--emit" if emit.is_none() => emit = Some(value.as_str()),
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
            if emit == "native" {
                if output_path.ends_with(".loboc") {
                    return Err(error(
                        "invocation.cli.invalid",
                        ".loboc is reserved for --emit checked",
                    ));
                }
                artifact
                    .compile_native(std::path::Path::new(output_path))
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
                _ => {
                    return Err(error(
                        "invocation.cli.invalid",
                        "expected --emit native|checked|c; checked output needs .loboc extension",
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
            let result = artifact
                .run_native(&args[2..])
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
            let mut arguments = Vec::new();
            let mut i = 2;
            while i < args.len() {
                let option = &args[i];
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| error("invocation.cli.invalid", "missing option value"))?;
                match option.as_str() {
                    "--entry" if entry.is_none() => entry = Some(value.as_str()),
                    "--fuel" => {
                        fuel = value
                            .parse::<u64>()
                            .map_err(|e| error("invocation.cli.invalid", e))?
                    }
                    "--arg" => {
                        let (ty, text) = value
                            .split_once(':')
                            .ok_or_else(|| error("invocation.cli.invalid", "expected TYPE:VALUE"))?;
                        let value = match ty {
                            "I64" => Value::I64(
                                text.parse()
                                    .map_err(|e| error("invocation.arguments.invalid", e))?,
                            ),
                            "Bool" => Value::Bool(
                                text.parse()
                                    .map_err(|e| error("invocation.arguments.invalid", e))?,
                            ),
                            _ => return Err(error("invocation.arguments.invalid", "unknown argument type")),
                        };
                        arguments.push(value);
                    }
                    _ => return Err(error("invocation.cli.invalid", "unknown or repeated option")),
                }
                i += 2;
            }
            let entry = entry.ok_or_else(|| error("invocation.cli.invalid", "--entry is required"))?;
            artifact
                .run(entry, arguments, fuel)
                .map(|value| json!({"status":"ok", "kind":"value", "value":value}))
                .map_err(|e| json!({"status":"error", "kind":e.kind(), "diagnostic":e}))
        }
        _ => unreachable!(),
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
