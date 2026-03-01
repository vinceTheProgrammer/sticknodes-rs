use core::cell::RefCell;

use core2::io::Cursor;
use core2::io::Read;
use core2::io::Seek;
use hashbrown::HashMap;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::visit::EdgeRef;
use petgraph::visit::VisitMap;
use petgraph::visit::Visitable;
use petgraph::Direction;
use petgraph::Graph;
extern crate alloc;
use alloc::{format, rc::Rc, vec, vec::Vec};
use serde::Deserialize;
use serde::Serialize;

use crate::serialization::read::read_stickfigure;
use crate::serialization::write::write_stickfigure;
use crate::structs::node::*;
use crate::Color;
use crate::LibraryError;
use crate::Polyfill;
use crate::StickfigureError;

const NODE_LIMIT: usize = 400;

#[cfg(feature = "tryreadanyway")]
pub const SUPPORTED_APP_VERSION: i32 = 9999999;

#[cfg(feature = "tryreadanyway")]
pub const SUPPORTED_APP_BUILD: i32 = 9999999;


#[cfg(not(feature = "tryreadanyway"))]
pub const SUPPORTED_APP_VERSION: i32 = 423;

#[cfg(not(feature = "tryreadanyway"))]
pub const SUPPORTED_APP_BUILD: i32 = 72;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Default)]
#[serde(transparent)]
pub struct DrawOrderIndex(pub i32);

pub struct IWillNotAbuseUnlimitedNodes(pub bool);

impl Into<DrawOrderIndex> for i32 {
    fn into(self) -> DrawOrderIndex {
        DrawOrderIndex(self)
    }
}

pub(crate) struct NodeIndices {
    draw_index: DrawOrderIndex,
    node_index: NodeIndex,
}

#[derive(Debug, Clone)]
pub struct Stickfigure {
    pub version: i32,
    pub build: i32,
    pub scale: f32,
    pub color: Color,
    pub nodes: Graph<Rc<RefCell<Node>>, ()>,
    pub polyfills: Vec<Rc<RefCell<Polyfill>>>,
    polyfill_anchors: Vec<DrawOrderIndex>,
    next_draw_index: DrawOrderIndex,
    draw_index_map: HashMap<NodeIndex, DrawOrderIndex>,
    node_index_map: HashMap<DrawOrderIndex, NodeIndex>,
    is_node_limit_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableStickfigure {
    pub version: i32,
    pub build: i32,
    pub scale: f32,
    pub color: Color,
    pub nodes: Vec<SerializableNode>,
    pub polyfills: Vec<Polyfill>,
}

impl Default for Stickfigure {
    fn default() -> Self {
        Self {
            version: SUPPORTED_APP_VERSION,
            build: SUPPORTED_APP_BUILD,
            scale: 1.0,
            color: Color::default(),
            nodes: Graph::new(),
            polyfills: Vec::new(),
            next_draw_index: DrawOrderIndex(0),
            draw_index_map: HashMap::new(),
            node_index_map: HashMap::new(),
            polyfill_anchors: Vec::new(),
            is_node_limit_enabled: true,
        }
    }
}

/// Public Methods
impl Stickfigure {
    /// Creates a new, empty `Stickfigure`, set to latest version and build.
    pub fn new() -> Self {
        let mut stickfigure = Stickfigure::default();
        stickfigure.add_root_node();

        stickfigure
    }

    /// Creates a new, empty `Stickfigure`, set to the specified version and build.
    pub fn from_version_and_build(version: i32, build: i32) -> Result<Self, LibraryError> {
        if version > Stickfigure::default().version {
            return Err(LibraryError::UnsupportedVersion(version));
        } else if version == Stickfigure::default().version && build > Stickfigure::default().build
        {
            return Err(LibraryError::UnsupportedBuild(version, build));
        }

        let mut stickfigure = Stickfigure::new();
        stickfigure.version = version;
        stickfigure.build = build;
        stickfigure.add_root_node();

        Ok(stickfigure)
    }

