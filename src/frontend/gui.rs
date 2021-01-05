/*
Copyright 2020 Anish Jewalikar

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::path::PathBuf;

use ferrous_core::CPU;

use glium::{
    glutin::{
        dpi::LogicalSize,
        event_loop::EventLoop,
        window::{Icon, WindowBuilder},
        ContextBuilder,
    },
    texture::{RawImage2d, Texture2d},
    uniforms::MagnifySamplerFilter,
    BlitTarget, Display, Surface,
};

use imgui::{
    im_str, ColorEdit, FontConfig, FontSource, MenuItem, Slider, Window,
};

use super::{audio, file_dialog};

#[derive(PartialEq)]
pub enum State {
    Idle,
    Paused,
    Running,
}

pub struct Application {
    /// Chip8 CPU
    pub cpu: ferrous_core::CPU,

    // File Dialog Handler
    pub file_dialog: file_dialog::FileDialog,

    // Current ROM,
    pub rom: Option<PathBuf>,

    // The interpreters current state.
    pub state: State,

    /// OpenGL backed display.
    pub display: Display,

    /// RGB framebuffer for the display.
    pub framebuffer: Box<[u8; 128 * 64 * 3]>,

    /// Dear ImGui context.
    pub imgui: imgui::Context,

    /// ImGui winit support.
    pub platform: imgui_winit_support::WinitPlatform,

    /// ImGui glium renderer support.
    pub renderer: imgui_glium_renderer::Renderer,

    /// Height taken up by the main menu bar.
    pub menu_height: Option<u32>,

    pub larger_font: imgui::FontId,

    pub audio: audio::Audio,

    // ----- State ----- //
    /// Is the about window currently opened?
    pub about_window: bool,

    /// Is the Dear ImGui Metrics window currently opened?
    pub metrics_window: bool,

    /// Is the color picker active?
    pub pallete_window: bool,

    /// Draw Color
    pub fg_color: [f32; 3],

    /// Background Color
    pub bg_color: [f32; 3],

    /// CPU cycles to execute per frame.
    pub cycles_per_frame: u16,
}

impl Application {
    /// Create a new `Application` instance.
    pub fn new(events_loop: &EventLoop<()>) -> Self {
        let image = image::load_from_memory_with_format(
            include_bytes!("../images/rust-logo-64x64.png"),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8();

        let (w, h) = image.dimensions();
        let actual_icon = Icon::from_rgba(image.into_raw(), w, h).unwrap();

        let cb = ContextBuilder::new();
        let wb = WindowBuilder::new()
            .with_window_icon(Some(actual_icon))
            .with_decorations(true)
            .with_title("Ferrous Chip-8")
            .with_min_inner_size(LogicalSize::new(128, 64))
            .with_inner_size(LogicalSize::new(1152, 576));

        // Create Glium display.
        let display = Display::new(wb, cb, events_loop).unwrap();

        // Clear the screen.
        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame.finish().unwrap();

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(
                imgui.io_mut(),
                window,
                imgui_winit_support::HiDpiMode::Default,
            );
        }

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (7.0 * hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size * 2.0,
                ..FontConfig::default()
            }),
        }]);

        let font_id = imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size * 3.0,
                ..FontConfig::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        let renderer =
            imgui_glium_renderer::Renderer::init(&mut imgui, &display).unwrap();

        Self {
            cpu: CPU::new(),
            file_dialog: file_dialog::FileDialog::new(),
            rom: None,
            state: State::Idle,
            display,
            framebuffer: Box::new([0; 128 * 64 * 3]),
            imgui,
            platform,
            renderer,
            audio: audio::Audio::new(),
            larger_font: font_id,
            about_window: false,
            metrics_window: false,
            pallete_window: false,
            menu_height: None,
            fg_color: [1.0; 3],
            bg_color: [0.0; 3],
            cycles_per_frame: 10,
        }
    }

    /// Render video memory onto the screen.
    pub fn render_frame(&mut self) {
        // Prepare framebuffer for rendering.
        self.prepare_framebuffer();
        let (height, width) = self.cpu.get_height_width();

        // Create texture.
        let buffer_length = height * width * 3;

        let image = RawImage2d::from_raw_rgb_reversed(
            &self.framebuffer[..buffer_length],
            (width as u32, height as u32),
        );

        let texture = Texture2d::new(&self.display, image).unwrap();
        let window_size = self.display.gl_window().window().inner_size();

        // Blit the texture onto the screen.
        let mut frame = self.display.draw();

        texture.as_surface().blit_whole_color_to(
            &frame,
            &BlitTarget {
                left: 0,
                bottom: 0,
                width: window_size.width as i32,
                height: (window_size.height - self.menu_height.unwrap_or(0))
                    as i32,
            },
            MagnifySamplerFilter::Nearest,
        );

        self.render_ui(&mut frame);

        frame.finish().unwrap();
    }

    /// Convert the raw vram data to RGB.
    fn prepare_framebuffer(&mut self) {
        let data = self.cpu.get_video_buffer();

        let fg = self
            .fg_color
            .iter()
            .map(|x| ((*x) * 255.0).round() as u8)
            .collect::<Vec<u8>>();

        let bg = self
            .bg_color
            .iter()
            .map(|x| ((*x) * 255.0).round() as u8)
            .collect::<Vec<u8>>();

        self.framebuffer.chunks_exact_mut(3).enumerate().for_each(
            |(i, rgb)| {
                if data[i] == 0 {
                    rgb.copy_from_slice(&bg);
                } else {
                    rgb.copy_from_slice(&fg);
                }
            },
        );
    }

    /// Render Ui built with Dear ImGui.
    fn render_ui(&mut self, frame: &mut glium::Frame) {
        if self.file_dialog.is_open && self.state == State::Idle {
            let result = self.file_dialog.query_result();

            if let file_dialog::DialogResult::RomFile(path) = result {
                self.rom = Some(path);
            }
        }

        let ui = self.imgui.frame();
        let gl_window = self.display.gl_window();

        // --- Main Menu Bar --- //
        if let Some(main_menu) = ui.begin_main_menu_bar() {
            if let Some(file_menu) = ui.begin_menu(im_str!("File"), true) {
                if MenuItem::new(im_str!("Open ROM")).build(&ui)
                    && !self.file_dialog.is_open
                {
                    self.state = State::Idle;
                    self.cpu.reset();

                    self.file_dialog.create_rom_dialog();
                }

                file_menu.end(&ui);
            }

            if let Some(emu_menu) = ui.begin_menu(im_str!("Emulation"), true) {
                if MenuItem::new(im_str!("Start"))
                    .enabled(self.state != State::Running && self.rom.is_some())
                    .build(&ui)
                {
                    if self.state == State::Idle {
                        // Load ROM and start.
                        let rom =
                            std::fs::read(self.rom.as_ref().unwrap()).unwrap();

                        self.cpu.load_rom(&rom).unwrap();
                    }

                    self.state = State::Running;
                }

                if MenuItem::new(im_str!("Pause"))
                    .enabled(self.state == State::Running)
                    .build(&ui) {
                        self.state = State::Paused;
                        self.audio.pause_beep();
                    }

                if MenuItem::new(im_str!("Stop"))
                    .enabled(self.state != State::Idle)
                    .build(&ui)
                {
                    self.cpu.reset();
                    self.state = State::Idle;
                }

                if let Some(cycles_menu) =
                    ui.begin_menu(im_str!("Cycles per Frame"), true)
                {
                    Slider::<u16>::new(im_str!("cycles"))
                        .range(1..=2000)
                        .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                        .build(&ui, &mut self.cycles_per_frame);

                    cycles_menu.end(&ui);
                }

                MenuItem::new(im_str!("Pallete"))
                    .build_with_ref(&ui, &mut self.pallete_window);

                if let Some(quirks_menu) =
                    ui.begin_menu(im_str!("Quirk Settings"), true)
                {
                    MenuItem::new(im_str!("Shift Quirk"))
                        .build_with_ref(&ui, &mut self.cpu.shift_quirk);
                    MenuItem::new(im_str!("Load and Store Quirk"))
                        .build_with_ref(&ui, &mut self.cpu.load_store_quirk);
                    MenuItem::new(im_str!("Jump Quirk"))
                        .build_with_ref(&ui, &mut self.cpu.jump_quirk);

                    quirks_menu.end(&ui);
                }

                emu_menu.end(&ui);
            }

            if let Some(help_menu) = ui.begin_menu(im_str!("Help"), true) {
                MenuItem::new(im_str!("Metrics"))
                    .build_with_ref(&ui, &mut self.metrics_window);
                MenuItem::new(im_str!("About"))
                    .build_with_ref(&ui, &mut self.about_window);

                help_menu.end(&ui);
            }

            self.menu_height = Some(ui.window_size()[1] as u32);
            main_menu.end(&ui);
        }

        // --- Windows --- //
        if self.about_window {
            let font_id = self.larger_font;

            Window::new(im_str!("About"))
                .bg_alpha(1.0)
                .resizable(false)
                .opened(&mut self.about_window)
                .build(&ui, || {
                    let token = ui.push_font(font_id);
                    ui.text_colored([0.6, 0.25, 0.1, 1.0], im_str!("Ferrous Chip-8"));
                    token.pop(&ui);

                    ui.text(im_str!(
                        "A simple, accurate (super) Chip-8 interpreter written in Rust."
                    ));
                    ui.separator();
                    ui.text(im_str!("Author: Anish Jewalikar"));
                    ui.text(im_str!(
                        "Licensed under the terms of the Apache-2.0 license."
                    ));
                });
        }

        if self.metrics_window {
            ui.show_metrics_window(&mut self.metrics_window);
        }

        if self.pallete_window {
            if let Some(window) = Window::new(im_str!("Pallete"))
                .always_auto_resize(true)
                .resizable(false)
                .opened(&mut self.pallete_window)
                .begin(&ui)
            {
                ColorEdit::new(
                    im_str!("Foreground Colour"),
                    &mut self.fg_color,
                )
                .picker(true)
                .format(imgui::ColorFormat::U8)
                .alpha(false)
                .build(&ui);

                ColorEdit::new(
                    im_str!("Background Colour"),
                    &mut self.bg_color,
                )
                .picker(true)
                .format(imgui::ColorFormat::U8)
                .alpha(false)
                .build(&ui);

                window.end(&ui);
            }
        }

        // -- Rendering -- //
        // Prepare for rendering.
        self.platform.prepare_render(&ui, gl_window.window());

        // Render ImGui.
        let draw_data = ui.render();
        self.renderer.render(frame, draw_data).unwrap();
    }
}
