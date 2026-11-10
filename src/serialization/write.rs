use std::io::{self, Write};

use crate::{
    error::{LibraryError, StickfigureError},
    structs::{
        node::{AngleLockMode, GradientMode, Node, NodeType, TriangleType},
        polyfill::Polyfill,
        stickfigure::{DrawOrderIndex, Stickfigure},
    },
    ConnectorMethod,
};

/// Writes big-endian numeric values into an underlying `Write` implementation.
///
/// The stickfigure file format currently writes its numeric fields in big-endian
/// order, so the byte order is fixed here rather than selected for every field.
struct BinaryWriter<W> {
    inner: W,
}

impl<W: Write> BinaryWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }

    fn write_i8(&mut self, value: i8) -> io::Result<()> {
        self.inner.write_all(&value.to_be_bytes())
    }

    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.inner.write_all(&value.to_be_bytes())
    }

    fn write_i16(&mut self, value: i16) -> io::Result<()> {
        self.inner.write_all(&value.to_be_bytes())
    }

    fn write_i32(&mut self, value: i32) -> io::Result<()> {
        self.inner.write_all(&value.to_be_bytes())
    }

    fn write_f32(&mut self, value: f32) -> io::Result<()> {
        self.inner.write_all(&value.to_be_bytes())
    }
}

impl<W: Write> Write for BinaryWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn write_stickfigure_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut byte_writer = BinaryWriter::new(&mut byte_vec);
    byte_writer.write_i32(stickfigure.version)?;

    if stickfigure.version >= 403 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i32(stickfigure.build)?;
    }

    let mut segment_color_writer = BinaryWriter::new(&mut byte_vec);
    segment_color_writer.write_f32(stickfigure.scale)?;
    segment_color_writer.write_all(&[
        stickfigure.color.alpha,
        stickfigure.color.blue,
        stickfigure.color.green,
        stickfigure.color.red,
    ])?;

    Ok(byte_vec)
}

pub fn write_stickfigure(stickfigure: &Stickfigure) -> Result<Vec<u8>, LibraryError> {
    let mut byte_vec = Vec::new();

    byte_vec.append(&mut write_stickfigure_header(stickfigure)?);

    byte_vec.append(&mut write_child_nodes(
        stickfigure.version,
        stickfigure.build,
        DrawOrderIndex(0),
        stickfigure,
    )?);

    if stickfigure.version >= 230 {
        byte_vec.append(&mut write_polyfill_header(stickfigure)?);
    }

    if stickfigure.version >= 403 && stickfigure.build >= 38 {
        byte_vec.append(&mut write_connector_data(stickfigure)?);
    }

    Ok(byte_vec)
}

fn write_connector_data(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let connector_nodes = stickfigure.get_nodes_with_property(|node| node.connector_data.is_some());

    let mut byte_writer = BinaryWriter::new(&mut byte_vec);
    // Allow justified because wrapping and truncation would only matter at absurdly large node counts
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    byte_writer.write_i32(connector_nodes.len() as i32)?;

    for draw_index in connector_nodes {
        let connector_node = stickfigure.get_node(draw_index).ok_or_else(|| StickfigureError::InvalidDrawIndex(draw_index.0, "Attempted to get connector node that does not exist when writing .nodes. Probably a library bug.".to_string()))?;
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i32(connector_node.get_draw_order_index().0)?;

        let connector_data = &connector_node.connector_data;

        match connector_data {
            Some(data) => {
                field_writer
                    .write_i32(data.end_node_draw_index.0)?;
            },
            None => return Err(StickfigureError::GenericError(format!("Attempted to get undefined connector data of node (while writing .nodes file). Node {draw_index:?}.")))?,
        }
    }

    Ok(byte_vec)
}

fn write_child_nodes(
    version: i32,
    build: i32,
    draw_index: DrawOrderIndex,
    stickfigure: &Stickfigure,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    if !stickfigure.all_draw_indices_exist(&[draw_index]) {
        return Err(StickfigureError::InvalidDrawIndex(
            draw_index.0,
            "Cannot finish writing this stickfigure file.".to_string(),
        ));
    }

    let node_index = stickfigure.node_index_from_draw_order(draw_index);

    if let Some(node) = stickfigure.nodes.node_weight(node_index) {
        byte_vec.append(&mut write_node(version, build, node, stickfigure)?);

        let children = stickfigure.get_children(draw_index);
        // Allow justified because wrapping and truncation would only matter at absurdly large node counts
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let number_of_child_nodes = children.len() as i32;

        let mut byte_writer = BinaryWriter::new(&mut byte_vec);
        byte_writer.write_i32(number_of_child_nodes)?;

        if version >= 403 && build > 38 {
            for draw_index in &children {
                if let Some(node) = stickfigure.get_node(*draw_index) {
                    let connector_data_present = node.connector_data.is_some();
                    let mut byte_writer = BinaryWriter::new(&mut byte_vec);
                    byte_writer.write_i32(i32::from(connector_data_present))?;
                } else {
                    return Err(StickfigureError::GenericError(
                        "Failed to get child node from index (while writing .nodes).".to_string(),
                    ));
                }
            }
        }

        for child_draw_index in children.iter().rev() {
            byte_vec.append(&mut write_child_nodes(
                version,
                build,
                *child_draw_index,
                stickfigure,
            )?);
        }
    }

    Ok(byte_vec)
}