    pub fn to_serializable(&self) -> SerializableStickfigure {
        let root_node =self.get_node(DrawOrderIndex(0)).expect("Possibly an internal bug, but please check if the stickfigure you're trying to serialize has its root node draw order index set as 0. If it is, this is probably a library bug. If it is not 0, then that might be on your end :p");
        let mut nodes = Vec::new();
        let serializable_nodes = root_node.borrow().build_serializable_tree(
            &self.nodes,
            self.node_index_from_draw_order(root_node.borrow().draw_order_index),
        );
        let serializable_polyfills = self
            .polyfills
            .iter()
            .map(|poly| poly.borrow().clone())
            .collect();
        nodes.push(serializable_nodes);
        SerializableStickfigure {
            version: self.version,
            build: self.build,
            scale: self.scale,
            color: self.color,
            nodes: nodes,
            polyfills: serializable_polyfills,
        }
    }

    pub fn set_is_node_limit_enabled(
        &mut self,
        is_enabled: bool,
        agree: IWillNotAbuseUnlimitedNodes,
    ) {
        if agree.0 {
            self.is_node_limit_enabled = is_enabled;
        }
    }

    pub(crate) fn add_root_node(&mut self) {
        let rc_node = Rc::new(RefCell::new(Node::new()));
        let draw_index = DrawOrderIndex(0);

        Node::update(&rc_node, |n| {
            n.node_type = NodeType::RootNode;
            n.draw_order_index = draw_index;
        });

        let node_index = self.nodes.add_node(Rc::clone(&rc_node));

        self.remap_draw_index(node_index, draw_index);
    }

    /// Creates a new `Stickfigure` from raw bytes of a `.nodes` file.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, LibraryError> {
        let mut reader = Cursor::new(bytes);

        let stickfigure = read_stickfigure(&mut reader)?;
        Ok(stickfigure)
    }

    /// Get raw bytes of a `.nodes` file from an existing `Stickfigure`.
    pub fn to_bytes(&self) -> Result<Vec<u8>, LibraryError> {
        let mut byte_vec = Vec::new();

        byte_vec.append(&mut write_stickfigure(self)?);

        Ok(byte_vec)
    }

    /// Adds a new node to the stickfigure.
    ///
    /// The node is given a unique `DrawOrderIndex`.
    ///
    /// # Parameters
    ///
    /// * `node` - The `Node` to add to the `Stickfigure`.
    /// * `parent_draw_index` - The `DrawOrderIndex` of the node to add this node as a child of
    ///
    /// # Returns
    ///
    /// Result of `DrawOrderIndex` of the newly added node.
    pub fn add_node(
        &mut self,
        node: Node,
        parent_draw_index: DrawOrderIndex,
    ) -> Result<DrawOrderIndex, StickfigureError> {
        self.check_if_can_add_node(1)?;

        let rc_node = Rc::new(RefCell::new(node));
        let draw_index = self.get_next_draw_index();

        Node::update(&rc_node, |n| {
            n.draw_order_index = draw_index;
        });

        let node_index = self.nodes.add_node(rc_node);

        self.remap_draw_index(node_index, draw_index);

        self.add_edge(parent_draw_index, draw_index);

        Ok(draw_index)
    }

    pub fn add_node_at_index(
        &mut self,
        node: Node,
        parent_draw_index: DrawOrderIndex,
        draw_index: DrawOrderIndex,
    ) -> Result<DrawOrderIndex, StickfigureError> {
        let temp_draw_index = self.add_node(node, parent_draw_index)?;

        self.change_draw_index(temp_draw_index, draw_index)?;

        Ok(draw_index)
    }

