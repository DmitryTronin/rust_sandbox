use std::{
    env,
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use temp_1::analyze_clipboard_text;

const MAX_BODY_BYTES: usize = 1_048_576;
const ROOT_JSON: &str = "{\"service\":\"clipboard hidden character analyzer\",\"health\":\"GET /health\",\"analyze\":\"POST /analyze-clipboard\",\"ui\":\"GET /ui\"}";
const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Clipboard Hidden Character Analyzer</title>
  <style>
    :root {
      color-scheme: light;
      --background: #f7f7f4;
      --surface: #ffffff;
      --surface-muted: #eeeeea;
      --text: #222426;
      --muted: #61666d;
      --border: #d8d8d2;
      --accent: #0f766e;
      --accent-strong: #115e59;
      --danger: #b42318;
      --shadow: 0 16px 48px rgba(24, 28, 32, 0.12);
    }

    * {
      box-sizing: border-box;
    }

    body {
      margin: 0;
      min-height: 100vh;
      background: var(--background);
      color: var(--text);
      font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      line-height: 1.5;
    }

    main {
      width: min(1120px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 40px 0;
    }

    header {
      display: flex;
      align-items: end;
      justify-content: space-between;
      gap: 24px;
      margin-bottom: 24px;
    }

    h1 {
      margin: 0;
      font-size: clamp(1.75rem, 4vw, 3.25rem);
      line-height: 1.05;
      font-weight: 750;
      letter-spacing: 0;
    }

    .status {
      min-width: 150px;
      padding: 8px 12px;
      border: 1px solid var(--border);
      border-radius: 999px;
      background: var(--surface);
      color: var(--muted);
      font-size: 0.9rem;
      text-align: center;
      white-space: nowrap;
    }

    .workspace {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(340px, 0.9fr);
      gap: 20px;
      align-items: start;
    }

    .panel {
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--surface);
      box-shadow: var(--shadow);
      overflow: hidden;
    }

    .panel-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: 14px 16px;
      border-bottom: 1px solid var(--border);
      background: var(--surface-muted);
    }

    h2 {
      margin: 0;
      font-size: 1rem;
      font-weight: 700;
      letter-spacing: 0;
    }

    textarea {
      display: block;
      width: 100%;
      min-height: 430px;
      padding: 16px;
      border: 0;
      resize: vertical;
      color: var(--text);
      background: #fff;
      font: 1rem/1.55 ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
      outline: none;
    }

    .actions {
      display: flex;
      gap: 10px;
      padding: 14px 16px;
      border-top: 1px solid var(--border);
      background: var(--surface);
    }

    button {
      min-height: 42px;
      border: 1px solid transparent;
      border-radius: 6px;
      padding: 0 14px;
      font: inherit;
      font-weight: 700;
      cursor: pointer;
    }

    button.primary {
      background: var(--accent);
      color: #ffffff;
    }

    button.primary:hover {
      background: var(--accent-strong);
    }

    button.secondary {
      background: var(--surface);
      border-color: var(--border);
      color: var(--text);
    }

    button.secondary:hover {
      background: var(--surface-muted);
    }

    .summary {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
      padding: 16px;
    }

    .metric {
      min-height: 82px;
      padding: 12px;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: #fbfbf9;
    }

    .metric span {
      display: block;
      color: var(--muted);
      font-size: 0.82rem;
    }

    .metric strong {
      display: block;
      margin-top: 4px;
      font-size: 1.75rem;
      line-height: 1.1;
    }

    .results {
      border-top: 1px solid var(--border);
      overflow-x: auto;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      min-width: 720px;
      font-size: 0.92rem;
    }

    th,
    td {
      padding: 10px 12px;
      border-bottom: 1px solid var(--border);
      text-align: left;
      vertical-align: top;
    }

    th {
      background: #fbfbf9;
      color: var(--muted);
      font-size: 0.78rem;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }

    code {
      display: inline-block;
      padding: 2px 6px;
      border-radius: 5px;
      background: var(--surface-muted);
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
      font-size: 0.9em;
    }

    .empty,
    .error {
      padding: 20px 16px;
      color: var(--muted);
    }

    .error {
      color: var(--danger);
      font-weight: 700;
    }

    [hidden] {
      display: none !important;
    }

    @media (max-width: 860px) {
      main {
        width: min(100vw - 24px, 680px);
        padding: 24px 0;
      }

      header,
      .workspace {
        display: block;
      }

      .status {
        display: inline-block;
        margin-top: 14px;
      }

      .panel + .panel {
        margin-top: 16px;
      }

      textarea {
        min-height: 320px;
      }
    }
  </style>
