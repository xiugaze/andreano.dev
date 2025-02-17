use pulldown_cmark::{Parser, Options};
use server::serve;
use tidier::FormatOptions;
use std::fs::{self};
use ramhorns::{Template, Content};

mod preprocessor;
mod server;

use std::collections::HashMap;
use std::io;
//use tidy::Document;

fn parse_frontmatter_content(file_path: &str) -> io::Result<(Option<HashMap<String, String>>, String)> {
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

fn parse_post_markdown(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let (frontmatter, content) = parse_frontmatter_content(&input_path)?;
    let options = Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES;
    let parser = preprocessor::Preprocessor::new(Parser::new_ext(&content, options));


    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    /* note: in mustache, {{{ is an unescaped template */
    let template = 
        "<!DOCTYPE html>
            <html>
            <head>
                <title>{{ title }}</title>
                <link href=\"styles/style.css\" rel=\"stylesheet\"/>
            </head>
            <body>
                <div class=\"container\">
                    {{{ content }}}
                </div>
            </body>
            </html>
        ";

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


#[derive(Content)]
struct Post<'a> {
    title: &'a str,
    content: &'a str,
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "serve" {
        serve("output", "9090");
    } else {
        let _ = parse_post_markdown("./input/test.md", "./output/test.html");
    }

    Ok(())
}
