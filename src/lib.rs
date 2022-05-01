pub mod delaunay_voronoi;
pub mod geometry;
pub mod mesh;
pub mod topological;

#[cfg(test)]
mod tests {
    use crate::{
        delaunay_voronoi::{DelaunayMesh, VoronoiVertex},
        topological::TopologicalMesh,
    };

    #[test]
    fn make_floating_edge() {
        let mut mesh = DelaunayMesh::new();
        let a = mesh.insert_vertex((0.0, 1.0));
        let b = mesh.insert_vertex((5.0, 1.0));
        let inf = mesh.insert_face(VoronoiVertex::Infinite);
        let e = mesh.make_edge(a, b, inf, inf);

        let ring = mesh.get_primal_onext_ring(e);
        assert_eq!(ring.collect::<Vec<_>>().len(), 1);
    }

    #[test]
    fn make_dangling_edge() {
        let mut mesh = TopologicalMesh::new();
        let a = mesh.insert_vertex("A");
        let b = mesh.insert_vertex("B");
        let c = mesh.insert_vertex("C");
        let dummy = mesh.insert_face("(face)");

        let e1 = mesh.make_edge(a, b, dummy, dummy);
        let e2 = mesh.connect_vertex(e1, c);

        // check primal topology
        assert_eq!(mesh.primal(e1).lnext().id(), e2);
        assert_eq!(mesh.primal(e1).onext().id(), e1);
        assert_eq!(mesh.primal(e1).sym().id(), e1.sym());
        assert_eq!(mesh.primal(e1).sym().onext().id(), e2);
        assert_eq!(mesh.primal(e2).onext().id(), e1.sym());
        assert_eq!(mesh.primal(e2).sym().onext().id(), e2.sym());

        // check dual topology
        // assert_eq
    }

    #[test]
    fn make_triangle() {
        let mut mesh = TopologicalMesh::new();
        let a = mesh.insert_vertex("A");
        let b = mesh.insert_vertex("B");
        let c = mesh.insert_vertex("C");
        let infinity = mesh.insert_face("(infinity)");
        let inside = mesh.insert_face("(inside)");

        let e1 = mesh.make_edge(a, b, inside, infinity);
        let e2 = mesh.connect_vertex(e1, c);
        let e3 = mesh.connect_primal(e2, e1);
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
        for e in [e1, e1.sym(), e2, e2.sym()] {
            println!("ONext Ring ({:?}", e);
            for i in mesh.get_primal_onext_ring(e) {
                println!("{:?}={:?}", i, mesh.get_primal(i));
            }
            println!("");
        }
    }
}
