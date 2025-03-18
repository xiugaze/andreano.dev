use std::collections::HashMap;

use warp::Filter;

use super::comment::{get_comments, options_handler, post_comment};



pub async fn serve() {

    let file_server = warp::fs::dir("./");
    let get_route = warp::path("get-comments")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|query: HashMap<String, String>| async move {
            let post = query.get("post").cloned().unwrap_or_else(|| "test".to_string());
            get_comments(post).await
        });

    // POST /comments
    let post_route = warp::path("comments")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(post_comment);

    // OPTIONS /comments
    let options_route = warp::path("comments")
        .and(warp::options())
        .and_then(options_handler);


    warp::serve(
        file_server
            .or(get_route)
            .or(post_route)
            .or(options_route)
    ).run(([127, 0, 0, 1], 8080)).await;

}
