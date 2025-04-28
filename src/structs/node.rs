use petgraph::graph::NodeIndex;
use petgraph::Graph;
use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;

use super::stickfigure::DrawOrderIndex;

use core::cell::RefCell;
extern crate alloc;
use alloc::{rc::Rc, vec::Vec};

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub(crate) draw_order_index: DrawOrderIndex,
    pub is_static: bool,
    pub is_stretchy: bool,
    pub is_smart_stretch: bool,
    pub do_not_apply_smart_stretch: bool,
    pub use_segment_color: bool,
    pub use_circle_outline: bool,
    pub circle_is_hollow: bool,
    pub use_gradient: bool,
    pub reverse_gradient: bool,
    pub gradient_mode: i16,
    pub use_segment_scale: bool,
    pub local_x: f32,
    pub local_y: f32,
    pub scale: f32,
    pub default_length: f32,
    pub length: f32,
    pub default_thickness: i32,
    pub thickness: i32,
    pub segment_curve_radius_and_default_curve_radius: i32,
    pub curve_circulization: bool,
    pub segment_curve_polyfill_precision: i16,
    pub half_arc: bool,
    pub right_triangle_direction: i16,
    pub triangle_upside_down: bool,
    pub trapezoid_top_thickness_ratio: f32,
    pub num_polygon_vertices: i16,
    pub default_local_angle: f32,
    pub local_angle: f32,
    pub default_angle: f32,
    pub color: Color,
    pub gradient_color: Color,
    pub circle_outline_color: Color,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            node_type: NodeType::default(),
            draw_order_index: DrawOrderIndex(0),
            is_static: false,
            is_stretchy: false,
            is_smart_stretch: false,
            do_not_apply_smart_stretch: false,
            use_segment_color: false,
            use_circle_outline: false,
            circle_is_hollow: false,
            use_gradient: false,
            reverse_gradient: false,
            gradient_mode: 0,
            use_segment_scale: false,
            local_x: 0.0,
            local_y: 0.0,
            scale: 1.0,
            default_length: 0.0,
            length: 0.0,
            default_thickness: 0,
            thickness: 0,
            segment_curve_radius_and_default_curve_radius: 0,
            curve_circulization: false,
            segment_curve_polyfill_precision: 0,
            half_arc: false,
            right_triangle_direction: 0,
            triangle_upside_down: false,
            trapezoid_top_thickness_ratio: 0.0,
            num_polygon_vertices: 0,
            default_local_angle: 0.0,
            local_angle: 0.0,
            default_angle: 0.0,
            color: Color::default(),
            gradient_color: Color::default(),
            circle_outline_color: Color::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableNode {
    pub node_type: NodeType,
    pub is_static: bool,
    pub is_stretchy: bool,
    pub is_smart_stretch: bool,
    pub do_not_apply_smart_stretch: bool,
    pub use_segment_color: bool,
    pub use_circle_outline: bool,
    pub circle_is_hollow: bool,
    pub use_gradient: bool,
    pub reverse_gradient: bool,
    pub gradient_mode: i16,
    pub use_segment_scale: bool,
    pub local_x: f32,
    pub local_y: f32,
    pub scale: f32,
    pub default_length: f32,
    pub length: f32,
    pub default_thickness: i32,
    pub thickness: i32,
    pub segment_curve_radius_and_default_curve_radius: i32,
    pub curve_circulization: bool,
    pub segment_curve_polyfill_precision: i16,
    pub half_arc: bool,
    pub right_triangle_direction: i16,
    pub triangle_upside_down: bool,
    pub trapezoid_top_thickness_ratio: f32,
    pub num_polygon_vertices: i16,
    pub default_local_angle: f32,
    pub local_angle: f32,
    pub default_angle: f32,
    pub color: Color,
    pub gradient_color: Color,
    pub circle_outline_color: Color,
    pub children: Vec<SerializableNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeOptions {
    pub node_type: NodeType,
    pub is_static: bool,
    pub is_stretchy: bool,
    pub is_smart_stretch: bool,
    pub do_not_apply_smart_stretch: bool,
    pub use_segment_color: bool,
    pub use_circle_outline: bool,
    pub circle_is_hollow: bool,
    pub use_gradient: bool,
    pub reverse_gradient: bool,
    pub gradient_mode: i16,
    pub use_segment_scale: bool,
    pub local_x: f32,
    pub local_y: f32,
    pub scale: f32,
    pub default_length: f32,
    pub length: f32,
    pub default_thickness: i32,
    pub thickness: i32,
    pub segment_curve_radius_and_default_curve_radius: i32,
    pub curve_circulization: bool,
    pub segment_curve_polyfill_precision: i16,
    pub half_arc: bool,
    pub right_triangle_direction: i16,
    pub triangle_upside_down: bool,
    pub trapezoid_top_thickness_ratio: f32,
    pub num_polygon_vertices: i16,
    pub default_local_angle: f32,
    pub local_angle: f32,
    pub default_angle: f32,
    pub color: Color,
    pub gradient_color: Color,
    pub circle_outline_color: Color,
}

impl Default for NodeOptions {
    fn default() -> Self {
        Self {
            node_type: NodeType::default(),
            is_static: false,
            is_stretchy: false,
            is_smart_stretch: false,
            do_not_apply_smart_stretch: false,
            use_segment_color: false,
            use_circle_outline: false,
            circle_is_hollow: false,
            use_gradient: false,
            reverse_gradient: false,
            gradient_mode: 0,
            use_segment_scale: false,
            local_x: 0.0,
            local_y: 0.0,
            scale: 1.0,
            default_length: 0.0,
            length: 0.0,
            default_thickness: 0,
            thickness: 0,
            segment_curve_radius_and_default_curve_radius: 0,
            curve_circulization: false,
            segment_curve_polyfill_precision: 0,
            half_arc: false,
            right_triangle_direction: 0,
            triangle_upside_down: false,
            trapezoid_top_thickness_ratio: 0.0,
            num_polygon_vertices: 0,
            default_local_angle: 0.0,
            local_angle: 0.0,
            default_angle: 0.0,
            color: Color::default(),
            gradient_color: Color::default(),
            circle_outline_color: Color::default(),
        }
    }
}

impl Node {
    /// Creates a new, empty `Node`.
    pub fn new() -> Self {
        Node::default()
    }

    pub fn from_options(options: NodeOptions) -> Self {
        let mut node_type = options.node_type.clone();
        if options.node_type.to_integer() == NodeType::RootNode.to_integer() {
            node_type = NodeType::RoundedSegment;
        }
        Self {
            node_type: node_type,
            is_static: options.is_static,
            is_stretchy: options.is_stretchy,
            is_smart_stretch: options.is_smart_stretch,
            do_not_apply_smart_stretch: options.do_not_apply_smart_stretch,
            use_segment_color: options.use_segment_color,
            use_circle_outline: options.use_circle_outline,
            circle_is_hollow: options.circle_is_hollow,
            use_gradient: options.use_gradient,
            reverse_gradient: options.reverse_gradient,
            gradient_mode: options.gradient_mode,
            use_segment_scale: options.use_segment_scale,
            local_x: options.local_x,
            local_y: options.local_y,
            scale: options.scale,
            default_length: options.default_length,
            length: options.length,
            default_thickness: options.default_thickness,
            thickness: options.thickness,
            segment_curve_radius_and_default_curve_radius: options
                .segment_curve_radius_and_default_curve_radius,
            curve_circulization: options.curve_circulization,
            segment_curve_polyfill_precision: options.segment_curve_polyfill_precision,
            half_arc: options.half_arc,
            right_triangle_direction: options.right_triangle_direction,
            triangle_upside_down: options.triangle_upside_down,
            trapezoid_top_thickness_ratio: options.trapezoid_top_thickness_ratio,
            num_polygon_vertices: options.num_polygon_vertices,
            default_local_angle: options.default_local_angle,
            local_angle: options.local_angle,
            default_angle: options.default_angle,
            color: options.color,
            gradient_color: options.gradient_color,
            circle_outline_color: options.circle_outline_color,
            ..Default::default()
        }
    }

    pub fn to_options(&self) -> NodeOptions {
        NodeOptions {
            node_type: self.node_type.clone(),
            is_static: self.is_static,
            is_stretchy: self.is_stretchy,
            is_smart_stretch: self.is_smart_stretch,
            do_not_apply_smart_stretch: self.do_not_apply_smart_stretch,
            use_segment_color: self.use_segment_color,
            use_circle_outline: self.use_circle_outline,
            circle_is_hollow: self.circle_is_hollow,
            use_gradient: self.use_gradient,
            reverse_gradient: self.reverse_gradient,
            gradient_mode: self.gradient_mode,
            use_segment_scale: self.use_segment_scale,
            local_x: self.local_x,
            local_y: self.local_y,
            scale: self.scale,
            default_length: self.default_length,
            length: self.length,
            default_thickness: self.default_thickness,
            thickness: self.thickness,
            segment_curve_radius_and_default_curve_radius: self
                .segment_curve_radius_and_default_curve_radius,
            curve_circulization: self.curve_circulization,
            segment_curve_polyfill_precision: self.segment_curve_polyfill_precision,
            half_arc: self.half_arc,
            right_triangle_direction: self.right_triangle_direction,
            triangle_upside_down: self.triangle_upside_down,
            trapezoid_top_thickness_ratio: self.trapezoid_top_thickness_ratio,
            num_polygon_vertices: self.num_polygon_vertices,
            default_local_angle: self.default_local_angle,
            local_angle: self.local_angle,
            default_angle: self.default_angle,
            color: self.color,
            gradient_color: self.gradient_color,
            circle_outline_color: self.circle_outline_color,
        }
    }

    pub fn build_serializable_tree(
        &self,
        graph: &Graph<Rc<RefCell<Node>>, ()>,
        current: NodeIndex,
    ) -> SerializableNode {
        let node = graph.node_weight(current).expect("Bug with library. NodeIndex passed to build_serializable_tree in Node is not valid.").borrow().clone();
        let mut children = Vec::new();

        for neighbor in graph.neighbors_directed(current, petgraph::Direction::Outgoing) {
            children.push(self.build_serializable_tree(graph, neighbor));
        }

        SerializableNode {
            children,
            node_type: node.node_type.clone(),
            is_static: node.is_static,
            is_stretchy: node.is_stretchy,
            is_smart_stretch: node.is_smart_stretch,
            do_not_apply_smart_stretch: node.do_not_apply_smart_stretch,
            use_segment_color: node.use_segment_color,
            use_circle_outline: node.use_circle_outline,
            circle_is_hollow: node.circle_is_hollow,
            use_gradient: node.use_gradient,
            reverse_gradient: node.reverse_gradient,
            gradient_mode: node.gradient_mode,
            use_segment_scale: node.use_segment_scale,
            local_x: node.local_x,
            local_y: node.local_y,
            scale: node.scale,
            default_length: node.default_length,
            length: node.length,
            default_thickness: node.default_thickness,
            thickness: node.thickness,
            segment_curve_radius_and_default_curve_radius: node
                .segment_curve_radius_and_default_curve_radius,
            curve_circulization: node.curve_circulization,
            segment_curve_polyfill_precision: node.segment_curve_polyfill_precision,
            half_arc: node.half_arc,
            right_triangle_direction: node.right_triangle_direction,
            triangle_upside_down: node.triangle_upside_down,
            trapezoid_top_thickness_ratio: node.trapezoid_top_thickness_ratio,
            num_polygon_vertices: node.num_polygon_vertices,
            default_local_angle: node.default_local_angle,
            local_angle: node.local_angle,
            default_angle: node.default_angle,
            color: node.color,
            gradient_color: node.gradient_color,
            circle_outline_color: node.circle_outline_color,
        }
    }

    pub fn get_draw_order_index(&self) -> DrawOrderIndex {
        return self.draw_order_index;
    }

    pub fn update<F>(rc_node: &Rc<RefCell<Self>>, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = rc_node.borrow_mut();
        f(&mut *inner);
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum NodeType {
    /// Should only be one RootNode per `Stickfigure`, and it should be the first node.
    RootNode = -1,
    #[default]
    RoundedSegment = 0,
    Segment = 1,
    Circle = 2,
    Triangle = 3,
    FilledCircle = 4,
    Ellipse = 5,
    Trapezoid = 6,
    Polygon = 7,
}

impl NodeType {
    pub fn from_integer(int: i8) -> Option<Self> {
        match int {
            -1 => Some(NodeType::RootNode),
            0 => Some(NodeType::RoundedSegment),
            1 => Some(NodeType::Segment),
            2 => Some(NodeType::Circle),
            3 => Some(NodeType::Triangle),
            4 => Some(NodeType::FilledCircle),
            5 => Some(NodeType::Ellipse),
            6 => Some(NodeType::Trapezoid),
            7 => Some(NodeType::Polygon),
            _ => None,
        }
    }
    pub fn to_integer(&self) -> i8 {
        self.clone() as i8
    }
}
