use std::cell::RefCell;

use super::Mesh;


#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct UnspecifiedDEdgeEntity(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct PrimalDEdgeEntity(pub usize);

impl PrimalDEdgeEntity {
    pub fn rot(self) -> DualDEdgeEntity {
        DualDEdgeEntity(self.0)
    }
    pub fn rot_inv(self) -> DualDEdgeEntity {
        DualDEdgeEntity(self.0 ^ 1)
    }
    pub fn sym(self) -> PrimalDEdgeEntity {
        PrimalDEdgeEntity(self.0 ^ 1)
    }
    pub fn orientation(self) -> bool {
        self.0.is_power_of_two()
    }
}

impl From<UnspecifiedDEdgeEntity> for PrimalDEdgeEntity {
    fn from(e: UnspecifiedDEdgeEntity) -> Self {
        PrimalDEdgeEntity(e.0)
    }
}

impl Into<UnspecifiedDEdgeEntity> for PrimalDEdgeEntity {
    fn into(self) -> UnspecifiedDEdgeEntity {
        UnspecifiedDEdgeEntity(self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct DualDEdgeEntity(pub usize);

impl DualDEdgeEntity {
    pub fn rot(self) -> PrimalDEdgeEntity {
        PrimalDEdgeEntity(self.0 ^ 1)
    }
    pub fn rot_inv(self) -> PrimalDEdgeEntity {
        PrimalDEdgeEntity(self.0)
    }
    pub fn sym(self) -> DualDEdgeEntity {
        DualDEdgeEntity(self.0 ^ 1)
    }
}

impl From<UnspecifiedDEdgeEntity> for DualDEdgeEntity {
    fn from(e: UnspecifiedDEdgeEntity) -> Self {
        DualDEdgeEntity(e.0)
    }
}

impl Into<UnspecifiedDEdgeEntity> for DualDEdgeEntity {
    fn into(self) -> UnspecifiedDEdgeEntity {
        UnspecifiedDEdgeEntity(self.0)
    }
}

/// Convenience type for traversing the mesh.
pub struct MeshCursor<'a, V, F, T, Cache> {
    mesh: &'a Mesh<V, F, Cache>,
    entity: T,
}

impl<'a, V, F, T, Cache> MeshCursor<'a, V, F, T, Cache> {
    /// Drop reference to mesh.
    pub fn id(self) -> T {
        self.entity
    }
}

impl<'a, V, F, Cache> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
    pub fn new(mesh: &'a Mesh<V, F, Cache>, entity: PrimalDEdgeEntity) -> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
        MeshCursor { mesh, entity }
    }
    pub fn org(&self) -> &RefCell<V> {
        self.mesh.get_vertex(self.mesh.get_primal(self.entity).borrow().org)
    }
    pub fn dest(&self) -> &RefCell<V> {
        self.mesh.get_vertex(self.mesh.get_primal(self.entity.sym()).borrow().org)
    }
    pub fn get(&self) -> &RefCell<PrimalDirectedEdge>{
        self.mesh.get_primal(self.entity)
    }

    fn extend(&self, entity: PrimalDEdgeEntity) -> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
        MeshCursor {
            mesh: self.mesh,
            entity,
        }
    }
    fn extend_other(&self, entity: DualDEdgeEntity) -> MeshCursor<'a, V, F, DualDEdgeEntity, Cache> {
        todo!()
    }

    pub fn onext(&self) -> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
        self.extend(self.mesh.get_primal(self.entity).borrow().onext)
    }
    pub fn oprev(&self) -> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
        self.extend(self.mesh.get_dual(self.entity.rot()).borrow().onext.rot())
    }
    pub fn lnext(&self) -> MeshCursor<'a, V, F, PrimalDEdgeEntity, Cache> {
        self.extend(self.mesh.get_dual(self.entity.rot_inv()).borrow().onext.rot())
    }
}



#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VertexEntity(pub usize);
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FaceEntity(pub usize);


#[derive(Debug, Default)]
pub struct PrimalDirectedEdge {
    pub org: VertexEntity,
    pub onext: PrimalDEdgeEntity,
}

#[derive(Debug, Default)]
pub struct DualDirectedEdge {
    pub org: FaceEntity,
    pub onext: DualDEdgeEntity,
}