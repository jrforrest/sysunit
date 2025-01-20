//! Loads units from various sources

use async_std::path::PathBuf;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};

mod unitfile;
use unitfile::UnitFile;

type NodeArc = Arc<Mutex<Node>>;
type NodeResult = Result<Option<NodeArc>>;

/// Finds and loads scripts from the set of search paths its instantiated with
pub struct Loader {
    search_paths: Vec<NodeArc>,
}

impl Loader {
    pub fn from_search_paths(search_paths: Vec<PathBuf>) -> Self {
        let search_paths = search_paths
            .into_iter()
            .map(|p| Node::Directory(Dir::new(p)).into() )
            .collect::<Vec<NodeArc>>();
        Self { search_paths }
    }

    pub async fn load(&self, loc: &str) -> Result<String> {
        match self.search(loc).await? {
            Some(node) => {
                let locked_node = node.lock().unwrap();
                match &*locked_node {
                    Node::Script(script) => Ok(script.clone()),
                    _ => Err(anyhow!("{} is not a unit script", loc)),
                }
            },
            None => Err(anyhow!("Could not find script at location: {}", loc)),
        }
    }

    async fn search(&self, loc: &str) -> NodeResult {
        for node in &self.search_paths {
            let path = PathBuf::from(loc);
            if let Some(node) = self.search_node(node.clone(), &path).await? {
                return Ok(Some(node));
            }
        }
        return Ok(None);
    }

    async fn search_node(&self, node: NodeArc, path: &PathBuf) -> NodeResult {
        let mut cur_node = node;

        for component in path.iter() {
            let name = component.to_str().map(String::from);
            cur_node = if let Some(name) = name {
                let mut locked_node = cur_node.lock().unwrap();
                if let Some(node) = locked_node.search(&name).await? {
                    node
                } else {
                    return Ok(None);
                }
            } else {
                let lossy_component = component.to_string_lossy();
                return Err(anyhow!("Could not parse component: {} in path: {}", lossy_component, display_path(&path)));
            }
        }

        Ok(Some(cur_node))
    }
}

/// Loading is modeled as a graph traversal. A Node can be a directory which
/// contains scripts and unitfiles, unitfiles, which contain scripts, and scripts
/// which should be terminal nodes.
#[derive(Debug)]
enum Node {
    Directory(Dir),
    UnitFile(UnitFile),
    Script(String),
}

impl Node {
    pub async fn search(&mut self, loc: &str) -> NodeResult {
        match self {
            Node::Directory(dir) => dir.search(loc).await,
            Node::UnitFile(uf) => get_script_from_unitfile(uf, loc).await,
            Node::Script(_) => todo!("errr script cant be searched"),
        }
    }
}

impl From<Node> for NodeArc {
    fn from(node: Node) -> Self {
        Arc::new(Mutex::new(node))
    }
}

fn node_res(node: Node) -> NodeResult {
    Ok(Some(node.into()))
}

#[derive(Debug)]
struct Dir {
    location: PathBuf,
    children: HashMap<String, NodeArc>,
}

impl Dir {
    pub fn new(location: PathBuf) -> Self {
        Self { location, children: HashMap::new(), }
    }

    pub async fn search(&mut self, name: &str) -> NodeResult {
        if let Some(node) = self.children.get(&name.to_string()) {
            return Ok(Some(node.clone()));
        }

        let path = self.location.join(name);

        if path.is_dir().await {
            return node_res(Node::Directory(Dir::new(path)))
        };

        if path.is_file().await {
            let ext = path.extension().map(|e| e.to_str()).unwrap_or(None);
            return match ext {
                Some("sysu") => {
                    let unitfile = UnitFile::load(path.clone()).await?;
                    node_res(Node::UnitFile(unitfile))
                },
                Some("sh") => {
                    let script = load_script(&path)?;
                    node_res(Node::Script(script))
                },
                _ => Err(anyhow!("Invalid file extension on file: {}", display_path(&path))),
            }
        }

        return Ok(None);
    }
}

fn load_script(path: &PathBuf) -> Result<String> {
    use std::fs;
    fs::read_to_string(path).map_err(|e| e.into())
}

fn display_path(path: &PathBuf) -> String {
    path.to_str().unwrap_or("<invalid unicode>").to_string()
}

async fn get_script_from_unitfile(uf: &mut UnitFile, loc: &str) -> NodeResult {
    match uf.get(loc).await {
        Some(script) => Ok(Some(Node::Script(script).into())),
        None => Err(anyhow!("Could not find script: {} in unitfile: {}", loc, uf.display_path())),
    }
}
