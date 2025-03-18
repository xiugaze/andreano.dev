use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::{fs, io};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};







#[derive(Clone)]
struct Router {
    routes: HashMap<String, String>,
}

impl Router {
    fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    fn add_route(&mut self, path: &str, endpoint: &str) {
        self.routes.insert(path.to_string(), endpoint.to_string());
    }

    fn resolve(&self, path: &str) -> Option<&String> {
        self.routes.get(path)
    }
}

pub fn serve(dir: &str, port: &str) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    println!("listening at http://127.0.0.1:{}", port);

    let mut router = Router::new();

    router.add_route("/", "/index.html");
    router.add_route("/blog", "/blog/index.html");

    let routes: HashMap<String, String> = serde_json::from_str(&fs::read_to_string("routes.json").unwrap()).unwrap();
    for (k, v) in routes.iter() {
        router.add_route(k, v);
    }

    let router = Arc::new(router);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = dir.to_string();
                let router = Arc::clone(&router);
                std::thread::spawn(move || {
                    handle_request(stream, &dir, &router);
                });
            }
            Err(e) => {
                eprintln!("failed to accept connection: {}", e);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Comment {
    id: u32,
    author: String,
    content: String,
    timestamp: String,
}

#[derive(Deserialize)]
struct CommentInput {
    author: String,
    content: String,
}



struct AppState {
    comments: Mutex<Vec<Comment>>,
    next_id: Mutex<u32>,
}

fn handle_post_comment(stream: &mut std::net::TcpStream, request: &str, state: &AppState) -> io::Result<()> {
    let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
    println!("Raw request body: {:?}", body); // Debug: print the raw body
    
    let cors_headers = "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n";

    match serde_json::from_str::<CommentInput>(body) {
        Ok(input) if !input.author.is_empty() && !input.content.is_empty() => {
            println!("Parsed input: author={}, content={}", input.author, input.content); // Debug: confirm parsing
            let timestamp = chrono::Utc::now().to_rfc3339();
            let mut next_id = state.next_id.lock().unwrap();
            let comment = Comment {
                id: *next_id,
                author: input.author,
                content: input.content,
                timestamp,
            };
            *next_id += 1;

            let mut comments = state.comments.lock().unwrap();
            comments.push(comment.clone());
            save_comments(&comments)?;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{}\r\n{}",
                cors_headers,
                serde_json::to_string(&comment)?
            );
            stream.write_all(response.as_bytes())?;
        }
        Ok(input) => {
            println!("Empty fields: author={}, content={}", input.author, input.content); // Debug: log empty case
            let response = format!(
                "HTTP/1.1 400 BAD REQUEST\r\n{}Empty author or content",
                cors_headers
            );
            stream.write_all(response.as_bytes())?;
        }
        Err(e) => {
            println!("JSON parsing error: {}", e); // Debug: log the specific error
            let response = format!(
                "HTTP/1.1 400 BAD REQUEST\r\n{}Invalid JSON: {}",
                cors_headers,
                e
            );
            stream.write_all(response.as_bytes())?;
        }
    }
    Ok(())
}

fn handle_options(stream: &mut std::net::TcpStream) -> io::Result<()> {
    let response = "HTTP/1.1 204 NO CONTENT\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn load_comments() -> io::Result<Vec<Comment>> {
    let path = Path::new("comments.json");
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(Vec::new())
    }
}

fn save_comments(comments: &[Comment]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("comments.json")?;
    let json = serde_json::to_string_pretty(comments)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn get_last_id() -> Option<u32> {
    load_comments().ok().and_then(|comments| {
        comments.into_iter().map(|c| c.id).max()
    })
}

fn handle_request(mut stream: TcpStream, dir: &str, router: &Router) {
    let state = AppState {
        comments: Mutex::new(load_comments().unwrap_or_else(|_| Vec::new())),
        next_id: Mutex::new(get_last_id().unwrap_or(0) + 1),
    };

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string();

    println!("got request {:?}", request.lines().next());


    if request.starts_with("OPTIONS /comments") {
        let _ = handle_options(&mut stream);
        return;
    }

    if request.starts_with("POST /comments") {
        println!("got comment request...");
        let _ = handle_post_comment(&mut stream, &request, &state);
        return;
    }

    /* if path is an external link */
    if path.starts_with("http://") || path.starts_with("https://") {
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: {}\r\n\r\n",
            path
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    let file_name = router.resolve(&path).unwrap_or(&path);
    let file_path = Path::new(dir).join(&file_name[1..]);
    println!("got request for {:?}", file_path);
    if file_path.exists() && file_path.is_file() {
        println!("Fetching {}", file_path.display());

        let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("wasm") => "application/wasm",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("webm") => "video/webm",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("otf") => "font/otf",
            _ => "application/octet-stream", // Default to binary data
        };

        let contents = fs::read(&file_path).unwrap_or_else(|_| Vec::new());

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes'\r\n\r\n",
            content_type,
            contents.len()
        );

        stream.write(response.as_bytes()).unwrap();
        stream.write(&contents).unwrap();

    } else {
        println!("Could not find {}", file_path.display());
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }
    stream.flush().unwrap();
}
