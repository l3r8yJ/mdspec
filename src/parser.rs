use crate::model::{Block, BlockKind, Definition, Document, Link, Position};
use pulldown_cmark::{BrokenLink, Event, LinkType, Options, Parser, Tag, TagEnd};

pub fn parse(source: &str) -> Document {
    let positions = SourcePositions::new(source);
    let parser = Parser::new_with_broken_link_callback(
        source,
        Options::ENABLE_TABLES,
        Some(|_: BrokenLink<'_>| Some(("".into(), "".into()))),
    );
    let mut definitions: Vec<_> = parser
        .reference_definitions()
        .iter()
        .map(|(_, definition)| Definition {
            key: definition.span.start.to_string(),
            url: definition.dest.to_string(),
            position: positions.at(definition.span.start),
        })
        .collect();
    definitions.sort_by_key(|definition| definition.position.offset);
    let definition_ranges: Vec<_> = parser
        .reference_definitions()
        .iter()
        .map(|(_, definition)| definition.span.clone())
        .collect();
    let mut document = Document {
        blocks: Vec::new(),
        definitions,
        links: Vec::new(),
        headings: Vec::new(),
        nested_headings: Vec::new(),
    };
    let mut events = parser.into_offset_iter();
    let mut depth = 0_usize;
    let mut heading: Option<(String, Position)> = None;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut cell: Option<String> = None;
    let mut in_html_comment = false;
    while let Some((event, span)) = events.next() {
        let start = positions.at(span.start);
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    let kind = match &tag {
                        Tag::Heading { level, .. } => BlockKind::Heading(*level as u8),
                        Tag::Table(_) => BlockKind::Table(Vec::new()),
                        Tag::Paragraph => BlockKind::Paragraph,
                        _ => BlockKind::Content,
                    };
                    document.blocks.push(Block {
                        kind,
                        position: start,
                        end: span.end,
                        text: String::new(),
                    });
                }
                match tag {
                    Tag::Heading { .. } => {
                        heading = Some((String::new(), start));
                        if depth > 0 {
                            document.nested_headings.push(start);
                        }
                    }
                    Tag::Table(_) => table_rows.clear(),
                    Tag::TableHead | Tag::TableRow => table_rows.push(Vec::new()),
                    Tag::TableCell => cell = Some(String::new()),
                    Tag::Link {
                        link_type,
                        dest_url,
                        id,
                        ..
                    }
                    | Tag::Image {
                        link_type,
                        dest_url,
                        id,
                        ..
                    } => {
                        let unknown = matches!(
                            link_type,
                            LinkType::ReferenceUnknown
                                | LinkType::CollapsedUnknown
                                | LinkType::ShortcutUnknown
                        );
                        let reference = matches!(
                            link_type,
                            LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut
                        ) || unknown;
                        let key = reference.then(|| {
                            events.reference_definitions().get(&id).map_or_else(
                                || id.to_string(),
                                |definition| definition.span.start.to_string(),
                            )
                        });
                        document.links.push(Link {
                            key,
                            url: (!unknown).then(|| dest_url.to_string()),
                            position: start,
                        });
                    }
                    _ => {}
                }
                depth += 1;
            }
            Event::End(tag) => {
                depth = depth.saturating_sub(1);
                match tag {
                    TagEnd::Heading(_) => {
                        if let Some(value) = heading.take() {
                            document.headings.push(value);
                        }
                    }
                    TagEnd::TableCell => {
                        if let (Some(row), Some(value)) = (table_rows.last_mut(), cell.take()) {
                            row.push(value);
                        }
                    }
                    TagEnd::Table if depth == 0 => {
                        if let Some(block) = document.blocks.last_mut() {
                            block.kind = BlockKind::Table(std::mem::take(&mut table_rows));
                        }
                    }
                    TagEnd::Paragraph | TagEnd::Item => {
                        if let Some(block) = document.blocks.last_mut() {
                            block.text.push('\n');
                        }
                    }
                    _ => {}
                }
            }
            Event::Html(ref text) | Event::InlineHtml(ref text)
                if in_html_comment || text.trim().starts_with("<!--") =>
            {
                in_html_comment = !text.contains("-->");
            }
            Event::InlineHtml(text) => {
                if heading.is_none()
                    && let Some(block) = document.blocks.last_mut()
                {
                    block.text.push_str(&text);
                }
            }
            Event::Text(text)
            | Event::Code(text)
            | Event::Html(text)
            | Event::InlineMath(text)
            | Event::DisplayMath(text) => {
                if depth == 0 {
                    document.blocks.push(Block {
                        kind: BlockKind::Content,
                        position: start,
                        end: span.end,
                        text: text.to_string(),
                    });
                } else if let Some(block) = document.blocks.last_mut() {
                    block.text.push_str(&text);
                }
                if let Some((value, _)) = &mut heading {
                    value.push_str(&text);
                }
                if let Some(value) = &mut cell {
                    value.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(block) = document.blocks.last_mut() {
                    block.text.push('\n');
                }
                if let Some((value, _)) = &mut heading {
                    value.push(' ');
                }
                if let Some(value) = &mut cell {
                    value.push(' ');
                }
            }
            Event::Rule if depth == 0 => document.blocks.push(Block {
                kind: BlockKind::Content,
                position: start,
                end: span.end,
                text: String::new(),
            }),
            _ => {}
        }
    }
    for span in definition_ranges {
        if !document
            .blocks
            .iter()
            .any(|block| block.position.offset <= span.start && span.start < block.end)
        {
            document.blocks.push(Block {
                kind: BlockKind::Definition,
                position: positions.at(span.start),
                end: span.end,
                text: String::new(),
            });
        }
    }
    document.blocks.sort_by_key(|block| block.position.offset);
    document
}