    pub fn change_draw_index(
        &mut self,
        draw_index: DrawOrderIndex,
        new_draw_index: DrawOrderIndex,
    ) -> Result<(), StickfigureError> {
        if draw_index == new_draw_index {
            return Ok(());
        }

        if !self.node_index_map.contains_key(&draw_index) {
            return Err(StickfigureError::InvalidDrawIndex(
                draw_index.0,
                format!("Not attempting to change any indices."),
            ));
        }

        let node_index = self.node_index_from_draw_order(draw_index);

        if self.node_index_map.contains_key(&new_draw_index) {
            let mut affected_nodes: Vec<NodeIndices> = self
                .draw_index_map
                .iter()
                .filter(|(_, &draw_index)| draw_index.0 >= new_draw_index.0)
                .map(|(&node_index, &draw_index)| NodeIndices {
                    node_index,
                    draw_index,
                })
                .collect();

            affected_nodes.sort_by_key(|node_indices| node_indices.draw_index.0);

            for indices in affected_nodes.iter_mut() {
                let draw_index_ = {
                    let node = self.nodes.node_weight(indices.node_index).expect("This node index was retrieved from the node_index_map, so it should be valid, but apparently is not - bug in library logic");

                    Node::update(&node, |n| {
                        n.draw_order_index.0 += 1;
                    });

                    node.borrow().draw_order_index
                };

                self.draw_index_map.insert(indices.node_index, draw_index_);
                self.node_index_map.insert(draw_index_, indices.node_index);
            }
        }

        let node = self.nodes.node_weight(node_index).expect("Earlier in this method call the draw order index was evaluated to be valid, which means its associated node index must exist, but apparently does not - bug in library logic");
        Node::update(&node, |n| {
            n.draw_order_index = new_draw_index;
        });

        self.remap_draw_index(node_index, new_draw_index);
        self.compact_draw_indices();

        Ok(())
    }

    /// Gets a reference to a node.
    ///
    /// # Parameters
    ///
    /// * `draw_index` - The `DrawOrderIndex` of the node.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the node if it exists.
    pub fn get_node(&self, draw_index: DrawOrderIndex) -> Option<&Rc<RefCell<Node>>> {
        if !self.node_index_map.contains_key(&draw_index) {
            return None;
        }
        let node_index = self.node_index_from_draw_order(draw_index);
        self.nodes.node_weight(node_index)
    }

    // Gets a mutable reference to a node.
    //
    // # Parameters
    //
    // * `draw_index` - The `DrawOrderIndex` of the node.
    //
    // # Returns
    //
    // An `Option` containing a mutable reference to the node if it exists.
    // pub fn get_node_mut(&mut self, draw_index: DrawOrderIndex) -> Option<&mut Node> {
    //     if !self.node_index_map.contains_key(&draw_index) {
    //         return None;
    //     }
    //     let node_index = self.node_index_from_draw_order(draw_index);
    //     self.nodes.node_weight_mut(node_index)
    // }

    /// Get the `DrawOrderIndex` of the direct parent of the `Node` at the specified `DrawOrderIndex`.
    pub fn get_parent(&self, draw_index: DrawOrderIndex) -> Option<DrawOrderIndex> {
        let child_node_index = self.node_index_from_draw_order(draw_index);

        if let Some(parent_node_index) = self
            .nodes
            .neighbors_directed(child_node_index, petgraph::Direction::Incoming)
            .next()
        {
            let parent_draw_index = self.draw_order_from_node_index(parent_node_index);

            Some(parent_draw_index)
        } else {
            None
        }
    }

    /// Get a `Vec<DrawOrderIndex>` containing all `DrawOrderIndex`s of `Node`s that are direct children of the `Node` at the specified `DrawOrderIndex`.
    pub fn get_children(&self, parent_draw_index: DrawOrderIndex) -> Vec<DrawOrderIndex> {
        let parent_node_index = self.node_index_from_draw_order(parent_draw_index);

        let child_node_indices: Vec<NodeIndex> = self
            .nodes
            .neighbors_directed(parent_node_index, petgraph::Direction::Outgoing)
            .collect();

        let child_draw_indices = self.draw_order_indices_from_node_indices(&child_node_indices);

        child_draw_indices
    }

