use core::cell::RefCell;

use byteorder::{BigEndian, WriteBytesExt};
use core2::io::{Cursor, Write};
extern crate alloc;
use alloc::{format, rc::Rc, vec, vec::Vec};

use crate::{
    error::*,
    structs::{node::*, polyfill::*, stickfigure::{self, *}}, ConnectorMethod,
};

fn write_stickfigure_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    cursor
        .write_i32::<BigEndian>(stickfigure.version)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer));

    if stickfigure.version >= 403 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i32::<BigEndian>(stickfigure.build)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer2 = [0u8; 8]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor2 = Cursor::new(&mut buffer2[..]);
    cursor2
        .write_f32::<BigEndian>(stickfigure.scale)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor2
        .write_all(&[
            stickfigure.color.alpha,
            stickfigure.color.blue,
            stickfigure.color.green,
            stickfigure.color.red,
        ])
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer2));

    Ok(byte_vec)
}

pub fn write_stickfigure(stickfigure: &Stickfigure) -> Result<Vec<u8>, LibraryError> {
    let mut byte_vec = Vec::new();

    byte_vec.append(&mut write_stickfigure_header(&stickfigure)?);

    byte_vec.append(&mut write_child_nodes(
        stickfigure.version,
        stickfigure.build,
        DrawOrderIndex(0),
        stickfigure,
    )?);

    if stickfigure.version >= 230 {
        byte_vec.append(&mut write_polyfill_header(&stickfigure)?);
    }

    if stickfigure.version >= 403 && stickfigure.build >= 38 {
        byte_vec.append(&mut write_connector_data(&stickfigure)?);
    }

    Ok(byte_vec)
}

