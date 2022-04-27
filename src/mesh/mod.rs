use std::cell::RefCell;

use self::quad::{
    DualDEdgeEntity, DualDirectedEdge, FaceEntity, PrimalDEdgeEntity, PrimalDirectedEdge,
    VertexEntity, MeshCursor,
};

pub mod quad;
/// Tools for constructing, navigating and manipulating meshes.
///
#[derive(Debug)]
pub struct Mesh<V, F, Cache> {
    pub primal_dedges: Vec<Option<RefCell<PrimalDirectedEdge>>>,
    pub dual_dedges: Vec<Option<RefCell<DualDirectedEdge>>>,
    pub vertices: Vec<Option<RefCell<V>>>,
    pub faces: Vec<Option<RefCell<F>>>,
    cache: Option<Cache>,
}

impl<V, F, Cache> Default for Mesh<V, F, Cache> {
    fn default() -> Self {
        Self {
            primal_dedges: Default::default(),
            dual_dedges: Default::default(),
            vertices: Default::default(),
            faces: Default::default(),
            cache: None,
        }
    }
}

impl<'a, V, F, Cache> Mesh<V, F, Cache> {
    pub fn new() -> Self {
        Mesh::default()
    }

    pub fn get_primal(&self, entity: PrimalDEdgeEntity) -> &RefCell<PrimalDirectedEdge> {
        self.primal_dedges.get(entity.0).unwrap().as_ref().unwrap()
    }
    pub fn get_dual(&self, entity: DualDEdgeEntity) -> &RefCell<DualDirectedEdge> {
        self.dual_dedges.get(entity.0).unwrap().as_ref().unwrap()
    }
    pub fn get_vertex(&self, entity: VertexEntity) -> &RefCell<V> {
        self.vertices.get(entity.0).unwrap().as_ref().unwrap()
    }
    pub fn get_face(&self, entity: FaceEntity) -> &RefCell<F> {
        self.faces.get(entity.0).unwrap().as_ref().unwrap()
    }

    pub fn insert_vertex<U: Into<V>>(&mut self, v: U) -> VertexEntity {
        let e = VertexEntity(self.vertices.len());
        self.vertices.push(Some(RefCell::new(v.into())));
        e
    }

    pub fn insert_face(&mut self, f: F) -> FaceEntity {
        let e = FaceEntity(self.faces.len());
        self.faces.push(Some(RefCell::new(f)));
        e
    }

    pub fn get_primal_onext_ring(&'a self, entity: PrimalDEdgeEntity) -> PrimalOnextRing<'a, V, F, Cache> {
        PrimalOnextRing {
            first: entity,
            current: None,
            mesh: self,
        }
    }

    pub fn make_edge(
        &mut self,
        org: VertexEntity,
        dest: VertexEntity,
        left: FaceEntity,
        right: FaceEntity,
    ) -> PrimalDEdgeEntity {
        let entity = PrimalDEdgeEntity(self.primal_dedges.len());

        self.primal_dedges
            .push(Some(RefCell::new(PrimalDirectedEdge {
                org: org,
                onext: entity,
            })));
        self.primal_dedges
            .push(Some(RefCell::new(PrimalDirectedEdge {
                org: dest,
                onext: entity.sym(),
            })));
        self.dual_dedges.push(Some(RefCell::new(DualDirectedEdge {
            org: left,
            onext: entity.rot_inv(),
        })));
        self.dual_dedges.push(Some(RefCell::new(DualDirectedEdge {
            org: right,
            onext: entity.rot(),
        })));

        entity
    }

    pub fn splice_primal(&self, a: PrimalDEdgeEntity, b: PrimalDEdgeEntity) {
        let alpha = self.get_primal(a).borrow().onext.rot();
        let beta = self.get_primal(b).borrow().onext.rot();

        // relabel entities a,b,alpha,beta as the actual directed edges
        let mut a = self.get_primal(a).borrow_mut();
        let mut b = self.get_primal(b).borrow_mut();
        let mut alpha = self.get_dual(alpha).borrow_mut();
        let mut beta = self.get_dual(beta).borrow_mut();

        // swap onext values
        let temp = a.onext;
        a.onext = b.onext;
        b.onext = temp;

        let temp = alpha.onext;
        alpha.onext = beta.onext;
        beta.onext = temp;
    }

    // fn splice_dual(&self, )

    // /// Create new primal edge from the end of `from` to the begining of `to`
    pub fn connect_primal(&mut self, from: PrimalDEdgeEntity, to: PrimalDEdgeEntity) -> PrimalDEdgeEntity {
        let org = self.get_primal(from.sym()).borrow().org;
        let dest = self.get_primal(to).borrow().org;
        let left = self.get_dual(from.rot_inv()).borrow().org;
        let right = self.get_dual(from.rot()).borrow().org;

        let e = self.make_edge(org, dest, left, right);
        let from_lnext = self.get_dual(from.rot_inv()).borrow().onext.rot();

        self.splice_primal(e, from_lnext);
        self.splice_primal(e.sym(), to);

        e
    }

    pub fn delete_primal(&mut self, e: PrimalDEdgeEntity) {
        let e_oprev = self.get_dual(e.rot()).borrow().onext.rot();
        let e_sym_oprev = self.get_dual(e.rot_inv()).borrow().onext.rot();
        self.splice_primal(e, e_oprev);
        self.splice_primal(e.sym(), e_sym_oprev);

        self.primal_dedges.get_mut(e.0).unwrap().take();
        self.primal_dedges.get_mut(e.sym().0).unwrap().take();
        self.dual_dedges.get_mut(e.rot().0).unwrap().take();
        self.dual_dedges.get_mut(e.rot_inv().0).unwrap().take();
    }

    pub fn swap(&self, e: PrimalDEdgeEntity) {
        let a = self.get_dual(e.rot()).borrow().onext.rot();
        let b = self.get_dual(e.rot_inv()).borrow().onext.rot();
        let a_lnext = self.get_dual(a.rot_inv()).borrow().onext.rot();
        let b_lnext = self.get_dual(b.rot_inv()).borrow().onext.rot();

        self.splice_primal(e, a);
        self.splice_primal(e.sym(), b);
        self.splice_primal(e, a_lnext);
        self.splice_primal(e.sym(), b_lnext);

        let org = self.get_primal(a.sym()).borrow().org;
        let dest = self.get_primal(b.sym()).borrow().org;

        self.get_primal(e).borrow_mut().org = org;
        self.get_primal(e.sym()).borrow_mut().org = dest;
    }

    pub fn primal(&'a self, e: PrimalDEdgeEntity) -> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
        MeshCursor::new(self, e)
    }
}

pub struct PrimalOnextRing<'a, V, F, Cache> {
    first: PrimalDEdgeEntity,
    current: Option<PrimalDEdgeEntity>,
    mesh: &'a Mesh<V, F, Cache>,
}

impl<'a, V, F, Cache> Iterator for PrimalOnextRing<'a, V, F, Cache> {
    type Item = PrimalDEdgeEntity;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(current) => {
                self.current = Some(self.mesh.get_primal(current).borrow().onext);
                if Some(self.first) == self.current {
                    return None;
                } else {
                    self.current
                }
            }
            None => {
                self.current = Some(self.mesh.get_primal(self.first).borrow().onext);
                self.current
            }
        }
    }
}
