use std::{
    env,
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use temp_1::analyze_clipboard_text;

const MAX_BODY_BYTES: usize = 1_048_576;

fn main() -> io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "7878".to_owned());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;

    println!("Listening on http://127.0.0.1:{port}");
    println!("POST pasted text to /analyze-clipboard");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(error) = handle_connection(stream) {
                        eprintln!("Failed to handle request: {error}");
                    }
                });
            }
            Err(error) => eprintln!("Failed to accept connection: {error}"),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let mut buffer = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];

    loop {
        let bytes_read = match stream.read(&mut chunk) {
            Ok(bytes_read) => bytes_read,
            Err(error) if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => {
                break;
            }
            Err(error) => return Err(error),
        };

        if bytes_read == 0 {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        if request_body_complete(&buffer) || buffer.len() > MAX_BODY_BYTES + 8192 {
            break;
        }
    }

    let response = build_response(&buffer);
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn request_body_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = find_header_end(buffer) else {
        return false;
    };
    let Some(body_start) = header_end.checked_add(4) else {
        return true;
    };

    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let Ok(content_length) = content_length(&headers) else {
        return true;
    };

    if content_length > MAX_BODY_BYTES {
        return true;
    }

    match body_start.checked_add(content_length) {
        Some(expected_length) => buffer.len() >= expected_length,
        None => true,
    }
}

fn build_response(request: &[u8]) -> String {
    let Some(header_end) = find_header_end(request) else {
        return http_response(
            400,
            "Bad Request",
            "text/plain; charset=utf-8",
            "Malformed request",
        );
    };

    let headers = String::from_utf8_lossy(&request[..header_end]);
    let mut request_line = headers
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    let method = request_line.next().unwrap_or_default();
    let path = request_line.next().unwrap_or_default();

    if method == "GET" && path == "/health" {
        return http_response(
            200,
            "OK",
            "application/json; charset=utf-8",
            "{\"status\":\"ok\"}",
        );
    }

    if method != "POST" || path != "/analyze-clipboard" {
        return http_response(
            404,
            "Not Found",
            "application/json; charset=utf-8",
            "{\"error\":\"Use POST /analyze-clipboard with the pasted text as the request body.\"}",
        );
    }

    let content_length =
        match content_length(&headers) {
            Ok(content_length) => content_length,
            Err(ContentLengthError::Missing) => return http_response(
                411,
                "Length Required",
                "application/json; charset=utf-8",
                "{\"error\":\"POST /analyze-clipboard requires a valid Content-Length header.\"}",
            ),
            Err(ContentLengthError::Invalid) => {
                return http_response(
                    400,
                    "Bad Request",
                    "application/json; charset=utf-8",
                    "{\"error\":\"Content-Length must be a valid non-negative integer.\"}",
                )
            }
        };

    if content_length > MAX_BODY_BYTES {
        return http_response(
            413,
            "Payload Too Large",
            "application/json; charset=utf-8",
            "{\"error\":\"Request body must be 1048576 bytes or smaller.\"}",
        );
    }

    let Some(body_start) = header_end.checked_add(4) else {
        return http_response(
            400,
            "Bad Request",
            "application/json; charset=utf-8",
            "{\"error\":\"Malformed request.\"}",
        );
    };
    let Some(expected_length) = body_start.checked_add(content_length) else {
        return http_response(
            413,
            "Payload Too Large",
            "application/json; charset=utf-8",
            "{\"error\":\"Request body must be 1048576 bytes or smaller.\"}",
        );
    };

    if request.len() < expected_length {
        return http_response(
            400,
            "Bad Request",
            "application/json; charset=utf-8",
            "{\"error\":\"Incomplete request body.\"}",
        );
    }

    let body = &request[body_start..expected_length];
    let pasted_text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(_) => {
            return http_response(
                400,
                "Bad Request",
                "application/json; charset=utf-8",
                "{\"error\":\"Request body must be valid UTF-8 text.\"}",
            )
        }
    };

    let analysis = analyze_clipboard_text(pasted_text).to_json();
    http_response(200, "OK", "application/json; charset=utf-8", &analysis)
}

fn find_header_end(request: &[u8]) -> Option<usize> {
    request.windows(4).position(|window| window == b"\r\n\r\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentLengthError {
    Missing,
    Invalid,
}

fn content_length(headers: &str) -> Result<usize, ContentLengthError> {
    let mut found = None;

    for line in headers.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };

        if name.eq_ignore_ascii_case("content-length") {
            if found.is_some() {
                return Err(ContentLengthError::Invalid);
            }

            found = Some(
                value
                    .trim()
                    .parse()
                    .map_err(|_| ContentLengthError::Invalid)?,
            );
        }
    }

    found.ok_or(ContentLengthError::Missing)
}

fn http_response(status: u16, reason: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_response_analyzes_post_body() {
        let response = build_response(
            b"POST /analyze-clipboard HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4\r\n\r\na\xc2\xa0b",
        );

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"name\":\"NO-BREAK SPACE\""));
    }

    #[test]
    fn test_build_response_rejects_unknown_route() {
        let response = build_response(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");

        assert!(response.starts_with("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn test_build_response_rejects_missing_content_length() {
        let response =
            build_response(b"POST /analyze-clipboard HTTP/1.1\r\nHost: localhost\r\n\r\n");

        assert!(response.starts_with("HTTP/1.1 411 Length Required"));
    }

    #[test]
    fn test_build_response_rejects_invalid_content_length() {
        let response = build_response(
            b"POST /analyze-clipboard HTTP/1.1\r\nHost: localhost\r\nContent-Length: nope\r\n\r\n",
        );

        assert!(response.starts_with("HTTP/1.1 400 Bad Request"));
    }

    #[test]
    fn test_oversized_content_length_completes_without_body() {
        let request = b"POST /analyze-clipboard HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1048577\r\n\r\n";

        assert!(request_body_complete(request));
        assert!(build_response(request).starts_with("HTTP/1.1 413 Payload Too Large"));
    }
}