    pub fn get_children_recursive(&self, draw_index: DrawOrderIndex) -> Vec<DrawOrderIndex> {
        let node_index = self.node_index_from_draw_order(draw_index);
        let mut children = Vec::new();
        let mut dfs = Dfs::new(&self.nodes, node_index);

        while let Some(nx) = dfs.next(&self.nodes) {
            if nx != node_index {
                children.push(nx);
            }
        }

        self.draw_order_indices_from_node_indices(&children)
    }

    pub fn get_parents_recursive(&self, draw_index: DrawOrderIndex) -> Vec<DrawOrderIndex> {
        let start = self.node_index_from_draw_order(draw_index);
        let mut ancestors = Vec::new();

        // Use our own DFS stack for incoming edges
        let mut stack = vec![start];
        let mut visited = self.nodes.visit_map();

        while let Some(node) = stack.pop() {
            if !visited.visit(node) {
                continue;
            }

            for neighbor in self.nodes.neighbors_directed(node, Direction::Incoming) {
                if visited.is_visited(&neighbor) {
                    continue;
                }

                ancestors.push(neighbor);
                stack.push(neighbor);
            }
        }

        self.draw_order_indices_from_node_indices(&ancestors)
    }

    pub fn get_all_node_indices(&self) -> Vec<DrawOrderIndex> {
        let indices: Vec<NodeIndex> = self.nodes.node_indices().collect();

        self.draw_order_indices_from_node_indices(&indices)
    }

    /// Gets the sibling nodes of a given node.
    ///
    /// Sibling nodes are nodes that share the same parent.
    ///
    /// # Parameters
    ///
    /// * `draw_index` - The `DrawOrderIndex` of the node whose siblings are to be found.
    ///
    /// # Returns
    ///
    /// A `Vec` of `DrawOrderIndex` representing the siblings of the given node.
    pub fn get_siblings(&self, draw_index: DrawOrderIndex) -> Vec<DrawOrderIndex> {
        let mut siblings = Vec::new();
        let mut parents = Vec::new();

        let node_index = self.node_index_from_draw_order(draw_index);

        for edge in self.nodes.edges_directed(node_index, petgraph::Incoming) {
            parents.push(edge.source());
        }

        for parent in parents {
            for edge in self.nodes.edges_directed(parent, petgraph::Outgoing) {
                let child = edge.target();
                if child != node_index {
                    siblings.push(child);
                }
            }
        }

        let siblings_draw_indices = self.draw_order_indices_from_node_indices(&siblings);

        siblings_draw_indices
    }

    pub fn get_nodes_with_property<F>(&self, mut predicate: F) -> Vec<DrawOrderIndex>
    where
        F: FnMut(&Rc<RefCell<Node>>) -> bool,
    {
        let node_indices: Vec<NodeIndex> = self
            .nodes
            .node_indices()
            .filter(|&i| {
                if let Some(node_data) = self.nodes.node_weight(i) {
                    predicate(node_data)
                } else {
                    false
                }
            })
            .collect();

        let draw_indices = self.draw_order_indices_from_node_indices(&node_indices);

        draw_indices
    }

    pub fn update_nodes_with_property<F>(&mut self, mut predicate: F, mut updater: F)
    where
        F: FnMut(&Rc<RefCell<Node>>),
    {
        for node_index in self.nodes.node_indices() {
            if let Some(node_data) = self.nodes.node_weight(node_index) {
                predicate(node_data);
                updater(node_data);
            }
        }
    }

    pub fn remove_node(&mut self, draw_index: DrawOrderIndex) -> Result<(), StickfigureError> {
        if !self.node_index_map.contains_key(&draw_index) {
            return Err(StickfigureError::InvalidDrawIndex(
                draw_index.0,
                format!("Cancelling node removal."),
            ));
        }
        let node_index = self.node_index_from_draw_order(draw_index);

        if let Some(parent_draw_index) = self.get_parent(draw_index) {
            let child_draw_indices: Vec<DrawOrderIndex> = self.get_children(draw_index);

            for child_draw_index in child_draw_indices.iter() {
                self.add_edge(parent_draw_index, *child_draw_index);
            }
        }

        self.nodes.remove_node(node_index);
        self.compact_draw_indices();

        Ok(())
    }

