use std::cell::{Cell, Ref, RefCell};
use std::rc::{Rc, Weak};

type Mesh = Vec<QEdge>;
struct QEdge {
    dedges: [DEdge; 4],
}

type Ptr<A> = Option<Rc<RefCell<A>>>;
type WeakPtr<A> = Option<Weak<RefCell<A>>>;

#[derive(Debug)]
struct DEdge {
    origin: Rc<Node>,
    onext: WeakPtr<DEdge>,
    rot: WeakPtr<DEdge>,
    flip: WeakPtr<DEdge>,
}
impl DEdge {
    fn new(origin: Rc<Node>) -> DEdge {
        DEdge {
            origin: origin,
            onext: None,
            rot: None,
            flip: None,
        }
    }
    fn onext(&self) -> &Ptr<DEdge> {
        &self.onext.and_then(|o| o.upgrade())
    }
    fn flip(&self) -> &Ptr<DEdge> {
        &self.flip.and_then(|f| f.upgrade())
    }
}
impl PartialEq for DEdge {
    fn eq(&self, other: &DEdge) -> bool {
        *self.origin == *other.origin
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Node {
    Data { label: String },
    Inf,
}

impl Node {
    fn from(label: &str) -> Node {
        Node::Data { label: String::from(label) }
    }
}

impl QEdge {
    fn new(from: Rc<Node>, to: Rc<Node>) -> QEdge {
        let dedges = [
            DEdge::new(from),
            DEdge::new(Rc::new(Node::Inf)),
            DEdge::new(to),
            DEdge::new(Rc::new(Node::Inf)),
        ];
        QEdge { dedges: dedges }
    }
}

#[cfg(test)]
mod tests {
    // use super::List::*;
    use super::*;

    #[test]
    fn basic_qedge() {
        let A = Rc::new(Node::from("A"));
        let B = Rc::new(Node::from("B"));
        let q = QEdge::new(A, B);
        let d = &q.dedges[0];
        assert_eq!(*d, *d);
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
