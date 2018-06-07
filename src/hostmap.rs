//! This module

extern crate serde_json;
use std::{collections::HashMap, fmt, fs::File, io, path::Path};

/// HOSTMAP STRUCT
#[derive(Default, PartialEq, Eq, Clone)]
pub struct HostMap {
    mappings: HashMap<String, String>,
}

impl HostMap {
    /// NAYSU METHOD
    pub fn new_from_file(path: &Path) -> io::Result<Self> {
        let mapfile = File::open(path)?;
        let mappings: HashMap<String, String> = serde_json::from_reader(mapfile)?;
        Ok(HostMap { mappings })
    }

    /// NAYSU METHOD
    pub fn resolve_callback(&self, id: &str) -> Option<String> {
        match self.mappings.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// NAYSU METHOD
    pub fn insert(&mut self, callback_id: String, url: String) {
        self.mappings.insert(callback_id, url);
    }

    /// NAYSU METHOD
    pub fn remove(&mut self, callback_id: &str) -> Option<String> {
        self.mappings.remove(callback_id)
    }
}

impl fmt::Debug for HostMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self.mappings)
    }
}
