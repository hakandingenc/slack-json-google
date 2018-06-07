//! Callback id to url mappings

extern crate serde_json;
use std::{collections::HashMap, fmt, fs::File, io, path::Path};

/// Callback id to url mappings
#[derive(Default, PartialEq, Eq, Clone)]
pub struct HostMap {
    mappings: HashMap<String, String>,
}

impl HostMap {
    /// Creates a new HostMap from a JSON encoded file
    pub fn new_from_file(path: &Path) -> io::Result<Self> {
        let mapfile = File::open(path)?;
        let mappings: HashMap<String, String> = serde_json::from_reader(mapfile)?;
        Ok(HostMap { mappings })
    }

    /// Given a callback id, returns `Some` containing the
    /// corresponding url, or `None` if one doesn't exist
    pub fn resolve_callback(&self, id: &str) -> Option<String> {
        match self.mappings.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// Given a callback id and a url, inserts a new mapping
    /// to HostMap, or updates if the callback id already
    /// exists
    pub fn insert(&mut self, callback_id: String, url: String) {
        self.mappings.insert(callback_id, url);
    }

    /// Given a callback id, removes it from the HostMap,
    /// returning `Some` containing the corresponding url,
    /// or `None` if the callback id doesn't exits
    pub fn remove(&mut self, callback_id: &str) -> Option<String> {
        self.mappings.remove(callback_id)
    }
}

impl fmt::Debug for HostMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self.mappings)
    }
}
