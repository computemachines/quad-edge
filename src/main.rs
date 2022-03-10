use quad_edge::{
  delaunay_voronoi::{DelaunayMesh, VoronoiVertex},
  topological::TopologicalMesh,
};
#[cfg(feature = "gui")]
mod gui;

fn main() {
    let x = 0.05;

    let mut mesh = DelaunayMesh::new();
    let a = mesh.insert_vertex((0.0, 1.0));
    let b = mesh.insert_vertex((0.0, -1.0));
    let c = mesh.insert_vertex((x, 0.0));
    let d = mesh.insert_vertex((-x, 0.0));
    let inf = mesh.insert_face(VoronoiVertex::Infinite);

    let e1 = mesh.make_edge(d, a, inf, inf);
    let e2 = mesh.make_edge(b, c, inf, inf);
    let e3 = mesh.connect_primal(e2.sym(), e1);
    let e4 = mesh.connect_primal(e2, e1.sym());

    let e5 = mesh.connect_primal(e3.sym(), e1.sym());
    // let e5 = mesh.connect_primal(e3, e4);
    println!("isDelaunay: {}", mesh.is_delaunay(e5));

    #[cfg(feature = "gui")]
    gui::explore_mesh(mesh);
}
