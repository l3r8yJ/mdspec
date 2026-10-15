use crate::config::Labels;
use crate::model::{Block, BlockKind, Document, NestedHeading};

pub struct Outline<'a> {
    pub preamble: Vec<&'a Block>,
    pub parts: Vec<Part<'a>>,
    pub references: Vec<&'a [Block]>,
    pub stray: Vec<&'a Block>,
    pub nested: &'a [NestedHeading],
}

pub enum Part<'a> {
    Endpoint(Endpoint<'a>),
    General(General<'a>),
    Other(&'a Block),
}

pub struct Endpoint<'a> {
    pub heading: &'a Block,
    pub name: &'a str,
    pub intro: &'a [Block],
    pub sections: Vec<Section<'a>>,
    pub detached: Vec<Section<'a>>,
}

pub struct General<'a> {
    pub heading: &'a Block,
    pub items: Vec<Item<'a>>,
}

pub struct Section<'a> {
    pub heading: &'a Block,
    pub kind: SectionKind,
    pub body: &'a [Block],
    pub items: Vec<Item<'a>>,
}

pub struct Item<'a> {
    pub heading: &'a Block,
    pub body: &'a [Block],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Logic,
    Components,
    Mappings,
    Examples,
    References,
    Unknown,
}

impl SectionKind {
    const KNOWN: [Self; 5] = [
        Self::Logic,
        Self::Components,
        Self::Mappings,
        Self::Examples,
        Self::References,
    ];

    fn of(name: &str, labels: &Labels) -> Self {
        labels
            .sections()
            .into_iter()
            .zip(Self::KNOWN)
            .find_map(|(label, kind)| (label == name).then_some(kind))
            .unwrap_or(Self::Unknown)
    }

    pub fn rank(self) -> Option<usize> {
        Self::KNOWN.iter().position(|kind| *kind == self)
    }

    const fn has_items(self) -> bool {
        matches!(self, Self::Components | Self::Mappings | Self::Examples)
    }
}

impl<'a> Part<'a> {
    pub const fn heading(&self) -> &'a Block {
        match self {
            Self::Endpoint(endpoint) => endpoint.heading,
            Self::General(general) => general.heading,
            Self::Other(heading) => heading,
        }
    }
}

impl<'a> Outline<'a> {
    pub fn part_at(&self, offset: usize) -> Option<(usize, &Part<'a>)> {
        self.parts
            .iter()
            .enumerate()
            .rev()
            .find(|(_, part)| (..=offset).contains(&part.heading().span.start.offset))
    }

    pub fn endpoints(&self) -> impl Iterator<Item = &Endpoint<'a>> {
        self.parts.iter().filter_map(|part| match part {
            Part::Endpoint(endpoint) => Some(endpoint),
            Part::General(_) | Part::Other(_) => None,
        })
    }

    pub fn general(&self) -> impl Iterator<Item = &General<'a>> {
        self.parts.iter().filter_map(|part| match part {
            Part::General(general) => Some(general),
            Part::Endpoint(_) | Part::Other(_) => None,
        })
    }
}

const TO_END_OF_DOCUMENT: u8 = 0;

pub fn children(blocks: &[Block], index: usize, depth: u8) -> &[Block] {
    let rest = blocks.get(index + 1..).unwrap_or_default();
    let length = rest
        .iter()
        .position(|block| matches!(block.kind, BlockKind::Heading { level, .. } if level <= depth))
        .unwrap_or(rest.len());
    rest.get(..length).unwrap_or_default()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Sections,
    Detached,
    Closed,
}

struct Builder<'a, 'l> {
    blocks: &'a [Block],
    labels: &'l Labels,
    outline: Outline<'a>,
    current_section: Option<SectionKind>,
    in_component: bool,
    target: Target,
}

pub fn build<'a>(document: &'a Document, labels: &Labels) -> Outline<'a> {
    let mut builder = Builder {
        blocks: &document.blocks,
        labels,
        outline: Outline {
            preamble: Vec::new(),
            parts: Vec::new(),
            references: Vec::new(),
            stray: Vec::new(),
            nested: &document.nested_headings,
        },
        current_section: None,
        in_component: false,
        target: Target::Sections,
    };
    for (index, block) in document.blocks.iter().enumerate() {
        if let BlockKind::Heading { level, .. } = block.kind {
            builder.place(index, block, level);
        }
    }
    builder.outline
}

