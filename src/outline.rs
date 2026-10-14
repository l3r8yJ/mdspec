use crate::config::Labels;
use crate::model::{Block, BlockKind, Document, NestedHeading};

pub struct Outline<'a> {
    pub preamble: Vec<&'a Block>,
    pub parts: Vec<Part<'a>>,
    pub stray: Vec<&'a Block>,
    pub nested: &'a [NestedHeading],
}

pub enum Part<'a> {
    Endpoint(Endpoint<'a>),
    General(General<'a>),
    Other,
}

pub struct Endpoint<'a> {
    pub heading: &'a Block,
    pub name: &'a str,
    pub sections: Vec<Section<'a>>,
}

pub struct General<'a> {
    pub heading: &'a Block,
}

pub struct Section<'a> {
    pub heading: &'a Block,
    pub kind: SectionKind,
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

    const fn has_items(self) -> bool {
        matches!(self, Self::Components | Self::Mappings | Self::Examples)
    }
}

impl<'a> Outline<'a> {
    pub fn endpoints(&self) -> impl Iterator<Item = &Endpoint<'a>> {
        self.parts.iter().filter_map(|part| match part {
            Part::Endpoint(endpoint) => Some(endpoint),
            Part::General(_) | Part::Other => None,
        })
    }
}

struct Builder<'a, 'l> {
    labels: &'l Labels,
    outline: Outline<'a>,
    section: Option<SectionKind>,
    component: bool,
    attached: bool,
}

pub fn build<'a>(document: &'a Document, labels: &Labels) -> Outline<'a> {
    let mut builder = Builder {
        labels,
        outline: Outline {
            preamble: Vec::new(),
            parts: Vec::new(),
            stray: Vec::new(),
            nested: &document.nested_headings,
        },
        section: None,
        component: false,
        attached: true,
    };
    for block in &document.blocks {
        if let BlockKind::Heading { level, .. } = block.kind {
            builder.heading(block, level);
        }
    }
    builder.outline
}

impl<'a> Builder<'a, '_> {
    fn heading(&mut self, heading: &'a Block, level: u8) {
        let fits = match level {
            1 => self.title(),
            2 => {
                self.part(heading);
                true
            }
            _ => match self.outline.parts.last() {
                Some(Part::Other) => true,
                Some(Part::General(_)) => self.general(heading, level),
                Some(Part::Endpoint(_)) | None => self.section(heading, level),
            },
        };
        if !fits {
            self.outline.stray.push(heading);
        }
    }

    const fn title(&mut self) -> bool {
        let first = self.outline.parts.is_empty();
        if !first {
            self.attached = false;
        }
        first
    }

    fn part(&mut self, heading: &'a Block) {
        let name = heading.text.trim();
        let part = if name == self.labels.components {
            Part::General(General { heading })
        } else {
            name.strip_prefix(self.labels.endpoint_prefix.as_str())
                .map_or(Part::Other, |name| {
                    Part::Endpoint(Endpoint {
                        heading,
                        name: name.trim(),
                        sections: Vec::new(),
                    })
                })
        };
        self.outline.parts.push(part);
        self.section = None;
        self.component = false;
        self.attached = true;
    }

    fn general(&self, heading: &Block, level: u8) -> bool {
        let name = heading.text.trim();
        match level {
            3 => !name.is_empty(),
            4 => name == self.labels.description,
            _ => false,
        }
    }

    fn section(&mut self, heading: &'a Block, level: u8) -> bool {
        let name = heading.text.trim();
        match level {
            3 => {
                self.open(heading);
                true
            }
            4 => {
                self.component = self.section == Some(SectionKind::Components);
                self.section.is_some_and(SectionKind::has_items) && !name.is_empty()
            }
            5 => self.component && name == self.labels.description,
            _ => false,
        }
    }

    fn open(&mut self, heading: &'a Block) {
        let kind = SectionKind::of(heading.text.trim(), self.labels);
        self.section = Some(kind);
        self.component = false;
        match self.outline.parts.last_mut() {
            None => self.outline.preamble.push(heading),
            Some(Part::Endpoint(endpoint)) if self.attached => {
                endpoint.sections.push(Section { heading, kind });
            }
            Some(Part::Endpoint(_) | Part::General(_) | Part::Other) => {}
        }
    }
}
