use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;

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
    router.add_route("/blog/formula", "/blog/formula/test.html");

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

fn handle_request(mut stream: TcpStream, dir: &str, router: &Router) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);
    let mut path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/").to_string();

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
    println!("file_name: {}", file_name);
    println!("dir: {}", dir);
    let file_path = Path::new(dir).join(&file_name[1..]);
    println!("file_path: {}", file_path.to_str().unwrap());
    if file_path.exists() && file_path.is_file() {
        println!("Fetching {}", file_path.display());

        let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            _ => "application/octet-stream", // Default to binary data
        };

        let contents = fs::read(&file_path).unwrap_or_else(|_| Vec::new());

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
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
