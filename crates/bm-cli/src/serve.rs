//! Development server: the session API over HTTP, plus the built interface as
//! static files, so the app can be run and tested in a plain browser.
//!
//! One match at a time, one request at a time, bound to the loopback interface.
//! The desktop app does not use this; it calls the same `bm_session::App` in
//! process.

use bm_session::App;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};

const MAX_BODY: usize = 1 << 20;

struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(stream: &TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?;
    let path = target.split('?').next().unwrap_or("/").to_string();
    let mut length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).ok()? == 0 {
            break;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().ok()?;
            }
        }
    }
    if length > MAX_BODY {
        return None;
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some(Request { method, path, body })
}

fn respond(mut stream: &TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// Resolve a request path inside the static directory; `None` if it tries to leave it.
fn static_path(root: &Path, request_path: &str) -> Option<PathBuf> {
    let rel = Path::new(request_path.trim_start_matches('/'));
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(root.join(rel))
}

fn handle(stream: &TcpStream, app: &mut App, static_dir: Option<&Path>) {
    let Some(req) = read_request(stream) else {
        respond(stream, "400 Bad Request", "text/plain", b"bad request");
        return;
    };
    if req.method == "OPTIONS" {
        respond(stream, "204 No Content", "text/plain", b"");
        return;
    }
    if let Some(cmd) = req.path.strip_prefix("/api/") {
        let payload = if req.body.is_empty() { Ok(Value::Null) } else { serde_json::from_slice::<Value>(&req.body) };
        let (status, body) = match payload {
            Err(e) => ("400 Bad Request", json!({ "code": "bad_request", "message": e.to_string() })),
            Ok(payload) => match app.call(cmd, payload) {
                Ok(value) => ("200 OK", value),
                Err(e) => ("400 Bad Request", serde_json::to_value(&e).unwrap_or(Value::Null)),
            },
        };
        respond(stream, status, "application/json", body.to_string().as_bytes());
        return;
    }
    let Some(root) = static_dir else {
        respond(stream, "404 Not Found", "text/plain", b"no static directory configured");
        return;
    };
    let wanted = if req.path == "/" { "/index.html" } else { req.path.as_str() };
    let file = static_path(root, wanted).filter(|p| p.is_file()).unwrap_or_else(|| root.join("index.html"));
    match std::fs::read(&file) {
        Ok(bytes) => respond(stream, "200 OK", mime(&file), &bytes),
        Err(_) => respond(stream, "404 Not Found", "text/plain", b"not found"),
    }
}

pub fn run(port: u16, static_dir: Option<PathBuf>) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("Baseball Manager development server on http://127.0.0.1:{port}");
    match &static_dir {
        Some(dir) => println!("serving the interface from {}", dir.display()),
        None => println!("API only (pass --static <dir> to serve the built interface)"),
    }
    let mut app = App::new();
    for stream in listener.incoming().flatten() {
        handle(&stream, &mut app, static_dir.as_deref());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_paths_stay_inside_the_root() {
        let root = Path::new("/srv/app");
        assert_eq!(static_path(root, "/assets/a.js"), Some(PathBuf::from("/srv/app/assets/a.js")));
        assert_eq!(static_path(root, "/../etc/passwd"), None);
        assert_eq!(static_path(root, "/assets/../../x"), None);
        assert_eq!(mime(Path::new("x/index.html")), "text/html; charset=utf-8");
    }
}
