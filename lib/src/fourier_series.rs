use crate::point::PointF64;

pub struct QuadraticBezierCurve {
    pub p0: PointF64,
    pub p1: PointF64,
    pub p2: PointF64,
}

pub type BezierCurve = QuadraticBezierCurve;

pub fn quadratic_bezier_curve_length(curve: &QuadraticBezierCurve) -> f64 {
    let x0 = curve.p0.x;
    let x1 = curve.p1.x;
    let x2 = curve.p2.x;
    let y0 = curve.p0.y;
    let y1 = curve.p1.y;
    let y2 = curve.p2.y;

    let dx0 = x1 - x0;
    let dx1 = x2 - x1;
    let dy0 = y1 - y0;
    let dy1 = y2 - y1;

    let a = (dx1 - dx0).powf(2.0) + (dy1 - dy0).powf(2.0);
    let b = dx0 * (dx1 - dx0) + dy0 * (dy1 - dy0);
    let c = dx0.powf(2.0) + dy0.powf(2.0);
    let d = a * c - b.powf(2.0);

    let mut t = 1.0;
    let t1 = (t + b / a) * f64::sqrt(a * t.powf(2.0) + 2.0 * b * t + c)
        + ((d) / a.powf(3.0 / 2.0)) * f64::asinh((a * t + b) / f64::sqrt(d));

    t = 0.0;

    let t0 = (t + b / a) * f64::sqrt(a * t.powf(2.0) + 2.0 * b * t + c)
        + ((d) / a.powf(3.0 / 2.0)) * f64::asinh((a * t + b) / f64::sqrt(d));

    return t1 - t0;
}
