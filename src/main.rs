use quad_edge::{delaunay_voronoi::DelaunayMesh, topological::TopologicalMesh};

fn main() {
    let mut mesh = TopologicalMesh::new();
    let e = mesh.make_edge("A", "B");
    let f = mesh.make_edge("D", "E");

    mesh.splice_primal(e, f);

    // for i in mesh.get_primal_onext_ring(e) {
    //     println!("{:?}", mesh.get_primal(i));
    // }
}