fn write_connector_data(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let connector_nodes = stickfigure.get_nodes_with_property(|node| node.borrow().connector_data.is_some());

    let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    cursor
        .write_i32::<BigEndian>(connector_nodes.len() as i32)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer));

    for draw_index in connector_nodes {
        let connector_node = stickfigure.get_node(draw_index).ok_or_else(|| StickfigureError::InvalidDrawIndex(draw_index.0, format!("Attempted to get connector node that does not exist when writing .nodes. Probably a library bug.")))?;
        let mut buffer_: [u8; 8] = [0u8; 8]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i32::<BigEndian>(connector_node.borrow().get_draw_order_index().0)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;

        let connector_data= &mut connector_node.borrow_mut().connector_data;

        match connector_data {
            Some(data) => {
                cursor_
                    .write_i32::<BigEndian>(data.end_node_draw_index.0)
                    .or_else(|err| return Err(StickfigureError::Io(err)))?;
            },
            None => return Err(StickfigureError::GenericError(format!("Attempted to get undefined connector data of node (while writing .nodes file). Node {:?}.", draw_index)))?,
        }

        byte_vec.append(&mut Vec::from(buffer_));
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

    if !stickfigure.all_draw_indices_exist(&vec![draw_index]) {
        return Err(StickfigureError::InvalidDrawIndex(
            draw_index.0,
            format!("Cannot finish writing this stickfigure file."),
        ));
    }

    let node_index = stickfigure.node_index_from_draw_order(draw_index);

    if let Some(node) = stickfigure.nodes.node_weight(node_index) {
        byte_vec.append(&mut write_node(version, build, node, stickfigure)?);

        let children = stickfigure.get_children(draw_index);
        let number_of_child_nodes = children.len() as i32;

        let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor = Cursor::new(&mut buffer[..]);
        cursor
            .write_i32::<BigEndian>(number_of_child_nodes)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer));

        if version >= 403 && build > 38 {
            for draw_index in &children {
                if let Some(node) = stickfigure.get_node(*draw_index) {
                    let connector_data_present = node.borrow().connector_data.is_some();
                    let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
                    let mut cursor = Cursor::new(&mut buffer[..]);
                    cursor
                        .write_i32::<BigEndian>(connector_data_present as i32)
                        .or_else(|err| return Err(StickfigureError::Io(err)))?;
                    byte_vec.append(&mut Vec::from(buffer));
                } else {
                    return Err(StickfigureError::GenericError(format!("Failed to get child node from index (while writing .nodes).")));
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
    stickfigure: &Stickfigure
) -> Result<Vec<u8>, StickfigureError> {
    {
        let mut node = rc_node.borrow_mut();

        node.local_x = node.get_local_x();
        node.local_y = node.get_local_y();

        match node.triangle_type {
            TriangleType::Isosceles => {
                node.right_triangle_direction = 0;
            },
            TriangleType::RightTriangle => {
                if node.triangle_flipped {
                    node.right_triangle_direction = -1;
                } else {
                    node.right_triangle_direction = 1;
                }
            },
        }

        if node.node_type.to_integer() == NodeType::RootNode.to_integer() {
            node.is_angle_locked = false;
            node.angle_lock_is_main_node = false;
            node.angle_lock_relative_start = 0.0;
            node.angle_lock_stickfigure_start = 0.0;
            node.angle_lock_offset_minuend = 0.0;
            node.angle_lock_offset_subtrahend = 0.0;
            node.angle_lock_offset = 0.0;
            
        } else {
            if let Some(parent_node_draw_index) = stickfigure.get_parent(node.get_draw_order_index()) {
                if let Some(parent_node) = stickfigure.get_node(parent_node_draw_index) {
                    if let Some(root_node) = stickfigure.get_node(DrawOrderIndex(0)) {
                        match node.angle_lock_mode {
                            AngleLockMode::None => {
                                node.is_angle_locked = false;
                                node.angle_lock_is_main_node = false;
                                node.angle_lock_relative_start = 0.0;
                                node.angle_lock_stickfigure_start = 0.0;
                            },
                            AngleLockMode::Absolute => {
                                node.is_angle_locked = true;
                                node.angle_lock_is_main_node = true;
                                node.angle_lock_relative_start = 0.0;
                                node.angle_lock_stickfigure_start = 0.0;
                            },
                            AngleLockMode::Relative => {
                                node.is_angle_locked = true;
                                node.angle_lock_is_main_node = false;
                                node.angle_lock_relative_start = parent_node.borrow().get_global_angle(stickfigure);
                                node.angle_lock_stickfigure_start = root_node.borrow().local_angle;
                            },
                        }
    
                        node.angle_lock_offset_minuend = node.get_global_angle(stickfigure);
                        node.angle_lock_offset_subtrahend = parent_node.borrow().get_global_angle(stickfigure);
                        node.angle_lock_offset = node.angle_lock_offset_minuend - node.angle_lock_offset_subtrahend;
                        
                    } else {
                        return Err(StickfigureError::GenericError(format!("Failed to get parent node draw index when setting private properties (while writing .nodes).")));
                    }
                } else {
                    return Err(StickfigureError::GenericError(format!("Failed to get parent node when setting private properties (while writing .nodes).")));
                }
            } else {
                return Err(StickfigureError::GenericError(format!("Failed to get root node when setting private properties (while writing .nodes).")));
    
            }
        }
    }
    let node = rc_node.borrow();

    let mut byte_vec = Vec::new();

    if version >= 403 && build >= 38 {
        if let Some(connector_data) = &rc_node.borrow().connector_data {
            let mut buffer_c1 = [0u8; 12]; // fixed-size buffer. should be set to max size of bytes written in this section.
            let mut cursor_c1 = Cursor::new(&mut buffer_c1[..]);
            cursor_c1
                .write_f32::<BigEndian>(connector_data.local_x)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            cursor_c1
                .write_f32::<BigEndian>(connector_data.local_y)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            cursor_c1
                .write_f32::<BigEndian>(connector_data.percent)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            byte_vec.append(&mut Vec::from(buffer_c1));

            if build >= 44 {
                let mut buffer_c2 = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
                let mut cursor_c2 = Cursor::new(&mut buffer_c2[..]);
                cursor_c2
                    .write_f32::<BigEndian>(connector_data.percent_default)
                    .or_else(|err| return Err(StickfigureError::Io(err)))?;
                byte_vec.append(&mut Vec::from(buffer_c2));
            }

            let mut buffer_c3 = [0u8; 9]; // fixed-size buffer. should be set to max size of bytes written in this section.
            let mut cursor_c3 = Cursor::new(&mut buffer_c3[..]);
            cursor_c3
                .write_f32::<BigEndian>(connector_data.value)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            cursor_c3
                .write_i32::<BigEndian>(ConnectorMethod::to_integer(&connector_data.method) as i32)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            cursor_c3
                .write_u8(connector_data.reversed as u8)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            byte_vec.append(&mut Vec::from(buffer_c3));

            if build >= 65 {
                let mut buffer_c4 = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
                let mut cursor_c4 = Cursor::new(&mut buffer_c4[..]);
                cursor_c4
                    .write_f32::<BigEndian>(connector_data.smart_stretch_ancestral_value)
                    .or_else(|err| return Err(StickfigureError::Io(err)))?;
                byte_vec.append(&mut Vec::from(buffer_c4));
            }
        }
    }

    let mut buffer1 = [0u8; 7]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor1 = Cursor::new(&mut buffer1[..]);
    cursor1
        .write_i8(node.node_type.to_integer())
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor1
        .write_i32::<BigEndian>(node.draw_order_index.0)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor1
        .write_u8(node.is_static as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor1
        .write_u8(node.is_stretchy as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer1));

    if version >= 403 && build >= 48 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.is_floaty as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    if version >= 248 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.is_smart_stretch as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 252 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.do_not_apply_smart_stretch as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    if version >= 403 && build >= 50 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.smart_stretch_reset_impulse as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer2 = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor2 = Cursor::new(&mut buffer2[..]);
    cursor2
        .write_u8(node.use_segment_color as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer2));

    if version >= 256 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.use_circle_outline as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 21 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.circle_is_hollow as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 176 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.use_gradient as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_u8(node.reverse_gradient as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 20 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i16::<BigEndian>(GradientMode::to_integer(&node.gradient_mode) as i16)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer3 = [0u8; 29]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor3 = Cursor::new(&mut buffer3[..]);
    cursor3
        .write_u8(node.use_segment_scale as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.local_x)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.local_y)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.scale)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.default_length)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.length)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_i32::<BigEndian>(node.default_thickness)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_i32::<BigEndian>(node.thickness)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer3));

    if version >= 320 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i32::<BigEndian>(node.segment_curve_radius_and_default_curve_radius)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 20 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.curve_circulization as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 21 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i16::<BigEndian>(node.segment_curve_polyfill_precision)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 256 {
        let mut buffer_ = [0u8; 3]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.half_arc as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_i16::<BigEndian>(node.right_triangle_direction)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 300 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.triangle_upside_down as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 36 {
        let mut buffer_ = [0u8; 8]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        if build < 64 {
            cursor_
                .write_i32::<BigEndian>(node.trapezoid_thickness_start as i32)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            cursor_
                .write_i32::<BigEndian>(node.trapezoid_thickness_end as i32)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
        } else {
            cursor_
                .write_f32::<BigEndian>(node.trapezoid_thickness_start)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            cursor_
                .write_f32::<BigEndian>(node.trapezoid_thickness_end)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
        }
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build == 36 {
        let buffer_ = [0u8; 8]; // fixed-size buffer. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 256 && build != 36 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.trapezoid_top_thickness_ratio)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));

    }
    if version >= 403 && build >= 36 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.trapezoid_is_rounded_start as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_u8(node.trapezoid_is_rounded_end as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 256 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i16::<BigEndian>(node.num_polygon_vertices)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 248 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.default_local_angle)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer4 = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor4 = Cursor::new(&mut buffer4[..]);
    cursor4
        .write_f32::<BigEndian>(node.local_angle)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer4));

    if version >= 248 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.default_angle)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer5 = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor5 = Cursor::new(&mut buffer5[..]);
    cursor5
        .write_all(&[
            node.color.alpha,
            node.color.blue,
            node.color.green,
            node.color.red,
        ])
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer5));

    if version >= 176 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_all(&[
                node.gradient_color.alpha,
                node.gradient_color.blue,
                node.gradient_color.green,
                node.gradient_color.red,
            ])
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 256 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_all(&[
                node.circle_outline_color.alpha,
                node.circle_outline_color.blue,
                node.circle_outline_color.green,
                node.circle_outline_color.red,
            ])
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 39 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.is_angle_locked as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && (build >= 39 && build <= 50) {
        let buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 51 {
        let mut bool = node.angle_lock_is_main_node;
        if build < 56 {
            bool = !bool;
        }
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(bool as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && (build >= 51 && build <= 56) {
        let mut buffer_ = [0u8; 8]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.angle_lock_offset_minuend)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_f32::<BigEndian>(node.angle_lock_offset_subtrahend)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 57 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.angle_lock_offset)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 63 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.angle_lock_relative_start)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 67 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.angle_lock_stickfigure_start)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 63 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i8(node.angle_lock_relative_multiplier)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 39 {
        if build <= 40 {
            let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
            let mut cursor_ = Cursor::new(&mut buffer_[..]);
            cursor_
                .write_i16::<BigEndian>(node.is_drag_locked as i16)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            byte_vec.append(&mut Vec::from(buffer_));
        } else {
            let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
            let mut cursor_ = Cursor::new(&mut buffer_[..]);
            cursor_
                .write_u8(node.is_drag_locked as u8)
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            byte_vec.append(&mut Vec::from(buffer_));
        }
        
    }
    if version >= 403 && (build >= 41 && build <= 45) {
        let buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 46 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.drag_lock_angle)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_))
    }
    if version >= 403 && build >= 41 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.smart_stretch_multiplier)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_))
    }
    if version >= 403 && (build >= 41 && build <= 45) {
        let buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        byte_vec.append(&mut Vec::from(buffer_));
    }

    Ok(byte_vec)
}