struct SourcePositions<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> SourcePositions<'a> {
    fn new(source: &'a str) -> Self {
        let mut line_starts = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' || (byte == b'\r' && source.as_bytes().get(offset + 1) != Some(&b'\n'))
            {
                line_starts.push(offset + 1);
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    fn at(&self, offset: usize) -> Position {
        let line = self.line_starts.partition_point(|start| *start <= offset);
        let line_start = self.line_starts[line - 1];
        Position {
            line,
            column: self.source[line_start..offset].chars().count() + 1,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::model::BlockKind;
    use asserting::prelude::*;

    #[test]
    fn treats_lone_carriage_returns_as_line_breaks() {
        let document = parse("## Тест\r\rЯ [нет]\r");
        assert_that!(document.links[0].position.line).is_equal_to(3);
        assert_that!(document.links[0].position.column).is_equal_to(3);
    }

    #[test]
    fn keeps_visible_heading_text_without_inline_html_tags() {
        let document = parse("## <em>Имя</em>\n");
        assert_that!(document.headings[0].0.as_str()).is_equal_to("Имя");
    }

    #[test]
    fn reports_unicode_columns_and_byte_offsets() {
        let document = parse("## Тест\r\n\r\nЯ [нет]\r\n");
        assert_that!(document.links[0].position.line).is_equal_to(3);
        assert_that!(document.links[0].position.column).is_equal_to(3);
        assert_that!(document.links[0].position.offset).is_equal_to(18);
    }

    #[test]
    fn resolves_case_folded_references_and_preserves_unknown_links() {
        let document =
            parse("[текст][STRASSE] [missing][] ` [code] ` \\[escaped]\n\n[Straße]: target.md\n");
        assert_that!(document.links.len()).is_equal_to(2);
        assert_that!(document.links[0].key.as_deref())
            .is_equal_to(Some(document.definitions[0].key.as_str()));
        assert_that!(document.links[1].url.as_deref()).is_equal_to(None);
    }

    #[test]
    fn preserves_table_cells_and_ignores_code_headings() {
        let document = parse(
            "## Заголовок\n\n> ### Вложенный\n\n```md\n# Fake\n```\n\n| Source | Target |\n| --- | --- |\n| id | |\n",
        );
        assert_that!(document.headings.len()).is_equal_to(2);
        assert_that!(document.nested_headings.len()).is_equal_to(1);
        assert_that!(matches!(document.blocks.last().map(|block| &block.kind), Some(BlockKind::Table(rows)) if rows[1][1].is_empty())).is_equal_to(true);
    }

    #[test]
    fn excludes_html_comments_from_meaningful_content() {
        let document = parse("<!--\n empty \n-->\n\nText <!-- hidden --> visible.\n");
        assert_that!(document.blocks[0].text.is_empty()).is_equal_to(true);
        assert_that!(document.blocks[1].text.trim()).is_equal_to("Text  visible.");
    }
}
