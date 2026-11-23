use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

const HEADER_LIMIT: usize = 8192;
const IO_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, PartialEq)]
struct Request {
    method: String,
    path: String,
}

fn parse_request(bytes: &[u8]) -> Result<Option<Request>, u16> {
    if bytes.len() > HEADER_LIMIT {
        return Err(431);
    }
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = httparse::Request::new(&mut headers);
    match request.parse(bytes) {
        Ok(httparse::Status::Partial) => return Ok(None),
        Err(httparse::Error::TooManyHeaders) => return Err(431),
        Err(_) => return Err(400),
        Ok(httparse::Status::Complete(_)) => (),
    }
    if !matches!(request.version, Some(0 | 1)) {
        return Err(505);
    }
    let mut hosts = 0;
    let mut content_length = None;
    for header in request.headers.iter() {
        if header.name.eq_ignore_ascii_case("host") {
            hosts += 1;
            if header.value.is_empty() || header.value.iter().any(u8::is_ascii_whitespace) {
                return Err(400);
            }
        }
        if header.name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(400);
        }
        if header.name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some()
                || header.value.is_empty()
                || !header.value.iter().all(u8::is_ascii_digit)
            {
                return Err(400);
            }
            content_length = Some(
                std::str::from_utf8(header.value)
                    .map_err(|_| 400_u16)?
                    .parse::<u64>()
                    .map_err(|_| 400_u16)?,
            );
        }
    }
    if hosts > 1 || (request.version == Some(1) && hosts != 1) {
        return Err(400);
    }
    if content_length.unwrap_or(0) > 0 {
        return Err(413);
    }
    let target = request.path.ok_or(400_u16)?;
    if target.len() > 2048 {
        return Err(414);
    }
    if !target.starts_with('/') || target.contains('#') {
        return Err(400);
    }
    let encoded = target.split('?').next().ok_or(400_u16)?.as_bytes();
    let mut decoded = Vec::with_capacity(encoded.len());
    let mut n = 0;
    while n < encoded.len() {
        if encoded[n] == b'%' {
            let pair = encoded.get(n + 1..n + 3).ok_or(400_u16)?;
            let hex = std::str::from_utf8(pair).map_err(|_| 400_u16)?;
            decoded.push(u8::from_str_radix(hex, 16).map_err(|_| 400_u16)?);
            n += 3;
        } else {
            decoded.push(encoded[n]);
            n += 1;
        }
    }
    let path = String::from_utf8(decoded).map_err(|_| 400_u16)?;
    if path.chars().any(char::is_control) {
        return Err(400);
    }
    Ok(Some(Request {
        method: request.method.ok_or(400_u16)?.into(),
        path,
    }))
}

fn response_bytes(status: u16, content_type: &str, body: &str, head: bool) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Content Too Large",
        414 => "URI Too Long",
        431 => "Request Header Fields Too Large",
        505 => "HTTP Version Not Supported",
        _ => "Internal Server Error",
    };
    let allow = if status == 405 { "Allow: GET, HEAD\r\n" } else { "" };
    let date = httpdate::fmt_http_date(SystemTime::now());
    let mut response = format!("HTTP/1.1 {status} {reason}\r\nDate: {date}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n{allow}\r\n",body.len()).into_bytes();
    if !head {
        response.extend_from_slice(body.as_bytes());
    }
    response
}

fn read_request(stream: &mut TcpStream) -> Result<Request, u16> {
    let deadline = Instant::now() + IO_TIMEOUT;
    let mut bytes = Vec::new();
    loop {
        let remaining = deadline.checked_duration_since(Instant::now()).ok_or(408_u16)?;
        stream.set_read_timeout(Some(remaining)).map_err(|_| 400_u16)?;
        let mut buffer = [0; 1024];
        let count = stream.read(&mut buffer).map_err(|e| {
            if matches!(e.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) {
                408_u16
            } else {
                400_u16
            }
        })?;
        if count == 0 {
            return Err(400);
        }
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(request) = parse_request(&bytes)? {
            return Ok(request);
        }
    }
}

