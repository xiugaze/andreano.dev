use std::{collections::HashMap, convert::Infallible, fs, path::Path, sync::Arc};

use rusqlite::Connection;
use warp::Filter;

use super::comment::{get_challenge, get_comments, options_handler, post_comment, CommentInput};

pub fn get_db_connection(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    Connection::open(db_path)
}

pub fn init_db(db_path: &Path) -> Result<(), rusqlite::Error> {
    let conn = match get_db_connection(&db_path) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id TEXT NOT NULL,
            comment TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE challenges (
            challenge_id TEXT PRIMARY KEY,
            value1 INTEGER NOT NULL,
            value2 INTEGER NOT NULL
        );",
        [],
    )?;
    Ok(())
}

pub async fn serve(dir: &Path, db_dir: String) {
    let db_path = Path::new(&db_dir);
    if !db_path.exists() {
        fs::create_dir_all(&db_path.parent().unwrap());
        init_db(&db_path);
    }

    let file_server = warp::fs::dir(dir.to_path_buf());
    let four_oh_four = warp::fs::file(dir.join("404/index.html"));


    let arc = Arc::new(db_dir);

    // GET /comments
    let get_route = warp::path("comments")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::any().map({
            let string: Arc<String> = Arc::clone(&arc);
            move || Arc::clone(&string)
        }))
        .and_then(
            |query: HashMap<String, String>, db_dir: Arc<String>| async move {
                let post = query
                    .get("post")
                    .cloned()
                    .unwrap_or_else(|| "test".to_string());
                get_comments(post, &db_dir).await
            },
        );
    let challenge_route = warp::path("challenge")
        .and(warp::get())
        .and(warp::any().map({
            let string: Arc<String> = Arc::clone(&arc);
            move || Arc::clone(&string)
        }))
        .and_then(|db_dir: Arc<String> | {
            get_challenge(db_dir)
        });

    // POST /comments
    let post_route = warp::path("comments")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map({ move || Arc::clone(&arc) }))
        .and_then(|input: CommentInput, db_dir: Arc<String>| {
            let db_dir = Arc::clone(&db_dir);
            post_comment(input, db_dir)
        });

    // OPTIONS /comments
    let options_route = warp::path("comments")
        .and(warp::options())
        .and_then(options_handler);

    warp::serve(
        file_server
            .or(challenge_route)
            .or(get_route)
            .or(post_route)
            .or(options_route)
            .or(four_oh_four),
    )
    .run(([127, 0, 0, 1], 8080))
    .await;
}