    pub fn add_polyfill(&mut self, polyfill: Polyfill) -> DrawOrderIndex {
        let rc_polyfill = Rc::new(RefCell::new(polyfill));
        let draw_index = { rc_polyfill.borrow().anchor_node_draw_index };
        self.polyfill_anchors
            .push(rc_polyfill.borrow().anchor_node_draw_index);
        self.polyfills.push(rc_polyfill);
        draw_index
    }

    pub fn get_polyfill(&self, draw_index: DrawOrderIndex) -> Option<Rc<RefCell<Polyfill>>> {
        if let Some(polyfill) = self
            .polyfills
            .iter()
            .find(|poly| poly.borrow().anchor_node_draw_index == draw_index)
        {
            Some(Rc::clone(polyfill))
        } else {
            None
        }
    }

    pub fn remove_polyfill(
        &mut self,
        anchor_draw_order: DrawOrderIndex,
    ) -> Result<(), StickfigureError> {
        let rc_polyfills: Vec<Rc<RefCell<Polyfill>>> = self
            .polyfills
            .iter()
            .map(|rc_poly| Rc::clone(rc_poly))
            .filter(|polyfill| polyfill.borrow().anchor_node_draw_index != anchor_draw_order)
            .collect();

        let anchor_to_remove_index = self
            .polyfill_anchors
            .iter()
            .position(|el| *el == anchor_draw_order);

        match anchor_to_remove_index {
            Some(index) => {
                self.polyfill_anchors.remove(index);
            },
            None => {
                return Err(StickfigureError::InvalidDrawIndex(anchor_draw_order.0, format!("Anchor node draw index not found in internal polyfill_anchors Vec. Likely some kind of logical error. Cancelling polyfill removal.")))
            },
        }

        self.polyfills = rc_polyfills;
        Ok(())
    }

    /// Returns true if all provided DrawOrderIndices exist in the node_index_map.
    pub fn all_draw_indices_exist(&self, indices: &[DrawOrderIndex]) -> bool {
        indices
            .iter()
            .all(|index| self.node_index_map.contains_key(index))
    }

    /// Returns true if provided DrawOrderIndex exists in the node_index_map.
    pub fn draw_index_exists(&self, draw_index: DrawOrderIndex) -> bool {
        self.node_index_map.contains_key(&draw_index)
    }

    pub fn draw_index_is_polyfill_anchor(&self, draw_index: DrawOrderIndex) -> bool {
        self.polyfill_anchors.contains(&draw_index)
    }

    /// Returns a Vec of DrawOrderIndices that do not exist in node_index_map.
    pub fn missing_draw_indices(&self, indices: &[DrawOrderIndex]) -> Vec<DrawOrderIndex> {
        indices
            .iter()
            .filter(|index| !self.node_index_map.contains_key(*index))
            .cloned()
            .collect()
    }
}

/// Private Methods
impl Stickfigure {
    pub(crate) fn add_node_at_unique_index(
        &mut self,
        node: Node,
        parent_draw_index: DrawOrderIndex,
        draw_index: DrawOrderIndex,
    ) -> Result<NodeIndices, StickfigureError> {
        if self.node_index_map.contains_key(&draw_index) {
            return Err(StickfigureError::OccupiedDrawIndex(draw_index.0 ,format!("add_node_at_unique_index will not attempt to shift indices. This is likely being called while reading a stickfigure file. If that's the case, the file likely has duplicate draw order indices. If that's not the case or the file has all unique draw order indices, then there's a bug with the library.")).into());
        }

        let rc_node = Rc::new(RefCell::new(node));

        Node::update(&rc_node, |n| {
            n.draw_order_index = draw_index;
        });

        let node_index = self.nodes.add_node(rc_node);

        self.remap_draw_index(node_index, draw_index);

        self.add_edge(parent_draw_index, draw_index);

        Ok(NodeIndices {
            draw_index,
            node_index,
        })
    }

