use image::{image_dimensions, ImageFormat};
use image::io::Reader as ImageReader;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};
use ramhorns::{Content, Ramhorns, Template};
use serde_yaml::Value;

use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::{env, io, str};

mod preprocessor;
mod server;

struct Post {
    title: String,
    content: String,
    draft: bool,
    url: String,
    date: chrono::DateTime<chrono::FixedOffset>,
    styles: String,
    scripts: String,
}

impl Post {
    fn from_path(file_path: &Path) -> io::Result<Post> {
        let content = fs::read_to_string(file_path)?;

        if content.starts_with("---\n") {
            let mut parts = content.splitn(3, "---\n");
            parts.next();

            if let (Some(frontmatter), Some(body)) = (parts.next(), parts.next()) {
                let frontmatter: HashMap<String, Value> = serde_yaml::from_str(frontmatter)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                let title = match frontmatter.get("title") {
                    Some(Value::String(title)) => title,
                    _ => "no title",
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

                let draft = match frontmatter.get("draft") {
                    Some(Value::Bool(d)) => *d,
                    _ => false,
                };

                let styles = match frontmatter.get("styles") {
                    Some(Value::Sequence(sty)) => {
                        let styles_vec = sty
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        styles_vec
                    }
                    _ => Vec::new(),
                };

                let mut post = Post {
                    title: title.to_string(),
                    content: body.to_string(),
                    draft,
                    url: String::from(""),
                    date,
                    styles: String::new(),
                    scripts: String::new(),
                };

                for s in styles {
                    post.add_style(&format!("styles/{}", s));
                }

                return Ok(post);
            }
        }

        Ok(Post {
            title: "default title".to_string(),
            content,
            draft: false,
            url: "".to_string(),
            date: chrono::DateTime::default(),
            styles: String::new(),
            scripts: String::new(),
        })
    }

    fn add_style(&mut self, style: &str) {
        self.styles
            .push_str(&format!("<link href=\"{}\" rel=\"stylesheet\">\n", style));
    }

    fn add_script(&mut self, script: &str) {
        self.scripts
            .push_str(&format!("<script src=\"{}\"></script>\n", script));
    }
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

struct PostMetadata {
    title: String,
    url: String,
    date: chrono::DateTime<chrono::FixedOffset>,
    draft: bool,
}

struct HtmlContent {
    content: String,
    has_code: bool,
    has_math: bool,
}

fn chew(content: &mut String, path: &Path) -> HtmlContent {
    let options = Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_MATH
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_SMART_PUNCTUATION;

    let mut parser = Parser::new_ext(&content, options).peekable();
    let mut has_code = false;
    let mut has_math = false;

    let mut events = Vec::new();

    while let Some(event) = parser.next() {
        let e = match event {
            Event::Start(Tag::Image {
                link_type,
                mut dest_url,
                title,
                id,
            }) => {
                let mut alttext = String::new();
                let parent = path.parent().unwrap();
                let image_path = match dest_url.strip_prefix("./") {
                    Some(p) => p,
                    None => &dest_url,
                };
                let img_path = parent.join(image_path);
                let mut img_path = Path::new("./").join(img_path);
                img_path.set_extension("webp");
                let dimensions = match image_dimensions(&img_path) {
                    Ok(dim) => dim,
                    Err(_) => (800, 400),
                };
                println!("{:?}: {:?}", img_path, dimensions);

                
                if let Some(Event::Text(alt)) = parser.next() {
                    alttext.push_str(&alt);
                }
                let mut html = String::new();
                html.push_str("<figure>\n");
                html.push_str(format!("<a href=\"{}\">\n", dest_url).as_str());
                html.push_str("<picture>\n");

                if dest_url.ends_with(".jpg") | dest_url.ends_with(".png") | dest_url.ends_with(".jpeg") {
                    let dest_str = dest_url.to_string();
                    let extension = Path::new(&dest_str).extension().and_then(|ext| ext.to_str()).unwrap();
                    html.push_str(
                        format!(
                            "
                                <source srcset=\"{}\" type=\"image/{}\">
                            ",
                            dest_url.replace(extension, "webp"),
                            extension
                        )
                        .as_str(),
                    );
                }

                html.push_str(
                    format!(
                        "
                            <img loading=\"lazy\" width=\"{}\" height=\"{}\" src=\"{}\" alt=\"{}\" title=\"{}\">
                        ",
                        dimensions.0, dimensions.1, dest_url, alttext, title
                    )
                    .as_str(),
                );

                html.push_str("</picture>\n");
                html.push_str("</a>\n");
                html.push_str(format!("<figcaption>{}</figcaption>", title).as_str());
                html.push_str("</figure>\n");
                Event::Html(html.into())
            }

            Event::Start(tag) => {
                if let Tag::CodeBlock(_) = tag {
                    has_code = true;
                }
                Event::Start(tag)
            }
            Event::DisplayMath(c) => {
                let text: CowStr<'_> =
                    latex2mathml::latex_to_mathml(c.as_ref(), latex2mathml::DisplayStyle::Block)
                        .unwrap_or_else(|e| e.to_string())
                        .into();
                has_math = true;
                Event::Html(text)
            }
            Event::InlineMath(c) => {
                let text: CowStr<'_> =
                    latex2mathml::latex_to_mathml(c.as_ref(), latex2mathml::DisplayStyle::Inline)
                        .unwrap_or_else(|e| e.to_string())
                        .into();
                has_math = true;
                Event::Html(text)
            }
            _ => event,
        };

        events.push(e);
    }

    let parser = events.into_iter();
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);
    HtmlContent {
        content: html_content,
        has_code,
        has_math,
    }
}

fn parse_post_markdown(
    input_path: &Path,
    output_path: &Path,
    commit: &str,
) -> std::io::Result<PostMetadata> {
    let mut post = Post::from_path(&input_path)?;

    if post.draft {
        return Ok(PostMetadata {
            title: post.title,
            date: post.date,
            url: "".to_string(),
            draft: post.draft,
        });
    }

    let rendered_content: HtmlContent = chew(&mut post.content, &input_path);

    /* render template */
    let tpls: Ramhorns = Ramhorns::from_folder("./templates").unwrap();
    let template = tpls.get("post.html").unwrap();

    if rendered_content.has_code {
        post.add_style("/styles/prism.css");
        post.add_script("/scripts/prism.js");
    }

    if rendered_content.has_math {
        println!("adding math to {:?}", input_path);
        post.add_style("/styles/math.css");
    }

    let rel_path: Vec<_> = output_path.components().collect();
    let rel_path2: PathBuf = rel_path[1..].iter().collect();
    let mut rel_path_parent = rel_path2.parent().unwrap().to_str().unwrap().to_string();
    if !rel_path_parent.is_empty() {
        rel_path_parent.push('/');
    }

    println!(
        "output path parent: {}",
        output_path.parent().unwrap().to_str().unwrap()
    );

    let rendered = template.render(&BaseTemplate {
        title: &post.title,
        path: &format!("/{}", rel_path_parent),
        content: &rendered_content.content,
        styles: &post.styles,
        scripts: &post.scripts,
        commit,
    });

    /* format the output html */

    let output_path = Path::new(output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, rendered)?;

    if let Some(r) = rel_path_parent.strip_suffix("/") {
        rel_path_parent = r.to_string();
    }

    /* metadata */
    let post = PostMetadata {
        title: post.title,
        url: format!("/{}", rel_path_parent),
        date: post.date.fixed_offset(),
        draft: post.draft,
    };
    Ok(post)
}

fn get_git_commit_hash() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("failed to execute git command: {}", e))?;

    if output.status.success() {
        let hash = str::from_utf8(&output.stdout)
            .map_err(|e| format!("invalid UTF-8 in git output: {}", e))?
            .trim()
            .to_string();
        Ok(hash)
    } else {
        let error = str::from_utf8(&output.stderr)
            .map_err(|e| format!("invalid UTF-8 in git error: {}", e))?
            .trim()
            .to_string();
        Err(format!("git command failed: {}", error))
    }
}

