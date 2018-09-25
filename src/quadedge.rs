// ids only for now.
// all operations will have to be conjugate to mesh memory arena.

// memory arena
type Mesh = Vec<DEdge>;
type DEdgeID = usize;

// A view into a Quad Edge.
#[derive(PartialEq, Debug, Clone)]
struct DEdge {
    // orientation: Orientation,
    origin: Node,
    onext: DEdgeID,
    rot: DEdgeID,
}

impl DEdge {
    fn org(&self) -> &Node {
        &self.origin
    }
    fn onext(&self, mesh: &Mesh) -> DEdge {
        mesh[self.onext].clone()
    }
    fn rot(&self, mesh: &Mesh) -> DEdge {
        mesh[self.rot].clone()
    }
    fn sym(&self, mesh: &Mesh) -> DEdge {
        self.rot(mesh).rot(mesh)
    }

    fn splice(&mut self, mesh: &Mesh, that: &mut DEdge) {
        let mut alpha = self.onext(mesh).rot(mesh);
        let mut beta = that.onext(mesh).rot(mesh);

        let temp = self.onext;
        self.onext = that.onext;
        that.onext = temp;
        
        let temp = alpha.onext;
        alpha.onext = beta.onext;
        beta.onext = temp;
    }
}

enum Orientation {
    CW,
    CCW,
}

fn makeEdge(mesh: &mut Mesh, a: &Node, b: &Node) -> DEdge {
    let a = a.clone();
    let b = b.clone();
    let base_id = mesh.len();
    let r0 = mesh.push(DEdge {
        origin: a,
        onext: base_id,
        rot: base_id+1,
    });
    let r1 = mesh.push(DEdge {
        origin: Node::Infinite,
        onext: base_id+1,
        rot: base_id+2,
    });
    let r2 = mesh.push(DEdge {
        origin: b,
        onext: base_id+2,
        rot: base_id+3,
    });
    let r3 = mesh.push(DEdge {
        origin: Node::Infinite,
        onext: base_id+3,
        rot: base_id,
    });
    return mesh[base_id].clone()
}


#[derive(PartialEq, Debug, Clone)]
enum Node {
    Finite(String),
    Infinite,
}


#[cfg(test)]
mod tests {
    // use super::List::*;
    use super::*;

    #[test]
    fn test_make_edge_topological_operator() {
        let mut mesh = vec![];
        let e = makeEdge(&mut mesh, &Node::Finite("A".to_string()), &Node::Finite("B".to_string()));
        assert_ne!(e.org(), e.sym(&mut mesh).org());
        assert_eq!(e, e.onext(&mut mesh));
    }

    #[test]
    fn splice_doesnt_crash() {
        let mut mesh = vec![];
        let mut f = makeEdge(&mut mesh, &Node::Finite("A".to_string()), &Node::Finite("B".to_string()));
        let mut g = makeEdge(&mut mesh, &Node::Finite("C".to_string()), &Node::Finite("D".to_string()));
        f.splice(&mut mesh, &mut g);
    }

    // #[test]
    // fn ref_counting_cons_list() {
    //     let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));

    //     let b = Cons(3, a.clone());
    //     let c = Cons(10, a.clone());
    //     assert_eq!(2 + 2, 4);
    // }

    // use std::cell::RefCell;

    // #[test]
    // fn interior_mut() {
    //     let a = vec!["hello"];
    //     let b = RefCell::new(a);
    //     {
    //         let mut c = b.borrow_mut();
    //         c.push("worlds");
    //     }
    //     assert_eq!(*b.borrow(), vec!["hello", "worlds"]);
    // }
}
