use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

const FORMAT: &str = "lobo-test/0.1";
const MAX_REQUEST: usize = 4 * 1024 * 1024;

#[path = "../agent_testing/mod.rs"]
mod testing;
use testing::{Action, Session};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    format: String,
    action: Action,
}

fn error(code: &str, message: impl ToString) -> Value {
    let text = message.to_string();
    let preview = text.chars().take(1024).collect::<String>();
    json!({"status":"invalid_request","code":code,"message":preview,"message_truncated":preview.len() < text.len()})
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3
        || args[0] != "--session"
        || args[1] != "--clang"
        || !std::path::Path::new(&args[2]).is_absolute()
    {
        println!(
            "{}",
            json!({"format":FORMAT,"status":"help","usage":"lobo-test --session --clang /absolute/path/to/clang"})
        );
        return;
    }
    let mut session = Session::new(args[2].clone().into());
    let identity = format!(
        "{}:{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        match (&mut input)
            .take((MAX_REQUEST + 1) as u64)
            .read_until(b'\n', &mut line)
        {
            Ok(0) | Err(_) => break,
            _ => {}
        }
        let oversized = line.len() > MAX_REQUEST;
        let (mut result, close) = if oversized {
            (error("request.limit", "request exceeds 4 MiB"), true)
        } else {
            match serde_json::from_slice::<Request>(&line) {
                Ok(request) if request.format == FORMAT => {
                    let close = matches!(request.action, Action::Close {});
                    (session.apply(request.action), close)
                }
                Ok(_) => (error("request.version", "unsupported request format"), false),
                Err(e) => (error("request.invalid", e), false),
            }
        };
        result["format"] = json!(FORMAT);
        result["session"] = json!(identity);
        let mut encoded = result.to_string();
        if encoded.len() > 8 * 1024 * 1024 {
            encoded = json!({"format":FORMAT,"session":identity,"status":"infrastructure_error","code":"tool.response_limit","message":"detail exceeds 8 MiB response allowance"}).to_string();
        }
        if writeln!(output, "{encoded}")
            .and_then(|_| output.flush())
            .is_err()
            || close
        {
            break;
        }
    }
}
