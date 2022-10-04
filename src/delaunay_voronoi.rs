use cgmath::Point2;

use log::info;

use crate::{
    geometry::ccw,
    geometry::in_circle,
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

        !in_circle(a, x, y, b)
    }
    /// Finds a dedge `e` such that given point `x` either lies on `e` or is strictly inside the left face of `e`.
    pub fn locate_point(&mut self, x: GeometricVertex) -> PrimalDEdgeEntity {
        if self.cache.last_found_point.is_none() {
            self.cache.last_found_point = self
                .primal_dedges
                .iter()
                .enumerate()
                .find(|(_, e)| e.is_some())
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
            } else if e.left().borrow().is_infinite() {
                info!("reached boundary. x is outside convex hull.");
                break e.id();
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
                info!("found face");
                break e.id();
            }
        }
    }
    pub fn insert_delaunay_vertex(&mut self, v: GeometricVertex) {
        let mut e = self.locate_point(v);
        if self.primal(e).left().borrow().is_infinite() {
            self.insert_delaunay_exterior_vertex(v, e);
        } else {
            // insert interior
        }
    }
    fn insert_delaunay_exterior_vertex(&mut self, v: GeometricVertex, e: PrimalDEdgeEntity) {
        let mut boundary_edge = self.primal(e);
        // find fan start
        while ccw(
            v,
            *boundary_edge.org().borrow(),
            *boundary_edge.dest().borrow(),
        ) {
            boundary_edge.lnext_mut();
        }
        let boundary_edge = boundary_edge.id();
        // insert dangling
        let new_vertex = self.insert_vertex(v);
        let dangling_edge = self.connect_vertex(boundary_edge.sym(), new_vertex);

        let fan_start = self.primal(dangling_edge.sym()).rprev().id();
        let mut active_edge = fan_start.clone();
        // complete fan
        while !ccw(
            v,
            *self.primal(active_edge).org().borrow(),
            *self.primal(active_edge).dest().borrow(),
        ) {
            println!("completing fan");
            let e_rnext_id = self.primal(active_edge).rnext().id();
            let e_id = active_edge.clone();
            let e_rprev_id = self.primal(active_edge).rprev().id();
            let new_face = self.insert_face(VoronoiVertex::Finite(0.0, 0.0));
            let new_edge = self.connect_primal(e_rprev_id, fan_start);
            self.get_dual(e_id.rot()).borrow_mut().org = new_face;
            self.get_dual(e_rnext_id.rot()).borrow_mut().org = new_face;
            self.get_dual(new_edge.rot()).borrow_mut().org = new_face;
            active_edge = e_rprev_id;
        }
        println!("done");
    }
}

#[cfg(tests)]
mod tests {}
