use anyhow::{Context, Result, bail};
use printer_site::build;
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    time::Duration,
};

fn serve(output: &Path, port: u16) -> Result<()> {
    let root = output.canonicalize()?;
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("Preview: http://127.0.0.1:{port} (Ctrl-C to stop)");
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        while request.len() < 8192 && !request.windows(4).any(|w| w == b"\r\n\r\n") {
            let Ok(count) = stream.read(&mut buffer) else {
                break;
            };
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
        }
        let request = String::from_utf8_lossy(&request);
        let mut words = request.lines().next().unwrap_or("").split_whitespace();
        let method = words.next().unwrap_or("");
        let path = words.next().unwrap_or("/").split('?').next().unwrap_or("/");
        let path = if path == "/" { "/index.html" } else { path };
        let file = root
            .join(path.trim_start_matches('/'))
            .canonicalize()
            .ok()
            .filter(|p| p.starts_with(&root) && p.is_file());
        let (status, content_type, body) = if !matches!(method, "GET" | "HEAD") {
            (
                "405 Method Not Allowed",
                "text/plain",
                b"GET or HEAD required".to_vec(),
            )
        } else if let Some(file) = file {
            let mime = match file.extension().and_then(|v| v.to_str()).unwrap_or("") {
                "html" => "text/html; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "js" => "text/javascript; charset=utf-8",
                "svg" => "image/svg+xml",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                "woff2" => "font/woff2",
                _ => "application/octet-stream",
            };
            ("200 OK", mime, std::fs::read(file)?)
        } else {
            ("404 Not Found", "text/plain", b"Not found".to_vec())
        };
        let header = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        if stream.write_all(header.as_bytes()).is_ok() && method != "HEAD" {
            let _ = stream.write_all(&body);
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    let mut source = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut output = None;
    let mut port = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => source = args.next().context("--source needs a directory")?.into(),
            "--output" => {
                output = Some(PathBuf::from(
                    args.next().context("--output needs a directory")?,
                ))
            }
            "--serve" => {
                port = Some(
                    args.next()
                        .context("--serve needs a port")?
                        .parse::<u16>()?,
                )
            }
            "--help" | "-h" => {
                println!(
                    "Build the printer page from TOML and Markdown.\nUsage: cargo run -- [--source DIRECTORY] [--output DIRECTORY] [--serve PORT]"
                );
                return Ok(());
            }
            _ => bail!("Unknown argument: {arg}"),
        }
    }
    let output = output.unwrap_or_else(|| source.join("_site"));
    let count = build(&source, &output)?;
    println!("Built {count} projects → {}", output.display());
    if let Some(port) = port {
        serve(&output, port)?;
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Build failed: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
