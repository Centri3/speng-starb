use super::Patch;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Compact {
    general: General,
    compatibility: Compatibility,
    start: Option<Start>,
    toggle: Option<Toggle>,
}

impl Patch for Compact {
    fn start(&self) {}

    fn toggle(&self) {
        let update = match self.toggle.as_ref() {
            Some(up) => up,
            None => todo!(),
        };

        for opcode in update.opcodes.clone() {}
    }
}

#[derive(Deserialize)]
struct General {
    name: String,
    description: String,
    category: Category,
}

#[derive(Deserialize)]
enum Category {
    #[serde(rename = "general")]
    General,
    #[serde(rename = "feature")]
    Feature,
    #[serde(rename = "bugfix")]
    Bugfix,
    #[serde(rename = "filter")]
    Filter,
    #[serde(rename = "qol")]
    QoL,
    #[serde(rename = "other")]
    Other,
}

#[derive(Deserialize)]
struct Compatibility {
    versions: [Versions; 5usize],
    experimental: bool,
}

#[derive(Deserialize)]
enum Versions {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "deny")]
    Deny,
}

#[derive(Deserialize)]
struct Start {
    opcodes: Vec<Opcodes>,
}

#[derive(Deserialize)]
struct Toggle {
    opcodes: Vec<Opcodes>,
}

#[derive(Clone, Deserialize)]
struct Opcodes {
    #[serde(rename = "addr")]
    address: usize,
    on: Vec<u8>,
    off: Vec<u8>,
}
