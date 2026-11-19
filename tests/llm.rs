//! Contract tests for the OpenAI-compatible client, driven by a loopback server.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use target::llm::LlmClient;
use target::message::Message;

/// Start a one-shot HTTP server that replies with `body` and returns the raw
/// request bytes to the caller.
fn spawn_mock_server(status: u16, body: &'static str) -> (String, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("local address");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        let request = read_request(&mut stream);

        let reason = if status == 200 {
            "OK"
        } else {
            "Internal Server Error"
        };
        let response = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        stream.flush().expect("flush response");

        request
    });

    (format!("http://{address}"), handle)
}

/// Read one HTTP request, including a body delimited by `Content-Length`.
fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut data = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let read = stream.read(&mut buffer).expect("read request");
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..read]);

        if let Some(header_end) = find_header_end(&data) {
            let headers = String::from_utf8_lossy(&data[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    if name.eq_ignore_ascii_case("content-length") {
                        value.trim().parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            if data.len() >= header_end + 4 + content_length {
                break;
            }
        }
    }

    data
}

/// Byte index of the `\r\n\r\n` separator, if present.
fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

#[tokio::test]
async fn parses_tool_calls_and_sends_authentication() {
    let body = r#"{
        "choices": [{
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "list_files", "arguments": "{\"path\":\".\"}"}
                }]
            }
        }]
    }"#;
    let (base_url, server) = spawn_mock_server(200, body);

    let client = LlmClient::new(
        format!("{base_url}/v1"),
        "test-model",
        Some("secret-key".to_string()),
    )
    .expect("build client");

    let messages = vec![Message::system("system prompt"), Message::user("hello")];
    let turn = client
        .complete(&messages, serde_json::json!([]), 0.2)
        .await
        .expect("complete request");

    let calls = turn.tool_calls.expect("tool calls");
    assert_eq!(calls[0]["id"], "call_1");
    assert_eq!(calls[0]["function"]["name"], "list_files");
    assert!(turn.content.is_none());

    let request = server.join().expect("server thread");
    let text = String::from_utf8_lossy(&request);
    assert!(text.contains("authorization: Bearer secret-key"));
    assert!(text.contains("test-model"));
    assert!(text.contains("system prompt"));
    assert!(text.contains("\"tool_choice\":\"auto\""));
}

#[tokio::test]
async fn returns_final_text_without_tool_calls() {
    let body = r#"{"choices":[{"message":{"role":"assistant","content":"done"}}]}"#;
    let (base_url, _server) = spawn_mock_server(200, body);

    let client = LlmClient::new(base_url, "test-model", None).expect("build client");
    assert!(!client.has_api_key());

    let turn = client
        .complete(&[], serde_json::json!([]), 0.0)
        .await
        .expect("complete request");

    assert_eq!(
        turn.content.as_ref().and_then(|value| value.as_str()),
        Some("done")
    );
    assert!(turn.tool_calls.is_none());
}

#[tokio::test]
async fn surfaces_http_errors() {
    let (base_url, _server) = spawn_mock_server(500, "upstream exploded");

    let client = LlmClient::new(base_url, "test-model", None).expect("build client");
    let error = client
        .complete(&[], serde_json::json!([]), 0.0)
        .await
        .expect_err("request should fail")
        .to_string();

    assert!(error.contains("HTTP 500"));
    assert!(error.contains("upstream exploded"));
}
