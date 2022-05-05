use std::cell::RefCell;

use super::{
    quad::{DualDEdgeEntity, PrimalDEdgeEntity, PrimalDirectedEdge},
    DualMeshCursor, Mesh,
};

/// Convenience type for traversing the mesh.
pub struct PrimalMeshCursor<'a, V, F, Cache> {
    mesh: &'a Mesh<V, F, Cache>,
    entity: PrimalDEdgeEntity,
}

impl<'a, V, F, Cache> PrimalMeshCursor<'a, V, F, Cache> {
    /// Drop reference to mesh.
    pub fn id(self) -> PrimalDEdgeEntity {
        self.entity
    }
}

impl<'a, V, F, Cache> PrimalMeshCursor<'a, V, F, Cache> {
    pub fn new(
        mesh: &'a Mesh<V, F, Cache>,
        entity: PrimalDEdgeEntity,
    ) -> PrimalMeshCursor<'a, V, F, Cache> {
        PrimalMeshCursor { mesh, entity }
    }
    pub fn org(&self) -> &RefCell<V> {
        self.mesh
            .get_vertex(self.mesh.get_primal(self.entity).borrow().org)
    }
    pub fn dest(&self) -> &RefCell<V> {
        self.mesh
            .get_vertex(self.mesh.get_primal(self.entity.sym()).borrow().org)
    }
    pub fn get(&self) -> &RefCell<PrimalDirectedEdge> {
        self.mesh.get_primal(self.entity)
    }

    fn extend(&self, entity: PrimalDEdgeEntity) -> PrimalMeshCursor<'a, V, F, Cache> {
        PrimalMeshCursor {
            mesh: self.mesh,
            entity,
        }
    }
    fn extend_other(&self, entity: DualDEdgeEntity) -> DualMeshCursor<'a, V, F, Cache> {
        DualMeshCursor::new(self.mesh, entity)
    }

    pub fn onext(&self) -> PrimalMeshCursor<'a, V, F, Cache> {
        self.extend(self.mesh.get_primal(self.entity).borrow().onext)
    }
    pub fn oprev(&self) -> PrimalMeshCursor<'a, V, F, Cache> {
        self.extend(self.mesh.get_dual(self.entity.rot()).borrow().onext.rot())
    }
    pub fn lnext(&self) -> PrimalMeshCursor<'a, V, F, Cache> {
        self.extend(
            self.mesh
                .get_dual(self.entity.rot_inv())
                .borrow()
                .onext
                .rot(),
        )
    }
    pub fn sym(&self) -> PrimalMeshCursor<V, F, Cache> {
        self.extend(self.entity.sym())
    }
}