fn write_node(
    version: i32,
    build: i32,
    node: &Node,
    stickfigure: &Stickfigure,
) -> Result<Vec<u8>, StickfigureError> {
    let mut node = node.clone();
    prepare_node_for_write(&mut node, stickfigure)?;
    let mut byte_vec = Vec::new();

    byte_vec.append(&mut write_node_connector_fields(version, build, &node)?);
    byte_vec.append(&mut write_node_header_fields(version, build, &node)?);
    byte_vec.append(&mut write_node_geometry_fields(version, build, &node)?);
    byte_vec.append(&mut write_node_angle_and_color_fields(
        version, build, &node,
    )?);

    Ok(byte_vec)
}

fn prepare_node_for_write(
    node: &mut Node,
    stickfigure: &Stickfigure,
) -> Result<(), StickfigureError> {
    node.local_x = node.get_local_x(stickfigure);
    node.local_y = node.get_local_y(stickfigure);

    match node.triangle_type {
        TriangleType::Isosceles => {
            node.right_triangle_direction = 0;
        }
        TriangleType::RightTriangle => {
            if node.triangle_flipped {
                node.right_triangle_direction = -1;
            } else {
                node.right_triangle_direction = 1;
            }
        }
    }

    if node.node_type.to_integer() == NodeType::RootNode.to_integer() {
        node.is_angle_locked = false;
        node.angle_lock_is_main_node = false;
        node.angle_lock_relative_start = 0.0;
        node.angle_lock_stickfigure_start = 0.0;
        node.angle_lock_offset_minuend = 0.0;
        node.angle_lock_offset_subtrahend = 0.0;
        node.angle_lock_offset = 0.0;
    } else if let Some(parent_node_draw_index) = stickfigure.get_parent(node.get_draw_order_index())
    {
        if let Some(parent_node) = stickfigure.get_node(parent_node_draw_index) {
            if let Some(root_node) = stickfigure.get_node(DrawOrderIndex(0)) {
                match node.angle_lock_mode {
                    AngleLockMode::None => {
                        node.is_angle_locked = false;
                        node.angle_lock_is_main_node = false;
                        node.angle_lock_relative_start = 0.0;
                        node.angle_lock_stickfigure_start = 0.0;
                    }
                    AngleLockMode::Absolute => {
                        node.is_angle_locked = true;
                        node.angle_lock_is_main_node = true;
                        node.angle_lock_relative_start = 0.0;
                        node.angle_lock_stickfigure_start = 0.0;
                    }
                    AngleLockMode::Relative => {
                        node.is_angle_locked = true;
                        node.angle_lock_is_main_node = false;
                        node.angle_lock_relative_start = parent_node.get_global_angle(stickfigure);
                        node.angle_lock_stickfigure_start = root_node.local_angle;
                    }
                }

                node.angle_lock_offset_minuend = node.get_global_angle(stickfigure);
                node.angle_lock_offset_subtrahend = parent_node.get_global_angle(stickfigure);
                node.angle_lock_offset =
                    node.angle_lock_offset_minuend - node.angle_lock_offset_subtrahend;
            } else {
                return Err(StickfigureError::GenericError("Failed to get parent node draw index when setting private properties (while writing .nodes).".to_string()));
            }
        } else {
            return Err(StickfigureError::GenericError(
                "Failed to get parent node when setting private properties (while writing .nodes)."
                    .to_string(),
            ));
        }
    } else {
        return Err(StickfigureError::GenericError(
            "Failed to get root node when setting private properties (while writing .nodes)."
                .to_string(),
        ));
    }
    Ok(())
}
fn write_node_connector_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    if version >= 403 && build >= 38 {
        if let Some(connector_data) = &node.connector_data {
            let mut cursor_c1 = BinaryWriter::new(&mut byte_vec);
            cursor_c1.write_f32(connector_data.local_x)?;
            cursor_c1.write_f32(connector_data.local_y)?;
            cursor_c1.write_f32(connector_data.percent)?;

            if build >= 44 {
                let mut cursor_c2 = BinaryWriter::new(&mut byte_vec);
                cursor_c2.write_f32(connector_data.percent_default)?;
            }

            let mut cursor_c3 = BinaryWriter::new(&mut byte_vec);
            cursor_c3.write_f32(connector_data.value)?;
            cursor_c3.write_i32(i32::from(ConnectorMethod::to_integer(
                &connector_data.method,
            )))?;
            cursor_c3.write_u8(u8::from(connector_data.reversed))?;

            if build >= 65 {
                let mut cursor_c4 = BinaryWriter::new(&mut byte_vec);
                cursor_c4.write_f32(connector_data.smart_stretch_ancestral_value)?;
            }
        }
    }
    Ok(byte_vec)
}

