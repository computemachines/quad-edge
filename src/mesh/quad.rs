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
