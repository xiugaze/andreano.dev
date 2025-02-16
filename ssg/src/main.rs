use pulldown_cmark::{Parser, Options};
use std::{fmt::write, fs::{self, File}, io::Write};
use ramhorns::{Template, Content};

mod preprocessor;

#[derive(Content)]
struct Post<'a> {
    title: &'a str,
}

fn main() -> std::io::Result<()> {
    let ex = "# {{title}}\n\nHello, $x^2 = 2$";
    let options = Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(ex, options);
    let preprocessor = preprocessor::Preprocessor::new(parser);

    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, preprocessor);

    let rendered = Template::new(output).unwrap().render(
        &Post {
            title: "Test title!",
        },
    );

    fs::write("test.html", rendered)?;
    Ok(())
}
