use glam::Vec2;
use std::ops::RangeInclusive;
use vizia::*;

/// Convert a screen value to its data position
pub fn ui_to_data_pos(
    cx: &Context,
    ui_point: &Vec2,
    range_data: impl Lens<Target = RangeInclusive<f32>>,
    max_data: f32,
) -> Vec2 {
    _ui_to_data_pos(
        cx.cache.get_bounds(cx.current),
        *ui_point,
        range_data.get(cx).clone(),
        max_data,
    )
}
pub fn data_to_ui_pos(
    cx: &Context,
    point: Vec2,
    range_data: impl Lens<Target = RangeInclusive<f32>>,
    max: f32,
) -> Vec2 {
    _data_to_ui_pos(
        cx.cache.get_bounds(cx.current),
        point,
        range_data.get(cx).clone(),
        max,
    )
}

fn _ui_to_data_pos(
    bounds: BoundingBox,
    ui_point: Vec2,
    range: RangeInclusive<f32>,
    max: f32,
) -> Vec2 {
    let (width, height) = (bounds.w, bounds.h);
    // Assume `ui_point` is an absolute coordinate. We must convert it to relative coordinates
    let mut ui_point = ui_point;
    let offset = { Vec2::new(bounds.x, bounds.y) };
    // Convert to relative point
    ui_point -= offset;
    // Scale points to fit within `(x,y) = ([0..=max], [0..=1])`
    let y = (height - ui_point.y) / height;
    let offset_data = range.start() * max;
    let scale = (range.end() - range.start()) * max;
    let x = ((ui_point.x / width) * scale) + offset_data;
    Vec2::new(x, y)
}

fn _data_to_ui_pos(bounds: BoundingBox, point: Vec2, range: RangeInclusive<f32>, max: f32) -> Vec2 {
    let (width, height) = (bounds.w, bounds.h);
    // y value is a simple scale
    let y = height - (point.y * height);
    // x value requires us to calculate our zoomed position TODO: Zoom too

    // Calculate the x-offset determined by the current view zoom window
    // This value shifts points to the left and right We calculate the
    // offset by getting the view's starting x value, which is normalized.
    // We then see how much time that offsets by multiplying that normalized
    // value times the maximum X of our MSEG.
    let offset = range.start() * max;
    // Calculate the x-zoom scale to apply to points
    let scale = 1f32 / ((range.end() - range.start()) * max);
    let x = ((point.x - offset) * scale) * width;
    let relative = Vec2::new(x, y);
    // adjust to be absolute by adding the container coords
    let offset = { Vec2::new(bounds.x, bounds.y) };
    relative + offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    fn rect() -> BoundingBox {
        BoundingBox {
            x: 10f32,
            y: 20f32,
            w: 200f32,
            h: 80f32,
        }
    }

    #[test]
    fn gets_ui_point_from_data() {
        let rect = rect();
        let ui_point = _data_to_ui_pos(rect, Vec2::new(0.6, 0.5), 0.2..=0.4, 2f32);
        assert_eq!(ui_point.x.round(), 110f32);
        assert_eq!(ui_point.y.round(), 60f32);
    }

    #[test]
    fn gets_data_point_from_ui() {
        let rect = rect();
        let data_point = _ui_to_data_pos(rect, Vec2::new(110f32, 60f32), 0.2..=0.4, 2f32);
        assert_approx_eq!(data_point.x, 0.6);
        assert_approx_eq!(data_point.y, 0.5);
    }
}
