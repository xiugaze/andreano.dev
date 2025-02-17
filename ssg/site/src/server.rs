use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

pub fn serve(dir: &str, port: &str) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    println!("listening at http://127.0.0.1:{}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = dir.to_string();
                std::thread::spawn(move || {
                    handle_request(stream, &dir);
                });
            }
            Err(e) => {
                eprintln!("failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_request(mut stream: TcpStream, dir: &str) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let file_path = Path::new(dir).join(&path[1..]);
    if file_path.exists() && file_path.is_file() {
        println!("Fetching {}", file_path.display());

        // Determine the Content-Type based on the file extension
        let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            _ => "application/octet-stream", // Default to binary data
        };

        // Read the file as binary data
        let contents = fs::read(&file_path).unwrap_or_else(|_| Vec::new());

        // Construct the HTTP response
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            content_type,
            contents.len()
        );

        // Send the headers and the file contents
        stream.write(response.as_bytes()).unwrap();
        stream.write(&contents).unwrap();
    } else {
        println!("Could not find {}", file_path.display());
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }
    stream.flush().unwrap();
}
