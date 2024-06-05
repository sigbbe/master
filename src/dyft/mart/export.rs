use super::structure::MartIndex;
use crate::config::master_result_dir;
use crate::dyft::*;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TrieNode {
    node_id: MartNodeId,
    node_type: MartNodeTypes,
    children: Vec<(MartByteLabel, TrieNode)>,
}

impl TrieNode {
    pub fn new(node_id: MartNodeId, node_type: MartNodeTypes) -> Self {
        Self {
            node_id,
            node_type,
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self, node: TrieNode, label: MartByteLabel) {
        self.children.push((label, node));
    }

    pub fn id(&self) -> MartNodeId {
        self.node_id
    }

    pub fn ntype(&self) -> MartNodeTypes {
        self.node_type
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MartTrieExport {
    root: Option<TrieNode>,
}

pub trait MartExporter {
    fn export(self) -> MartTrieExport;
}

impl MartTrieExport {
    pub fn new() -> Self {
        MartTrieExport { root: None }
    }

    pub fn insert(&mut self, ptr: &MartPointer, node: TrieNode) {
        if let Some(root) = &mut self.root {
            // set of visited nodes
            let mut visited_nodes = HashSet::<MartPointer>::new();

            // queue for BFS traversal
            let mut queue = VecDeque::from_iter([root]);

            while let Some(current_node) = queue.pop_front() {
                let current_ptr = MartPointer::new(current_node.node_id, current_node.node_type);
                if visited_nodes.contains(&current_ptr)
                    || current_ptr.is_null_ptr()
                    || current_ptr.is_leaf()
                {
                    continue;
                }
                visited_nodes.insert(current_ptr);

                if let Some((_label, parent)) = current_node
                    .children
                    .iter_mut()
                    .find(|(_, child)| MartPointer::new(child.id(), child.ntype()) == *ptr)
                {
                    for (label, node) in node.children.into_iter() {
                        parent.insert(node, label);
                    }
                    return;
                } else {
                    queue.extend(current_node.children.iter_mut().map(|(_, child)| child));
                }
            }
        } else {
            self.root = Some(node);
        }
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = master_result_dir()
            .map(|p| p.join(path.as_ref()))
            .unwrap_or(path.as_ref().to_path_buf());
        let mut writer = std::fs::File::create(path)?;
        let buf = rmp_serde::to_vec(self)?;
        writer.write_all(&buf).map_err(|e| anyhow::anyhow!(e))
    }
}

impl<'a> MartExporter for MartIndex<'a> {
    fn export(self) -> MartTrieExport {
        // get the root node
        // let root_ptr = MartPointer::from();

        // set of visited nodes
        let mut visited_nodes = HashSet::<MartPointer>::new();

        // queue for BFS traversal
        let mut queue = VecDeque::<MartPointer>::new();

        // Create a new MartTrieExport
        let mut mart_export = MartTrieExport::new();

        // Add the root node to the queue
        queue.push_back(*self.root());

        // Perform BFS traversal of the trie structure
        while let Some(ptr) = queue.pop_front() {
            // Check if the node has already been visited
            if !(visited_nodes.contains(&ptr) || ptr.is_null_ptr() || ptr.is_leaf()) {
                // mark the current node as visited
                visited_nodes.insert(ptr);

                // Get the children of the current node
                let children = self.perform_find_children(&ptr);

                // Create a new TrieNode for the current node
                let mut trie_node = TrieNode::new(ptr.nid(), ptr.ntype());

                // iterate through the children and add them to the TrieNode
                for MartEdge { label, ptr } in children.into_iter() {
                    trie_node.insert(TrieNode::new(ptr.nid(), ptr.ntype()), label);
                    // Add the child to the queue for further traversal
                    queue.push_back(ptr);
                }
                // Insert the TrieNode into the MartTrieExport
                mart_export.insert(&ptr, trie_node);
            }
        }
        mart_export
    }
}
