use pulldown_cmark::{Parser, Options};
use server::serve;
use std::{fmt::write, fs::{self, File}, io::Write};
use ramhorns::{Template, Content};

mod preprocessor;
mod server;

#[derive(Content)]
struct Post<'a> {
    title: &'a str,
    content: &'a str,
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "serve" {
        serve("output", "9090");
    }

    let input = fs::read_to_string("./input/test.md")?;

    let options = Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(&input, options);
    let preprocessor = preprocessor::Preprocessor::new(parser);

    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, preprocessor);

    
    /* note: in mustache, {{{ is an unescaped template */
    let template = 
        "<!DOCTYPE html>
            <html>
            <head>
                <title>{{ title }}</title>
                <link href=\"styles/style.css\" rel=\"stylesheet\"/>
            </head>
            <body>
                <h1>{{ title }}</h1>
                <div>{{{ content }}}</div> 
            </body>
            </html>
        ";
    let rendered = Template::new(template).unwrap().render(
        &Post {
            title: "test title",
            content: &content,
        },
    );

    fs::write("./output/test.html", rendered)?;
    Ok(())
}
