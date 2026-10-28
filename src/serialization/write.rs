use core::cell::RefCell;

use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Cursor, Write};
use std::{format, rc::Rc, vec::Vec};

use crate::{
    error::{LibraryError, StickfigureError},
    structs::{
        node::{AngleLockMode, GradientMode, Node, NodeType, TriangleType},
        polyfill::Polyfill,
        stickfigure::{DrawOrderIndex, Stickfigure},
    },
    ConnectorMethod,
};

fn write_stickfigure_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut output_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut byte_writer = Cursor::new(&mut output_bytes[..]);
    byte_writer
        .write_i32::<BigEndian>(stickfigure.version)
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(output_bytes));

    if stickfigure.version >= 403 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i32::<BigEndian>(stickfigure.build)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    let mut segment_color_bytes = [0u8; 8]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut segment_color_writer = Cursor::new(&mut segment_color_bytes[..]);
    segment_color_writer
        .write_f32::<BigEndian>(stickfigure.scale)
        .map_err(StickfigureError::Io)?;
    segment_color_writer
        .write_all(&[
            stickfigure.color.alpha,
            stickfigure.color.blue,
            stickfigure.color.green,
            stickfigure.color.red,
        ])
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(segment_color_bytes));

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

    let connector_nodes =
        stickfigure.get_nodes_with_property(|node| node.borrow().connector_data.is_some());

    let mut output_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut byte_writer = Cursor::new(&mut output_bytes[..]);
    // Allow justified because wrapping and truncation would only matter at absurdly large node counts
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    byte_writer
        .write_i32::<BigEndian>(connector_nodes.len() as i32)
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(output_bytes));

    for draw_index in connector_nodes {
        let connector_node = stickfigure.get_node(draw_index).ok_or_else(|| StickfigureError::InvalidDrawIndex(draw_index.0, "Attempted to get connector node that does not exist when writing .nodes. Probably a library bug.".to_string()))?;
        let mut field_bytes: [u8; 8] = [0u8; 8]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i32::<BigEndian>(connector_node.borrow().get_draw_order_index().0)
            .map_err(StickfigureError::Io)?;

        let connector_data = &mut connector_node.borrow_mut().connector_data;

        match connector_data {
            Some(data) => {
                field_writer
                    .write_i32::<BigEndian>(data.end_node_draw_index.0).map_err(StickfigureError::Io)?;
            },
            None => return Err(StickfigureError::GenericError(format!("Attempted to get undefined connector data of node (while writing .nodes file). Node {draw_index:?}.")))?,
        }

        byte_vec.append(&mut Vec::from(field_bytes));
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

        let mut output_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut byte_writer = Cursor::new(&mut output_bytes[..]);
        byte_writer
            .write_i32::<BigEndian>(number_of_child_nodes)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(output_bytes));

        if version >= 403 && build > 38 {
            for draw_index in &children {
                if let Some(node) = stickfigure.get_node(*draw_index) {
                    let connector_data_present = node.borrow().connector_data.is_some();
                    let mut output_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
                    let mut byte_writer = Cursor::new(&mut output_bytes[..]);
                    byte_writer
                        .write_i32::<BigEndian>(i32::from(connector_data_present))
                        .map_err(StickfigureError::Io)?;
                    byte_vec.append(&mut Vec::from(output_bytes));
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
    rc_node: &Rc<RefCell<Node>>,
    stickfigure: &Stickfigure,
) -> Result<Vec<u8>, StickfigureError> {
    {
        let mut node = rc_node.borrow_mut();

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
        } else if let Some(parent_node_draw_index) =
            stickfigure.get_parent(node.get_draw_order_index())
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
                            node.angle_lock_relative_start =
                                parent_node.borrow().get_global_angle(stickfigure);
                            node.angle_lock_stickfigure_start = root_node.borrow().local_angle;
                        }
                    }

                    node.angle_lock_offset_minuend = node.get_global_angle(stickfigure);
                    node.angle_lock_offset_subtrahend =
                        parent_node.borrow().get_global_angle(stickfigure);
                    node.angle_lock_offset =
                        node.angle_lock_offset_minuend - node.angle_lock_offset_subtrahend;
                } else {
                    return Err(StickfigureError::GenericError("Failed to get parent node draw index when setting private properties (while writing .nodes).".to_string()));
                }
            } else {
                return Err(StickfigureError::GenericError("Failed to get parent node when setting private properties (while writing .nodes).".to_string()));
            }
        } else {
            return Err(StickfigureError::GenericError(
                "Failed to get root node when setting private properties (while writing .nodes)."
                    .to_string(),
            ));
        }
    }
    let node = rc_node.borrow();

    let mut byte_vec = Vec::new();

    if version >= 403 && build >= 38 {
        if let Some(connector_data) = &rc_node.borrow().connector_data {
            let mut buffer_c1 = [0u8; 12]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
            let mut cursor_c1 = Cursor::new(&mut buffer_c1[..]);
            cursor_c1
                .write_f32::<BigEndian>(connector_data.local_x)
                .map_err(StickfigureError::Io)?;
            cursor_c1
                .write_f32::<BigEndian>(connector_data.local_y)
                .map_err(StickfigureError::Io)?;
            cursor_c1
                .write_f32::<BigEndian>(connector_data.percent)
                .map_err(StickfigureError::Io)?;
            byte_vec.append(&mut Vec::from(buffer_c1));

            if build >= 44 {
                let mut buffer_c2 = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
                let mut cursor_c2 = Cursor::new(&mut buffer_c2[..]);
                cursor_c2
                    .write_f32::<BigEndian>(connector_data.percent_default)
                    .map_err(StickfigureError::Io)?;
                byte_vec.append(&mut Vec::from(buffer_c2));
            }

            let mut buffer_c3 = [0u8; 9]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
            let mut cursor_c3 = Cursor::new(&mut buffer_c3[..]);
            cursor_c3
                .write_f32::<BigEndian>(connector_data.value)
                .map_err(StickfigureError::Io)?;
            cursor_c3
                .write_i32::<BigEndian>(i32::from(ConnectorMethod::to_integer(
                    &connector_data.method,
                )))
                .map_err(StickfigureError::Io)?;
            cursor_c3
                .write_u8(u8::from(connector_data.reversed))
                .map_err(StickfigureError::Io)?;
            byte_vec.append(&mut Vec::from(buffer_c3));

            if build >= 65 {
                let mut buffer_c4 = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
                let mut cursor_c4 = Cursor::new(&mut buffer_c4[..]);
                cursor_c4
                    .write_f32::<BigEndian>(connector_data.smart_stretch_ancestral_value)
                    .map_err(StickfigureError::Io)?;
                byte_vec.append(&mut Vec::from(buffer_c4));
            }
        }
    }

    let mut node_header_bytes = [0u8; 7]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut node_header_writer = Cursor::new(&mut node_header_bytes[..]);
    node_header_writer
        .write_i8(node.node_type.to_integer())
        .map_err(StickfigureError::Io)?;
    node_header_writer
        .write_i32::<BigEndian>(node.draw_order_index.0)
        .map_err(StickfigureError::Io)?;
    node_header_writer
        .write_u8(u8::from(node.is_static))
        .map_err(StickfigureError::Io)?;
    node_header_writer
        .write_u8(u8::from(node.is_stretchy))
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(node_header_bytes));

    if version >= 403 && build >= 48 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.is_floaty))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    if version >= 248 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.is_smart_stretch))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 252 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.do_not_apply_smart_stretch))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    if version >= 403 && build >= 50 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.smart_stretch_reset_impulse))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    let mut segment_color_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut segment_color_writer = Cursor::new(&mut segment_color_bytes[..]);
    segment_color_writer
        .write_u8(u8::from(node.use_segment_color))
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(segment_color_bytes));

    if version >= 256 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.use_circle_outline))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 21 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.circle_is_hollow))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 176 {
        let mut field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.use_gradient))
            .map_err(StickfigureError::Io)?;
        field_writer
            .write_u8(u8::from(node.reverse_gradient))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 20 {
        let mut field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i16::<BigEndian>(i16::from(GradientMode::to_integer(&node.gradient_mode)))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    let mut node_geometry_bytes = [0u8; 29]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut node_geometry_writer = Cursor::new(&mut node_geometry_bytes[..]);
    node_geometry_writer
        .write_u8(u8::from(node.use_segment_scale))
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_f32::<BigEndian>(node.local_x)
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_f32::<BigEndian>(node.local_y)
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_f32::<BigEndian>(node.scale)
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_f32::<BigEndian>(node.default_length)
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_f32::<BigEndian>(node.length)
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_i32::<BigEndian>(node.default_thickness)
        .map_err(StickfigureError::Io)?;
    node_geometry_writer
        .write_i32::<BigEndian>(node.thickness)
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(node_geometry_bytes));

    if version >= 320 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i32::<BigEndian>(node.segment_curve_radius_and_default_curve_radius)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 20 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.curve_circulization))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 21 {
        let mut field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i16::<BigEndian>(node.segment_curve_polyfill_precision)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 256 {
        let mut field_bytes = [0u8; 3]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.half_arc))
            .map_err(StickfigureError::Io)?;
        field_writer
            .write_i16::<BigEndian>(node.right_triangle_direction)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 300 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.triangle_upside_down))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 36 {
        let mut field_bytes = [0u8; 8]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        // Allow justified because we can't avoid truncating without adding redundant i32 trapezoid thickness start/end fields
        // Only figures from a small range of build versions will be effected
        #[allow(clippy::cast_possible_truncation)]
        if build < 64 {
            field_writer
                .write_i32::<BigEndian>(node.trapezoid_thickness_start as i32)
                .map_err(StickfigureError::Io)?;
            field_writer
                .write_i32::<BigEndian>(node.trapezoid_thickness_end as i32)
                .map_err(StickfigureError::Io)?;
        } else {
            field_writer
                .write_f32::<BigEndian>(node.trapezoid_thickness_start)
                .map_err(StickfigureError::Io)?;
            field_writer
                .write_f32::<BigEndian>(node.trapezoid_thickness_end)
                .map_err(StickfigureError::Io)?;
        }
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build == 36 {
        let field_bytes = [0u8; 8]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 256 && build != 36 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.trapezoid_top_thickness_ratio)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 36 {
        let mut field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.trapezoid_is_rounded_start))
            .map_err(StickfigureError::Io)?;
        field_writer
            .write_u8(u8::from(node.trapezoid_is_rounded_end))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 256 {
        let mut field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i16::<BigEndian>(node.num_polygon_vertices)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 248 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.default_local_angle)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    let mut node_angle_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut node_angle_writer = Cursor::new(&mut node_angle_bytes[..]);
    node_angle_writer
        .write_f32::<BigEndian>(node.local_angle)
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(node_angle_bytes));

    if version >= 248 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.default_angle)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    let mut node_color_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut node_color_writer = Cursor::new(&mut node_color_bytes[..]);
    node_color_writer
        .write_all(&[
            node.color.alpha,
            node.color.blue,
            node.color.green,
            node.color.red,
        ])
        .map_err(StickfigureError::Io)?;
    byte_vec.append(&mut Vec::from(node_color_bytes));

    if version >= 176 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_all(&[
                node.gradient_color.alpha,
                node.gradient_color.blue,
                node.gradient_color.green,
                node.gradient_color.red,
            ])
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 256 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_all(&[
                node.circle_outline_color.alpha,
                node.circle_outline_color.blue,
                node.circle_outline_color.green,
                node.circle_outline_color.red,
            ])
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 39 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(node.is_angle_locked))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && (39..=50).contains(&build) {
        let field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 51 {
        let mut bool = node.angle_lock_is_main_node;
        if build < 56 {
            bool = !bool;
        }
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_u8(u8::from(bool))
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && (51..=56).contains(&build) {
        let mut field_bytes = [0u8; 8]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.angle_lock_offset_minuend)
            .map_err(StickfigureError::Io)?;
        field_writer
            .write_f32::<BigEndian>(node.angle_lock_offset_subtrahend)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 57 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.angle_lock_offset)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 63 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.angle_lock_relative_start)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 67 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.angle_lock_stickfigure_start)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 63 {
        let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_i8(node.angle_lock_relative_multiplier)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 39 {
        if build <= 40 {
            let mut field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
            let mut field_writer = Cursor::new(&mut field_bytes[..]);
            field_writer
                .write_i16::<BigEndian>(i16::from(node.is_drag_locked))
                .map_err(StickfigureError::Io)?;
            byte_vec.append(&mut Vec::from(field_bytes));
        } else {
            let mut field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
            let mut field_writer = Cursor::new(&mut field_bytes[..]);
            field_writer
                .write_u8(u8::from(node.is_drag_locked))
                .map_err(StickfigureError::Io)?;
            byte_vec.append(&mut Vec::from(field_bytes));
        }
    }
    if version >= 403 && (41..=45).contains(&build) {
        let field_bytes = [0u8; 2]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 46 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.drag_lock_angle)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && build >= 41 {
        let mut field_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut field_writer = Cursor::new(&mut field_bytes[..]);
        field_writer
            .write_f32::<BigEndian>(node.smart_stretch_multiplier)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(field_bytes));
    }
    if version >= 403 && (41..=45).contains(&build) {
        let field_bytes = [0u8; 1]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(field_bytes));
    }

    Ok(byte_vec)
}

