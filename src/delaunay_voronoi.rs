use crate::{quad::{PrimalDEdgeEntity, DualDEdgeEntity, PrimalDirectedEdge, DualDirectedEdge}, mesh::Mesh};



#[derive(Default, Debug, PartialEq)]
pub struct DelaunayDEdge {
    vertex: (f64, f64),
    onext: PrimalDEdgeEntity,
}

impl Into<DelaunayDEdge> for (f64, f64) {
    fn into(self) -> DelaunayDEdge {
        DelaunayDEdge {
            vertex: self,
            onext: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum VoronoiVertex {
    Infinite,
    Finite(f64, f64),
}

impl Default for VoronoiVertex {
    fn default() -> Self {
        VoronoiVertex::Infinite
    }
}

#[derive(Default, Debug)]
pub struct VoronoiDEdge { 
    vertex: VoronoiVertex,
    onext: DualDEdgeEntity,
}

impl PrimalDirectedEdge for DelaunayDEdge {
    type Vertex = (f64, f64);

    fn get_org(&self) -> &Self::Vertex {
        &self.vertex
    }

    fn get_mut_org(&mut self) -> &mut Self::Vertex {
        &mut self.vertex
    }

    fn onext(&self) -> PrimalDEdgeEntity {
        self.onext
    }

    fn set_onext(&mut self, onext: PrimalDEdgeEntity) {
        self.onext = onext;
    }
}

impl DualDirectedEdge for VoronoiDEdge {
    type Face = VoronoiVertex;

    fn get_org(&self) -> &Self::Face {
        &self.vertex
    }

    fn get_mut_org(&mut self) -> &mut Self::Face {
        &mut self.vertex
    }

    fn onext(&self) -> DualDEdgeEntity {
        self.onext
    }
    fn set_onext(&mut self, onext: DualDEdgeEntity) {
        self.onext = onext;
    }
}

pub type DelaunayMesh = Mesh<DelaunayDEdge, VoronoiDEdge>;
