use cgmath::Point2;

use crate::{
    geometry::ccw,
    geometry::{ccw_or_linear, in_circle},
    mesh::{
        quad::{FaceEntity, PrimalDEdgeEntity},
        Mesh,
    },
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
            if x == *e.org().borrow() || x == *e.dest().borrow() {
                break e.id();
            } else if !ccw_or_linear(x, *e.org().borrow(), *e.dest().borrow()) {
                // rightof x, e
                e.sym_mut();
                continue;
            } else if e.left().borrow().is_infinite() {
                break e.id();
            } else if ccw(x, *e.onext().org().borrow(), *e.onext().dest().borrow()) {
                // leftof x, e.onext
                e.onext_mut();
                continue;
            } else if ccw(x, *e.dprev().org().borrow(), *e.dprev().dest().borrow()) {
                // leftof x, e.dprev
                e.dprev_mut();
                continue;
            } else {
                break e.id();
            }
        }
    }
    pub fn insert_delaunay_vertex(&mut self, v: GeometricVertex) {
        let e = self.locate_point(v);
        if self.primal(e).left().borrow().is_infinite() {
            self.insert_delaunay_exterior_vertex(v, e);
        } else {
            self.insert_delaunay_interior_vertex(v, e);
        }
    }
    fn insert_delaunay_exterior_vertex(&mut self, v: GeometricVertex, e: PrimalDEdgeEntity) {
        println!("inserting exterior");
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

        let mut active_edge = self.primal(dangling_edge.sym()).rprev().id();
        let fan_start = dangling_edge.sym();
        // complete fan
        while !ccw(
            v,
            *self.primal(active_edge).org().borrow(),
            *self.primal(active_edge).dest().borrow(),
        ) {
            let e_rnext_id = self.primal(active_edge).rnext().id();
            let e_id = active_edge.clone();
            let e_rprev_id = self.primal(active_edge).rprev().id();
            let new_face = self.insert_face(VoronoiVertex::Finite(0.0, 0.0));
            let new_edge = self.connect_primal(e_rprev_id.sym(), fan_start);
            self.get_dual(e_id.rot()).borrow_mut().org = new_face;
            self.get_dual(e_rnext_id.rot()).borrow_mut().org = new_face;
            self.get_dual(new_edge.rot()).borrow_mut().org = new_face;

            active_edge = e_rprev_id;
        }
        active_edge = fan_start;
        println!("active_edge: {}", active_edge.0);
        dbg!(active_edge);
        // walk down the fan. check for invalid edges
        loop {
            println!("looping");
            let rprev = self.primal(active_edge).rprev().id();
            dbg!(rprev.0);
            if !self.is_delaunay(rprev) {
                self.swap_primal(rprev);
                continue;
            }
            active_edge = self.primal(active_edge).oprev().id();
            if (active_edge.0 == fan_start.0)
                //|| !self.primal(rprev).sym().left().borrow().is_infinite()
            {
                break;
            }
        }
        println!("done");
    }
    fn insert_delaunay_interior_vertex(&mut self, v: GeometricVertex, e: PrimalDEdgeEntity) {
        println!("inserting interior");
        // insert dangling
        let new_vertex = self.insert_vertex(v);
        let dangling_edge = self.connect_vertex(e.sym(), new_vertex);

        // fan about edge
        let fan_end = dangling_edge.sym();
        let mut active_id = e;
        let mut face: FaceEntity = FaceEntity::default();
        while self.primal(active_id).lnext().id().0 != fan_end.sym().0 {
            let old_lprev_id = self.primal(active_id).lprev().id(); // last radial out
            let old_lnext_id = self.primal(active_id).lnext().id();
            face = self.get_dual(old_lprev_id.rot_inv()).borrow().org;
            assert!(!self.get_face(face).borrow().is_infinite());

            self.get_dual(active_id.rot_inv()).borrow_mut().org = face;
            let new_edge = self.connect_primal(active_id, old_lprev_id);
            self.get_dual(new_edge.rot_inv()).borrow_mut().org = face;

            let new_face = self.insert_face(VoronoiVertex::Finite(0.0, 0.0));

            self.get_dual(new_edge.rot()).borrow_mut().org = new_face;

            active_id = old_lnext_id;
        }
        self.get_dual(fan_end.rot()).borrow_mut().org = face;

        active_id = fan_end;
        loop {
            let suspect_edge = self.primal(active_id).lnext().id();
            if !self.is_delaunay(suspect_edge) {
                self.swap_primal(suspect_edge);
                continue;
            }
            active_id = self.primal(active_id).onext().id();
            if active_id.0 == fan_end.0 {
                break;
            }
        }
        if !self.is_delaunay(active_id) {
            self.swap_primal(active_id);
        }
    }
}
