//! Contains implementations for UIs with help
//! of Dear ImGui.

use std::io::prelude::*;

use glium::glutin::event::Event;
use glium::{texture::RawImage2d, uniforms::MagnifySamplerFilter, BlitTarget, Surface, Texture2d};
use imgui::{
    im_str, ColorEdit, FontConfig, FontId, FontSource, ImString, MenuItem, Slider, SliderFlags, Ui,
    Window,
};

const EMULATOR_VERSION: &str = env!("CARGO_PKG_VERSION");
const FONT_SOURCE: &[u8] = include_bytes!("../assets/FiraMono.ttf");

#[derive(Clone, Copy, PartialEq)]
pub enum EmulatorState {
    Idle,
    Running,
    Paused,
    Quit,
}

/// Stores the UserInterface state.
pub struct State {
    /// Is about window currently open?
    about_window: bool,

    /// Is metrics window currently open?
    metrics_window: bool,

    /// FontId of the larger sized font.
    big_font: FontId,

    /// Is color picker active?
    palette_window: bool,

    /// Is memory view window active.
    debug_memory_view: bool,

    /// Is stack view active.
    debug_stack_view: bool,

    /// Is register view active.
    debug_register_view: bool,

    /// Are debug controls active.
    debug_controls: bool,

    /// ImGui Memory Editor widget.
    memory_edit: imgui_memory_editor::MemoryEditor,

    /// CPU cycles to execute frame.
    pub cycles_per_frame: u16,

    /// Current state of the CPU.
    pub emulator_state: EmulatorState,

    /// Foreground color.
    fg_color: [f32; 3],

    /// Background color.
    bg_color: [f32; 3],

    /// Height of the main menu bar.
    menu_height: Option<u32>,

    /// Is a ROM currently loaded?
    rom_loaded: bool,
}

/// Implementation of the UI with Dear ImGui.
pub struct UserInterface {
    /// Dear ImGui context.
    imgui: imgui::Context,

    /// Dear ImGui glium backed renderer.
    renderer: imgui_glium_renderer::Renderer,

    /// Dear ImGui winit backed platform implementation.
    platform: imgui_winit_support::WinitPlatform,

    /// RGB framebuffer.
    framebuffer: Box<[u8; 128 * 64 * 3]>,

    /// Ui State
    pub state: State,
}

