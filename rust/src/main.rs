extern crate glfw;
extern crate gl;

use glfw::{Context, fail_on_errors};
use std::time::Instant;

use eui_neo_rust::state::UIState;
use eui_neo_rust::renderer::renderer::Renderer;
use eui_neo_rust::color::Color;
use eui_neo_rust::pages::main_page::MainPage;

/// Synchronize UIState screen/framebuffer/dpi metrics from the GLFW window.
fn sync_metrics(window: &glfw::Window, state: &mut UIState) {
    let (sw, sh) = window.get_size();
    let (fw, fh) = window.get_framebuffer_size();
    state.screen_w = sw as f32;
    state.screen_h = sh as f32;
    state.framebuffer_w = fw as f32;
    state.framebuffer_h = fh as f32;
    if sw > 0 {
        state.dpi_scale_x = fw as f32 / sw as f32;
    }
    if sh > 0 {
        state.dpi_scale_y = fh as f32 / sh as f32;
    }
}

fn main() {
    // ── Check for --dump flag (auto-dump first frame) ─────────────────
    let auto_dump = std::env::args().any(|a| a == "--dump");

    // ── Initialize GLFW ──────────────────────────────────────────────
    let mut glfw = glfw::init(fail_on_errors!()).expect("Failed to initialize GLFW");

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // ── Create window ────────────────────────────────────────────────
    let (mut window, events) = glfw
        .create_window(800, 600, "EUI-NEO Rust", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_all_polling(true);

    // ── Load OpenGL function pointers ────────────────────────────────
    gl::load_with(|symbol| {
        let proc = glfw.get_proc_address_raw(symbol);
        match proc {
            Some(f) => f as *const std::os::raw::c_void,
            None => std::ptr::null(),
        }
    });

    // ── Enable blending ──────────────────────────────────────────────
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // ── Initialize state ─────────────────────────────────────────────
    let mut state = UIState::default();
    sync_metrics(&window, &mut state);

    // ── Initialize renderer ──────────────────────────────────────────
    let mut renderer = Renderer::new();
    renderer.init();
    renderer.set_viewport(state.screen_w, state.screen_h);
    renderer.set_framebuffer_size(state.framebuffer_w, state.framebuffer_h);

    // ── Dump control ────────────────────────────────────────────────
    let mut dump_requested = auto_dump;

    // ── Font loading (matching C++ exactly) ────────────────────────
    const UI_FONT_FILE: &str = "YouSheBiaoTiHei-2.ttf";
    const ICON_FONT_FILE: &str = "Font Awesome 7 Free-Solid-900.otf";
    const UI_SDF_LOAD_SIZE: f32 = 72.0;
    const ICON_SDF_LOAD_SIZE: f32 = 96.0;

    let font_dirs: &[&str] = &["font/", "src/font/", "../font/", "../src/font/"];

    // Helper: try loading from project font dirs
    let try_load_project_font = |atlas: &mut eui_neo_rust::renderer::font::FontAtlas,
                                  file: &str, size: f32, start: u32, end: u32, sdf: bool| -> bool {
        for dir in font_dirs {
            let path = format!("{}{}", dir, file);
            if std::path::Path::new(&path).exists() {
                if atlas.load_font(&path, size, start, end, sdf) {
                    return true;
                }
            }
        }
        false
    };

    // Load main UI font (ASCII range 32..128)
    let mut font_loaded = try_load_project_font(
        &mut renderer.font_atlas, UI_FONT_FILE, UI_SDF_LOAD_SIZE, 32, 128, true,
    );

    // Load icon glyphs individually from Font Awesome (matching C++ loadProjectIcon calls)
    let icon_codepoints: &[u32] = &[
        0xF009, // grid
        0xF013, // gear
        0xF015, // home
        0xF031, // font
        0xF04B, // play
        0xF078, // chevron-down
        0xF106, // chevron-up
        0xF107, // chevron-down
        0xF185, // sun
        0xF186, // moon
    ];
    for &cp in icon_codepoints {
        try_load_project_font(
            &mut renderer.font_atlas, ICON_FONT_FILE, ICON_SDF_LOAD_SIZE, cp, cp + 1, false,
        );
    }

    // Fallback: system fonts at same size as C++
    if !font_loaded {
        let fallback_paths = [
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/arial.ttf",
        ];
        for path in &fallback_paths {
            if std::path::Path::new(path).exists() {
                if renderer.font_atlas.load_font(path, UI_SDF_LOAD_SIZE, 32, 128, true) {
                    font_loaded = true;
                    eprintln!("[main] Fallback font loaded: {}", path);
                    break;
                }
            }
        }
    }
    if !font_loaded {
        eprintln!("[main] Warning: No font loaded! Text will not render.");
    }

    // ── Create MainPage ──────────────────────────────────────────────
    let mut main_page = MainPage::new();

    // ── Timing ───────────────────────────────────────────────────────
    let mut last_time = Instant::now();

    // ── Main loop ────────────────────────────────────────────────────
    while !window.should_close() {
        // -- Calculate delta time (capped at 0.05s = 20fps minimum) --
        let now = Instant::now();
        let mut dt = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        if dt > 0.05 {
            dt = 0.05;
        }
        state.delta_time = dt;

        // -- Process events ───────────────────────────────────────────
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                    sync_metrics(&window, &mut state);
                    renderer.set_viewport(state.screen_w, state.screen_h);
                    renderer.set_framebuffer_size(state.framebuffer_w, state.framebuffer_h);
                    state.needs_repaint = true;
                }

                glfw::WindowEvent::Size(_width, _height) => {
                    sync_metrics(&window, &mut state);
                    state.needs_repaint = true;
                }

                glfw::WindowEvent::CursorPos(x, y) => {
                    state.mouse_x = x as f32;
                    state.mouse_y = y as f32;
                    state.pointer_moved = true;
                }

                glfw::WindowEvent::MouseButton(button, action, _modifiers) => {
                    match button {
                        glfw::MouseButtonLeft => {
                            if action == glfw::Action::Press {
                                state.mouse_down = true;
                                state.mouse_clicked = true;
                            } else if action == glfw::Action::Release {
                                state.mouse_down = false;
                            }
                        }
                        glfw::MouseButtonRight => {
                            if action == glfw::Action::Press {
                                state.mouse_right_down = true;
                                state.mouse_right_clicked = true;
                            } else if action == glfw::Action::Release {
                                state.mouse_right_down = false;
                            }
                        }
                        _ => {}
                    }
                    state.needs_repaint = true;
                }

                glfw::WindowEvent::Char(codepoint) => {
                    let mut buf = [0u8; 4];
                    let encoded = codepoint.encode_utf8(&mut buf);
                    state.text_input.push_str(encoded);
                    state.needs_repaint = true;
                }

                glfw::WindowEvent::Scroll(xoffset, yoffset) => {
                    state.scroll_delta_x += xoffset as f32;
                    state.scroll_delta_y += yoffset as f32;
                    state.needs_repaint = true;
                }

                glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => {
                    let key_index = key as i32;
                    if key_index >= 0 && (key_index as usize) < state.keys.len() {
                        let idx = key_index as usize;
                        if action == glfw::Action::Press {
                            state.keys[idx] = true;
                            state.keys_pressed[idx] = true;
                        } else if action == glfw::Action::Release {
                            state.keys[idx] = false;
                        } else if action == glfw::Action::Repeat {
                            state.keys_pressed[idx] = true;
                        }
                    }

                    // Escape to close window
                    if key == glfw::Key::Escape && action == glfw::Action::Press {
                        window.set_should_close(true);
                    }

                    // F12 triggers dump
                    if key == glfw::Key::F12 && action == glfw::Action::Press {
                        dump_requested = true;
                        eprintln!("[EUI-NEO Rust] Dump requested (F12)");
                    }

                    state.needs_repaint = true;
                }

                _ => {}
            }
        }

        // -- Determine if we need to repaint ──────────────────────────
        let needs_repaint = state.needs_repaint
            || state.animation_time_left > 0.0
            || state.pointer_moved;

        if needs_repaint {
            // -- Update phase ─────────────────────────────────────────
            main_page.update(&state);

            // -- Tick renderer animation timer ────────────────────────
            renderer.tick(dt);

            // -- Draw phase ───────────────────────────────────────────
            let should_draw = renderer.should_repaint() || needs_repaint;
            if should_draw {
                // Begin dump recording if requested
                if dump_requested {
                    renderer.dump.begin_recording();
                }

                renderer.begin_frame();

                // ── Draw MainPage (direct rendering for dump comparison) ──
                main_page.draw_direct(&mut renderer, &state);

                // End dump recording and write to file
                if dump_requested {
                    renderer.dump.end_recording_and_write("eui_dump.json");
                    dump_requested = false;
                    if auto_dump {
                        eprintln!("[EUI-NEO Rust] Auto-dump complete, exiting.");
                        break;
                    }
                }

                // Swap buffers
                window.swap_buffers();
            }
        }

        // -- Clear per-frame state ────────────────────────────────────
        state.text_input.clear();
        state.scroll_delta_x = 0.0;
        state.scroll_delta_y = 0.0;
        state.scroll_consumed = false;
        state.mouse_clicked = false;
        state.mouse_right_clicked = false;
        state.pointer_moved = false;
        state.needs_repaint = false;
        state.frame_count += 1;

        // Reset keys_pressed array each frame
        for pressed in state.keys_pressed.iter_mut() {
            *pressed = false;
        }
    }

    // ── Shutdown ─────────────────────────────────────────────────────
    renderer.shutdown();
    eprintln!("[EUI-NEO Rust] Shutdown complete.");
}
