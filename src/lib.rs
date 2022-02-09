use std::{cell::RefCell};

pub mod mesh;
pub mod quad;
pub mod delaunay_voronoi;
pub mod simple;
pub mod topological;


#[cfg(test)]
mod tests {
    use crate::{delaunay_voronoi::DelaunayMesh, topological::TopologicalMesh};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn make_mesh() {
        let mut mesh = DelaunayMesh::new();
        let e = mesh.make_edge((0.0, 1.0), (5.0, 1.0));

        let ring = mesh.get_primal_onext_ring(e);
        assert_eq!(ring.collect::<Vec<_>>().len(), 1);
    }

    #[test]
    fn topological_mesh() {
        let mut mesh = TopologicalMesh::new();
        let e = mesh.make_edge("A", "B");
        let f = mesh.make_edge("D", "E");
        for i in mesh.get_primal_onext_ring(e) {
            println!("{:?}", mesh.get_primal(i));
        }
    }
}
