use std::cell::RefCell;

use super::{Mesh, quad::{PrimalDEdgeEntity, PrimalDirectedEdge, DualDEdgeEntity, DualDirectedEdge}, PrimalMeshCursor};


pub struct DualMeshCursor<'a, V, F, Cache> {
    mesh: &'a Mesh<V, F, Cache>,
    entity: DualDEdgeEntity,
}

impl<'a, V, F, Cache> DualMeshCursor<'a, V, F, Cache> {
    /// Drop reference to mesh.
    pub fn id(self) -> DualDEdgeEntity {
        self.entity
    }
}

impl<'a, V, F, Cache> DualMeshCursor<'a, V, F, Cache> {
    pub fn new(mesh: &'a Mesh<V, F, Cache>, entity: DualDEdgeEntity) -> DualMeshCursor<'a, V, F, Cache> {
        DualMeshCursor { mesh, entity }
    }
    pub fn org(&self) -> &RefCell<F> {
        self.mesh.get_face(self.mesh.get_dual(self.entity).borrow().org)
    }
    pub fn dest(&self) -> &RefCell<F> {
        self.mesh.get_face(self.mesh.get_dual(self.entity.sym()).borrow().org)
    }
    pub fn get(&self) -> &RefCell<DualDirectedEdge>{
        self.mesh.get_dual(self.entity)
    }

    fn extend(&self, entity: DualDEdgeEntity) -> DualMeshCursor<'a, V, F, Cache> {
        DualMeshCursor {
            mesh: self.mesh,
            entity,
        }
    }
    fn extend_other(&self, entity: PrimalDEdgeEntity) -> PrimalMeshCursor<'a, V, F, Cache> {
        PrimalMeshCursor::new(self.mesh, entity)
    }

    pub fn onext(&self) -> DualMeshCursor<'a, V, F, Cache> {
        self.extend(self.mesh.get_dual(self.entity).borrow().onext)
    }
    pub fn oprev(&self) -> DualMeshCursor<'a, V, F, Cache> {
        self.extend(self.mesh.get_primal(self.entity.rot()).borrow().onext.rot())
    }
    pub fn lnext(&self) -> DualMeshCursor<'a, V, F, Cache> {
        self.extend(self.mesh.get_primal(self.entity.rot_inv()).borrow().onext.rot())
    }
    pub fn sym(&self) -> DualMeshCursor<V, F, Cache> {
        self.extend(self.entity.sym())
    }
}


