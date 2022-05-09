use std::iter;

use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

pub fn build_circle(subdivisions: u32) -> Mesh {
    let mut circle = Mesh::new(PrimitiveTopology::TriangleList);

    // let mut v_uv = vec![];
    // let mut v_weight = vec![];
    // let mut i = 0;
    // for (&pos_x_split, &uv_x_split, &weight_x_split) in
    // multizip((&pos_x_splits, &uv_x_splits, &weight_x_splits))
    // {
    //     v_pos.push([pos_x_split, 1.0, 0.0]);
    //     v_uv.push([uv_x_split, 1.0]);
    //     v_weight.push(weight_x_split);
    //     indices.push(i);
    //     i += 1;

    //     v_pos.push([pos_x_split, -1.0, 0.0]);
    //     v_uv.push([uv_x_split, 0.0]);
    //     v_weight.push(weight_x_split);
    //     indices.push(i);
    //     i += 1;
    // }

    let mut v_pos = vec![];
    let mut v_normal = vec![];
    let mut v_uv = vec![];
    let mut indices = vec![];

    let ns = 2_i32.pow(subdivisions);
    let steps = (-ns..ns).map(|i| i as f32 / ns as f32);
    let negate = |i: f32| -i;

    let right = iter::repeat(1.0f32).zip(steps.clone());
    let top = steps.clone().map(negate).zip(iter::repeat(1.0f32));
    let left = iter::repeat(-1.0f32).zip(steps.clone().map(negate));
    let bottom = steps.clone().zip(iter::repeat(-1.0f32));

    v_pos.push([0.0, 0.0, 0.0]);
    v_normal.push([0.0, 0.0, 1.0]);
    v_uv.push([0.5, 0.5]);

    for (x, y) in right.chain(top).chain(left).chain(bottom) {
        let dist = (x * x + y * y).sqrt();
        // normalized position
        v_pos.push([x / dist, y / dist, 0.0]);
        v_normal.push([0.0, 0.0, 1.0]);
        v_uv.push([x * 0.5 + 0.5, y * 0.5 + 0.5]);
    }

    for i in 1..8 * ns as u32 {
        indices.append(&mut vec![0, i, i + 1]);
    }
    indices.append(&mut vec![0, 8 * ns as u32, 1]);

    // for (i, angle) in (1..).map(|i| (i, 2.0 * i as f32 * 3.1415 / segments as f32)) {
    //     v_pos.push([angle.cos(), angle.sin(), 0.0]);
    //     v_normal.push([0.0, 0.0, 1.0]);
    //     indices.append(&mut vec![0, i, i + 1]);
    // }
    // indices.append(&mut vec![0, segments, 1]);

    circle.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    circle.insert_attribute(Mesh::ATTRIBUTE_NORMAL, v_normal);
    circle.insert_attribute(Mesh::ATTRIBUTE_UV_0, v_uv);
    // quads.insert_attribute(ATTRIBUTE_WEIGHT, v_weight);
    circle.set_indices(Some(Indices::U32(indices)));

    circle
}

pub fn build_rect(a: Vec2, b: Vec2) -> Mesh {
    let mut quad = Mesh::new(PrimitiveTopology::TriangleList);

    let mut v_pos = vec![
        [a.x, -1.0, 0.0],
        [a.x, 1.0, 0.0],
        [b.x, 1.0, 0.0],
        [b.x, -1.0, 0.0],
    ];
    let mut v_normal = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];
    let mut v_uv = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

    let indices = vec![0, 2, 1, 0, 3, 2];

    quad.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    quad.insert_attribute(Mesh::ATTRIBUTE_NORMAL, v_normal);
    quad.insert_attribute(Mesh::ATTRIBUTE_UV_0, v_uv);
    // quads.insert_attribute(ATTRIBUTE_WEIGHT, v_weight);
    quad.set_indices(Some(Indices::U32(indices)));

    quad
}
