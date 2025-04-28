use byteorder::{BigEndian, ReadBytesExt};
use core2::io::Read;
extern crate alloc;
use alloc::{rc::Rc, vec::Vec};

use core::cell::RefCell;

use crate::{
    error::*,
    structs::{node::*, polyfill::*, stickfigure::*},
    Color,
};

fn read_stickfigure_header(
    reader: &mut impl Read,
    stickfigure: &mut Stickfigure,
) -> Result<(), StickfigureError> {
    stickfigure.version = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    if stickfigure.version >= 403 {
        stickfigure.build = reader
            .read_i32::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    stickfigure.scale = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    stickfigure.color = Color {
        alpha: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        blue: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        green: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        red: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
    };

    Ok(())
}

pub fn read_stickfigure(reader: &mut impl Read) -> Result<Stickfigure, LibraryError> {
    let mut stickfigure = Stickfigure::default();

    read_stickfigure_header(reader, &mut stickfigure)?;

    let version = stickfigure.version;
    let build = stickfigure.build;

    if version > Stickfigure::default().version {
        return Err(LibraryError::UnsupportedVersion(version));
    } else if version == Stickfigure::default().version && build > Stickfigure::default().build {
        return Err(LibraryError::UnsupportedBuild(version, build));
    }

    read_child_nodes(
        reader,
        stickfigure.version,
        stickfigure.build,
        DrawOrderIndex(-1),
        1,
        &mut stickfigure,
    )?;

    if stickfigure.version >= 230 {
        stickfigure.polyfills = read_polyfill_header(reader)?;
    }

    Ok(stickfigure)
}

fn read_child_nodes(
    reader: &mut impl Read,
    version: i32,
    build: i32,
    parent_draw_index: DrawOrderIndex,
    number_of_child_nodes_to_read: i32,
    stickfigure: &mut Stickfigure,
) -> Result<(), StickfigureError> {
    for _ in 0..number_of_child_nodes_to_read {
        let (node, number_of_child_nodes) = read_node(reader, version, build)?;

        let number_of_child_nodes = number_of_child_nodes;
        let draw_index = node.draw_order_index;

        if parent_draw_index.0 == -1 {
            stickfigure.add_root_node();
        } else {
            stickfigure.add_node_at_unique_index(node, parent_draw_index, draw_index)?;
        }

        read_child_nodes(
            reader,
            version,
            build,
            draw_index,
            number_of_child_nodes,
            stickfigure,
        )?;
    }

    Ok(())
}

fn read_node(
    reader: &mut impl Read,
    version: i32,
    build: i32,
) -> Result<(Node, i32), StickfigureError> {
    let mut node = Node::new();

    node.node_type = NodeType::from_integer(
        reader
            .read_i8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
    )
    .unwrap_or_default();
    node.draw_order_index.0 = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.is_static = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    node.is_stretchy = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;

    if version >= 248 {
        node.is_smart_stretch = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 252 {
        node.do_not_apply_smart_stretch = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    node.use_segment_color = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;

    if version >= 256 {
        node.use_circle_outline = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 21 {
        node.circle_is_hollow = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 176 {
        node.use_gradient = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.reverse_gradient = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 20 {
        node.gradient_mode = reader
            .read_i16::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    node.use_segment_scale = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    node.local_x = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.local_y = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.scale = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.default_length = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.length = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.default_thickness = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.thickness = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    if version >= 320 {
        node.segment_curve_radius_and_default_curve_radius = reader
            .read_i32::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 20 {
        node.curve_circulization = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 21 {
        node.segment_curve_polyfill_precision = reader
            .read_i16::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 256 {
        node.half_arc = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.right_triangle_direction = reader
            .read_i16::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        node.triangle_upside_down = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.trapezoid_top_thickness_ratio = reader
            .read_f32::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        node.num_polygon_vertices = reader
            .read_i16::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 248 {
        node.default_local_angle = reader
            .read_f32::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    node.local_angle = reader
        .read_f32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    if version >= 248 {
        node.default_angle = reader
            .read_f32::<BigEndian>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }

    node.color = Color {
        alpha: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        blue: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        green: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        red: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
    };

    if version >= 176 {
        node.gradient_color = Color {
            alpha: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            blue: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            green: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            red: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
        };
    }
    if version >= 256 {
        node.circle_outline_color = Color {
            alpha: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            blue: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            green: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            red: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
        };
    }

    let number_of_child_nodes = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    Ok((node, number_of_child_nodes))
}

fn read_polyfill_header(
    reader: &mut impl Read,
) -> Result<Vec<Rc<RefCell<Polyfill>>>, StickfigureError> {
    let number_of_polyfills = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let mut polyfills = Vec::with_capacity(number_of_polyfills as usize);
    for _ in 0..number_of_polyfills {
        let polyfill = Rc::new(RefCell::new(read_polyfill(reader)?));
        polyfills.push(polyfill);
    }

    return Ok(polyfills);
}

fn read_polyfill(reader: &mut impl Read) -> Result<Polyfill, StickfigureError> {
    let mut polyfill = Polyfill::default();

    polyfill.anchor_node_draw_index.0 = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    let [alpha, blue, green, red] = buf;

    polyfill.color = Color {
        alpha,
        blue,
        green,
        red,
    };
    polyfill.use_polyfill_color = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    let number_of_attached_nodes = reader
        .read_i32::<BigEndian>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let attached_node_draw_indices: Vec<DrawOrderIndex> = (0..number_of_attached_nodes)
        .map(|_| reader.read_i32::<BigEndian>().map(DrawOrderIndex))
        .collect::<Result<_, _>>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    polyfill.attached_node_draw_indices = attached_node_draw_indices;

    Ok(polyfill)
}
