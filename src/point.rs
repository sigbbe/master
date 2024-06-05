use nalgebra::Matrix2x1;

pub type Distance = f64;
pub type Point = [Distance; 2];
pub type PointMatrix = Matrix2x1<Distance>;

pub struct PolarCoordinate {
    pub r: f64,
    pub theta: f64,
}

impl From<PointMatrix> for PolarCoordinate {
    fn from(point: PointMatrix) -> Self {
        Self {
            r: (point[0].powi(2) + point[1].powi(2)).sqrt(),
            theta: point[1].atan2(point[0]),
        }
    }
}