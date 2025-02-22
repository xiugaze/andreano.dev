use warp::Filter;
use std::net::SocketAddr;

pub async fn run_server() {
 let file_route = warp::get()
        .and(warp::fs::dir("output"))
        .with(warp::log("file_route"));

    let blog_formula_files = warp::path!("blog" / "formula" / ..)
        .and(warp::fs::dir("output/blog/formula"));

    let routes = file_route
        .or(blog_formula_files);

    let addr: SocketAddr = ([127, 0, 0, 1], 3030).into();

    println!("Server running at http://{:?}", addr);

    warp::serve(routes)
        .run(addr).await;
}
