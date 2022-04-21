use bevy::prelude::*;

use super::arrow_instance::ATTRIBUTE_WEIGHT;

pub fn build_line_mesh() -> Mesh {

    let mut lines = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);

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