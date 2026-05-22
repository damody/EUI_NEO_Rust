/// Global UI state (matches C++ UIState).
#[derive(Debug, Clone)]
pub struct UIState {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_down: bool,
    pub mouse_clicked: bool,
    pub mouse_right_down: bool,
    pub mouse_right_clicked: bool,
    pub delta_time: f32,
    pub screen_w: f32,
    pub screen_h: f32,
    pub framebuffer_w: f32,
    pub framebuffer_h: f32,
    pub dpi_scale_x: f32,
    pub dpi_scale_y: f32,
    pub text_input: String,
    pub keys: [bool; 512],
    pub keys_pressed: [bool; 512],
    pub scroll_delta_x: f32,
    pub scroll_delta_y: f32,
    pub scroll_consumed: bool,
    pub pointer_moved: bool,
    pub needs_repaint: bool,
    pub animation_time_left: f32,
    pub frame_count: i32,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_down: false,
            mouse_clicked: false,
            mouse_right_down: false,
            mouse_right_clicked: false,
            delta_time: 0.0,
            screen_w: 800.0,
            screen_h: 600.0,
            framebuffer_w: 800.0,
            framebuffer_h: 600.0,
            dpi_scale_x: 1.0,
            dpi_scale_y: 1.0,
            text_input: String::new(),
            keys: [false; 512],
            keys_pressed: [false; 512],
            scroll_delta_x: 0.0,
            scroll_delta_y: 0.0,
            scroll_consumed: false,
            pointer_moved: false,
            needs_repaint: true,
            animation_time_left: 0.0,
            frame_count: 0,
        }
    }
}
