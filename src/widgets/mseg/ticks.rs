use std::ops::RangeInclusive;
use vizia::*;

pub(crate) struct MsegTicks<R>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    range: R,
    /// The max length of this MSEG, usually representing `f32` seconds.
    max: f32,
}
impl<R> MsegTicks<R>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub(crate) fn new(cx: &mut Context, range: R, max: f32) -> Handle<Self> {
        Self { range, max }
            .build(cx)
            .width(Stretch(1f32))
            .height(Stretch(1f32))
    }
}

impl<R> View for MsegTicks<R>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}
    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let rect = cx.cache.get_bounds(cx.current);
        
    }
}
