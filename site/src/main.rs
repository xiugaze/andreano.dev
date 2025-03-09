use pulldown_cmark::{Event, Options, Parser, Tag};
use ramhorns::{Content, Ramhorns, Template};
use serde_yaml::Value;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::exit;
use tidier::FormatOptions;

mod preprocessor;
mod server;

use std::collections::{HashMap, VecDeque};
use std::{env, io};

fn parse_frontmatter_content(
    file_path: &Path,
) -> io::Result<(Option<HashMap<String, Value>>, String)> {
    let content = fs::read_to_string(file_path)?;

    if content.starts_with("---\n") {
        let mut parts = content.splitn(3, "---\n");
        parts.next(); 

        if let (Some(frontmatter), Some(markdown)) = (parts.next(), parts.next()) {
            let frontmatter_map: HashMap<String, Value> = serde_yaml::from_str(frontmatter)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            return Ok((Some(frontmatter_map), markdown.trim().to_string()));
        }
    }

    Ok((None, content.trim().to_string()))
}

#[derive(Content)]
struct BaseTemplate<'a> {
    title: &'a str,
    path: &'a str,
    content: &'a str,
    styles: &'a str,
    scripts: &'a str,
    commit: &'a str,
}

struct Post {
    title: String,
    url: String,
    date: chrono::DateTime<chrono::FixedOffset>,
}

fn parse_post_markdown(input_path: &Path, output_path: &Path, commit: &str) -> std::io::Result<Post> {

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

    let mut has_code = false;
    let parser = preprocessor::Preprocessor::new(Parser::new_ext(&content, options)).map(|e| {
        if let Event::Start(tag) = &e {
            if let Tag::CodeBlock(_) = tag {
                has_code = true;
            }
        }
        e
    });

    /* convert markdown to html */
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    /* render template */
    let tpls: Ramhorns = Ramhorns::from_folder("./templates").unwrap();
    let template = tpls.get("post.html").unwrap();

    let parent = input_path.parent().unwrap().display();

    let frontmatter = frontmatter.unwrap_or(HashMap::new());


    let title = match frontmatter.get("title") {
        Some(Value::String(title)) => title,
        _ => "default title",
    };

    let date = match frontmatter.get("date") {
        Some(Value::String(date)) => match chrono::DateTime::parse_from_rfc3339(date) {
            Ok(dt) => dt,
            Err(e) => {
                println!("error parsing {}", e);
                chrono::DateTime::default()
            }
        },
        _ => chrono::DateTime::default(),
    };

    let fm_styles = match frontmatter.get("styles") {
        Some(Value::Sequence(sty)) => {
            let styles_vec = sty.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
            styles_vec
        },
        _ => Vec::new(),
    };

    let mut styles = String::new();
    let mut scripts = String::new();

    if has_code {
        styles.push_str("<link href=\"/styles/prism.css\" rel=\"stylesheet\">");
        scripts.push_str("<script src=\"/scripts/prism.js\"></script>");
    }

    for s in fm_styles {
        styles.push_str(&format!("<link href=\"/styles/{}\" rel=\"stylesheet\">", s));
    }

    let rel_path: Vec<_> = output_path.components().collect();
    let rel_path2: PathBuf = rel_path[1..].iter().collect();
    let mut rel_path_parent = rel_path2.parent().unwrap().to_str().unwrap().to_string();
    if !rel_path_parent.is_empty() {
        rel_path_parent.push('/');
    }

    println!("output path parent: {}", output_path.parent().unwrap().to_str().unwrap());
    let rendered = template.render(&BaseTemplate {
        title,
        path: &format!("/{}", rel_path_parent),
        content: &html_content,
        styles: &styles,
        scripts: &scripts,
        commit,
    });

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


    if let Some(r) = rel_path_parent.strip_suffix("/") {
        rel_path_parent = r.to_string();
    }

    /* metadata */
    let post = Post {
        title: title.to_string(),
        url: format!("/{}", rel_path_parent),
        date: date.fixed_offset(),
    };
    Ok(post)
}

use std::process::Command;
use std::str;

fn get_git_commit_hash() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if output.status.success() {
        let hash = str::from_utf8(&output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in git output: {}", e))?
            .trim()
            .to_string();
        Ok(hash)
    } else {
        let error = str::from_utf8(&output.stderr)
            .map_err(|e| format!("Invalid UTF-8 in git error: {}", e))?
            .trim()
            .to_string();
        Err(format!("Git command failed: {}", error))
    }
}

fn copy_traverse(input: &Path, output: &Path) -> io::Result<()> {
    if !input.is_dir() {
        println!("error: input is not a directory");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input is not a directory",
        ));
    }

    if !output.exists() {
        fs::create_dir_all(output)?;
    }

    let commit = &get_git_commit_hash().unwrap()[0..6];

    let mut todo = VecDeque::new();
    todo.push_back((input.to_path_buf(), output.to_path_buf()));

    let mut posts: Vec<Post> = Vec::new();

    while let Some((cur_in, cur_out)) = todo.pop_front() {
        for entry in fs::read_dir(&cur_in)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let mut new_path = cur_out.join(file_name);

            if path.is_dir() {
                println!("Found directory: {:?}", path);
                fs::create_dir_all(&new_path)?;
                todo.push_back((path, new_path)); 
            } else if path.is_file() {
                println!("Found file: {:?}", path);
                if let Some(ext) = path.extension() {
                    match ext.to_str().unwrap() {
                        "md" => { 
                            println!("{:?} is markdown", path); 
                            new_path.set_extension("html");
                            let Ok(post) = parse_post_markdown(&path, &new_path, &commit) else {
                                println!("error parsing post {:?}", &path);
                                break;
                            };
                            let Some(str) = path.to_str() else {
                                println!("error parsing path {:?} to string", &path);
                                break;
                            };
                            if str.contains("blog") {
                                posts.push(post);
                            }
                        },
                        _ => { fs::copy(&path, &new_path)?; },
                    }
                }
            }
        }
    }

    let mut blog_index_content = String::new();
    posts.sort_by(|i, j| (&j.date).cmp(&i.date)); // reverse comparison
    blog_index_content.push_str("<h1>blog</h1>");
    for p in posts {
        let date_str = p.date.format("%Y-%m-%d").to_string();
        blog_index_content.push_str(&format!(
            "<a href=\"{}\">&lt;{}&gt; {}</a>\n",
            p.url, date_str, p.title
        ));
    }

    let template = fs::read_to_string("templates/index.html")?;
    let rendered = Template::new(template).unwrap().render(&BaseTemplate {
        title: "blog",
        path: "/blog/",
        content: &blog_index_content,
        scripts: "",
        styles: "",
        commit: &commit,
    });

    fs::write("static/blog/index.html", rendered)?;

    Ok(())
}




fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cwd = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => {
            println!("Failed to get current directory: {}", e);
            exit(1)
        }
    };
    if args.len() > 1 && args[1] == "serve" {
        server::serve(&cwd, "8080");
    }

    let blog = PathBuf::from("input");
    let blog2 = PathBuf::from("static");
    copy_traverse(&blog, &blog2)
    //println!("Deploying from commit {}", &hash[0..6]);
    //render_blog("blog")
}
