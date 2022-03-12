/// Interpolate between two values with an adjustable curve
///
/// # Parameters
///
/// * `a` - starting value
/// * `b` - ending value
/// * `t` - value from `0..=1` defining amount
/// * `curve` - value from `-1..=1` defining curve
///
/// # Panics
///
/// If `t` or `curve` are outside their defined ranges
///
/// # Graph
///
/// https://www.desmos.com/calculator/25jtia4sbj
pub fn xlerp(a: f32, b: f32, t: f32, curve: f32) -> f32 {
    assert!((0f32..=1f32).contains(&t));
    assert!((-1f32..=1f32).contains(&curve));

    match curve == -1f32 {
        // We match for a curve of -1 as it would otherwise be undefined
        true => 0f32,
        false => {
            let exp = match curve > 0f32 {
                true => 1f32 - curve,
                false => 1f32 / (1f32 - f32::abs(curve)),
            };
            let adjusted = t.powf(exp);
            fundsp::math::lerp(a, b, adjusted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_xlerp() {
        let v = xlerp(0f32, 1f32, 0.5f32, 0f32);
        assert_eq!(0.5, v);
    }

    #[test]
    fn curved_neg_xlerp() {
        let v = xlerp(0f32, 1f32, 0.6f32, -0.67f32);
        // Obtain the approximate value from Desmos to compare
        assert!(f32::abs(0.2127 - v) < 0.0001);
    }

    #[test]
    fn curved_xlerp() {
        let v = xlerp(0f32, 1f32, 0.3f32, 0.67f32);
        // Obtain the approximate value from Desmos to compare
        assert!(f32::abs(0.6721 - v) < 0.0001);
    }

    #[test]
    fn scaled_curved_xlerp() {
        let v = xlerp(0f32, 2f32, 0.6f32, -0.67f32);
        // Obtain the approximate value from Desmos to compare
        assert!(f32::abs(0.4254 - v) < 0.001);
    }

    #[test]
    fn not_undefined_xlerp() {
        let v = xlerp(0f32, 2f32, 0.6f32, -1.0f32);
        // Obtain the approximate value from Desmos to compare
        assert_eq!(v, 0f32);
    }
}
