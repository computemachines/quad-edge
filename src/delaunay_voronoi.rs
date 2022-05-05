use cgmath::Point2;

use bevy::prelude::*;

use crate::{
    geometry::in_circle, geometry::ccw,
    mesh::{quad::PrimalDEdgeEntity, Mesh},
};

pub type GeometricVertex = Point2<f32>;

#[derive(Debug, PartialEq)]
pub enum VoronoiVertex {
    Infinite,
    Finite(f32, f32),
}

impl Default for VoronoiVertex {
    fn default() -> Self {
        VoronoiVertex::Infinite
    }
}

impl VoronoiVertex {
    pub fn is_infinite(&self) -> bool {
        VoronoiVertex::Infinite == *self
    }
}


#[derive(Default)]
pub struct LocatePointCache {
    last_found_point: Option<PrimalDEdgeEntity>,
}

pub type DelaunayMesh = Mesh<GeometricVertex, VoronoiVertex, LocatePointCache>;

impl DelaunayMesh {
    pub fn is_delaunay(&self, xy: PrimalDEdgeEntity) -> bool {
        let xy = self.primal(xy);
        let a = xy.onext().dest().borrow().clone();
        let x = xy.org().borrow().clone();
        let y = xy.dest().borrow().clone();
        let b = xy.oprev().dest().borrow().clone();

        in_circle(a, x, y, b)
    }
    /// Finds a dedge `e` such that given point `x` either lies on `e` or is strictly inside the left face of `e`.
    pub fn locate_point(&mut self, x: GeometricVertex) -> PrimalDEdgeEntity {
        if self.cache.last_found_point.is_none() {
            self.cache.last_found_point = self
                .primal_dedges
                .iter()
                .enumerate()
                .find(|(i, e)| e.is_some())
                .map(|(i, _)| PrimalDEdgeEntity(i));
        }
        let mut e = self.primal(self.cache.last_found_point.unwrap());
        loop {
            info!("e = {:?}", e.id());
            if x == *e.org().borrow() || x == *e.dest().borrow() {
                info!("x lies on {:?} endpoints", e.id());
                break e.id();
            } else if !ccw(x, *e.org().borrow(), *e.dest().borrow()) {
                // rightof x, e
                info!("x is right of {:?}", e.id());
                e.sym_mut();
                continue;
            } else if ccw(x, *e.onext().org().borrow(), *e.onext().dest().borrow()) {
                info!("x is left of {:?}", e.onext().id());
                // leftof x, e.onext
                e.onext_mut();
                continue;
            } else if ccw(x, *e.dprev().org().borrow(), *e.dprev().dest().borrow()) {
                info!("x is left of {:?}", e.dprev().id());
                // leftof x, e.dprev
                e.dprev_mut();
                continue;
            } else {
                info!("fallthrough");
                break e.id();
            }
        }
    }
    pub fn insert_delaunay_vertex(&mut self, v: GeometricVertex) {}
}

#[cfg(tests)]
mod tests {}
