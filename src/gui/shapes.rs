use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};

#[derive(Debug, Clone)]
pub struct Circle {
    radius: f32,
    num_segments: usize,
}

impl Circle {
    pub fn new(radius: f32, num_segments: usize) -> Self {
        Self {
            radius,
            num_segments,
        }
    }
}

impl From<Circle> for Mesh {
    fn from(circle: Circle) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut positions = Vec::with_capacity(circle.num_segments+1);
        let normals = (0..circle.num_segments+1).into_iter().map(|_| [0., 0., 1.,]).collect::<Vec<_>>();
        let mut uvs = Vec::with_capacity(circle.num_segments+1);

        positions.push([0., 0., 0.]);
        uvs.push([0.5, 0.5]);
        for i in 0..circle.num_segments {
            positions.push([
                circle.radius * f32::cos(i as f32 * std::f32::consts::TAU / circle.num_segments as f32),
                circle.radius * f32::sin(i as f32 * std::f32::consts::TAU / circle.num_segments as f32),
                0.,
            ]);
            uvs.push([
                0.5 * f32::cos(i as f32 * std::f32::consts::TAU / circle.num_segments as f32) + 0.5,
                0.5 * f32::sin(i as f32 * std::f32::consts::TAU / circle.num_segments as f32) + 0.5,
            ])
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);


        let mut indices = Vec::with_capacity(circle.num_segments);
        for i in 1..circle.num_segments as u32 {
            indices.extend_from_slice(&[
                0, i, i+1,
            ]);
        }
        indices.extend_from_slice(&[0, circle.num_segments as u32, 1]);
        mesh.set_indices(Some(Indices::U32(indices)));
        
        mesh
    }
}