impl UserInterface {
    /// Create a new `UserInterface` instance.
    pub fn new(display: &glium::Display) -> Self {
        // Create Dear ImGui context, and disable log and ini saving.
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            display.gl_window().window(),
            imgui_winit_support::HiDpiMode::Default,
        );

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (15.0 * hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::TtfData {
            data: FONT_SOURCE,
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                ..FontConfig::default()
            }),
        }]);

        let big_font = imgui.fonts().add_font(&[FontSource::TtfData {
            data: FONT_SOURCE,
            size_pixels: font_size * 1.75,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                ..FontConfig::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, display)
            .expect("Failed to initialize Dear ImGui glium renderer.");

        Self {
            imgui,
            renderer,
            platform,
            framebuffer: Box::new([0; 128 * 64 * 3]),
            state: State {
                menu_height: None,
                about_window: false,
                metrics_window: false,
                cycles_per_frame: 10,
                emulator_state: EmulatorState::Idle,
                big_font,
                fg_color: [1.0; 3],
                bg_color: [0.0; 3],
                rom_loaded: false,
                palette_window: false,
                debug_memory_view: false,
                memory_edit: imgui_memory_editor::MemoryEditor::default(),
                debug_stack_view: false,
                debug_register_view: false,
                debug_controls: false,
            },
        }
    }

    /// Update the framebuffer, with new data.
    pub fn update_framebuffer(&mut self, cpu: &ferrous::Ferrous) {
        let data = cpu.get_video_buffer();

        let fg = self
            .state
            .fg_color
            .iter()
            .map(|x| ((*x) * 255.0).round() as u8)
            .collect::<Vec<u8>>();

        let bg = self
            .state
            .bg_color
            .iter()
            .map(|x| ((*x) * 255.0).round() as u8)
            .collect::<Vec<u8>>();

        self.framebuffer
            .chunks_exact_mut(3)
            .enumerate()
            .for_each(|(i, rgb)| {
                if data[i] == 0 {
                    rgb.copy_from_slice(&bg);
                } else {
                    rgb.copy_from_slice(&fg);
                }
            });
    }

    /// Let Dear ImGui platform handle window events.
    pub fn handle_event(&mut self, display: &glium::Display, event: &Event<()>) {
        let gl_window = display.gl_window();

        self.platform
            .handle_event(self.imgui.io_mut(), gl_window.window(), event);
    }

    pub fn update_delta(&mut self, delta: std::time::Duration) {
        self.imgui.io_mut().update_delta_time(delta);
    }

    pub fn prepare_frame(&mut self, display: &glium::Display) {
        let gl_window = display.gl_window();

        self.platform
            .prepare_frame(self.imgui.io_mut(), gl_window.window())
            .expect("Failed to prepare Dear ImGui frame.");
        gl_window.window().request_redraw();
    }

    pub fn render_ui(&mut self, display: &glium::Display, cpu: &mut ferrous::Ferrous) {
        let mut ui = self.imgui.frame();
        let gl_window = display.gl_window();

        render_menu(&mut self.state, &mut ui, cpu);
        render_windows(&mut self.state, &mut ui, cpu);

        self.platform.prepare_render(&ui, gl_window.window());

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        // Create texture.
        let (height, width) = cpu.get_height_width();
        let buffer_length = height * width * 3;

        let image = RawImage2d::from_raw_rgb_reversed(
            &self.framebuffer[..buffer_length],
            (width as u32, height as u32),
        );

        let texture = Texture2d::new(display, image).unwrap();
        let window_size = gl_window.window().inner_size();

        texture.as_surface().blit_whole_color_to(
            &target,
            &BlitTarget {
                left: 0,
                bottom: 0,
                width: window_size.width as i32,
                height: (window_size
                    .height
                    .saturating_sub(self.state.menu_height.unwrap_or(0)))
                    as i32,
            },
            MagnifySamplerFilter::Nearest,
        );

        let draw_data = ui.render();
        self.renderer
            .render(&mut target, draw_data)
            .expect("Failed to render Dear ImGui based Ui.");

        target.finish().expect("Failed to swap buffers.");
    }
}

/// Construct a cell for register values.
fn register_cell(ui: &Ui, name: String, value: String) {
    ui.align_text_to_frame_padding();
    ui.text_colored([0.0, 1.0, 0.0, 1.0], &name);
    ui.same_line(0.0);

    let mut input = ImString::new(value);
    let width = ui.push_item_width(56.0);

    ui.input_text(&ImString::new(format!("##{}", name)), &mut input)
        .read_only(true)
        .build();

    width.pop(ui);
}

