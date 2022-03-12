//! Extensions for things not in vizia

use std::ops::{RangeInclusive, Sub};

pub trait RangeExt<T>
where
    T: Sub<Output = T> + Copy,
{
    fn width(&self) -> T;
}

impl<T> RangeExt<T> for RangeInclusive<T>
where
    T: Sub<Output = T> + Copy,
{
    fn width(&self) -> T {
        *self.end() - *self.start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn positive_range_width() {
        assert_approx_eq!((0.2f32..=0.8).width(), 0.6);
    }

    #[test]
    fn negative_range_width() {
        assert_approx_eq!((-0.2f32..=0.2).width(), 0.4);
    }
}
