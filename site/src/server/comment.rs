use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Arc;
use std::{net::TcpStream, path::Path};
use std::{io, sync::Mutex};

use serde::{Deserialize, Serialize};


pub async fn get_comments(
    post: String,
) -> Result<impl warp::Reply, warp::Rejection> {

    let comments = load_comments().unwrap_or_else(|_| HashMap::new());
    let empty = &vec![];
    let post_comments = comments.get(&post).unwrap_or(empty);
    
    Ok(warp::reply::with_header(
        warp::reply::json(post_comments),
        "Access-Control-Allow-Origin",
        "*",
    ))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    id: u32,
    author: String,
    content: String,
    timestamp: String,
}

#[derive(Deserialize)]
pub struct CommentInput {
    post: String,
    author: String,
    content: String,
}


#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
use warp::http::StatusCode;

pub async fn post_comment(
    input: CommentInput,
) -> Result<impl warp::Reply, warp::Rejection> {

    if input.author.is_empty() || input.content.is_empty() || input.post.is_empty() {
        let error = ErrorResponse {
            error: "Empty post, author, or content".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error),
            StatusCode::BAD_REQUEST,
        ));
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut next_id_map = get_last_ids().unwrap_or_else(|| HashMap::new());
    let next_id = next_id_map.entry(input.post.clone()).or_insert(0);
    let comment = Comment {
        id: *next_id,
        author: input.author.clone(),
        content: input.content.clone(),
        timestamp,
    };
    *next_id += 1;

    let mut comments_map = load_comments().unwrap_or_else(|_| HashMap::new());
    let post_comments = comments_map
        .entry(input.post.clone())
        .or_insert_with(Vec::new);
    post_comments.push(comment.clone());
    
    save_comments(&comments_map);
    return Ok(warp::reply::with_status(
        warp::reply::json(&comment),
        StatusCode::OK,
    ));
}

pub async fn options_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT))
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

