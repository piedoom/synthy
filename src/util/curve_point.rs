use std::ops::{Deref, DerefMut};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CurvePoints(Vec<CurvePoint>);

impl CurvePoints {
    pub fn new(points: Vec<CurvePoint>) -> Self {
        Self(points)
    }
}

impl Deref for CurvePoints {
    type Target = Vec<CurvePoint>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurvePoints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A point with an adjustable single-control exponential curve
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
    /// Defines the exponential curve between the current and last point
    pub curve: f32,
}

impl From<(f32, f32)> for CurvePoint {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y, curve: 0f32 }
    }
}

impl From<(f32, f32, f32)> for CurvePoint {
    fn from((x, y, curve): (f32, f32, f32)) -> Self {
        Self { x, y, curve }
    }
}
