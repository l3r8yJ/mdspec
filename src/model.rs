#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Block {
    pub kind: BlockKind,
    pub position: Position,
    pub end: usize,
    pub text: String,
}

#[derive(Debug)]
pub enum BlockKind {
    Paragraph,
    Heading { level: u8, anchor: String },
    Table(Vec<Vec<String>>),
    Code,
    Content,
    Definition,
}

#[derive(Debug)]
pub struct Definition {
    pub key: String,
    pub url: String,
    pub position: Position,
}

#[derive(Debug)]
pub struct Link {
    pub key: Option<String>,
    pub url: Option<String>,
    pub position: Position,
    pub table_of_contents: bool,
}

#[derive(Debug)]
pub struct NestedHeading {
    pub position: Position,
    pub anchor: String,
}

#[derive(Debug, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub definitions: Vec<Definition>,
    pub links: Vec<Link>,
    pub nested_headings: Vec<NestedHeading>,
}

impl Document {
    pub fn anchors(&self) -> impl Iterator<Item = &str> {
        self.blocks
            .iter()
            .filter_map(|block| match &block.kind {
                BlockKind::Heading { anchor, .. } => Some(anchor.as_str()),
                _ => None,
            })
            .chain(
                self.nested_headings
                    .iter()
                    .map(|heading| heading.anchor.as_str()),
            )
    }
}
