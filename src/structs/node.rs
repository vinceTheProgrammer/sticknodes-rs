use petgraph::graph::NodeIndex;
use petgraph::Graph;
use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;
use crate::Stickfigure;

use super::connector::ConnectorData;
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
    pub is_floaty: bool, //
    pub is_smart_stretch: bool,
    pub do_not_apply_smart_stretch: bool, // must be false if is_static is true?
    pub smart_stretch_reset_impulse: bool, //
    pub use_segment_color: bool,
    pub use_circle_outline: bool,
    pub circle_is_hollow: bool,
    pub use_gradient: bool,
    pub reverse_gradient: bool,
    pub gradient_mode: GradientMode,
    pub use_segment_scale: bool,
    pub(crate) local_x: f32,
    pub(crate) local_y: f32,
    pub scale: f32,
    pub default_length: f32,
    pub length: f32,
    pub default_thickness: i32,
    pub thickness: i32,
    pub segment_curve_radius_and_default_curve_radius: i32,
    pub curve_circulization: bool,
    pub segment_curve_polyfill_precision: i16,
    pub half_arc: bool,
    pub(crate) right_triangle_direction: i16, // triangle type Isosceles -> 0; flipped = false, triangle type RightTriangle -> 1; flipped = true, triangle type RightTriangle -> -1
    pub triangle_type: TriangleType,
    pub(crate) triangle_flipped: bool,
    pub triangle_upside_down: bool,
    /// Thickness of the start of the trapezoid node (the end nearest its parent node).
    pub trapezoid_thickness_start: f32, // trapezoid_thickness_1
    /// Thickness of the end of the trapezoid node (the end furthest from its parent node).
    pub trapezoid_thickness_end: f32, // trapezoid_thickness_2
    /// Whether the start thickness should be equal to trapezoid_thickness_start. If false, start thickness will equal node thickness.
    /// Only has effect specifically for Stick Nodes build 36. Otherwise, this property does nothing.
    pub use_trapezoid_thickness_start: bool, // use_trapezoid_thickness_1
    /// Whether the end thickness should be equal to trapezoid_thickness_end. If false, end thickness will equal node thickness.
    /// Only has effect specifically for Stick Nodes build 36. Otherwise, this property does nothing.
    pub use_trapezoid_thickness_end: bool, // use_trapezoid_thickness_2
    pub(crate) trapezoid_top_thickness_ratio: f32,
    pub trapezoid_is_rounded_start: bool, // trapezoid_is_rounded_1
    pub trapezoid_is_rounded_end: bool, // trapezoid_is_rounded_2
    pub num_polygon_vertices: i16,
    pub default_local_angle: f32,
    pub local_angle: f32,
    pub default_angle: f32,
    pub color: Color,
    pub gradient_color: Color,
    pub circle_outline_color: Color,
    pub angle_lock_mode: AngleLockMode, // None -> angle locked = false; Absolute -> angle locked = true, is main node = true; Relative -> angle locked = true, is main node = false
    pub(crate) is_angle_locked: bool, //
    pub(crate) angle_lock_is_main_node: bool, //
    pub(crate) angle_lock_offset_minuend: f32, // my angle
    pub(crate) angle_lock_offset_subtrahend: f32, // parent angle
    pub(crate) angle_lock_offset: f32, // should be my angle minus my parent's angle
    pub(crate) angle_lock_relative_start: f32, // if absolute: 0; if relative: parent angle
    pub(crate) angle_lock_stickfigure_start: f32, // if absolute: 0; if relative: main node angle
    pub angle_lock_relative_multiplier: i8, //
    pub is_drag_locked: bool, //
    pub drag_lock_angle: f32, //
    pub smart_stretch_multiplier: f32, //
    pub connector_data: Option<ConnectorData> //
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
            gradient_mode: GradientMode::default(),
            use_segment_scale: false,
            local_x: 0.0,
            local_y: 0.0,
            scale: 1.0,
            default_length: 0.0,
            length: 0.0,
            default_thickness: 32,
            thickness: 32,
            segment_curve_radius_and_default_curve_radius: 0,
            curve_circulization: false,
            segment_curve_polyfill_precision: 1,
            half_arc: false,
            triangle_type: TriangleType::default(),
            triangle_flipped: false,
            right_triangle_direction: 0,
            triangle_upside_down: false,
            trapezoid_top_thickness_ratio: -1.0, // unused?
            num_polygon_vertices: 5,
            default_local_angle: 0.0,
            local_angle: 0.0,
            default_angle: 0.0,
            color: Color::DEFAULT,
            gradient_color: Color::DEFAULT_GRADIENT,
            circle_outline_color: Color::DEFAULT_GRADIENT,
            is_floaty: false,
            smart_stretch_reset_impulse: false,
            trapezoid_thickness_start: 32.0,
            trapezoid_thickness_end: 16.0,
            use_trapezoid_thickness_start: true,
            use_trapezoid_thickness_end: true,
            trapezoid_is_rounded_start: false,
            trapezoid_is_rounded_end: false,
            angle_lock_mode: AngleLockMode::default(),
            is_angle_locked: false,
            angle_lock_is_main_node: false,
            angle_lock_offset_minuend: 0.0,
            angle_lock_offset_subtrahend: 0.0,
            angle_lock_offset: 0.0,
            angle_lock_relative_start: 0.0,
            angle_lock_stickfigure_start: 0.0,
            angle_lock_relative_multiplier: 1,
            is_drag_locked: false,
            drag_lock_angle: 0.0,
            smart_stretch_multiplier: 1.0,
            connector_data: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableNode {
    pub node_type: NodeType,
    pub draw_order_index: DrawOrderIndex,
    pub is_static: bool,
    pub is_stretchy: bool,
    pub is_floaty: bool,
    pub is_smart_stretch: bool,
    pub do_not_apply_smart_stretch: bool,
    pub smart_stretch_reset_impulse: bool,
    pub use_segment_color: bool,
    pub use_circle_outline: bool,
    pub circle_is_hollow: bool,
    pub use_gradient: bool,
    pub reverse_gradient: bool,
    pub gradient_mode: GradientMode,
    pub use_segment_scale: bool,
    pub scale: f32,
    pub default_length: f32,
    pub length: f32,
    pub default_thickness: i32,
    pub thickness: i32,
    pub segment_curve_radius_and_default_curve_radius: i32,
    pub curve_circulization: bool,
    pub segment_curve_polyfill_precision: i16,
    pub half_arc: bool,
    pub triangle_type: TriangleType,
    pub triangle_upside_down: bool,
    pub trapezoid_thickness_start: f32,
    pub trapezoid_thickness_end: f32,
    pub use_trapezoid_thickness_start: bool,
    pub use_trapezoid_thickness_end: bool,
    pub trapezoid_is_rounded_start: bool,
    pub trapezoid_is_rounded_end: bool,
    pub num_polygon_vertices: i16,
    pub default_local_angle: f32,
    pub local_angle: f32,
    pub default_angle: f32,
    pub color: Color,
    pub gradient_color: Color,
    pub circle_outline_color: Color,
    pub angle_lock_mode: AngleLockMode,
    pub angle_lock_relative_multiplier: i8,
    pub is_drag_locked: bool,
    pub drag_lock_angle: f32,
    pub smart_stretch_multiplier: f32,
    pub connector_data: Option<ConnectorData>,
    pub children: Vec<SerializableNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeOptions {
    pub node_type: NodeType,
    pub is_static: bool,
    pub is_stretchy: bool,
    pub is_floaty: bool,
    pub is_smart_stretch: bool,
    pub do_not_apply_smart_stretch: bool,
    pub smart_stretch_reset_impulse: bool,
    pub use_segment_color: bool,
    pub use_circle_outline: bool,
    pub circle_is_hollow: bool,
    pub use_gradient: bool,
    pub reverse_gradient: bool,
    pub gradient_mode: GradientMode,
    pub use_segment_scale: bool,
    pub scale: f32,
    pub default_length: f32,
    pub length: f32,
    pub default_thickness: i32,
    pub thickness: i32,
    pub segment_curve_radius_and_default_curve_radius: i32,
    pub curve_circulization: bool,
    pub segment_curve_polyfill_precision: i16,
    pub half_arc: bool,
    pub triangle_type: TriangleType,
    pub triangle_upside_down: bool,
    pub trapezoid_thickness_start: f32,
    pub trapezoid_thickness_end: f32,
    pub use_trapezoid_thickness_start: bool,
    pub use_trapezoid_thickness_end: bool,
    pub trapezoid_is_rounded_start: bool,
    pub trapezoid_is_rounded_end: bool,
    pub num_polygon_vertices: i16,
    pub default_local_angle: f32,
    pub local_angle: f32,
    pub default_angle: f32,
    pub color: Color,
    pub gradient_color: Color,
    pub circle_outline_color: Color,
    pub angle_lock_mode: AngleLockMode,
    pub angle_lock_relative_multiplier: i8,
    pub is_drag_locked: bool,
    pub drag_lock_angle: f32,
    pub smart_stretch_multiplier: f32,
    pub connector_data: Option<ConnectorData>,
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
            gradient_mode: GradientMode::default(),
            use_segment_scale: false,
            scale: 1.0,
            default_length: 100.0,
            length: 100.0,
            default_thickness: 32,
            thickness: 32,
            segment_curve_radius_and_default_curve_radius: 0,
            curve_circulization: false,
            segment_curve_polyfill_precision: 1,
            half_arc: false,
            triangle_upside_down: false,
            num_polygon_vertices: 0,
            default_local_angle: 0.0,
            local_angle: 0.0,
            default_angle: 0.0,
            color: Color::default(),
            gradient_color: Color::default(),
            circle_outline_color: Color::default(),
            is_floaty: false,
            smart_stretch_reset_impulse: false,
            trapezoid_thickness_start: 32.0,
            trapezoid_thickness_end: 16.0,
            use_trapezoid_thickness_start: true,
            use_trapezoid_thickness_end: true,
            trapezoid_is_rounded_start: false,
            trapezoid_is_rounded_end: false,
            angle_lock_mode: AngleLockMode::default(),
            angle_lock_relative_multiplier: 1,
            is_drag_locked: false,
            drag_lock_angle: 0.0,
            smart_stretch_multiplier: 1.0,
            connector_data: None,
            triangle_type: TriangleType::Isosceles,
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
            scale: options.scale,
            default_length: options.default_length,
            length: options.length,
            default_thickness: options.default_thickness,
            thickness: options.thickness,
            segment_curve_radius_and_default_curve_radius: options.segment_curve_radius_and_default_curve_radius,
            curve_circulization: options.curve_circulization,
            segment_curve_polyfill_precision: options.segment_curve_polyfill_precision,
            half_arc: options.half_arc,
            triangle_upside_down: options.triangle_upside_down,
            num_polygon_vertices: options.num_polygon_vertices,
            default_local_angle: options.default_local_angle,
            local_angle: options.local_angle,
            default_angle: options.default_angle,
            color: options.color,
            gradient_color: options.gradient_color,
            circle_outline_color: options.circle_outline_color,
            is_floaty: options.is_floaty,
            smart_stretch_reset_impulse: options.smart_stretch_reset_impulse,
            triangle_type: options.triangle_type,
            trapezoid_thickness_start: options.trapezoid_thickness_start,
            trapezoid_thickness_end: options.trapezoid_thickness_end,
            use_trapezoid_thickness_start: options.use_trapezoid_thickness_start,
            use_trapezoid_thickness_end: options.use_trapezoid_thickness_end,
            trapezoid_is_rounded_start: options.trapezoid_is_rounded_start,
            trapezoid_is_rounded_end: options.trapezoid_is_rounded_end,
            angle_lock_mode: options.angle_lock_mode,
            angle_lock_relative_multiplier: options.angle_lock_relative_multiplier,
            is_drag_locked: options.is_drag_locked,
            drag_lock_angle: options.drag_lock_angle,
            smart_stretch_multiplier: options.smart_stretch_multiplier,
            connector_data: options.connector_data,
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
            gradient_mode: self.gradient_mode.clone(),
            use_segment_scale: self.use_segment_scale,
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
            triangle_upside_down: self.triangle_upside_down,
            num_polygon_vertices: self.num_polygon_vertices,
            default_local_angle: self.default_local_angle,
            local_angle: self.local_angle,
            default_angle: self.default_angle,
            color: self.color,
            gradient_color: self.gradient_color,
            circle_outline_color: self.circle_outline_color,
            is_floaty: self.is_floaty,
            smart_stretch_reset_impulse: self.smart_stretch_reset_impulse,
            triangle_type: self.triangle_type.clone(),
            trapezoid_thickness_start: self.trapezoid_thickness_start,
            trapezoid_thickness_end: self.trapezoid_thickness_end,
            use_trapezoid_thickness_start: self.use_trapezoid_thickness_start,
            use_trapezoid_thickness_end: self.use_trapezoid_thickness_end,
            trapezoid_is_rounded_start: self.trapezoid_is_rounded_start,
            trapezoid_is_rounded_end: self.trapezoid_is_rounded_end,
            angle_lock_mode: self.angle_lock_mode.clone(),
            angle_lock_relative_multiplier: self.angle_lock_relative_multiplier,
            is_drag_locked: self.is_drag_locked,
            drag_lock_angle: self.drag_lock_angle,
            smart_stretch_multiplier: self.smart_stretch_multiplier,
            connector_data: self.connector_data.clone()
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
            draw_order_index: node.get_draw_order_index(),
            children,
            node_type: node.node_type,
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
            scale: node.scale,
            default_length: node.default_length,
            length: node.length,
            default_thickness: node.default_thickness,
            thickness: node.thickness,
            segment_curve_radius_and_default_curve_radius: node.segment_curve_radius_and_default_curve_radius,
            curve_circulization: node.curve_circulization,
            segment_curve_polyfill_precision: node.segment_curve_polyfill_precision,
            half_arc: node.half_arc,
            triangle_upside_down: node.triangle_upside_down,
            num_polygon_vertices: node.num_polygon_vertices,
            default_local_angle: node.default_local_angle,
            local_angle: node.local_angle,
            default_angle: node.default_angle,
            color: node.color,
            gradient_color: node.gradient_color,
            circle_outline_color: node.circle_outline_color,
            is_floaty: node.is_floaty,
            smart_stretch_reset_impulse: node.smart_stretch_reset_impulse,
            triangle_type: node.triangle_type,
            trapezoid_thickness_start: node.trapezoid_thickness_start,
            trapezoid_thickness_end: node.trapezoid_thickness_end,
            use_trapezoid_thickness_start: node.use_trapezoid_thickness_start,
            use_trapezoid_thickness_end: node.use_trapezoid_thickness_end,
            trapezoid_is_rounded_start: node.trapezoid_is_rounded_start,
            trapezoid_is_rounded_end: node.trapezoid_is_rounded_end,
            angle_lock_mode: node.angle_lock_mode,
            angle_lock_relative_multiplier: node.angle_lock_relative_multiplier,
            is_drag_locked: node.is_drag_locked,
            drag_lock_angle: node.drag_lock_angle,
            smart_stretch_multiplier: node.smart_stretch_multiplier,
            connector_data: node.connector_data,
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

    pub fn get_global_angle(&self, stickfigure: &Stickfigure) -> f32 {
        let ancestors = stickfigure.get_parents_recursive(self.get_draw_order_index());
        let mut global_angle = 0.0;

        ancestors.iter().for_each(|ancestor| {
            if let Some(node) = stickfigure.get_node(*ancestor) {
                let angle = node.borrow().local_angle;

                global_angle += angle;
            }
            
        });

        let angle = self.local_angle;

        global_angle += angle;

        global_angle
    }

    pub fn get_local_x(&self) -> f32 {
        self.length_angle_to_xy().0
    }

    pub fn get_local_y(&self) -> f32 {
        self.length_angle_to_xy().1
    }

    fn length_angle_to_xy(&self) -> (f32, f32) {
        let length = self.length;
        let angle_degrees = self.local_angle;
        let angle_radians = angle_degrees.to_radians();
        let x = length * libm::cosf(angle_radians);
        let y = length * libm::sinf(angle_radians);
        (x, y)
    }
}

#[repr(i8)]
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

#[repr(u8)]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum GradientMode {
    Sideways = 0,
    #[default]
    Normal = 1
}

impl GradientMode {
    pub fn from_integer(int: i8) -> Option<Self> {
        match int {
            0 => Some(GradientMode::Sideways),
            1 => Some(GradientMode::Normal),
            _ => None,
        }
    }
    pub fn to_integer(&self) -> i8 {
        self.clone() as i8
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum AngleLockMode {
    #[default]
    None = 0,
    Absolute = 1,
    Relative = 2
}

impl AngleLockMode {
    pub fn from_integer(int: i8) -> Option<Self> {
        match int {
            0 => Some(AngleLockMode::None),
            1 => Some(AngleLockMode::Absolute),
            2 => Some(AngleLockMode::Relative),
            _ => None,
        }
    }
    pub fn to_integer(&self) -> i8 {
        self.clone() as i8
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum TriangleType {
    #[default]
    Isosceles = 0,
    RightTriangle = 1
}

impl TriangleType {
    pub fn from_integer(int: i8) -> Option<Self> {
        match int {
            0 => Some(TriangleType::Isosceles),
            1 => Some(TriangleType::RightTriangle),
            _ => None,
        }
    }
    pub fn to_integer(&self) -> i8 {
        self.clone() as i8
    }
}
