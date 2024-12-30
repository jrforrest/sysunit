//! Topological node sorter for dependency graphs
//!
//! Given a graph of nodes with dependencies, this module can resolve them
//! into a topological order.  Nodes are generic, and just need to implement
//! the ResolvableNode trait.  A loader must also be provided, which can lazily
//! load more nodes as needed.  This must satisfy the DependencyFetcher trait.
use anyhow::Result;

use std::fmt;
use std::error;
use core::hash::Hash;
use core::fmt::{Debug, Display};

pub trait ResolvableNode: Clone + Eq + PartialEq + Hash + Debug + Display + Sync + Send {
    fn get_id(&self) -> String;
}

pub trait DependencyFetcher<T: ResolvableNode> {
    async fn get_node_dependencies(&self, node: T) -> Result<Vec<T>>;
}

#[derive(Debug)]
pub struct CircularDependencyError {
    pub preceeding_nodes: Vec<String>,
    pub node_id: String,
}

impl fmt::Display for CircularDependencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Circular dependency detected: {}. stack: {}", self.node_id, self.preceeding_nodes.join(" -> "))
    }
}

impl error::Error for CircularDependencyError {}

enum NodeState {
    Visited,
    Visiting,
    Unvisited,
}

/// Resolves a dependency graph starting from the initial node.  Uses loader to lazily load
/// dependencies as graph traversal proceeds.
pub async fn resolve<'a, T, L>(initial_node: T, loader: &'a L) -> Result<Vec<T>> where
    T: ResolvableNode + 'a,
    L: DependencyFetcher<T>,
{
    use std::collections::HashMap;

    let mut visit_stack = vec![initial_node.clone()];
    let mut node_states = HashMap::new();
    let mut ordered_nodes = Vec::new();

    node_states.insert(initial_node.get_id(), NodeState::Unvisited);

    while let Some(node) = visit_stack.pop() {
        let state = node_states.get(&node.get_id()).unwrap();

        if matches!(state, NodeState::Visiting) {
            ordered_nodes.push(node.clone());
            node_states.insert(node.get_id(), NodeState::Visited);
            continue;
        }

        // Put the parent node back on the stack so we finish working on it,
        // after we traverse the dependencies
        node_states.insert(node.get_id(), NodeState::Visiting);
        visit_stack.push(node.clone());

        let dependencies = loader.get_node_dependencies(node.clone()).await?;

        for dep in dependencies {
            match node_states.get(&dep.get_id()) {
                // If we've already fully visited this node, it's been reached
                // in the dependency chain of another node and has already been
                // added to the ordered node list, so we don't need to do anything.
                Some(NodeState::Visited) => continue,
                // If we're reaching a node that is currently being visited in the
                // dependency chain of its children, we have a cycle
                Some(NodeState::Visiting) => {
                    let error = CircularDependencyError {
                        preceeding_nodes: ordered_nodes.iter().map(|node| node.get_id()).collect(),
                        node_id: dep.get_id(),
                    };

                    return Err(error.into());
                },
                // If this node has been loaded but not yet visited, we add it to the stack
                // so its children will also be loaded
                Some(NodeState::Unvisited) => {
                    node_states.insert(dep.get_id(), NodeState::Visiting);
                    visit_stack.push(dep);
                }
                // If this node is being lazy-loaded for the first time, we mark for an initial
                // visit
                None => {
                    node_states.insert(dep.get_id(), NodeState::Unvisited);
                    visit_stack.push(dep);
                }
            }
        }
    }

    Ok(ordered_nodes)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;
    use std::collections::HashMap;
    use futures::executor::block_on;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    struct Node {
        id: String,
        deps: Vec<String>
    }

    impl Display for Node {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.id)
        }
    }

    impl ResolvableNode for Node {
        fn get_id(&self) -> String {
            self.id.clone()
        }
    }

    struct NodeLoader {
        nodes: HashMap<String, Node>
    }

    impl NodeLoader {
        pub fn new() -> Self {
            Self {
                nodes: HashMap::new()
            }
        }

        pub fn add_node(&mut self, id: &str, deps: Vec<&str>) -> Node {
            let id = id.to_string();

            let node = Node {
                id: id.clone(),
                deps: deps.iter().map(|dep| dep.to_string()).collect(),
            };

            self.nodes.insert(id.clone(), node.clone());

            node
        }
    }

    impl DependencyFetcher<Node> for NodeLoader {
        async fn get_node_dependencies(&self, node: Node) -> Result<Vec<Node>> {
            node.deps.iter().map(|dep_id| {
                match self.nodes.get(dep_id) {
                    Some(dep) => Ok(dep.clone()),
                    None => Err(anyhow::anyhow!("Node not found"))
                }
            }).collect()
        }
    }

    #[test]
    fn test_basic() {
        let mut loader = NodeLoader::new();

        let top_node = loader.add_node("a", vec!["b", "c"]);
        loader.add_node("b", vec!["c"]);
        loader.add_node("c", vec![]);

        let result = block_on(resolve(top_node, &mut loader)).unwrap();

        let expected = vec!["c", "b", "a"];
        let result: Vec<String> = result.iter().map(|node| node.id.clone()).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cycle() {
        let mut loader = NodeLoader::new();

        let top_node = loader.add_node("a", vec!["b", "c"]);
        loader.add_node("b", vec!["c"]);
        loader.add_node("c", vec!["a"]);

        assert!(block_on(resolve(top_node, &mut loader)).is_err());
    }

    #[test]
    fn test_complex() {
        let mut loader = NodeLoader::new();

        let top_node = loader.add_node("a", vec!["b", "c"]);
        loader.add_node("b", vec!["c", "d"]);
        loader.add_node("c", vec!["d"]);
        loader.add_node("d", vec![]);

        let result = block_on(resolve(top_node, &mut loader)).unwrap();

        let expected = vec!["d", "c", "b", "a"];
        let result: Vec<String> = result.iter().map(|node| node.id.clone()).collect();
        assert_eq!(result, expected);
    }
}