fn write_node_header_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    let mut node_header_writer = BinaryWriter::new(&mut byte_vec);
    node_header_writer.write_i8(node.node_type.to_integer())?;
    node_header_writer.write_i32(node.draw_order_index.0)?;
    node_header_writer.write_u8(u8::from(node.is_static))?;
    node_header_writer.write_u8(u8::from(node.is_stretchy))?;

    if version >= 403 && build >= 48 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.is_floaty))?;
    }

    if version >= 248 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.is_smart_stretch))?;
    }
    if version >= 252 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.do_not_apply_smart_stretch))?;
    }

    if version >= 403 && build >= 50 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.smart_stretch_reset_impulse))?;
    }

    let mut segment_color_writer = BinaryWriter::new(&mut byte_vec);
    segment_color_writer.write_u8(u8::from(node.use_segment_color))?;

    if version >= 256 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.use_circle_outline))?;
    }
    if version >= 403 && build >= 21 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.circle_is_hollow))?;
    }
    if version >= 176 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.use_gradient))?;
        field_writer.write_u8(u8::from(node.reverse_gradient))?;
    }
    if version >= 403 && build >= 20 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i16(i16::from(GradientMode::to_integer(&node.gradient_mode)))?;
    }
    Ok(byte_vec)
}

fn write_node_geometry_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = write_node_base_geometry(node)?;
    byte_vec.append(&mut write_node_curve_fields(version, build, node)?);
    byte_vec.append(&mut write_node_shape_fields(version, build, node)?);
    Ok(byte_vec)
}

fn write_node_base_geometry(node: &Node) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    let mut node_geometry_writer = BinaryWriter::new(&mut byte_vec);
    node_geometry_writer.write_u8(u8::from(node.use_segment_scale))?;
    node_geometry_writer.write_f32(node.local_x)?;
    node_geometry_writer.write_f32(node.local_y)?;
    node_geometry_writer.write_f32(node.scale)?;
    node_geometry_writer.write_f32(node.default_length)?;
    node_geometry_writer.write_f32(node.length)?;
    node_geometry_writer.write_i32(node.default_thickness)?;
    node_geometry_writer.write_i32(node.thickness)?;
    Ok(byte_vec)
}

fn write_node_curve_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    if version >= 320 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i32(node.segment_curve_radius_and_default_curve_radius)?;
    }
    if version >= 403 && build >= 20 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.curve_circulization))?;
    }
    if version >= 403 && build >= 21 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i16(node.segment_curve_polyfill_precision)?;
    }
    Ok(byte_vec)
}

fn write_node_shape_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    if version >= 256 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.half_arc))?;
        field_writer.write_i16(node.right_triangle_direction)?;
    }
    if version >= 300 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.triangle_upside_down))?;
    }
    if version >= 403 && build >= 36 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        // Allow justified because we can't avoid truncating without adding redundant i32 trapezoid thickness start/end fields
        // Only figures from a small range of build versions will be effected
        #[allow(clippy::cast_possible_truncation)]
        if build < 64 {
            field_writer.write_i32(node.trapezoid_thickness_start as i32)?;
            field_writer.write_i32(node.trapezoid_thickness_end as i32)?;
        } else {
            field_writer.write_f32(node.trapezoid_thickness_start)?;
            field_writer.write_f32(node.trapezoid_thickness_end)?;
        }
    }
    if version >= 403 && build == 36 {
        byte_vec.extend_from_slice(&[0u8; 8]);
    }
    if version >= 256 && build != 36 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.trapezoid_top_thickness_ratio)?;
    }
    if version >= 403 && build >= 36 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.trapezoid_is_rounded_start))?;
        field_writer.write_u8(u8::from(node.trapezoid_is_rounded_end))?;
    }
    if version >= 256 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i16(node.num_polygon_vertices)?;
    }
    if version >= 248 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.default_local_angle)?;
    }
    Ok(byte_vec)
}

fn write_node_angle_and_color_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = write_node_angle_fields(version, node)?;
    byte_vec.append(&mut write_node_color_fields(version, node)?);
    byte_vec.append(&mut write_node_angle_lock_and_drag_fields(
        version, build, node,
    )?);
    Ok(byte_vec)
}

