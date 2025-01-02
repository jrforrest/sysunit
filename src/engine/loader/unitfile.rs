//! Unitfiles contain multiple units, separated by filename headers.
//! This module provides logic for loading units from them.

use futures::io::{AsyncRead, AsyncBufReadExt};
use futures::stream::StreamExt;
use async_std::io::BufReader;
use async_std::fs::File;
use async_std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;

use crate::parser::parse_unitfile_header;

#[derive(Debug)]
pub struct UnitFile {
    path: PathBuf,
    units: HashMap<String, String>,
}

impl UnitFile {
    pub async fn load(path: PathBuf) -> Result<Self> {
        let units = parse_defs(File::open(&path).await?).await?;
        Ok(Self { path, units })
    }

    pub async fn get(&self, name: &str) -> Option<String> {
        self.units.get(name).cloned()
    }

    pub fn display_path(&self) -> String {
        self.path.to_str().unwrap_or("<invalid unicode>").to_string()
    }
}

async fn parse_defs<R: AsyncRead + Unpin>(reader: R) -> Result<HashMap<String, String>> {
    // I should have used a nom parser for this probably but this works alright
    let bufread = BufReader::new(reader);

    let mut defs = HashMap::new();

    let mut current_name: Option<String> = None;
    let mut current_script = String::new();

    let mut lines = bufread.lines();
    while let Some(line_res) = lines.next().await {
        let line = line_res?;
        if let Ok(unit_name) = parse_unitfile_header(&line) {
            if let Some(name) = current_name {
                defs.insert(name, current_script);
            }
            current_name = Some(unit_name);
            current_script = String::new();
        } else {
            current_script.push_str(&format!("{}\n", line));
        }
    };

    if let Some(name) = current_name {
        defs.insert(name, current_script);
    }

    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task::block_on;

    #[test]
    fn test_parse_defs() {
        let input = r#"
            # [ Unit1 ]
            echo 'unit 1 script'

            # [ Unit2 ]
            echo 'unit 2 script'
        "#;

        let reader = input.as_bytes();
        let defs = block_on(parse_defs(reader)).unwrap();

        assert_eq!(defs.values().len(), 2);
        assert_eq!(defs.get("Unit1").unwrap().trim(), "echo 'unit 1 script'");
        assert_eq!(defs.get("Unit2").unwrap().trim(), "echo 'unit 2 script'");
    }
}
