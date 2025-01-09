use async_std::path::PathBuf;
use anyhow::Error;

/// Indicates where a units script should be loaded from
#[derive(Clone, Debug)]
pub struct Location {
    path: PathBuf,
    /// Stores the original form of the path as provided by the user
    /// for error reporting purposes
    original_path: String,
}

impl Location {
    pub fn new(path: PathBuf) -> Self {
        let original_path = path.clone().to_string_lossy().to_string();
        Self { path, original_path }
    }

    pub fn from_str(path: &str) -> Self {
        let path = PathBuf::from(path);
        let original_path = path.clone().to_string_lossy().to_string();
        Self { path, original_path }
    }

    /// Remove the highest-level directory from the path and returns it
    /// None if there are no more directory components left in the path
    pub fn shift_dir(&mut self) -> Option<String> {
        let path_clone = self.path.clone();
        let mut comps = path_clone.iter().collect::<Vec<_>>();

        if comps.len() <= 1 {
            return None
        };

        Some(comps.remove(0).to_string_lossy().to_string())
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn get_path_string(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn err(&self, msg: &str) -> Error {
        anyhow::anyhow!("{}. at location: {}", msg, self.original_path)
    }

    /// Indicates which type of path the current current path component is pointing at
    pub fn get_type(&self) -> Type {
        if self.is_unitfile() {
            Type::UnitFile
        } else if self.is_dir() {
            Type::Dir
        } else {
            Type::Script
        }
    }

    fn is_unitfile(&self) -> bool {
        let comps = self.get_components();
        if comps.len() >= 1 {
            let next = comps.first().unwrap();
            if next.as_os_str().to_string_lossy().to_string().ends_with(".sysu") {
                return true
            }
        }
        return false
    }

    fn is_dir(&self) -> bool {
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            if metadata.is_dir() { return true }
        }

        false
    }

    fn get_components(&self) -> Vec<std::path::Component> {
        self.path.components().collect()
    }

    fn basename(&self) -> String {
        todo!("get path basename")
    }
}

pub enum Type {
    UnitFile,
    Dir,
    Script
}
