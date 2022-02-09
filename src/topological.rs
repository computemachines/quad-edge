use crate::mesh::Mesh;
use crate::simple::SimpleDEdge;

pub type TopologicalMesh<T> = Mesh<SimpleDEdge<T>, SimpleDEdge<T>>;
