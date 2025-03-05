use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    if stream.read(&mut buffer).is_err() {
        return;
    }

    let request = String::from_utf8_lossy(&buffer);
    let request_line = request.lines().next().unwrap_or("");

    let (status, response_body) = match request_line.split_whitespace().collect::<Vec<&str>>()[..] {
        ["GET", "/fetch", ..] => handle_fetch(),
        ["POST", "/post", ..] => handle_post(&request),
        _ => (
            "HTTP/1.1 404 Not Found\r\n",
            "404 Not Found".to_string(),
        ),
    };

    let response = format!(
        "{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        response_body.len(),
        response_body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn handle_fetch() -> (&'static str, String) {
    let path = Path::new("comments.json");
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(contents) => ("HTTP/1.1 200 OK\r\n", contents),
            Err(_) => (
                "HTTP/1.1 500 Internal Server Error\r\n",
                "500 Internal Server Error".to_string(),
            ),
        }
    } else {
        // Return an empty array if file doesn’t exist
        ("HTTP/1.1 200 OK\r\n", "[]".to_string())
    }
}

fn handle_post(request: &str) -> (&'static str, String) {
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.trim())
        .unwrap_or("");

    if body.is_empty() {
        return (
            "HTTP/1.1 400 Bad Request\r\n",
            "400 Bad Request: No JSON data".to_string(),
        );
    }

    let path = Path::new("comments.json");
    let mut comments = if path.exists() {
        match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(_) => return (
                "HTTP/1.1 500 Internal Server Error\r\n",
                "500 Internal Server Error".to_string(),
            ),
        }
    } else {
        "[]".to_string()
    };

    let updated_comments = if comments == "[]" {
        format!("[{}]", body)
    } else {
        format!("{},{}]", &comments[..comments.len() - 1], body)
    };

    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
    {
        Ok(mut file) => {
            if file.write_all(updated_comments.as_bytes()).is_ok() {
                ("HTTP/1.1 200 OK\r\n", "{\"status\": \"Comment added\"}".to_string())
            } else {
                (
                    "HTTP/1.1 500 Internal Server Error\r\n",
                    "500 Internal Server Error".to_string(),
                )
            }
        }
        Err(_) => (
            "HTTP/1.1 500 Internal Server Error\r\n",
            "500 Internal Server Error".to_string(),
        ),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4000").expect("Failed to bind to address");
    println!("Comment engine running at http://127.0.0.1:4000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