/// Render main menu bar of the emulator.
fn render_menu(state: &mut State, ui: &mut Ui, cpu: &mut ferrous::Ferrous) {
    if let Some(main_menu_bar) = ui.begin_main_menu_bar() {
        if let Some(file_menu) = ui.begin_menu(im_str!("File"), true) {
            // I know it's ugly. It really is.
            if MenuItem::new(im_str!("Open")).build(ui) {
                if let Ok(nfd2::Response::Okay(path)) =
                    nfd2::open_file_dialog(Some("ch8,c8,fc8"), None)
                {
                    state.emulator_state = EmulatorState::Idle;

                    let is_correct_extension =
                        path.extension() == Some(&std::ffi::OsStr::new("fc8"));
                    let data = std::fs::read(path).expect("Failed to read ROM file.");

                    if is_correct_extension {
                        let sav: ferrous::Ferrous = serde_json::from_slice(&data)
                            .expect("Could not deserialize JSON input.");

                        let _ = std::mem::replace(cpu, sav);
                    } else {
                        cpu.reset();
                        cpu.load_rom(&data)
                            .expect("Failed to load ROM in interpreter memory.");
                    }

                    state.rom_loaded = true;
                }
            }

            if MenuItem::new(im_str!("Save State")).build(ui) {
                if let Ok(nfd2::Response::Okay(path)) = nfd2::open_save_dialog(Some("fc8"), None) {
                    let mut file =
                        std::fs::File::create(path).expect("Failed to create save file.");
                    let serialized = serde_json::to_vec(cpu).expect("Failed to serialize CPU.");

                    file.write_all(&serialized)
                        .expect("Failed to write save file.");
                }
            }

            if MenuItem::new(im_str!("Exit")).build(ui) {
                state.emulator_state = EmulatorState::Quit;
            }

            file_menu.end(ui);
        }

        if let Some(emulation_menu) = ui.begin_menu(im_str!("Emulation"), true) {
            if MenuItem::new(im_str!("Start"))
                .enabled(state.emulator_state != EmulatorState::Running && state.rom_loaded)
                .build(ui)
            {
                state.emulator_state = EmulatorState::Running;
            }

            if MenuItem::new(im_str!("Pause"))
                .enabled(state.emulator_state == EmulatorState::Running)
                .build(ui)
            {
                state.emulator_state = EmulatorState::Paused;
            }

            if MenuItem::new(im_str!("Reset"))
                .enabled(state.emulator_state != EmulatorState::Idle)
                .build(ui)
            {
                cpu.reset();

                state.rom_loaded = false;
                state.emulator_state = EmulatorState::Idle;
            }

            MenuItem::new(im_str!("Palette")).build_with_ref(ui, &mut state.palette_window);

            if let Some(cycles_menu) = ui.begin_menu(im_str!("Cycles per Frame"), true) {
                Slider::<u16>::new(im_str!("cycles"))
                    .range(1..=2000)
                    .flags(SliderFlags::ALWAYS_CLAMP)
                    .build(&ui, &mut state.cycles_per_frame);

                cycles_menu.end(&ui);
            }

            if let Some(quirks_menu) = ui.begin_menu(im_str!("Quirks"), true) {
                MenuItem::new(im_str!("Load and Store Quirk"))
                    .build_with_ref(ui, &mut cpu.load_store_quirk);

                MenuItem::new(im_str!("Shift Quirk")).build_with_ref(ui, &mut cpu.shift_quirk);

                MenuItem::new(im_str!("Jump Quirk")).build_with_ref(ui, &mut cpu.jump_quirk);

                quirks_menu.end(ui);
            }

            emulation_menu.end(ui);
        }

        if let Some(debug_menu) = ui.begin_menu(im_str!("Debug"), true) {
            MenuItem::new(im_str!("Debug Controls")).build_with_ref(ui, &mut state.debug_controls);
            MenuItem::new(im_str!("Registers")).build_with_ref(ui, &mut state.debug_register_view);
            MenuItem::new(im_str!("Address Stack")).build_with_ref(ui, &mut state.debug_stack_view);
            MenuItem::new(im_str!("Memory")).build_with_ref(ui, &mut state.debug_memory_view);

            debug_menu.end(ui);
        }

        if let Some(help_menu) = ui.begin_menu(im_str!("Help"), true) {
            MenuItem::new(im_str!("Dear ImGui Metrics"))
                .build_with_ref(ui, &mut state.metrics_window);

            MenuItem::new(im_str!("About")).build_with_ref(ui, &mut state.about_window);

            help_menu.end(ui);
        }

        state.menu_height = Some(ui.window_size()[1] as u32);
        main_menu_bar.end(ui);
    }
}

