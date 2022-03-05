use egui::Color32;

#[derive(Copy, Clone)]
pub struct Theme {
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
}

#[derive(Copy, Clone)]
pub struct ThemeColors {
    pub primary: Color32,
    pub background: Color32,
    pub background_light: Color32,
    pub border: Color32,
}

#[derive(Copy, Clone)]
pub struct ThemeSpacing {}

impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: ThemeColors {
                primary: Color32::from_rgb(73, 233, 137),
                background: Color32::from_rgb(27, 25, 32),
                background_light: Color32::from_rgb(46, 45, 91),
                border: Color32::from_rgba_unmultiplied(255, 255, 255, 50),
            },
            spacing: ThemeSpacing {},
        }
    }
}
