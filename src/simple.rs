use crate::quad::{
    DualDEdgeEntity, DualDirectedEdge, PrimalDEdgeEntity, PrimalDirectedEdge,
    UnspecifiedDEdgeEntity,
};

#[derive(Debug, Default)]
pub struct SimpleDEdge<T> {
    onext: UnspecifiedDEdgeEntity,
    node: T,
}

impl<T> From<T> for SimpleDEdge<T> {
    fn from(node: T) -> Self {
        SimpleDEdge {
            onext: Default::default(),
            node,
        }
    }
}

impl<T> PrimalDirectedEdge for SimpleDEdge<T> {
    type Vertex = T;

    fn get_org(&self) -> &Self::Vertex {
        &self.node
    }

    fn get_mut_org(&mut self) -> &mut Self::Vertex {
        &mut self.node
    }

    fn onext(&self) -> PrimalDEdgeEntity {
        self.onext.into()
    }

    fn set_onext(&mut self, onext: PrimalDEdgeEntity) {
        self.onext = onext.into();
    }
}

impl<T> DualDirectedEdge for SimpleDEdge<T> {
    type Face = T;

    fn get_org(&self) -> &Self::Face {
        &self.node
    }

    fn get_mut_org(&mut self) -> &mut Self::Face {
        &mut self.node
    }

    fn onext(&self) -> DualDEdgeEntity {
        self.onext.into()
    }

    fn set_onext(&mut self, onext: DualDEdgeEntity) {
        self.onext = onext.into();
    }
}