fn write_node_angle_fields(version: i32, node: &Node) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    let mut node_angle_writer = BinaryWriter::new(&mut byte_vec);
    node_angle_writer.write_f32(node.local_angle)?;

    if version >= 248 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.default_angle)?;
    }
    Ok(byte_vec)
}

fn write_node_color_fields(version: i32, node: &Node) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    let mut node_color_writer = BinaryWriter::new(&mut byte_vec);
    node_color_writer.write_all(&[
        node.color.alpha,
        node.color.blue,
        node.color.green,
        node.color.red,
    ])?;

    if version >= 176 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_all(&[
            node.gradient_color.alpha,
            node.gradient_color.blue,
            node.gradient_color.green,
            node.gradient_color.red,
        ])?;
    }
    if version >= 256 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_all(&[
            node.circle_outline_color.alpha,
            node.circle_outline_color.blue,
            node.circle_outline_color.green,
            node.circle_outline_color.red,
        ])?;
    }
    Ok(byte_vec)
}

fn write_node_angle_lock_and_drag_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = write_node_angle_lock_fields(version, build, node)?;
    byte_vec.append(&mut write_node_drag_fields(version, build, node)?);
    Ok(byte_vec)
}

fn write_node_angle_lock_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    if version >= 403 && build >= 39 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(node.is_angle_locked))?;
    }
    if version >= 403 && (39..=50).contains(&build) {
        byte_vec.extend_from_slice(&[0u8; 4]);
    }
    if version >= 403 && build >= 51 {
        let mut bool = node.angle_lock_is_main_node;
        if build < 56 {
            bool = !bool;
        }
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_u8(u8::from(bool))?;
    }
    if version >= 403 && (51..=56).contains(&build) {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.angle_lock_offset_minuend)?;
        field_writer.write_f32(node.angle_lock_offset_subtrahend)?;
    }
    if version >= 403 && build >= 57 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.angle_lock_offset)?;
    }
    if version >= 403 && build >= 63 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.angle_lock_relative_start)?;
    }
    if version >= 403 && build >= 67 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.angle_lock_stickfigure_start)?;
    }
    if version >= 403 && build >= 63 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_i8(node.angle_lock_relative_multiplier)?;
    }
    Ok(byte_vec)
}

fn write_node_drag_fields(
    version: i32,
    build: i32,
    node: &Node,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();
    if version >= 403 && build >= 39 {
        if build <= 40 {
            let mut field_writer = BinaryWriter::new(&mut byte_vec);
            field_writer.write_i16(i16::from(node.is_drag_locked))?;
        } else {
            let mut field_writer = BinaryWriter::new(&mut byte_vec);
            field_writer.write_u8(u8::from(node.is_drag_locked))?;
        }
    }
    if version >= 403 && (41..=45).contains(&build) {
        byte_vec.extend_from_slice(&[0u8; 2]);
    }
    if version >= 403 && build >= 46 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.drag_lock_angle)?;
    }
    if version >= 403 && build >= 41 {
        let mut field_writer = BinaryWriter::new(&mut byte_vec);
        field_writer.write_f32(node.smart_stretch_multiplier)?;
    }
    if version >= 403 && (41..=45).contains(&build) {
        byte_vec.extend_from_slice(&[0u8; 1]);
    }
    Ok(byte_vec)
}

fn write_polyfill_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut byte_writer = BinaryWriter::new(&mut byte_vec);
    // Allow justified because wrapping and truncation would only matter at absurdly large polyfill counts
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let number_of_polyfills = stickfigure.polyfills.len() as i32;
    byte_writer.write_i32(number_of_polyfills)?;

    for polyfill in &stickfigure.polyfills {
        byte_vec.append(&mut write_polyfill(polyfill)?);
    }

    Ok(byte_vec)
}

fn write_polyfill(polyfill: &Polyfill) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut byte_writer = BinaryWriter::new(&mut byte_vec);
    byte_writer.write_i32(polyfill.anchor_node_draw_index.0)?;
    byte_writer.write_all(&[
        polyfill.color.alpha,
        polyfill.color.blue,
        polyfill.color.green,
        polyfill.color.red,
    ])?;
    byte_writer.write_u8(u8::from(polyfill.use_polyfill_color))?;

    // Allow justified because wrapping and truncation would only matter at absurdly large node counts
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let number_of_attached_nodes = polyfill.attached_node_draw_indices.len() as i32;
    byte_writer.write_i32(number_of_attached_nodes)?;

    for draw_index in &polyfill.attached_node_draw_indices {
        let mut local_writer = BinaryWriter::new(&mut byte_vec);
        local_writer.write_i32(draw_index.0)?;
    }

    Ok(byte_vec)
}
