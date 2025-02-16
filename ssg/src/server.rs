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
        let contents = fs::read_to_string(&file_path).unwrap_or_else(|_| "not found".to_string());
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );
        stream.write(response.as_bytes()).unwrap();
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }
    stream.flush().unwrap();
}
