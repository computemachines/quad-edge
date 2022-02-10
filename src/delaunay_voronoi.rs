use crate::mesh::Mesh;


pub type DelaunayVertex = (f64, f64);

#[derive(Debug)]
pub enum VoronoiVertex {
    Infinite,
    Finite(f64, f64),
}

impl Default for VoronoiVertex {
    fn default() -> Self {
        VoronoiVertex::Infinite
    }
}

pub type DelaunayMesh = Mesh<DelaunayVertex, VoronoiVertex>;
