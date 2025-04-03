use image::{image_dimensions, ImageFormat};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ramhorns::{Content, Ramhorns, Template};
use serde_yaml::Value;

use server::serve::serve;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::{SyntaxReference, SyntaxSet};

use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::{env, io, str};

mod server;

struct Post {
    id: String,
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


                let filename = file_path.file_name().unwrap().to_str().unwrap().strip_suffix(".md").unwrap();
                println!("id: {:?}", filename);

                let mut post = Post {
                    id: filename.to_string(),
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

                let scripts_dir = file_path.parent().unwrap().join("scripts/");
                let index_js = scripts_dir.join("index.js");

                if index_js.exists() {
                    post.add_script("scripts/index.js");
                }
                //if scripts_dir.exists() && scripts_dir.is_dir() {
                //    for entry in fs::read_dir(scripts_dir)? {
                //        let entry = entry?;
                //        let path = entry.path();
                //        post.add_script(&format!("scripts/{}", path.file_name().unwrap().to_str().unwrap()));
                //    }
                //}

                let styles_dir = file_path.parent().unwrap().join("styles/");
                if styles_dir.exists() & styles_dir.is_dir() {
                    for entry in fs::read_dir(styles_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        post.add_style(&format!("styles/{}", path.file_name().unwrap().to_str().unwrap()));
                    }
                }


                return Ok(post);
            }
        }

        Ok(Post {
            id: "".to_string(),
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
    post_id: &'a str,
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
    toc: Option<String>,
}

fn headinglevel_to_i8(level: HeadingLevel) -> i8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn make_toc(toc: Vec<(String, String, i8)>) -> String {

    if toc.is_empty() {
        return String::from("");
    }

    let mut toc_string = String::from("<div class=\"toc\"><h4>Index</h4><ul>");
    let mut cur = 1; 

    for entry in toc {
        let level = entry.2; 

        while cur < level {
            toc_string.push_str("<ul>");
            cur += 1;
        }

        while cur > level {
            toc_string.push_str("</ul></li>");
            cur -= 1;
        }

        if level == cur {
            if !toc_string.ends_with("<ul>") && !toc_string.ends_with("<div class=\"toc\"><ul>") {
                toc_string.push_str("</li>");
            }
        }
        toc_string.push_str(&format!("<li><a href=\"#{}\">{}</a>", entry.1, entry.0));
    }

    while cur > 1 {
        toc_string.push_str("</ul></li>");
        cur -= 1;
    }

    toc_string.push_str("</li></ul></div>");
    toc_string
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

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let theme = &ts.themes["base16-ocean.light"];
    println!("{:?}", ts.themes.keys());
    let mut syntax: Option<&SyntaxReference> = None; 

    let mut parser = Parser::new_ext(&content, options).peekable();
    let mut has_code = false;
    let mut has_math = false;

    let mut to_highlight = String::new();
    let mut in_code_block = false;

    let mut toc = Vec::new();
    let mut events = Vec::new();

    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                println!("kind: {:?}", kind);
                // In actual use you'd probably want to keep track of what language this code is
                //syntax = ss.find_syntax_by_name(kind.)
                if let CodeBlockKind::Fenced(lang) = kind {
                    syntax = ss.find_syntax_by_token(&lang);
                }
                in_code_block = true;
            },
            Event::End(TagEnd::CodeBlock) => {
                has_code = true;
                if in_code_block {
                    let html = match syntax {
                        Some(syn) => highlighted_html_for_string(&to_highlight, &ss, &syn, &theme).unwrap(),
                        None => to_highlight,
                    };
                    events.push(Event::Html(html.into()));
                    to_highlight = String::new();
                    in_code_block = false;
                }
            },
            Event::Text(t) => {
                if in_code_block {
                    // If we're in a code block, build up the string of text
                    to_highlight.push_str(&t);
                } else {
                    events.push(Event::Text(t))
                }
            },
            Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs,
            }) => {
                if let Some(Event::Text(text)) = parser.next() {
                    let id_text: CowStr = text.to_lowercase().replace(" ", "-").into();
                    toc.push((text.to_string(), id_text.clone().to_string(), headinglevel_to_i8(level)));
                    events.push(Event::Start(Tag::Heading {
                        level,
                        id: Some(id_text),
                        classes,
                        attrs,
                    }));
                    events.push(Event::Text(text));
                } else {
                    events.push(Event::Start(Tag::Heading {
                        level,
                        id,
                        classes,
                        attrs,
                    }))
                }
            }
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

                /* NOTE: this is the biggest performance hit, this takes 4ever */
                let img_path = parent.join(image_path);
                let mut img_path = Path::new("./").join(img_path);
                img_path.set_extension("webp");
                let dimensions = match image_dimensions(&img_path) {
                    Ok(dim) => dim,
                    Err(_) => (800, 400),
                };

                if let Some(Event::Text(alt)) = parser.next() {
                    alttext.push_str(&alt);
                }
                let mut html = String::new();
                html.push_str("<figure>\n");
                html.push_str(format!("<a href=\"{}\">\n", dest_url).as_str());
                html.push_str("<picture>\n");

                if dest_url.ends_with(".jpg")
                    | dest_url.ends_with(".png")
                    | dest_url.ends_with(".jpeg")
                {
                    let dest_str = dest_url.to_string();
                    let extension = Path::new(&dest_str)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap();
                    html.push_str(
                        format!(
                            "
                                <source srcset=\"{}\" type=\"image/webp\">
                            ",
                            dest_url.replace(extension, "webp"),
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
                events.push(Event::Html(html.into()))
            },
            Event::DisplayMath(c) => {
                let text: CowStr<'_> =
                    latex2mathml::latex_to_mathml(c.as_ref(), latex2mathml::DisplayStyle::Block)
                        .unwrap_or_else(|e| e.to_string())
                        .into();
                has_math = true;
                events.push(Event::Html(text))
            },
            Event::InlineMath(c) => {
                let text: CowStr<'_> =
                    latex2mathml::latex_to_mathml(c.as_ref(), latex2mathml::DisplayStyle::Inline)
                        .unwrap_or_else(|e| e.to_string())
                        .into();
                has_math = true;
                events.push(Event::Html(text))
            }
            _ => events.push(event),
        };
    }

    let toc_string = make_toc(toc);

    println!("{}", toc_string);

    let parser = events.into_iter();
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    html_content = html_content.replace("{{ toc }}", &toc_string.clone()); // HACK: should fix this lol.
    HtmlContent {
        content: html_content,
        has_code,
        has_math,
        toc: Some(toc_string),
    }
}

