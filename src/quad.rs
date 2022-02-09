
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

pub trait PrimalDirectedEdge {
    type Vertex;

    /// Represents geometry of the mesh
    fn get_org(&self) -> &Self::Vertex;
    fn get_mut_org(&mut self) -> &mut Self::Vertex;

    /// Represents the topology of the mesh
    fn onext(&self) -> PrimalDEdgeEntity;
    fn set_onext(&mut self, onext: PrimalDEdgeEntity);
}

pub trait DualDirectedEdge {
    type Face;

    /// Represents geometry of the dual mesh
    fn get_org(&self) -> &Self::Face;
    fn get_mut_org(&mut self) -> &mut Self::Face;

    /// Represents the topology of the dual mesh
    fn onext(&self) -> DualDEdgeEntity;
    fn set_onext(&mut self, onext: DualDEdgeEntity);
}