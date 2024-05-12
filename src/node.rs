use crate::node_type::NodeType;
use crate::color::Color;
use raad::be::*;
use std::io::Read;

#[derive(Debug)]
pub struct Node {
    node_type: NodeType,
    id: i32,
    is_static: bool,
    is_stretchy: bool,
    is_smart_stretch: bool,
    do_not_apply_smart_stretch: bool,
    use_segment_color: bool,
    use_circle_outline: bool,
    circle_is_hollow: bool,
    use_gradient: bool,
    reverse_gradient: bool,
    gradient_mode: i16,
    use_segment_scale: bool,
    local_x: f32,
    local_y: f32,
    scale: f32,
    default_length: f32,
    length: f32,
    default_thickness: i32,
    thickness: i32,
    segment_curve_radius_and_default_curve_radius: i32,
    curve_circulization: bool,
    segment_curve_polyfill_precision: i16,
    half_arc: bool,
    right_triangle_direction: i16,
    triangle_upside_down: bool,
    trapezoid_top_thickness_ratio: f32,
    num_polygon_vertices: i16,
    default_local_angle: f32,
    local_angle: f32,
    default_angle: f32,
    color: Color,
    gradient_color: Color,
    circle_outline_color: Color,
    number_of_child_nodes: i32,
    child_nodes: Vec<Node>,
}

impl Node {
    pub fn read(reader: &mut impl Read) -> std::io::Result<Self> {

        let node_type = NodeType::from_integer(reader.r::<i8>()?).unwrap_or_default();
        let id = reader.r::<i32>()?;
        let is_static = reader.r::<u8>()? != 0;
        let is_stretchy = reader.r::<u8>()? != 0;
        let is_smart_stretch = reader.r::<u8>()? != 0;
        let do_not_apply_smart_stretch = reader.r::<u8>()? != 0;
        let use_segment_color = reader.r::<u8>()? != 0;
        let use_circle_outline = reader.r::<u8>()? != 0;
        let circle_is_hollow = reader.r::<u8>()? != 0;
        let use_gradient = reader.r::<u8>()? != 0;
        let reverse_gradient = reader.r::<u8>()? != 0;
        let gradient_mode = reader.r::<i16>()?;
        let use_segment_scale = reader.r::<u8>()? != 0;
        let local_x = reader.r::<f32>()?;
        let local_y = reader.r::<f32>()?;
        let scale = reader.r::<f32>()?;
        let default_length = reader.r::<f32>()?;
        let length = reader.r::<f32>()?;
        let default_thickness = reader.r::<i32>()?;
        let thickness = reader.r::<i32>()?;
        let segment_curve_radius_and_default_curve_radius = reader.r::<i32>()?;
        let curve_circulization = reader.r::<u8>()? != 0;
        let segment_curve_polyfill_precision = reader.r::<i16>()?;
        let half_arc = reader.r::<u8>()? != 0;
        let right_triangle_direction = reader.r::<i16>()?;
        let triangle_upside_down = reader.r::<u8>()? != 0;
        let trapezoid_top_thickness_ratio = reader.r::<f32>()?;
        let num_polygon_vertices = reader.r::<i16>()?;
        let default_local_angle = reader.r::<f32>()?;
        let local_angle = reader.r::<f32>()?;
        let default_angle = reader.r::<f32>()?;
        let color = Color {
            alpha: reader.r::<u8>()?,
            blue: reader.r::<u8>()?,
            green: reader.r::<u8>()?,
            red: reader.r::<u8>()?,
        };
        let gradient_color = Color {
            alpha: reader.r::<u8>()?,
            blue: reader.r::<u8>()?,
            green: reader.r::<u8>()?,
            red: reader.r::<u8>()?,
        };
        let circle_outline_color = Color {
            alpha: reader.r::<u8>()?,
            blue: reader.r::<u8>()?,
            green: reader.r::<u8>()?,
            red: reader.r::<u8>()?,
        };
        let number_of_child_nodes = reader.r::<i32>()?;
    
        // Reading child nodes recursively
        let mut child_nodes = Vec::new();
        for _ in 0..number_of_child_nodes {
            let child_node = Node::read(reader)?;
            child_nodes.push(child_node);
        }
    
        return Ok(Node {
            node_type,
            id,
            is_static,
            is_stretchy,
            is_smart_stretch,
            do_not_apply_smart_stretch,
            use_segment_color,
            use_circle_outline,
            circle_is_hollow,
            use_gradient,
            reverse_gradient,
            gradient_mode,
            use_segment_scale,
            local_x,
            local_y, scale, default_length, length,
            default_thickness, thickness,
            segment_curve_radius_and_default_curve_radius,
            curve_circulization,
            segment_curve_polyfill_precision,
            half_arc, 
            right_triangle_direction,
            triangle_upside_down,
            trapezoid_top_thickness_ratio,
            num_polygon_vertices,
            default_local_angle,
            local_angle,
            default_angle,
            color,
            gradient_color,
            circle_outline_color,
            number_of_child_nodes,
            child_nodes
        })
    }
}