    pub(crate) fn add_edge(
        &mut self,
        parent_draw_index: DrawOrderIndex,
        child_draw_index: DrawOrderIndex,
    ) {
        let parent_node_index = self.node_index_from_draw_order(parent_draw_index);
        let child_node_index = self.node_index_from_draw_order(child_draw_index);

        self.nodes.add_edge(parent_node_index, child_node_index, ());
    }

    /// Converts NodeIndex to DrawOrderIndex in O(1) time
    pub(crate) fn draw_order_from_node_index(&self, node_index: NodeIndex) -> DrawOrderIndex {
        self.draw_index_map
            .get(&node_index)
            .copied()
            .expect("NodeIndex missing DrawOrderIndex — bug in Stickfigure logic")
    }

    pub(crate) fn draw_order_indices_from_node_indices(
        &self,
        node_indices: &[NodeIndex],
    ) -> Vec<DrawOrderIndex> {
        node_indices
            .iter()
            .map(|ni| self.draw_order_from_node_index(*ni))
            .collect()
    }

    /// Converts DrawOrderIndex to NodeIndex in O(1) time
    pub(crate) fn node_index_from_draw_order(&self, draw_index: DrawOrderIndex) -> NodeIndex {
        self.node_index_map
            .get(&draw_index)
            .copied()
            .expect("NodeIndex missing DrawOrderIndex — bug in Stickfigure logic")
    }

    pub(crate) fn node_indices_from_draw_order_indices(
        &self,
        draw_indices: &[DrawOrderIndex],
    ) -> Vec<NodeIndex> {
        draw_indices
            .iter()
            .map(|di| self.node_index_from_draw_order(*di))
            .collect()
    }

    fn insert_new_indices(&mut self, node_indices: NodeIndices) {
        if self.draw_index_map.contains_key(&node_indices.node_index)
            || self.node_index_map.contains_key(&node_indices.draw_index)
        {
            panic!("Library is attempting to insert a new index pair that already exists - this is a bug with the library")
        }

        self.draw_index_map
            .insert(node_indices.node_index, node_indices.draw_index);
        self.node_index_map
            .insert(node_indices.draw_index, node_indices.node_index);
    }

    fn get_next_draw_index(&mut self) -> DrawOrderIndex {
        while self.node_index_map.contains_key(&self.next_draw_index) {
            self.next_draw_index.0 += 1;
        }

        self.next_draw_index
    }

    fn remap_draw_index(&mut self, node_index: NodeIndex, draw_index: DrawOrderIndex) {
        self.draw_index_map.insert(node_index, draw_index);
        self.node_index_map.insert(draw_index, node_index);
    }

    fn compact_draw_indices(&mut self) {
        // Get all current mappings, sorted by current draw index
        let mut indexed_nodes: Vec<(DrawOrderIndex, NodeIndex)> = self
            .node_index_map
            .iter()
            .map(|(draw_index, node_index)| (*draw_index, *node_index))
            .collect();

        indexed_nodes.sort_by_key(|(draw_index, _)| *draw_index);

        // Clear existing maps
        self.draw_index_map.clear();
        self.node_index_map.clear();

        // Reassign contiguous draw indices starting from 0
        for (new_draw_index, (_, node_index)) in indexed_nodes.into_iter().enumerate() {
            let new_draw_index = DrawOrderIndex(new_draw_index as i32);
            self.remap_draw_index(node_index, new_draw_index);
        }

        // Update next_draw_index to be one past the highest used
        self.next_draw_index = DrawOrderIndex(self.draw_index_map.len() as i32);
    }

    fn check_if_can_add_node(
        &self,
        number_of_nodes_being_added: usize,
    ) -> Result<(), StickfigureError> {
        let node_count = self.nodes.node_count();
        if self.is_node_limit_enabled {
            if node_count >= NODE_LIMIT {
                return Err(StickfigureError::NodeLimitError(
                    number_of_nodes_being_added,
                    node_count,
                    NODE_LIMIT,
                ));
            }
        }
        Ok(())
    }
}
