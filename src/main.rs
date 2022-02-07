use quad_edge::{delaunay_voronoi::DelaunayMesh};

fn main() {
  let mut mesh = DelaunayMesh::new();
  let e = mesh.make_edge((0.0, 1.0), (5.0, 1.0));
  println!("{:#?}", mesh);
  println!("-----------------");
  println!("");
  // println!("{:#?}", mesh);
  
  let ring = mesh.get_primal_onext_ring(e.sym());
  // println!("{:#?}", mesh);
  for i in ring {
    println!("{:?}, {:#?}", i, mesh.get_primal(i));
  }
  // println!("{:#?}", mesh);
}