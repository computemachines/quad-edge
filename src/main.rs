use quad_edge::topological::TopologicalMesh;

fn main() {
  let mut mesh = TopologicalMesh::new();
  let a = mesh.insert_vertex("A");
  let b = mesh.insert_vertex("B");
  let c = mesh.insert_vertex("C");
  let d = mesh.insert_vertex("D");
  let dummy = mesh.insert_face("(face)");

  let e1 = mesh.make_edge(a, b, dummy, dummy);
  let e3 = mesh.make_edge(c, d, dummy, dummy);

  let e2 = mesh.connect_primal(e1, e3);
  mesh.delete_primal(e2);

  for (i, e) in mesh.primal_dedges.iter().enumerate() {
    print!("PrimalDirectedEdge({}) := ", i);
    if let Some(e) = e {
      let e = e.borrow();
      println!("({:?}, onext: {:?})", e.org, e.onext);
    } else {
      println!("None");
    }

  }
}
