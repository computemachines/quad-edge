use std::fmt::Debug;

use quad_edge::{
    delaunay_voronoi::{DelaunayMesh, VoronoiVertex},
    mesh::{quad::{DualDEdgeEntity, PrimalDEdgeEntity}, Mesh},
//    topological::TopologicalMesh,
};
#[cfg(feature = "gui")]
mod gui;

fn main() {
    let mut mesh = DelaunayMesh::new();
    let a = mesh.insert_vertex((0.0, -100.0));
    let b = mesh.insert_vertex((0.0, 100.0));
    let c = mesh.insert_vertex((100.0, 0.0));
    let infinity = mesh.insert_face(VoronoiVertex::Infinite);
    let inside = mesh.insert_face(VoronoiVertex::Finite(0.0, 0.0));

    let e1 = mesh.make_edge(a, b, infinity, inside);
    let e2 = mesh.connect_vertex(e1, c);
    let _e3 = mesh.connect_primal(e2, e1);

    mesh.insert_delaunay_vertex((-200.0, 0.0).into());
    

    #[cfg(feature = "gui")]
    gui::explore_mesh(mesh);
}

fn print_primal_dedge_info<T: Debug, U: Debug, Cache: Default>(mesh: &Mesh<T, U, Cache>, e: PrimalDEdgeEntity) {
    let edge = mesh.get_primal(e).borrow();
    println!(
        "{:?} ~> (org={:?}, onext={:?})",
        e.0,
        mesh.get_vertex(edge.org).borrow(),
        edge.onext.0
    );
}
fn print_dual_dedge_info<T: Debug, U: Debug, Cache: Default>(mesh: &Mesh<T, U, Cache>, e: DualDEdgeEntity) {
    let edge = mesh.get_dual(e).borrow();
    println!(
        "{:?}\' ~> (org={:?}, onext={:?}\')",
        e.0,
        mesh.get_face(edge.org).borrow(),
        edge.onext.0
    );
}

fn print_edge_info<T: Debug, U: Debug, Cache: Default>(mesh: &Mesh<T, U, Cache>, e: PrimalDEdgeEntity) {
    println!("(Vertex) ONext Ring ({:?}", e);
    for i in mesh.get_primal_onext_ring(e) {
        // println!("{:?}={:?}", i, mesh.get_primal(i));
        print_primal_dedge_info(mesh, i);
    }
    println!("");
    println!("(Face) ONext Ring ({:?})", e.rot());
    for i in mesh.get_dual_onext_ring(e.rot()) {
        // println!("{:?}={:?}", i, mesh.get_dual(i));
        print_dual_dedge_info(mesh, i);
    }
    println!("");

    println!("{}.LNext = {}", e.0, mesh.primal(e).lnext().id().0);
    println!("");
}
fn show_primal<T: Debug, U: Debug, Cache: Default>(mesh: &Mesh<T, U, Cache>, e: PrimalDEdgeEntity) {
    let edge = mesh.get_primal(e).borrow();
    let edge_inv_rot = mesh.get_dual(e.rot_inv()).borrow();
    println!("{}.org = {:?}", e.0, mesh.get_vertex(edge.org).borrow(),);
    println!(
        "{}.left = {:?}",
        e.0,
        mesh.get_face(edge_inv_rot.org).borrow(),
    );
}

// fn manual_testing() {
//     println!("connect vertex -A-B-C-* into triangle");
//     let mut mesh = TopologicalMesh::new();
//     let a = mesh.insert_vertex("A");
//     let b = mesh.insert_vertex("B");
//     let c = mesh.insert_vertex("C");
//     let infinity = mesh.insert_face("(infinity)");
//     let inside = mesh.insert_face("(inside)");

//     let e1 = mesh.make_edge(a, b, inside, infinity);
//     show_primal(&mesh, e1);
//     let e2 = mesh.connect_vertex(e1, c);
//     let e3 = mesh.connect_primal(e2, e1);

//     for e in [e1, e1.sym(), e2, e2.sym(), e3, e3.sym()] {
//         print_edge_info(&mesh, e);
//     }

//     // println!("connect (A-B)-(C-D)");
//     // let mut mesh = TopologicalMesh::new();
//     // let a = mesh.insert_vertex("A");
//     // let b = mesh.insert_vertex("B");
//     // let c = mesh.insert_vertex("C");
//     // let d = mesh.insert_vertex("D");
//     // let dummy = mesh.insert_face("(face)");

//     // let e1 = mesh.make_edge(a, b, dummy, dummy);
//     // let e2 = mesh.make_edge(c, d, dummy, dummy);
//     // let e3 = mesh.connect_primal(e1, e2);
//     // for e in [e1, e1.sym(), e2, e2.sym(), e3, e3.sym()] {
//     // }
// }
