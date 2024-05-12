use crate::node::Node;
use std::io::Read;

#[derive(Debug)]
pub struct NodeData {
    root_node: Node
}

impl NodeData {
    pub fn read(reader: &mut impl Read) -> std::io::Result<Self> {

        let root_node = Node::read(reader)?;

        return Ok(NodeData {
            root_node
        })
    }
}