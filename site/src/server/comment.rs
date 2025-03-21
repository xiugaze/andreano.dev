use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::io;
use std::sync::Arc;
use chrono::Datelike;
use rand::Rng;
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub async fn get_comments(
    post: String,
    db_dir: &str,
) -> Result<impl warp::Reply, warp::Rejection> {

    let path = Path::new(db_dir);
    let comments = load_comments(&path).unwrap_or_else(|_| HashMap::new());
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
    id: String, 
    sum: u32,
    post: String,
    author: String,
    content: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
use serde_json::json;
use warp::http::StatusCode;

use super::serve::get_db_connection;


pub async fn post_comment(
    input: CommentInput,
    db_dir: Arc<String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let db_path = Path::new(db_dir.as_str());
    let mut challenges = match load_challenges(&db_path) {
        Ok(c) => c,
        Err(_) => HashMap::new(),
    };


    let mut pass = false;
    if challenges.contains_key(&input.id) {
        let (p, q) = challenges.get(&input.id).unwrap();

        let year = chrono::Utc::now().year() as u32;
        println!("({:?} ** {}) mod {}", year, p, q);
        let solution = mod_exp::mod_exp(year, *p, *q);
        println!("got {:?} for solution {:?}", input.sum, solution);
        if input.sum  == solution {
            pass = true;
        }
        challenges.remove(&input.id);
    }

    if !pass {
        let error = ErrorResponse {
            error: "challenge failed, comment not posted".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error),
            StatusCode::BAD_REQUEST,
        ));
    }

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
    let mut next_id_map = get_last_ids(db_path).unwrap_or_else(|| HashMap::new());
    let next_id = next_id_map.entry(input.post.clone()).or_insert(0);
    let comment = Comment {
        id: *next_id,
        author: input.author.clone(),
        content: input.content.clone(),
        timestamp,
    };
    *next_id += 1;

    let mut comments_map = load_comments(&db_path).unwrap_or_else(|_| HashMap::new());
    let post_comments = comments_map
        .entry(input.post.clone())
        .or_insert_with(Vec::new);
    post_comments.push(comment.clone());
    
    save_comments(&comments_map, &db_path);
    return Ok(warp::reply::with_status(
        warp::reply::json(&comment),
        StatusCode::OK,
    ));
}


pub async fn get_challenge(db_dir: Arc<String>) -> Result<impl warp::Reply, warp::Rejection> {
    println!("got here");
    let db_path = Path::new(db_dir.as_str());
    let mut challenges = match load_challenges(&db_path) {
        Ok(c) => c,
        Err(_) => HashMap::new(),
    };
    
    let p = rand::thread_rng().gen_range(1..100);
    let q = rand::thread_rng().gen_range(1..100);
    let id = uuid::Uuid::new_v4().to_string();

    challenges.insert(id.clone(), (p, q));
    let response_body = serde_json::to_string(
        &json!({ "p": p, "q": q, "id": id })
        ).unwrap();

    save_challenges(&challenges, &db_path);
    return Ok(warp::reply::with_status(
        warp::reply::json(&response_body),
        StatusCode::OK,
    ));
}

type Challenges = HashMap<String, (u32, u32)>;

fn load_challenges(db_dir: &Path) -> io::Result<Challenges> {
    let mut challenges = HashMap::new();

    if db_dir.exists() {
        let conn = get_db_connection(db_dir).map_err(|e| io::Error::new(io::ErrorKind::Other, e)).unwrap();
        let mut stmt = conn.prepare("SELECT challenge_id, value1, value2 FROM challenges").unwrap();
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0).unwrap(), // challenge_id
                (row.get(1).unwrap(), row.get(2).unwrap()) // (value1, value2)
            ))
        }).unwrap();

        for row in rows {
            let (challenge_id, values): (String, (u32, u32)) = row.unwrap();
            challenges.insert(challenge_id, values);
        }
    }
    Ok(challenges)
}

fn save_challenges(challenges: &Challenges, db_path: &Path) -> io::Result<()> {
    let conn = get_db_connection(db_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e)).unwrap();
    conn.execute("DELETE FROM challenges", []).unwrap();

    for (challenge_id, &(value1, value2)) in challenges {
        conn.execute(
            "INSERT INTO challenges (challenge_id, value1, value2) VALUES (?1, ?2, ?3)",
            params![challenge_id, value1, value2],
        ).unwrap();
    }
    Ok(())
}

pub async fn options_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT))
}


fn load_comments(db_dir: &Path) -> io::Result<HashMap<String, Vec<Comment>>> {
    let mut comments = HashMap::new();

    if db_dir.exists() {
        let conn = get_db_connection(&db_dir).unwrap();
        let mut stmt = conn.prepare("SELECT post_id, comment FROM comments").unwrap();
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        }).unwrap();

        for row in rows {
            let (post_id, comment_json): (String, String) = row.unwrap();
            let comment: Comment = serde_json::from_str(&comment_json)?;
            comments.entry(post_id).or_insert_with(Vec::new).push(comment);
        }
    }
    Ok(comments)
}

fn save_comments(comments: &HashMap<String, Vec<Comment>>, db_path: &Path) -> io::Result<()> {
    let conn = get_db_connection(&db_path).unwrap();
    conn.execute("DELETE FROM comments", []).unwrap();

    for (post_id, comment_list) in comments {
        for comment in comment_list {
            let comment_json = serde_json::to_string(comment).unwrap();
            conn.execute(
                "INSERT INTO comments (post_id, comment) VALUES (?1, ?2)",
                &[post_id, &comment_json],
            ).unwrap();
        }
    }
    Ok(())
}

fn get_last_ids(db_path: &Path) -> Option<HashMap<String, u32>> {
    load_comments(db_path).ok().map(|comments| {
        comments.into_iter().map(|(post, comments)| {
            let max_id = comments.iter().map(|c| c.id).max().unwrap_or(0);
            (post, max_id + 1)
        }).collect()
    })
}