fn text_argument(value: &str) -> String {
    format!(
        "Text:{}",
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn native_response(handler: &Path, request: &Request) -> Result<(u16, String, String), String> {
    let result = Command::new(handler)
        .args([
            "--entry",
            "example.web.handle",
            "--arg",
            &text_argument(&request.method),
            "--arg",
            &text_argument(&request.path),
            "--fuel",
            "10000",
            "--heap-cells",
            "10000",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() || !result.stderr.is_empty() || result.stdout.len() > 32768 {
        return Err("native handler failed".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).map_err(|e| e.to_string())?;
    if value["status"] != "ok"
        || value["value"]["type"] != "Record"
        || value["value"]["value"]["id"] != "example.web.response"
    {
        return Err("invalid response envelope".into());
    }
    let fields = &value["value"]["value"]["fields"];
    if fields["status"]["type"] != "I64"
        || fields["content_type"]["type"] != "Text"
        || fields["body"]["type"] != "Text"
    {
        return Err("invalid response field types".into());
    }
    let status = fields["status"]["value"]
        .as_str()
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or("invalid status")?;
    let content_type = fields["content_type"]["value"]
        .as_str()
        .ok_or("invalid content type")?;
    let body = fields["body"]["value"].as_str().ok_or("invalid body")?;
    if !matches!(status, 200 | 404 | 405)
        || !matches!(content_type, "text/plain; charset=utf-8" | "application/json")
        || body.len() > 4096
    {
        return Err("response exceeds sample policy".into());
    }
    Ok((status, content_type.into(), body.into()))
}

fn serve(stream: &mut TcpStream, handler: &Path) -> io::Result<()> {
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    let (status, content_type, body, head) = match read_request(stream) {
        Ok(request) => {
            let head = request.method == "HEAD";
            match native_response(handler, &request) {
                Ok((status, content_type, body)) => (status, content_type, body, head),
                Err(error) => {
                    eprintln!("{}", serde_json::json!({"event":"handler_error","message":error}));
                    (
                        500,
                        "text/plain; charset=utf-8".into(),
                        "Handler failed\n".into(),
                        head,
                    )
                }
            }
        }
        Err(status) => (
            status,
            "text/plain; charset=utf-8".into(),
            "Request rejected\n".into(),
            false,
        ),
    };
    stream.write_all(&response_bytes(status, &content_type, &body, head))
}

fn main() -> io::Result<()> {
    let mut address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut handler = std::env::current_exe()?.with_file_name("handler");
    let mut maximum = 0_usize;
    let mut args = std::env::args().skip(1);
    while let Some(option) = args.next() {
        let value = args.next().ok_or_else(|| {
            io::Error::other("expected --listen ADDRESS, --handler FILE or --max-requests N")
        })?;
        match option.as_str() {
            "--listen" => address = value.parse().map_err(io::Error::other)?,
            "--handler" => handler = PathBuf::from(value),
            "--max-requests" => maximum = value.parse().map_err(io::Error::other)?,
            _ => return Err(io::Error::other("unknown option")),
        }
    }
    if !address.ip().is_loopback() {
        return Err(io::Error::other("sample server accepts loopback listeners only"));
    }
    let handler = handler.canonicalize()?;
    if !handler.is_file() {
        return Err(io::Error::other("handler is not a file"));
    }
    let listener = TcpListener::bind(address)?;
    println!(
        "{}",
        serde_json::json!({"event":"listening","address":listener.local_addr()?.to_string(),"handler":handler})
    );
    io::stdout().flush()?;
    for (index, stream) in listener.incoming().enumerate() {
        if let Err(error) = stream.and_then(|mut stream| serve(&mut stream, &handler)) {
            eprintln!(
                "{}",
                serde_json::json!({"event":"connection_error","message":error.to_string()})
            );
        }
        if maximum != 0 && index + 1 >= maximum {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_complete_requests_and_decodes_only_the_path() {
        assert_eq!(
            parse_request(b"GET /greet/%C3%A9?ignored=true HTTP/1.1\r\nHost: localhost\r\n\r\n"),
            Ok(Some(Request {
                method: "GET".into(),
                path: "/greet/é".into()
            }))
        );
        assert_eq!(parse_request(b"GET / HTTP/1.1\r\nHost: local"), Ok(None));
    }

    #[test]
    fn rejects_ambiguous_framing_unsupported_bodies_and_invalid_paths() {
        for request in [
            "GET / HTTP/1.1\r\n\r\n",
            "GET / HTTP/1.1\r\nHost: a\r\nHost: b\r\n\r\n",
            "GET / HTTP/1.1\r\nHost: a\r\nContent-Length: 0\r\nContent-Length: 1\r\n\r\n",
            "GET / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n",
            "GET /greet/%0a HTTP/1.1\r\nHost: a\r\n\r\n",
            "GET /greet/%ff HTTP/1.1\r\nHost: a\r\n\r\n",
            "GET /greet/%zz HTTP/1.1\r\nHost: a\r\n\r\n",
            "GET http://other/ HTTP/1.1\r\nHost: a\r\n\r\n",
        ] {
            assert_eq!(parse_request(request.as_bytes()), Err(400), "{request}");
        }
        assert_eq!(
            parse_request(b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1\r\n\r\nx"),
            Err(413)
        );
    }

    #[test]
    fn response_length_counts_utf8_bytes_and_head_suppresses_the_body() {
        let get = String::from_utf8(response_bytes(200, "text/plain; charset=utf-8", "é", false)).unwrap();
        assert!(get.contains("Content-Length: 2\r\n"));
        assert!(get.ends_with("\r\n\r\né"));
        let head = String::from_utf8(response_bytes(405, "text/plain; charset=utf-8", "é", true)).unwrap();
        assert!(head.contains("Content-Length: 2\r\n"));
        assert!(head.contains("Allow: GET, HEAD\r\n"));
        assert!(head.ends_with("\r\n\r\n"));
    }
}