impl<'a> Builder<'a, '_> {
    fn place(&mut self, index: usize, heading: &'a Block, level: u8) {
        self.record_references_section(index, heading, level);
        let fits = match level {
            1 => self.accept_title(),
            2 => {
                self.open_part(index, heading);
                true
            }
            _ => match self.outline.parts.last() {
                Some(Part::Other(_)) => true,
                Some(Part::General(_)) => self.accept_general_heading(index, heading, level),
                Some(Part::Endpoint(_)) | None => {
                    self.accept_endpoint_heading(index, heading, level)
                }
            },
        };
        if !fits {
            self.outline.stray.push(heading);
        }
    }

    fn record_references_section(&mut self, index: usize, heading: &Block, level: u8) {
        if heading.text.trim() != self.labels.references {
            return;
        }
        let ends_before_level = match level {
            2 => TO_END_OF_DOCUMENT,
            3 => 2,
            _ => return,
        };
        self.outline
            .references
            .push(children(self.blocks, index, ends_before_level));
    }

    const fn accept_title(&mut self) -> bool {
        let first = self.outline.parts.is_empty();
        if !first {
            self.target = Target::Closed;
        }
        first
    }

    fn open_part(&mut self, index: usize, heading: &'a Block) {
        let name = heading.text.trim();
        let part = if name == self.labels.components {
            Part::General(General {
                heading,
                items: Vec::new(),
            })
        } else {
            name.strip_prefix(self.labels.endpoint_prefix.as_str())
                .map_or(Part::Other(heading), |name| {
                    Part::Endpoint(Endpoint {
                        heading,
                        name: name.trim(),
                        intro: children(self.blocks, index, 3),
                        sections: Vec::new(),
                        detached: Vec::new(),
                    })
                })
        };
        self.outline.parts.push(part);
        self.current_section = None;
        self.in_component = false;
        self.target = Target::Sections;
    }

    fn accept_general_heading(&mut self, index: usize, heading: &'a Block, level: u8) -> bool {
        let name = heading.text.trim();
        match level {
            3 => {
                if name != self.labels.references {
                    self.attach_general_item(index, heading);
                }
                !name.is_empty()
            }
            4 => name == self.labels.description,
            _ => false,
        }
    }

    fn attach_general_item(&mut self, index: usize, heading: &'a Block) {
        if self.target != Target::Sections {
            return;
        }
        if let Some(Part::General(general)) = self.outline.parts.last_mut() {
            general.items.push(Item {
                heading,
                body: children(self.blocks, index, 3),
            });
        }
    }

    fn accept_endpoint_heading(&mut self, index: usize, heading: &'a Block, level: u8) -> bool {
        let name = heading.text.trim();
        match level {
            3 => {
                self.open_section(index, heading);
                true
            }
            4 => {
                self.attach_section_item(index, heading);
                self.in_component = self.current_section == Some(SectionKind::Components);
                self.current_section.is_some_and(SectionKind::has_items) && !name.is_empty()
            }
            5 => self.in_component && name == self.labels.description,
            _ => false,
        }
    }

    fn open_section(&mut self, index: usize, heading: &'a Block) {
        let kind = SectionKind::of(heading.text.trim(), self.labels);
        self.current_section = Some(kind);
        self.in_component = false;
        let section = Section {
            heading,
            kind,
            body: children(self.blocks, index, 3),
            items: Vec::new(),
        };
        match self.outline.parts.last_mut() {
            None => self.outline.preamble.push(heading),
            Some(Part::Endpoint(endpoint)) if self.target == Target::Sections => {
                endpoint.sections.push(section);
            }
            Some(Part::Endpoint(endpoint)) => {
                endpoint.detached.push(section);
                self.target = Target::Detached;
            }
            Some(Part::General(_) | Part::Other(_)) => {}
        }
    }

    fn attach_section_item(&mut self, index: usize, heading: &'a Block) {
        let Some(Part::Endpoint(endpoint)) = self.outline.parts.last_mut() else {
            return;
        };
        let sections = match self.target {
            Target::Sections => &mut endpoint.sections,
            Target::Detached => &mut endpoint.detached,
            Target::Closed => return,
        };
        if let Some(section) = sections
            .last_mut()
            .filter(|section| section.kind.has_items())
        {
            section.items.push(Item {
                heading,
                body: children(self.blocks, index, 4),
            });
        }
    }
}
