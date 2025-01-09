use async_std::path::PathBuf;
use std::collections::HashMap;

use anyhow::Result;

use super::{
    location::{Location, Type as LocationType},
    unitfile::Unitfile,
    {Source, UnitScript, SearchResult},
};

/// Represents a directory from which units can be loaded
pub struct Dir {
    full_path: PathBuf,
    sources: HashMap<String, Source>,
}

impl Dir {
    pub fn new(full_path: PathBuf) -> Self {
        Self {
            full_path,
            sources: HashMap::new(),
        }
    }
}

impl Dir {
    pub fn display_path(&self) -> String {
        self.full_path.to_string_lossy().to_string()
    }

    pub async fn search(&mut self, loc: Location) -> Result<Option<UnitScript>> {
        dbg!(&loc);
        use LocationType::*;
        match &loc.get_type() {
            UnitFile | Dir => self.search_source(loc).await,
            LocationType::Script => self.load_script(loc).await,
        }
    }

    async fn search_source(&mut self, mut loc: Location) -> Result<Option<UnitScript>> {
        let filename = loc.shift_dir().ok_or_else(|| loc.err("is a directory, not a script"))?;

        if let Some(source) = self.sources.get_mut(&filename) {
            return source.search(loc).await;
        } else {
            let mut source = match loc.get_type() {
                LocationType::UnitFile => {
                    dbg!(&loc, &filename);
                    let unitfile_path = self.full_path.join(&filename);
                    Source::Unitfile(Unitfile::load(unitfile_path).await?)
                },
                LocationType::Dir => Source::Dir(Dir::new(loc.get_path())),
                _ => unreachable!(),
            };
            let unit = source.search(loc).await?;
            self.sources.insert(filename, source);
            Ok(unit)
        }
    }

    async fn load_script(&self, loc: Location) -> SearchResult {
        let script = async_std::fs::read_to_string(loc.get_path()).await?;
        Ok(Some(script))
    }
}
