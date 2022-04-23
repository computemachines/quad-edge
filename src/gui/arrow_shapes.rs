use std::iter::zip;

use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};

use super::arrow_instance::ATTRIBUTE_WEIGHT;

/*  1   3
 *  | / |
 *  2   4
 */
fn unit_quad_strip_x_splits(pos_x_splits: Vec<f32>, uv_x_splits: Vec<f32>) -> Mesh {
    let mut quads = Mesh::new(PrimitiveTopology::TriangleStrip);

    let mut v_pos = vec![];
    let mut v_uv = vec![];
    let mut indices = vec![];
    let mut i = 0;
    for (pos_x_split, uv_x_split) in zip(pos_x_splits, uv_x_splits) {
        v_pos.push([pos_x_split, 1.0, 0.0]);
        v_uv.push([uv_x_split, 1.0]);
        indices.push(i);
        i += 1;

        v_pos.push([pos_x_split, -1.0, 0.0]);
        v_uv.push([uv_x_split, 0.0]);
        indices.push(i);
        i += 1;
    }
    
    quads.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    quads.set_attribute(Mesh::ATTRIBUTE_UV_0, v_uv);
    quads.set_indices(Some(Indices::U32(indices)));

    quads
}

pub fn build_arrow_strip_mesh() -> Mesh {
    let mut quad = unit_quad_strip_x_splits(vec![-1.0, 1.0, -1.0, 1.0], vec![0.0, 0.1, 0.9, 1.0]);
    quad.set_attribute(ATTRIBUTE_WEIGHT, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

    let v_color = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    quad.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);
    quad
}

pub fn build_line_mesh() -> Mesh {
    let mut lines = Mesh::new(PrimitiveTopology::TriangleList);

    let v_color = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let v_pos = vec![
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let indices: Vec<u32> = vec![0, 2, 1, 2, 1, 3];
    lines.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    lines.set_attribute(ATTRIBUTE_WEIGHT, vec![0.0, 0.0, 1.0, 1.0]);

    let uvs = vec![
        [0.0, 0.0], //0
        [0.0, 1.0], //1
        [1.0, 0.0], //2
        [1.0, 1.0], //3
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    lines
}
