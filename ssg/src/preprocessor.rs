use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

#[derive(Debug, Default)]

pub struct Preprocessor<'a, I: Iterator<Item = Event<'a>>> {
    parent: I,
}


impl<'a, I: Iterator<Item = Event<'a>>> Preprocessor<'a, I> {
    pub fn new(parent: I) -> Self {
        Self { parent }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for Preprocessor<'a, I> {
    type Item = Event<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.parent.next()? {
            Event::InlineMath(c) => {
                return Some(Event::Html(
                    latex2mathml::latex_to_mathml(
                        c.as_ref(),
                        latex2mathml::DisplayStyle::Inline,
                    )
                    .unwrap_or_else(|e| e.to_string())
                    .into(),
                ));
            }
            Event::DisplayMath(c) => {
                return Some(Event::Html(
                    latex2mathml::latex_to_mathml(
                        c.as_ref(),
                        latex2mathml::DisplayStyle::Block,
                    )
                    .unwrap_or_else(|e| e.to_string())
                    .into(),
                ));
            }
            other => return Some(other),
        };
        //
        //let next = self.parent.next();
        //let code = match next {
        //    Some(Event::Text(c)) => {
        //        let mut code = c;
        //        loop {
        //            match self.parent.next() {
        //                Some(Event::Text(ref c)) => {
        //                    code = {
        //                        let mut s = code.into_string();
        //                        s.push_str(c);
        //                        CowStr::Boxed(s.into())
        //                    }
        //                }
        //                Some(Event::End(TagEnd::CodeBlock)) | None => break,
        //                Some(e) => {
        //                    return Some(Event::Text(
        //                        format!("Unexpected markdown event {:#?}", e).into(),
        //                    ))
        //                }
        //            }
        //        }
        //        code
        //    }
        //    Some(Event::End(TagEnd::CodeBlock)) | None => CowStr::Borrowed(""),
        //    Some(e) => {
        //        return Some(Event::Text(
        //            format!("Unexpected markdown event {:#?}", e).into(),
        //        ))
        //    }
        //};
        //
        //let mut html = String::with_capacity(code.len() + code.len() / 4 + 60);
        //html.push_str("<pre><code class=\"language-");
        //html.push_str(lang.as_ref());
        //html.push_str("\">");
        //
        //match lang.as_ref() {
        //    "rust" | "rs" => highlight::<languages::Rust>(&code, &mut html),
        //    "js" | "javascript" => highlight::<languages::JavaScript>(&code, &mut html),
        //    "toml" => highlight::<languages::Toml>(&code, &mut html),
        //    "sh" | "shell" | "bash" => highlight::<languages::Sh>(&code, &mut html),
        //    _ => write_escaped(&mut html, &code),
        //}
        //
        //html.push_str("</code></pre>");
        //
        //Some(Event::Html(html.into()))
    }
}
