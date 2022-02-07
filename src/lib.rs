use std::{cell::RefCell};

pub mod mesh;
pub mod quad;
pub mod delaunay_voronoi;


#[cfg(test)]
mod tests {
    use crate::delaunay_voronoi::DelaunayMesh;

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
        for i in ring {
            println!("{:?}, {:?}", i, mesh.get_primal(i));
        }
    }
}
