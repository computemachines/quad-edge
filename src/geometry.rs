use cgmath::{Matrix4, Point2, SquareMatrix};

pub fn in_circle(a: Point2<f64>, b: Point2<f64>, c: Point2<f64>, d: Point2<f64>) -> bool {
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
    ).determinant();
    
    test > 0.0
}