</head>
<body>
  <main>
    <header>
      <h1>Clipboard Hidden Character Analyzer</h1>
      <div class="status" id="status">Ready</div>
    </header>

    <section class="workspace">
      <div class="panel">
        <div class="panel-header">
          <h2>Pasted Text</h2>
        </div>
        <textarea id="input" spellcheck="false" autofocus placeholder="Paste text here"></textarea>
        <div class="actions">
          <button class="primary" id="analyze" type="button">Analyze</button>
          <button class="secondary" id="clear" type="button">Clear</button>
        </div>
      </div>

      <div class="panel" aria-live="polite">
        <div class="panel-header">
          <h2>Results</h2>
        </div>
        <div class="summary" id="summary">
          <div class="metric"><span>Total chars</span><strong id="totalChars">0</strong></div>
          <div class="metric"><span>Total bytes</span><strong id="totalBytes">0</strong></div>
          <div class="metric"><span>Lines</span><strong id="lineCount">0</strong></div>
          <div class="metric"><span>Invisible</span><strong id="invisibleCount">0</strong></div>
        </div>
        <div class="empty" id="empty">Paste text and run an analysis.</div>
        <div class="error" id="error" hidden></div>
        <div class="results" id="results" hidden>
          <table>
            <thead>
              <tr>
                <th>Marker</th>
                <th>Code point</th>
                <th>Line</th>
                <th>Column</th>
                <th>Name</th>
                <th>Category</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody id="findings"></tbody>
          </table>
        </div>
      </div>
    </section>
  </main>

  <script>
    const input = document.querySelector("#input");
    const status = document.querySelector("#status");
    const empty = document.querySelector("#empty");
    const error = document.querySelector("#error");
    const results = document.querySelector("#results");
    const findings = document.querySelector("#findings");
    const fields = {
      totalChars: document.querySelector("#totalChars"),
      totalBytes: document.querySelector("#totalBytes"),
      lineCount: document.querySelector("#lineCount"),
      invisibleCount: document.querySelector("#invisibleCount")
    };

    document.querySelector("#analyze").addEventListener("click", analyze);
    document.querySelector("#clear").addEventListener("click", () => {
      input.value = "";
      resetAnalysis("Paste text and run an analysis.");
      status.textContent = "Ready";
      input.focus();
    });

    async function analyze() {
      status.textContent = "Analyzing";
      error.hidden = true;

      try {
        const response = await fetch("/analyze-clipboard", {
          method: "POST",
          headers: { "Content-Type": "text/plain; charset=utf-8" },
          body: input.value
        });
        const payload = await response.json();

        if (!response.ok) {
          throw new Error(payload.error || "Analysis failed.");
        }

        renderAnalysis(payload);
        status.textContent = "Complete";
      } catch (err) {
        renderError(err instanceof Error ? err.message : "Analysis failed.");
        status.textContent = "Error";
      }
    }

    function renderAnalysis(analysis) {
      fields.totalChars.textContent = analysis.total_chars;
      fields.totalBytes.textContent = analysis.total_bytes;
      fields.lineCount.textContent = analysis.line_count;
      fields.invisibleCount.textContent = analysis.invisible_count;
      findings.replaceChildren();

      if (analysis.findings.length === 0) {
        renderEmptyFindings("No hidden characters found.");
        return;
      }

      empty.hidden = true;
      error.hidden = true;
      results.hidden = false;

      for (const item of analysis.findings) {
        const row = document.createElement("tr");
        appendCell(row, item.marker, true);
        appendCell(row, item.code_point, true);
        appendCell(row, item.line);
        appendCell(row, item.column);
        appendCell(row, item.name);
        appendCell(row, item.category);
        appendCell(row, item.description);
        findings.append(row);
      }
    }

    function appendCell(row, value, code = false) {
      const cell = document.createElement("td");
      const node = code ? document.createElement("code") : document.createTextNode("");

      if (code) {
        node.textContent = value;
        cell.append(node);
      } else {
        cell.textContent = value;
      }

      row.append(cell);
    }

    function renderEmptyFindings(message) {
      findings.replaceChildren();
      empty.textContent = message;
      empty.hidden = false;
      error.hidden = true;
      results.hidden = true;
    }

    function resetAnalysis(message) {
      resetMetrics();
      renderEmptyFindings(message);
    }

    function resetMetrics() {
      fields.totalChars.textContent = "0";
      fields.totalBytes.textContent = "0";
      fields.lineCount.textContent = "0";
      fields.invisibleCount.textContent = "0";
    }

    function renderError(message) {
      resetMetrics();
      findings.replaceChildren();
      error.textContent = message;
      error.hidden = false;
      empty.hidden = true;
      results.hidden = true;
    }
  </script>
</body>
</html>"##;

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

    if method == "GET" && path == "/" {
        return http_response(200, "OK", "application/json; charset=utf-8", ROOT_JSON);
    }

    if method == "GET" && path == "/ui" {
        return http_response(200, "OK", "text/html; charset=utf-8", INDEX_HTML);
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
    fn test_build_response_serves_service_metadata_at_root() {
        let response = build_response(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Type: application/json; charset=utf-8"));
        assert!(response.contains("\"analyze\":\"POST /analyze-clipboard\""));
        assert!(response.contains("\"ui\":\"GET /ui\""));
        assert!(!response.contains("<!doctype html>"));
    }

    #[test]
    fn test_build_response_serves_browser_ui_at_ui_route() {
        let response = build_response(b"GET /ui HTTP/1.1\r\nHost: localhost\r\n\r\n");

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Type: text/html; charset=utf-8"));
        assert!(response.contains("Clipboard Hidden Character Analyzer"));
        assert!(response.contains("fetch(\"/analyze-clipboard\""));
        assert!(response.contains("renderEmptyFindings(\"No hidden characters found.\")"));
        assert!(response.contains("resetAnalysis(\"Paste text and run an analysis.\")"));
        assert!(response.contains("function resetMetrics()"));
        assert!(response.contains("function renderError(message) {\n      resetMetrics();"));
    }

    #[test]
    fn test_build_response_rejects_unknown_route() {
        let response = build_response(b"GET /unknown HTTP/1.1\r\nHost: localhost\r\n\r\n");

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
