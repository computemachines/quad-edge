
pub mod mesh;
pub mod delaunay_voronoi;
pub mod topological;
pub mod geometry;

#[cfg(test)]
mod tests {
    use crate::{delaunay_voronoi::{DelaunayMesh, VoronoiVertex}, topological::TopologicalMesh};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn make_mesh() {
        let mut mesh = DelaunayMesh::new();
        let a = mesh.insert_vertex((0.0, 1.0));
        let b = mesh.insert_vertex((5.0, 1.0));
        let inf = mesh.insert_face(VoronoiVertex::Infinite);
        let e = mesh.make_edge(a, b, inf, inf);

        let ring = mesh.get_primal_onext_ring(e);
        assert_eq!(ring.collect::<Vec<_>>().len(), 1);
    }

    #[test]
    fn topological_mesh() {
        let mut mesh = TopologicalMesh::new();
        let a = mesh.insert_vertex("A");
        let b = mesh.insert_vertex("B");
        let c = mesh.insert_vertex("C");
        let d = mesh.insert_vertex("D");
        let dummy = mesh.insert_face("(face)");

        let e1 = mesh.make_edge(a, b, dummy, dummy);
        let e2 = mesh.make_edge(c, d, dummy, dummy);
        for i in mesh.get_primal_onext_ring(e1) {
            println!("{:?}", mesh.get_primal(i));
        }
    }
}