fn write_polyfill_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    let number_of_polyfills = stickfigure.polyfills.len() as i32;
    cursor
        .write_i32::<BigEndian>(number_of_polyfills)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    byte_vec.append(&mut Vec::from(buffer));

    for polyfill in &stickfigure.polyfills {
        byte_vec.append(&mut write_polyfill(&polyfill)?);
    }

    Ok(byte_vec)
}

fn write_polyfill(rc_polyfill: &Rc<RefCell<Polyfill>>) -> Result<Vec<u8>, StickfigureError> {
    let polyfill = rc_polyfill.borrow();

    let mut byte_vec = Vec::new();

    let mut buffer = [0u8; 13]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    cursor
        .write_i32::<BigEndian>(polyfill.anchor_node_draw_index.0)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor
        .write_all(&[
            polyfill.color.alpha,
            polyfill.color.blue,
            polyfill.color.green,
            polyfill.color.red,
        ])
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor
        .write_u8(if polyfill.use_polyfill_color { 1 } else { 0 })
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let number_of_attached_nodes = polyfill.attached_node_draw_indices.len() as i32;
    cursor
        .write_i32::<BigEndian>(number_of_attached_nodes)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    byte_vec.append(&mut Vec::from(buffer));

    for draw_index in &polyfill.attached_node_draw_indices {
        let mut local_buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut local_cursor = Cursor::new(&mut local_buffer[..]);
        local_cursor
            .write_i32::<BigEndian>(draw_index.0)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(local_buffer));
    }

    Ok(byte_vec)
}
