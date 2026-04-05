use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideBlock {
    Text(String),
    Image { alt: String, src: String },
}

pub fn parse_blocks(markdown: &str) -> Vec<SlideBlock> {
    let mut blocks = Vec::new();
    let mut text = String::new();
    let mut image_src: Option<String> = None;
    let mut image_alt = String::new();

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                flush_text(&mut blocks, &mut text);
                image_src = Some(dest_url.to_string());
                image_alt.clear();
            }
            Event::End(TagEnd::Image) => {
                if let Some(src) = image_src.take() {
                    blocks.push(SlideBlock::Image {
                        alt: image_alt.trim().to_string(),
                        src,
                    });
                    image_alt.clear();
                }
            }
            Event::Text(content) | Event::Code(content) => {
                if image_src.is_some() {
                    image_alt.push_str(&content);
                } else {
                    text.push_str(&content);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if image_src.is_some() {
                    image_alt.push(' ');
                } else {
                    text.push('\n');
                }
            }
            Event::Rule => {
                text.push_str("\n---\n");
            }
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::BlockQuote(_))
            | Event::End(TagEnd::CodeBlock)
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::List(_)) => {
                text.push('\n');
            }
            _ => {}
        }
    }

    flush_text(&mut blocks, &mut text);
    blocks
}

fn flush_text(blocks: &mut Vec<SlideBlock>, text: &mut String) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        blocks.push(SlideBlock::Text(trimmed.to_string()));
    }
    text.clear();
}

#[cfg(test)]
mod tests {
    use super::{parse_blocks, SlideBlock};

    #[test]
    fn parse_blocks_keeps_text_and_images_stable() {
        let markdown =
            "# Title\n\nHello **world**.\n\n![System diagram](images/flow.png)\n\n`code`";

        let blocks = parse_blocks(markdown);

        assert_eq!(
            blocks,
            vec![
                SlideBlock::Text(String::from("Title\nHello world.")),
                SlideBlock::Image {
                    alt: String::from("System diagram"),
                    src: String::from("images/flow.png"),
                },
                SlideBlock::Text(String::from("code")),
            ]
        );
    }
}
