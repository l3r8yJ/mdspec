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
    let document = Document {
        blocks: Vec::new(),
        definitions,
        links: Vec::new(),
        headings: Vec::new(),
        nested_headings: Vec::new(),
    };
    let mut state = ParseState {
        document,
        positions,
        depth: 0,
        heading: None,
        table_rows: Vec::new(),
        cell: None,
        in_html_comment: false,
        table_of_contents: false,
    };
    let mut events = parser.into_offset_iter();
    while let Some((event, span)) = events.next() {
        state.event(event, span, events.reference_definitions());
    }
    let mut document = state.document;
    let positions = state.positions;
    for span in definition_ranges {
        if !document
            .blocks
            .iter()
            .any(|block| (block.position.offset..block.end).contains(&span.start))
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

struct ParseState<'a> {
    document: Document,
    positions: SourcePositions<'a>,
    depth: usize,
    heading: Option<(String, Position)>,
    table_rows: Vec<Vec<String>>,
    cell: Option<String>,
    in_html_comment: bool,
    table_of_contents: bool,
}

impl ParseState<'_> {
    fn event(
        &mut self,
        event: Event<'_>,
        span: std::ops::Range<usize>,
        definitions: &pulldown_cmark::RefDefs<'_>,
    ) {
        let start = self.positions.at(span.start);
        match event {
            Event::Start(tag) => self.start(tag, span, definitions),
            Event::End(tag) => self.end(tag),
            Event::Html(ref text) | Event::InlineHtml(ref text)
                if self.in_html_comment || text.trim().starts_with("<!--") =>
            {
                self.in_html_comment = !text.contains("-->");
            }
            Event::InlineHtml(text) => {
                if self.heading.is_none()
                    && let Some(block) = self.document.blocks.last_mut()
                {
                    block.text.push_str(&text);
                }
            }
            Event::Text(text)
            | Event::Code(text)
            | Event::Html(text)
            | Event::InlineMath(text)
            | Event::DisplayMath(text) => {
                if self.depth == 0 {
                    self.document.blocks.push(Block {
                        kind: BlockKind::Content,
                        position: start,
                        end: span.end,
                        text: text.to_string(),
                    });
                } else if let Some(block) = self.document.blocks.last_mut() {
                    block.text.push_str(&text);
                }
                if let Some((value, _)) = &mut self.heading {
                    value.push_str(&text);
                }
                if let Some(value) = &mut self.cell {
                    value.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => self.line_break(),
            Event::Rule if self.depth == 0 => self.document.blocks.push(Block {
                kind: BlockKind::Content,
                position: start,
                end: span.end,
                text: String::new(),
            }),
            _ => {}
        }
    }

    fn line_break(&mut self) {
        if let Some(block) = self.document.blocks.last_mut() {
            block.text.push('\n');
        }
        if let Some((value, _)) = &mut self.heading {
            value.push(' ');
        }
        if let Some(value) = &mut self.cell {
            value.push(' ');
        }
    }

    fn start(
        &mut self,
        tag: Tag<'_>,
        span: std::ops::Range<usize>,
        definitions: &pulldown_cmark::RefDefs<'_>,
    ) {
        let start = self.positions.at(span.start);

        if self.depth == 0 {
            self.table_of_contents = matches!(tag, Tag::Paragraph)
                && matches!(
                    self.positions.source.get(span.clone()).map(str::trim),
                    Some("[TOC]" | "[[_TOC_]]")
                );
            let kind = match &tag {
                Tag::Heading { level, .. } => BlockKind::Heading(*level as u8),
                Tag::Table(_) => BlockKind::Table(Vec::new()),
                Tag::Paragraph => BlockKind::Paragraph,
                _ => BlockKind::Content,
            };
            self.document.blocks.push(Block {
                kind,
                position: start,
                end: span.end,
                text: String::new(),
            });
        }
        match tag {
            Tag::Heading { .. } => {
                self.heading = Some((String::new(), start));
                if self.depth > 0 {
                    self.document.nested_headings.push(start);
                }
            }
            Tag::TableHead | Tag::TableRow => self.table_rows.push(Vec::new()),
            Tag::TableCell => self.cell = Some(String::new()),
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
                    definitions.get(&id).map_or_else(
                        || id.to_string(),
                        |definition| definition.span.start.to_string(),
                    )
                });
                self.document.links.push(Link {
                    key,
                    url: (!unknown).then(|| dest_url.to_string()),
                    position: start,
                    table_of_contents: unknown && self.table_of_contents,
                });
            }
            _ => {}
        }
        self.depth += 1;
    }

    fn end(&mut self, tag: TagEnd) {
        self.depth = self.depth.saturating_sub(1);
        match tag {
            TagEnd::Heading(_) => {
                if let Some(value) = self.heading.take() {
                    self.document.headings.push(value);
                }
            }
            TagEnd::TableCell => {
                if let (Some(row), Some(value)) = (self.table_rows.last_mut(), self.cell.take()) {
                    row.push(value);
                }
            }
            TagEnd::Table => {
                let rows = std::mem::take(&mut self.table_rows);
                if let Some(block) = self.document.blocks.last_mut()
                    && matches!(block.kind, BlockKind::Table(_))
                {
                    block.kind = BlockKind::Table(rows);
                }
            }
            TagEnd::Paragraph | TagEnd::Item => {
                if let Some(block) = self.document.blocks.last_mut() {
                    block.text.push('\n');
                }
            }
            _ => {}
        }
    }
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
        let line_start = self.line_starts.get(line - 1).copied().unwrap_or_default();
        Position {
            line,
            column: self
                .source
                .get(line_start..offset)
                .unwrap_or_default()
                .chars()
                .count()
                + 1,
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

    #[test]
    fn preserves_breaks_between_words_in_multiline_content_and_headings() {
        let document =
            parse("First\nsecond\n======\n\nThird  \nfourth\n\n- Fifth\n\n  Sixth\n- Seventh\n");
        assert_that!(document.headings[0].0.as_str()).is_equal_to("First second");
        assert_that!(document.blocks[0].text.as_str()).is_equal_to("First\nsecond");
        assert_that!(document.blocks[1].text.as_str()).is_equal_to("Third\nfourth\n");
        assert_that!(document.blocks[2].text.as_str()).is_equal_to("Fifth\nSixth\n\nSeventh\n\n");
    }

    #[test]
    fn keeps_inline_html_out_of_heading_names_but_preserves_body_markup() {
        let document = parse("## <em>Name</em>\n\nText <em>body</em>\n");
        assert_that!(document.blocks[0].text.as_str()).is_equal_to("Name");
        assert_that!(document.blocks[1].text.trim()).is_equal_to("Text <em>body</em>");
    }

    #[test]
    fn keeps_nested_tables_inside_their_containing_blocks() {
        let document = parse(
            "> | Source | Target |\n> | --- | --- |\n> | hidden | nested |\n\n| Source | Target |\n| --- | --- |\n| actual | value |\n",
        );
        assert_that!(document.blocks.len()).is_equal_to(2);
        assert_that!(matches!(document.blocks[0].kind, BlockKind::Content)).is_equal_to(true);
        assert_that!(matches!(&document.blocks[1].kind, BlockKind::Table(rows) if rows.len() == 2 && rows[1] == ["actual", "value"])).is_equal_to(true);
    }

    #[test]
    fn keeps_nested_rules_inside_their_containing_blocks() {
        let document = parse("before\n\n---\n\n> after\n>\n> ---\n>\n> final\n");
        assert_that!(document.blocks.len()).is_equal_to(3);
        assert_that!(document.blocks[1].text.is_empty()).is_equal_to(true);
        assert_that!(document.blocks[2].text.trim()).is_equal_to("after\nfinal");
    }

    #[test]
    fn omits_multiline_html_comments_but_preserves_adjacent_html() {
        let document = parse("<!--\n hidden\n-->\n<div>\nvisible\n</div>\n");
        let text: String = document
            .blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect();
        assert_that!(text.as_str()).is_equal_to("<div>\nvisible\n</div>\n");
    }
}
