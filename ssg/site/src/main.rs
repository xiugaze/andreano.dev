use pulldown_cmark::{Parser, Options};
use tidier::FormatOptions;
use std::fs::{self};
use std::path::Path;
use std::process::exit;
use ramhorns::{Template, Content};

mod preprocessor;
mod server;

use std::collections::{HashMap, VecDeque};
use std::{env, io};

fn parse_frontmatter_content(file_path: &Path) -> io::Result<(Option<HashMap<String, String>>, String)> {
    let content = fs::read_to_string(file_path)?;
    if content.starts_with("---\n") {
        let mut parts = content.splitn(3, "---\n");
        parts.next();
        if let (Some(frontmatter), Some(markdown)) = (parts.next(), parts.next()) {
            let frontmatter_map: HashMap<String, String> = serde_yaml::from_str(frontmatter)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            return Ok((Some(frontmatter_map), markdown.trim().to_string()));
        }
    }
    Ok((None, content.trim().to_string()))
}


#[derive(Content)]
struct PostTemplate<'a> {
    title: &'a str,
    path: &'a str,
    content: &'a str,
}

struct Post {
    title: String,
    url: String,
    date: chrono::DateTime<chrono::FixedOffset>
}

fn parse_post_markdown(input_path: &Path, output_path: &str) -> std::io::Result<Post> {

    let (frontmatter, content) = parse_frontmatter_content(&input_path)?;

    let options = Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_MATH
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_SMART_PUNCTUATION;

    let parser = preprocessor::Preprocessor::new(Parser::new_ext(&content, options));

    /* convert markdown to html */
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    /* render template */
    let template = fs::read_to_string("templates/post.html")?;
    let parent = input_path.parent().unwrap().display();

    let frontmatter = frontmatter.unwrap();
    let title = match frontmatter.get("title") {
        Some(title) => title,
        None => "default title",
    };

    let date = match frontmatter.get("date") {
        Some(date) => {
            match chrono::DateTime::parse_from_rfc3339(date) {
                Ok(dt) => dt,
                Err(e) => { 
                    println!("error parsing {}", e); 
                    chrono::DateTime::default()
                },
            }
        },
        None => chrono::DateTime::default(),
    };

    let rendered = Template::new(template).unwrap().render(
        &PostTemplate {
            title,
            path: &format!("/{}/", parent.to_string()),
            content: &html_content,
        },
    );

    let post = Post {
        title: title.to_string(),
        url: format!("/{}", parent.to_string()),
        date: date.fixed_offset(),
    };

    /* format the output html */
    let opts = FormatOptions::new()
        .tabs(false)
        .strip_comments(true)
        .wrap(80)
        .indent(2);
    let formatted = tidier::format(rendered, false, &opts).unwrap();

    let output_path = Path::new(output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, formatted)?;

    Ok(post)
}

fn traverse_directory(start_dir: &str) -> std::io::Result<()> {
    let start_dir = Path::new(start_dir).to_path_buf();
    let mut stack = VecDeque::new();
    stack.push_back(start_dir);

    let mut posts: Vec<Post> = Vec::new();

    while let Some(cur) = stack.pop_front() {
        if cur.is_dir() {
            for entry in fs::read_dir(cur)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                    if let (Some(parent), Some(file_name)) = (path.parent().and_then(|p| p.file_stem()), path.file_stem()) {
                        if let (Some(parent), Some(file_name)) = (parent.to_str(), file_name.to_str()) {
                            println!("found {}/{}", parent, file_name);
                            match parse_post_markdown(path.as_path(), &format!("output/blog/{}/{}.html", parent, file_name)) {
                                Ok(post) => {
                                    println!("wrote output/blog/{}/{}.html", parent, file_name);
                                    posts.push(post);
                                }
                                Err(e) => println!("error: {}", e),
                            };
                        }
                    }
                }

                if path.is_dir() {
                    stack.push_back(path);
                }
            }
        }
    }

    let mut index_html = String::new();
    posts.sort_by(|i, j| (&i.date).cmp(&j.date));
    for p in posts {
        let date_str = p.date.format("%Y-%m-%d").to_string();
        println!("<{}> {} href={}", date_str, p.title, p.url);
        index_html.push_str(&format!("<a href=\"{}\">&lt{}&gt {}</a>\n", p.url, date_str, p.title));
    }
    fs::write("output/blog/index.html", index_html)?;

    Ok(())
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cwd = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => {println!("Failed to get current directory: {}", e); exit(1) }
    };
    if args.len() > 1 && args[1] == "serve" {
        server::serve(&cwd, "8080");
    } 
    traverse_directory("blog")
}
