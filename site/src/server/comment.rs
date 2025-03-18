use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::{net::TcpStream, path::Path};
use std::{io, sync::Mutex};

use serde::{Deserialize, Serialize};

pub struct CommentState {
    comments: Mutex<HashMap<String, Vec<Comment>>>,
    next_id: Mutex<HashMap<String, u32>>,
}

impl CommentState {
    pub fn call_forth() -> Self {
        Self {
            comments: Mutex::new(load_comments().unwrap_or_else(|_| HashMap::new())),
            next_id: Mutex::new(get_last_ids().unwrap_or_else(|| HashMap::new())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    id: u32,
    author: String,
    content: String,
    timestamp: String,
}

#[derive(Deserialize)]
struct CommentInput {
    post: String,
    author: String,
    content: String,
}


pub fn handle_get_comments(stream: &mut TcpStream, request: &str, state: &CommentState) {
    let post = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|p| p.split('?').nth(1))
        .and_then(|q| q.split('=').nth(1))
        .unwrap_or("test");
    let comments = state.comments.lock().unwrap();
    let empty = &vec![];
    let post_comments = comments.get(post).unwrap_or(empty);
    let json = serde_json::to_string(post_comments).unwrap_or_else(|_| "[]".to_string());
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        json.len(),
        json
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn handle_post_comment(stream: &mut TcpStream, request: &str, state: &CommentState) -> io::Result<()> {
    let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
    println!("Raw request body: {:?}", body);

    let cors_headers = "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n";

    match serde_json::from_str::<CommentInput>(body) {
        Ok(input)
            if !input.author.is_empty() && !input.content.is_empty() && !input.post.is_empty() =>
        {
            println!(
                "Parsed input: post={}, author={}, content={}",
                input.post, input.author, input.content
            );
            let timestamp = chrono::Utc::now().to_rfc3339();
            let mut next_id_map = state.next_id.lock().unwrap();
            let next_id = next_id_map.entry(input.post.clone()).or_insert(0);
            let comment = Comment {
                id: *next_id,
                author: input.author,
                content: input.content,
                timestamp,
            };
            *next_id += 1;

            let mut comments_map = state.comments.lock().unwrap();
            let post_comments = comments_map
                .entry(input.post.clone())
                .or_insert_with(Vec::new);
            post_comments.push(comment.clone());
            save_comments(&comments_map)?;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{}\r\n{}",
                cors_headers,
                serde_json::to_string(&comment)?
            );
            stream.write_all(response.as_bytes())?;
        }
        Ok(input) => {
            println!(
                "Empty fields: post={}, author={}, content={}",
                input.post, input.author, input.content
            );
            let response = format!(
                "HTTP/1.1 400 BAD REQUEST\r\n{}Empty post, author, or content",
                cors_headers
            );
            stream.write_all(response.as_bytes())?;
        }
        Err(e) => {
            println!("JSON parsing error: {}", e);
            let response = format!(
                "HTTP/1.1 400 BAD REQUEST\r\n{}Invalid JSON: {}",
                cors_headers, e
            );
            stream.write_all(response.as_bytes())?;
        }
    }
    Ok(())
}

pub fn handle_options(stream: &mut std::net::TcpStream) -> io::Result<()> {
    let response = "HTTP/1.1 204 NO CONTENT\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn load_comments() -> io::Result<HashMap<String, Vec<Comment>>> {
    let path = Path::new("comments.json");
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(HashMap::new())
    }
}

fn save_comments(comments: &HashMap<String, Vec<Comment>>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("comments.json")?;
    let json = serde_json::to_string_pretty(comments)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn get_last_ids() -> Option<HashMap<String, u32>> {
    load_comments().ok().map(|comments| {
        comments.into_iter().map(|(post, comments)| {
            let max_id = comments.iter().map(|c| c.id).max().unwrap_or(0);
            (post, max_id + 1)
        }).collect()
    })
}

