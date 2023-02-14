//! Config file handler for Star Browser Utilities

#[macro_use]
extern crate tracing;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use ron::de;
use ron::ser::PrettyConfig;
use ron::ser::{self};
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::fs::{self};
use std::io::Write;

// This will keep the PrettyConfig consistent everywhere even if updated, and is
// convenient of course. What can go wrong?
macro_rules! pretty_config {
    () => {
        PrettyConfig::default().struct_names(true)
    };
}

// Again, keeping things consistent everywhere even if updated (though in this
// case, that's a terrible idea). I might have a problem when it comes to
// this...
pub const CONFIG_FILE: &str = "starb.ron";

/// Mutable, global access to Star Browser Utilities' config.
pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::new(Config::init()));

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {}
    }
}

impl Config {
    // Private `new` functions should be called `init`. Don't ask why!
    #[inline]
    #[instrument]
    fn init() -> Self {
        File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(CONFIG_FILE)
            .unwrap();

        trace!("Initializing `CONFIG`");

        let deser = fs::read_to_string(CONFIG_FILE).expect("Reading `CONFIG_FILE` failed");

        // Return `Default::default()` if deserialization fails
        de::from_str::<Config>(&deser).unwrap_or_default()
    }

    #[inline]
    #[instrument]
    pub fn save(&self) {
        let mut cfg = File::options()
            .write(true)
            .truncate(true)
            .open(CONFIG_FILE)
            .unwrap();

        trace!("Saving `CONFIG`");

        let ser =
            ser::to_string_pretty(self, pretty_config!()).expect("Serializing `CONFIG` failed");

        writeln!(cfg, "{ser}").unwrap();
    }
}
