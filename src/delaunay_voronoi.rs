use cgmath::Point2;

use crate::{mesh::{Mesh, quad::PrimalDEdgeEntity}, geometry::in_circle};

pub type DelaunayVertex = Point2<f64>;

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

pub type DelaunayMesh = Mesh<DelaunayVertex, VoronoiVertex>;

impl DelaunayMesh {
    pub fn is_delaunay(&self, xy: PrimalDEdgeEntity) -> bool {
        let xy = self.primal(xy);
        let a = xy.onext().dest().borrow().clone();
        let x = xy.org().borrow().clone();
        let y = xy.dest().borrow().clone();
        let b = xy.oprev().dest().borrow().clone();
        
        in_circle(a, x, y, b)
    }
}

#[cfg(tests)]
mod tests {
    
}