/// Render additional windows, like about, metrics etc..
fn render_windows(state: &mut State, ui: &mut Ui, cpu: &mut ferrous::Ferrous) {
    if state.about_window {
        let font_id = state.big_font;

        Window::new(im_str!("About"))
            .bg_alpha(1.0)
            .resizable(false)
            .opened(&mut state.about_window)
            .build(ui, || {
                let token = ui.push_font(font_id);
                ui.text_colored([0.7, 0.25, 0.1, 1.0], im_str!("Ferrous"));
                token.pop(&ui);

                ui.text(im_str!("A (super) Chip-8 interpreter written in Rust."));
                ui.separator();
                ui.text(im_str!("v{}", EMULATOR_VERSION));
                ui.text(im_str!(
                    "Licensed under the terms of the Apache-2.0 license."
                ));
            });
    }

    if state.metrics_window {
        ui.show_metrics_window(&mut state.metrics_window);
    }

    if state.palette_window {
        if let Some(window) = Window::new(im_str!("Palette"))
            .always_auto_resize(true)
            .resizable(false)
            .opened(&mut state.palette_window)
            .begin(ui)
        {
            ColorEdit::new(im_str!("Foreground Colour"), &mut state.fg_color)
                .picker(true)
                .format(imgui::ColorFormat::U8)
                .alpha(false)
                .build(&ui);

            ColorEdit::new(im_str!("Background Colour"), &mut state.bg_color)
                .picker(true)
                .format(imgui::ColorFormat::U8)
                .alpha(false)
                .build(&ui);

            window.end(&ui);
        }
    }

    if state.debug_memory_view {
        state
            .memory_edit
            .draw_window(ui, im_str!("Memory"), cpu.ram.as_mut(), None);

        state.debug_memory_view = state.memory_edit.get_open();
    }

    if state.debug_stack_view {
        Window::new(im_str!("Address Stack"))
            .size([240.0, 270.0], imgui::Condition::Always)
            .resizable(false)
            .opened(&mut state.debug_stack_view)
            .build(ui, || {
                ui.columns(2, im_str!("address_stack"), true);

                // Stack Pointer.
                register_cell(ui, "SP  ".to_string(), format!("{:#04X}", cpu.sp));
                ui.separator();

                for (i, v) in cpu.stack.iter().enumerate() {
                    register_cell(ui, format!("{:#04X}", i), format!("{:#06X}", *v));

                    if i == 7 {
                        ui.next_column();
                    }
                }
            });
    }

    if state.debug_register_view {
        Window::new(im_str!("Registers"))
            .size([235.0, 300.0], imgui::Condition::Always)
            .resizable(false)
            .opened(&mut state.debug_register_view)
            .build(ui, || {
                ui.columns(2, im_str!("address_stack"), true);

                // Meta Registers.
                register_cell(ui, "PC  ".to_string(), format!("{:#06X}", cpu.pc));
                register_cell(ui, "DT  ".to_string(), format!("{:#04X}", cpu.dt));
                ui.next_column();
                register_cell(ui, "I   ".to_string(), format!("{:#06X}", cpu.id));
                register_cell(ui, "ST  ".to_string(), format!("{:#04X}", cpu.st));
                ui.next_column();
                ui.separator();

                for (i, v) in cpu.reg.iter().enumerate() {
                    register_cell(ui, format!("{:#04X}", i), format!("{:#04X}", *v));

                    if i == 7 {
                        ui.next_column();
                    }
                }
            });
    }

    if state.debug_controls {
        if let Some(token) = Window::new(im_str!("Debug Controls"))
            .resizable(false)
            .always_auto_resize(true)
            .opened(&mut state.debug_controls)
            .begin(ui)
        {
            if ui.button(im_str!("Pause"), [100.0, 20.0])
                && state.rom_loaded
                && state.emulator_state != EmulatorState::Idle
            {
                state.emulator_state = match state.emulator_state {
                    EmulatorState::Running => EmulatorState::Paused,
                    EmulatorState::Paused => EmulatorState::Running,

                    _ => state.emulator_state,
                }
            }

            ui.same_line(0.0);

            if ui.button(im_str!("Step"), [100.0, 20.0])
                && state.rom_loaded
                && state.emulator_state != EmulatorState::Idle
                && cpu.execute_cycle().is_none()
            {
                eprintln!("[WARN] invalid or unknown opcode encountered.")
            }

            ui.same_line(0.0);

            if ui.button(im_str!("Step Timers"), [100.0, 20.0])
                && state.rom_loaded
                && state.emulator_state != EmulatorState::Idle
            {
                cpu.step_timers();
            }

            ui.separator();

            register_cell(
                ui,
                "Next OpCode".to_string(),
                format!("{:#06X}", cpu.fetch_opcode()),
            );

            token.end(ui);
        }
    }
}