fn write_polyfill_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut output_bytes = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut byte_writer = Cursor::new(&mut output_bytes[..]);
    // Allow justified because wrapping and truncation would only matter at absurdly large polyfill counts
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let number_of_polyfills = stickfigure.polyfills.len() as i32;
    byte_writer
        .write_i32::<BigEndian>(number_of_polyfills)
        .map_err(StickfigureError::Io)?;

    byte_vec.append(&mut Vec::from(output_bytes));

    for polyfill in &stickfigure.polyfills {
        byte_vec.append(&mut write_polyfill(polyfill)?);
    }

    Ok(byte_vec)
}

fn write_polyfill(rc_polyfill: &Rc<RefCell<Polyfill>>) -> Result<Vec<u8>, StickfigureError> {
    let polyfill = rc_polyfill.borrow();

    let mut byte_vec = Vec::new();

    let mut output_bytes = [0u8; 13]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
    let mut byte_writer = Cursor::new(&mut output_bytes[..]);
    byte_writer
        .write_i32::<BigEndian>(polyfill.anchor_node_draw_index.0)
        .map_err(StickfigureError::Io)?;
    byte_writer
        .write_all(&[
            polyfill.color.alpha,
            polyfill.color.blue,
            polyfill.color.green,
            polyfill.color.red,
        ])
        .map_err(StickfigureError::Io)?;
    byte_writer
        .write_u8(u8::from(polyfill.use_polyfill_color))
        .map_err(StickfigureError::Io)?;

    // Allow justified because wrapping and truncation would only matter at absurdly large node counts
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let number_of_attached_nodes = polyfill.attached_node_draw_indices.len() as i32;
    byte_writer
        .write_i32::<BigEndian>(number_of_attached_nodes)
        .map_err(StickfigureError::Io)?;

    byte_vec.append(&mut Vec::from(output_bytes));

    for draw_index in &polyfill.attached_node_draw_indices {
        let mut local_buffer = [0u8; 4]; // fixed-size output_bytes. should be set to max size of bytes written in this section.
        let mut local_cursor = Cursor::new(&mut local_buffer[..]);
        local_cursor
            .write_i32::<BigEndian>(draw_index.0)
            .map_err(StickfigureError::Io)?;
        byte_vec.append(&mut Vec::from(local_buffer));
    }

    Ok(byte_vec)
}
