use serde::Deserialize;
use serde::Serialize;

pub type ImportTable<'a> = Vec<Import<'a>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Import<'a> {
    name: &'a str,
    functions: Vec<usize>,
}

impl<'a> Import<'a> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn functions(&self) -> &Vec<usize> {
        &self.functions
    }
}
