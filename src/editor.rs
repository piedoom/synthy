use baseview::WindowHandle;
use nih_plug::prelude::*;
use std::sync::Arc;
use vizia::{Application, WindowDescription};

pub use vizia::*;

const SIZE: (u32, u32) = (900, 600);

pub fn create_vizia_editor<U>(update: U) -> Option<Box<dyn Editor>>
where
    U: Fn(&mut Context, Arc<dyn GuiContext>) + 'static + Send + Sync,
{
    Some(Box::new(ViziaEditor {
        update: Arc::new(update),
    }))
}

pub struct ViziaEditor {
    update: Arc<dyn Fn(&mut Context, Arc<dyn GuiContext>) + 'static + Send + Sync>,
}

impl Editor for ViziaEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send + Sync> {
        let update = self.update.clone();

        let window_description = WindowDescription::new().with_inner_size(SIZE.0, SIZE.1);
        let window = Application::new(window_description, move |cx| {
            (update)(cx, context.clone());
        })
        .open_parented(&parent);

        Box::new(ViziaEditorHandle { window })
    }

    fn size(&self) -> (u32, u32) {
        (SIZE.0, SIZE.1)
    }
}

struct ViziaEditorHandle {
    window: WindowHandle,
}

unsafe impl Send for ViziaEditorHandle {}
unsafe impl Sync for ViziaEditorHandle {}

impl Drop for ViziaEditorHandle {
    fn drop(&mut self) {
        self.window.close();
    }
}
