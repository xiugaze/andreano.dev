use pulldown_cmark::{Parser, Options};
use server::serve;
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

fn parse_post_markdown(input_path: &Path, output_path: &str) -> std::io::Result<()> {
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


    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    let template = fs::read_to_string("templates/post.html")?;

    let rendered = Template::new(template).unwrap().render(
        &Post {
            title: "test title",
            content: &html_content,
        },
    );

    let opts = FormatOptions::new()
        .tabs(false)
        .strip_comments(true)
        .wrap(80)
        .indent(2);
    let formatted = tidier::format(rendered, false, &opts).unwrap();

    fs::write(&output_path, formatted)?;
    Ok(())
}

fn traverse_directory(start_dir: &str) -> std::io::Result<()> {
    let start_dir = Path::new(start_dir).to_path_buf();
    let mut stack = VecDeque::new();
    stack.push_back(start_dir);

    while let Some(cur) = stack.pop_front() {
        if cur.is_dir() {
            for entry in fs::read_dir(cur)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                    if let Some(file_name) = path.file_stem() {
                        if let Some(file_name) = file_name.to_str() {
                            println!("found {}", file_name);
                            let _ = parse_post_markdown(path.as_path(), &format!("output/blog/{}.html", file_name));
                        }
                    }
                }


                if path.is_dir() {
                    stack.push_back(path);
                }
            }
        }
    }

    Ok(())
}


#[derive(Content)]
struct Post<'a> {
    title: &'a str,
    content: &'a str,
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cwd = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => {println!("Failed to get current directory: {}", e); exit(1) }
    };
    if args.len() > 1 && args[1] == "serve" {
        serve(&cwd, "9090");
    }
    traverse_directory("./blog")
}
