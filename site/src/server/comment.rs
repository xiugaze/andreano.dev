use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::io;
use chrono::Datelike;
use rand::Rng;
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


pub async fn post_comment(
    input: CommentInput,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut challenges = match load_challenges() {
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


pub async fn get_challenge() -> Result<impl warp::Reply, warp::Rejection> {
    println!("got here");
    let mut challenges = match load_challenges() {
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

    save_challenges(&challenges);
    return Ok(warp::reply::with_status(
        warp::reply::json(&response_body),
        StatusCode::OK,
    ));
}

type Challenges = HashMap<String, (u32, u32)>;

fn load_challenges() -> io::Result<Challenges> {
    let path = Path::new("challenges.json");
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(HashMap::new())
    }
}

fn save_challenges(challenges: &Challenges) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("challenges.json")?;
    let json = serde_json::to_string_pretty(challenges)?;
    println!("writing {:?}", json);
    file.write_all(json.as_bytes())?;
    Ok(())
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

