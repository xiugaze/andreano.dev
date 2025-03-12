use std::path::Path;

use pulldown_cmark::{Event, Tag};

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
            Event::Start(Tag::Image {
                link_type,
                mut dest_url,
                title,
                id,
            }) => {
                let mut alttext = String::new();
                if let Some(Event::Text(alt)) = self.parent.next() {
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
                            dest_url,
                            extension
                        )
                        .as_str(),
                    );
                    dest_url = dest_url.replace(extension, "webp").into();
                }
                html.push_str(
                    format!(
                        "
                            <img loading=\"lazy\" src=\"{}\" alt=\"{}\" title=\"{}\">
                        ",
                        dest_url, alttext, title
                    )
                    .as_str(),
                );
                html.push_str("</picture>\n");
                html.push_str("</a>\n");
                html.push_str(format!("<figcaption>{}</figcaption>", title).as_str());
                html.push_str("</figure>\n");
                Some(Event::Html(html.into()))
            }
            other => return Some(other),
        };
        return event;
    }
}