fn copy_traverse(input: &Path, output: &Path, full: bool) -> io::Result<()> {
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

    let mut posts: Vec<PostMetadata> = Vec::new();

    let mut routes: HashMap<String, String> = HashMap::new();

    while let Some((cur_in, cur_out)) = todo.pop_front() {
        for entry in fs::read_dir(&cur_in)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let mut new_path = cur_out.join(file_name);

            if path.is_dir() {
                fs::create_dir_all(&new_path)?;
                todo.push_back((path, new_path));
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    match ext.to_str().unwrap() {
                        "md" => {
                            new_path.set_extension("html");
                            let Ok(post) = parse_post_markdown(&path, &new_path, &commit) else {
                                break;
                            };

                            if post.draft {
                                break;
                            }

                            let Some(str) = path.to_str() else {
                                break;
                            };
                            routes.insert(
                                String::from(&post.url),
                                String::from(
                                    new_path.to_str().unwrap().strip_prefix("static").unwrap(),
                                ),
                            );
                            if str.contains("blog") {
                                posts.push(post);
                            }
                        }
                        "jpg" | "jpeg" | "png" => {
                            fs::copy(&path, &new_path)?;
                            if full {
                                new_path.set_extension("webp");
                                let img = image::open(path).unwrap();
                                let _ = img.save_with_format(new_path, ImageFormat::WebP);
                            }
                        }
                        _ => {
                            fs::copy(&path, &new_path)?;
                            println!("{:?} => {:?}", path, new_path);
                        }
                    }
                }
            }
        }
    }

    fs::write(
        "static/routes.json",
        serde_json::to_string(&routes).unwrap(),
    )?;

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

    let input = PathBuf::from("input");
    let output = PathBuf::from("static");

    if args.len() > 1 {
        if args[1] == "serve" {
            server::serve(&cwd, "8080");
        }
        if args[1] == "full" {
            copy_traverse(&input, &output, true)?;
        }
    }

    let path = Path::new("./input/blog/formula-hybrid-2024/img/paddock.jpg");
    println!("{:?}", path.exists());
    copy_traverse(&input, &output, false)

    //println!("Deploying from commit {}", &hash[0..6]);
    //render_blog("blog")
}
