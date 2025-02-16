use pulldown_cmark::{Parser, Options};
use std::{fs::{self, File}, io::Write};

mod preprocessor;

fn main() -> std::io::Result<()> {
    let ex = "Hello, $x^2 = 2$";
    let options = Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(ex, options);
    let preprocessor = preprocessor::Preprocessor::new(parser);

    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, preprocessor);


    fs::write("test.html", output)?;
    Ok(())
}
