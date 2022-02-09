use std::cell::RefCell;

use crate::quad::{PrimalDirectedEdge, DualDirectedEdge, PrimalDEdgeEntity, DualDEdgeEntity};

/// Tools for constructing, navigating and manipulating meshes.
/// 
#[derive(Debug, Default)]
pub struct Mesh<T: PrimalDirectedEdge, U: DualDirectedEdge> {
    primal_dedges: Vec<Option<RefCell<T>>>,
    dual_dedges: Vec<Option<RefCell<U>>>,
}

impl<'a, T, U, V, F> Mesh<T, U>
where
    T: PrimalDirectedEdge<Vertex = V>,
    U: DualDirectedEdge<Face = F> + Default,
    V: Into<T>,
    F: Into<F>,
{
    pub fn new() -> Self {
        Self {
            primal_dedges: Vec::new(),
            dual_dedges: Vec::new(),
        }
    }

    pub fn get_primal(&self, entity: PrimalDEdgeEntity) -> &RefCell<T> {
        self.primal_dedges.get(entity.0).unwrap().as_ref().unwrap()
    }
    pub fn get_dual(&self, entity: DualDEdgeEntity) -> &RefCell<U> {
        self.dual_dedges.get(entity.0).unwrap().as_ref().unwrap()
    }

    pub fn get_primal_onext_ring(&'a self, entity: PrimalDEdgeEntity) -> PrimalOnextRing<'a, T, U> {
        PrimalOnextRing {
            first: entity,
            current: None,
            mesh: self,
        }
    }

    pub fn make_edge(&mut self, org: V, dest: V) -> PrimalDEdgeEntity {
        let entity = PrimalDEdgeEntity(self.primal_dedges.len());

        let mut e: T = org.into();
        let mut e_rot = U::default();
        let mut e_sym = dest.into();
        let mut e_inv_rot = U::default();

        // set up the topology of disconnected edge
        e.set_onext(entity);
        e_rot.set_onext(entity.rot_inv());
        e_sym.set_onext(entity.sym());
        e_inv_rot.set_onext(entity.rot());

        self.primal_dedges.push(Some(RefCell::new(e)));
        self.primal_dedges.push(Some(RefCell::new(e_sym)));
        self.dual_dedges.push(Some(RefCell::new(e_rot)));
        self.dual_dedges.push(Some(RefCell::new(e_inv_rot)));

        entity
    }

    pub fn splice_primal(&self, a: PrimalDEdgeEntity, b: PrimalDEdgeEntity) {
        let alpha = self.get_primal(a).borrow().onext().rot();
        let beta = self.get_primal(b).borrow().onext().rot();

        // relabel entities a,b,alpha,beta as the actual directed edges
        let mut a = self.get_primal(a).borrow_mut();
        let mut b = self.get_primal(b).borrow_mut();
        let mut alpha = self.get_dual(alpha).borrow_mut();
        let mut beta = self.get_dual(beta).borrow_mut();

        // swap onext values
        let temp = a.onext();
        a.set_onext(b.onext());
        b.set_onext(temp);

        let temp = alpha.onext();
        alpha.set_onext(beta.onext());
        beta.set_onext(temp);
    }

    // fn splice_dual(&self, )
}



pub struct PrimalOnextRing<'a, T: PrimalDirectedEdge, U: DualDirectedEdge + Default> {
  first: PrimalDEdgeEntity,
  current: Option<PrimalDEdgeEntity>,
  mesh: &'a Mesh<T, U>,
}

impl<'a, T, U, V, F> Iterator for PrimalOnextRing<'a, T, U>
where
  T: PrimalDirectedEdge<Vertex = V>,
  U: DualDirectedEdge<Face = F> + Default,
  V: Into<T>,
  F: Into<F>,
{
  type Item = PrimalDEdgeEntity;

  fn next(&mut self) -> Option<Self::Item> {
      match self.current {
          Some(current) => {
              self.current = Some(self.mesh.get_primal(current).borrow().onext());
              if Some(self.first) == self.current {
                  return None
              } else {
                  self.current
              }
          },
          None => {
              self.current = Some(self.mesh.get_primal(self.first).borrow().onext());
              self.current
          },
      }
  }
}