fn parse_post_markdown(
    input_path: &Path,
    output_path: &Path,
    commit: &str,
    template: &Template
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
    //let tpls: Ramhorns = Ramhorns::from_folder("./templates").unwrap();
    //let template = tpls.get("post.html").unwrap();

    if rendered_content.has_math {
        post.add_style("/styles/math.css");
    }

    let rel_path: Vec<_> = output_path.components().collect();
    let rel_path2: PathBuf = rel_path[1..].iter().collect();
    let mut rel_path_parent = rel_path2.parent().unwrap().to_str().unwrap().to_string();
    if !rel_path_parent.is_empty() {
        rel_path_parent.push('/');
    }

    let toc_str = match rendered_content.toc {
        Some(toc_str) => toc_str,
        None => String::from(""),
    };

    println!("toc_str: {:?}", toc_str);

    let rendered = template.render(&BaseTemplate {
        post_id: &post.id,
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

fn copy_traverse(input: &Path, output: &Path, hash: &str, full: bool) -> io::Result<()> {
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

    let commit = &hash[0..7];

    let mut todo = VecDeque::new();
    todo.push_back((input.to_path_buf(), output.to_path_buf()));

    let mut posts: Vec<PostMetadata> = Vec::new();
    let tpls: Ramhorns = Ramhorns::from_folder("./templates").unwrap();

    while let Some((cur_in, cur_out)) = todo.pop_front() {
        for entry in fs::read_dir(&cur_in)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let mut new_path = cur_out.join(file_name);

            if path.is_dir() {
                println!("created dir {:?}", new_path);
                fs::create_dir_all(&new_path)?;
                todo.push_back((path, new_path));
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    match ext.to_str().unwrap() {
                        "md" => {
                            new_path = cur_out.join("index.html");
                            let Some(str) = path.to_str() else {
                                break;
                            };

                            let mut template = "base.html";
                            let mut blog = false;
                            if str.contains("blog") {
                                template = "post.html";
                                blog = true;
                            }

                            let Ok(post) = parse_post_markdown(&path, &new_path, &commit, &tpls.get(template).unwrap()) else {
                                break;
                            };

                            if post.draft {
                                break;
                            }

                            if blog {
                                posts.push(post);
                            }
                        },
                        "html" => {
                            new_path = cur_out.join("index.html");
                            fs::copy(&path, &new_path)?;
                        },
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

    let mut blog_index_content = String::new();
    posts.sort_by(|i, j| (&j.date).cmp(&i.date)); // reverse comparison
    blog_index_content.push_str("<h1>blog</h1>\n\t<div class=\"blog-index\">\n");
    for p in posts {
        blog_index_content.push_str("<div class=\"blog-entry\">\n");

        let date_str = p.date.format("%Y-%m-%d").to_string();
        blog_index_content.push_str(&format!("<div class=\"blog-date\">&lt;{}&gt;</div>", date_str));
        blog_index_content.push_str(&format!("<div class=\"blog-title\"><a href=\"{}\">{}</a></div>", p.url, p.title));
        blog_index_content.push_str("</div>");
    }
    blog_index_content.push_str("</div>");

    let template = tpls.get("index.html").unwrap();
    let rendered = template.render(&BaseTemplate {
        post_id: "",
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


const DB_DIR: &'static str = "/var/lib/andreano-dev/db.sqlite3";

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2{
        println!("error: provide a command");
    } else {
        match args[1].as_str() {
            "serve" => {
                println!("serving...");
                let rt = tokio::runtime::Runtime::new()?;
                if args.len() > 2 {
                    rt.block_on(serve(Path::new(&args[2]), String::from(DB_DIR)));
                }
            },
            "crunch" => {
                let input = PathBuf::from("input");
                let output = PathBuf::from("static");
                let mut hash = "000000";
                if args.len() > 2 {
                    hash = &args[2];
                }
                return copy_traverse(&input, &output, &hash, false)
            },
            _ => { println!("unknown argument"); }
        }
    }
    return Ok(());
}


