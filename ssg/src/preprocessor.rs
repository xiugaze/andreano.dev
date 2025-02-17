use std::fmt::format;

// adapted from grego/cmark-syntax
use pulldown_cmark::{Event, Tag, TagEnd};

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
        let event = match self.parent.next()? {
            /* math */
            Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                let mut alttext = String::new();
                if let Some(Event::Text(alt)) = self.parent.next() {
                    alttext.push_str(&alt);
                }
                let mut html = String::new();
                html.push_str("<figure>\n");
                html.push_str(format!("<img src=\"{}\" alt=\"{}\" title=\"{}\">", dest_url, alttext, title).as_str());
                html.push_str(format!("<figcaption>{}</figcaption>", title).as_str());
                html.push_str("</figure>\n");
                Some(Event::Html(html.into()))
            },
            Event::InlineMath(c) => {
                Some(Event::Html(
                    latex2mathml::latex_to_mathml(
                        c.as_ref(),
                        latex2mathml::DisplayStyle::Inline,
                    )
                    .unwrap_or_else(|e| e.to_string())
                    .into(),
                ))
            },
            Event::DisplayMath(c) => {
                Some(Event::Html(
                latex2mathml::latex_to_mathml(
                    c.as_ref(),
                    latex2mathml::DisplayStyle::Block,
                )
                .unwrap_or_else(|e| e.to_string())
                .into(),
                ))
            },
            //Some(Event::Start((Tag::Image(linktype, url, title)))) => {
            //    Some(Event::Start(Tag::Link(linktype, url, title)))
            //},
            //Event::End(Tag::Image(linktype, url, title)) =>
            //    Event::End(Tag::Link(linktype, url, title)),
            other => return Some(other),
        };
        return event
    }
}
