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
    Heading(u8),
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

#[derive(Debug, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub definitions: Vec<Definition>,
    pub links: Vec<Link>,
    pub headings: Vec<(String, Position)>,
    pub nested_headings: Vec<Position>,
}
