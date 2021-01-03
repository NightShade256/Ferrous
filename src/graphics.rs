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

use ferrous_core::CPU;

use glium::{
    glutin::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
    texture::{RawImage2d, Texture2d},
    uniforms::MagnifySamplerFilter,
    BlitTarget, Display, Surface,
};

pub struct Renderer {
    /// OpenGL backed display.
    pub display: Display,

    /// RGB framebuffer for the display.
    pub framebuffer: Box<[u8; 128 * 64 * 3]>,
}

impl Renderer {
    /// Create a new `Ui` instance.
    pub fn new(events_loop: &EventLoop<()>) -> Self {
        let cb = ContextBuilder::new();
        let wb = WindowBuilder::new()
            .with_decorations(true)
            .with_title("Ferrous Chip-8")
            .with_min_inner_size(LogicalSize::new(128, 64))
            .with_inner_size(LogicalSize::new(640, 320));

        // Create Glium display.
        let display = Display::new(wb, cb, events_loop).unwrap();

        // Clear the screen.
        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame.finish().unwrap();

        Self {
            display,
            framebuffer: Box::new([0; 128 * 64 * 3]),
        }
    }

    /// Render video memory onto the screen.
    pub fn render_frame(&mut self, cpu: &CPU) {
        // Prepare framebuffer for rendering.
        self.prepare_framebuffer(cpu.get_video_buffer());
        let (height, width) = cpu.get_height_width();

        // Create texture.
        let buffer_length = height * width * 3;

        let image = RawImage2d::from_raw_rgb_reversed(
            &self.framebuffer[..buffer_length],
            (width as u32, height as u32),
        );

        let texture = Texture2d::new(&self.display, image).unwrap();
        let window_size = self.display.gl_window().window().inner_size();

        // Blit the texture onto the screen.
        let frame = self.display.draw();
        texture.as_surface().blit_whole_color_to(
            &frame,
            &BlitTarget {
                left: 0,
                bottom: 0,
                width: window_size.width as i32,
                height: window_size.height as i32,
            },
            MagnifySamplerFilter::Nearest,
        );
        frame.finish().unwrap();
    }

    /// Convert the raw vram data to RGB.
    fn prepare_framebuffer(&mut self, data: &[u8]) {
        self.framebuffer
            .chunks_exact_mut(3)
            .enumerate()
            .for_each(|(i, rgb)| {
                if data[i] == 0 {
                    rgb.copy_from_slice(&[0, 0, 0]);
                } else {
                    rgb.copy_from_slice(&[255, 255, 255]);
                }
            });
    }
}
