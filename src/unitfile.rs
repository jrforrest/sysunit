//! Unitfiles contain multiple units, separated by filename headers.
//! This module provides logic for loading units from them.

use async_std::path::PathBuf;
use async_std::io::BufReader;
use async_std::fs::File;
use std::collections::HashMap;
use futures::io::AsyncBufReadExt;
use anyhow::Error;

struct UnitFile {
    path: PathBuf,
    units: HashMap<String, UnitDefinition>,
}

impl UnitFile {
    pub async fn init(path: PathBuf) -> Result<Self, Error> {
        let text = self.get_text().await?;
        let units = self.parse_units(&text)?;

        Ok(Self { path, units })
    }

    async fn get_text(&self) -> String {
        let mut file = File::open(&self.path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(contents)
    }

}

type UnitDefinitions = HashMap<String, String>;

fn parse_defs(text: &str) -> UnitDefinitions {
    let mut defs = HashMap::new();
    let mut current_name = None;
    for line in text.lines() {
        if let Some(name) = try_parse_header(line) {
            current_name = Some(name);
        } else if let Some(name) = current_name {
            defs.entry(name).or_insert_with(String::new).push_str(line);
        }
    }
    defs
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_defs() {
        let text = r#"
          # [ Unit 1 ] 
          echo -n "Hi world!"
          
          # [ Unit 2 ]

          echo "booyeaaah"
        "#;

        let defs = parse_defs(text);
        assert_eq!(defs.len(), 2);
    }
}
