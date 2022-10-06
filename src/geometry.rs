// use bevy::utils::HashMap;
use cgmath::{Matrix4, Matrix3, Point2, SquareMatrix};

// // Quad here indicates /Quad-Tree/ hierachical data structure not /Quad-Edge/.
// struct Rect<T> {
//     bottom_left: Point2<T>,
//     upper_right: Point2<T>,
// }
// 
// struct QEntity(usize);
// 
// struct QuadTree {
//     root: QEntity,
//     // world: HashMap<QEntity, QuadNode>,
// }
// 
// enum QuadNode {
//     Branch {
//         aabb: Rect<f32>,
//         // children: []
//     },
//     Leaf {
//         aabb: Rect<f32>,
//     },
// }

pub fn ccw(a: Point2<f32>, b: Point2<f32>, c: Point2<f32>) -> bool {
    let test = Matrix3::new(
        a.x,
        b.x,
        c.x,
        a.y,
        b.y,
        c.y,
        1.0,
        1.0,
        1.0,
    ).determinant();

    test > 0.0
}
pub fn ccw_or_linear(a: Point2<f32>, b: Point2<f32>, c: Point2<f32>) -> bool {
    let test = Matrix3::new(
        a.x,
        b.x,
        c.x,
        a.y,
        b.y,
        c.y,
        1.0,
        1.0,
        1.0,
    ).determinant();

    test >= 0.0
}


pub fn in_circle(a: Point2<f32>, b: Point2<f32>, c: Point2<f32>, d: Point2<f32>) -> bool {
    let test = Matrix4::new(
        a.x,
        b.x,
        c.x,
        d.x,
        a.y,
        b.y,
        c.y,
        d.y,
        a.x * a.x + a.y * a.y,
        b.x * b.x + b.y * b.y,
        c.x * c.x + c.y * c.y,
        d.x * d.x + d.y * d.y,
        1.0,
        1.0,
        1.0,
        1.0,
    )
    .determinant();

    test > 0.0
}
