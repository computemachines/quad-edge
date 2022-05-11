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
    fn dangling_edge() {
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

        let d1 = e1.rot();
        let d2 = e2.rot();

        assert_eq!(mesh.dual(d1).get().borrow().org, dummy);

        // check dual topology
        assert_eq!(mesh.dual(d1).lnext().id(), d1);
        assert_eq!(mesh.dual(d2).lnext().id(), d1.sym());
        assert_eq!(mesh.dual(d1).onext().id(), d1.sym());
        assert_eq!(mesh.dual(d2).onext().id(), d1);
        assert_eq!(mesh.dual(d1).oprev().id(), d2);
        assert_eq!(mesh.dual(d1).sym().lnext().id(), d2);
        assert_eq!(mesh.dual(d2).sym().lnext().id(), d2.sym());
    }

    #[test]
    fn simple_triangle() {
        let mut mesh = TopologicalMesh::new();
        let a = mesh.insert_vertex("A");
        let b = mesh.insert_vertex("B");
        let c = mesh.insert_vertex("C");
        let infinity = mesh.insert_face("(infinity)");
        let inside = mesh.insert_face("(inside)");

        let e1 = mesh.make_edge(a, b, inside, infinity);
        let e2 = mesh.connect_vertex(e1, c);
        let e3 = mesh.connect_primal(e2, e1);

        let d1 = e1.rot();
        let d2 = e2.rot();
        let d3 = e3.rot();

        assert_eq!(mesh.primal(e3).sym().get().borrow().org, a);
        assert_eq!(mesh.primal(e2.sym()).dest().borrow().to_string(), "B");
        assert_eq!(mesh.primal(e1).lnext().id(), e2);
        assert_eq!(mesh.primal(e1).sym().lnext().id(), e3.sym());
        assert_eq!(mesh.primal(e3).lnext().id(), e1);
        assert_eq!(mesh.primal(e3).sym().lnext().id(), e2.sym());
        assert_eq!(mesh.primal(e2).lnext().id(), e3);
        assert_eq!(mesh.primal(e2).sym().lnext().id(), e1.sym());
        assert_eq!(mesh.primal(e1.sym()).onext().id(), e2);
    }

    #[test]
    fn connect_splits_face() {
        let mut mesh = TopologicalMesh::new();
        let a = mesh.insert_vertex("A");
        let b = mesh.insert_vertex("B");
        let c = mesh.insert_vertex("C");
        let d = mesh.insert_vertex("D");
        let f = mesh.insert_vertex("F");
        let infinity = mesh.insert_face("(infinity)");

        let e1 = mesh.make_edge(a, b, infinity, infinity);
        let e2 = mesh.connect_vertex(e1, c);
        let e3 = mesh.make_edge(d, f, infinity, infinity);
        let e4 = mesh.connect_primal(e2.sym(), e3);

        // for (i, dedge) in mesh.primal_dedges.drain(..).enumerate() {
        //     let temp = dedge.unwrap();
        //     let dedge = temp.borrow();
        //     println!("{i}: org={}, onext={}", dedge.org.0, dedge.onext.0);
        // }

        assert_eq!(mesh.primal(e2).onext().id(), e1.sym());
        assert_eq!(mesh.primal(e1).rprev().id(), e4);
        assert_eq!(mesh.primal(e2).oprev().id(), e4);
    }

    #[test]
    fn simple_swap() {
        let mut mesh = TopologicalMesh::new();
        let a = mesh.insert_vertex("A");
        let b = mesh.insert_vertex("B");
        let c = mesh.insert_vertex("C");
        let d = mesh.insert_vertex("D");
        let dummy = mesh.insert_face("(face)");

        let e1 = mesh.make_edge(a, b, dummy, dummy);
        let e2 = mesh.make_edge(c, d, dummy, dummy);
    }
}
