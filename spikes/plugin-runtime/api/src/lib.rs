pub type Point = [f64; 2];

#[derive(Clone, Copy)]
pub enum Role {
    OuterWall,
    InnerWall,
}

pub struct Path<'a> {
    pub inset_index: u32,
    pub role: Role,
    pub points: &'a [Point],
}

pub struct Layer<'a> {
    pub index: u32,
    pub z_bottom: f64,
    pub height: f64,
    pub paths: &'a [Path<'a>],
    pub options: &'a [ResolvedOption<'a>],
}

#[derive(Clone, Copy)]
pub enum OptionScope {
    Print,
    Printer,
    Filament,
}

#[derive(Clone, Copy)]
pub enum OptionKind {
    Boolean,
    Number { min: f64, max: f64 },
    Choice(&'static [&'static str]),
}

#[derive(Clone, Copy)]
pub enum Value<'a> {
    Boolean(bool),
    Number(f64),
    Choice(&'a str),
}

pub struct OptionDeclaration {
    pub key: &'static str,
    pub label: &'static str,
    pub scope: OptionScope,
    pub kind: OptionKind,
    pub default: Value<'static>,
}

pub struct ResolvedOption<'a> {
    pub key: &'a str,
    pub value: Value<'a>,
}

impl Layer<'_> {
    pub fn option(&self, key: &str) -> Option<Value<'_>> {
        self.options
            .iter()
            .find(|item| item.key == key)
            .map(|item| item.value)
    }
}

pub struct Edit {
    pub source: usize,
    pub z: f64,
    pub height: f64,
    pub flow_multiplier: f64,
}

pub trait Plugin {
    type Error;

    fn options(&self) -> &[OptionDeclaration];

    /// Return edits in print order. Repeating a source index copies that path.
    fn perimeters(&mut self, layer: Layer<'_>) -> Result<Vec<Edit>, Self::Error>;
    fn gcode_post_process(&mut self, input: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
