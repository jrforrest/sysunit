//! Loads units from various sources

use std::sync::{Arc, Mutex};
use async_std::path::PathBuf;
use anyhow::{Result, anyhow};

mod location;
mod unitfile;
mod dir;

use location::Location;
use dir::Dir;
use unitfile::Unitfile;

/// Loads units from the sources with which this loader was intialized
///
#[derive(Clone)]
pub struct Loader {
    root_sources: Arc<Mutex<Vec<Source>>>,
}
impl Loader {
    pub fn from_search_paths(paths: Vec<PathBuf>) -> Self {
        let root_sources: Vec<Source> = paths
            .iter()
            .map(|p| Source::Dir(Dir::new(p.clone())))
            .collect();
        Self { root_sources: Arc::new(Mutex::new(root_sources)) }
    }

    /// Searches for a unit by path or name
    ///
    /// Provides interior mutability of sources 
    pub async fn search(&self, unit_name: &str) -> Result<String> {
        let mut sources = self.root_sources.lock().unwrap();

        let location = Location::from_str(unit_name);

        for source in sources.iter_mut() {
            match source.search(location.clone()).await {
                Ok(Some(script)) => return Ok(script),
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        };

        let root_sources = sources.iter().map(|s| s.display_path()).collect::<Vec<_>>().join(", ");
        Err(anyhow!("Could not find unit: {}.  Searched: {}", unit_name, root_sources))
    }
}

pub type UnitScript = String;
type SearchResult = Result<Option<UnitScript>>;

/// A source is a place from which units can be loaded
enum Source {
    Dir(Dir),
    Unitfile(Unitfile),
}

use futures::Future;
use std::pin::Pin;

impl Source {
    async fn search(&mut self, loc: Location) -> SearchResult {
        self.search_inner(loc).await
    }

    fn search_inner(&mut self, loc: Location) -> Pin<Box<dyn Future<Output = SearchResult> + '_>> {
        match self {
            Source::Dir(dir) => Box::pin(dir.search(loc)),
            Source::Unitfile(unitfile) => Box::pin(unitfile.search(loc)),
        }
    }

    fn display_path(&self) -> String {
        match self {
            Source::Dir(dir) => dir.display_path(),
            Source::Unitfile(unitfile) => unitfile.display_path(),
        }
    }
}
