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
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl};

/// Holds the canvas for rendering to the screen.
pub struct Renderer {
    canvas: Canvas<Window>,
}

impl Renderer {
    /// Return a new `Renderer` instance.
    pub fn new(context: &Sdl) -> Self {
        let video_sys = context.video().unwrap();
        let window = video_sys
            .window("Oxidized Chip8", 640, 320)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self { canvas }
    }

    /// Render the VRAM buffer onto the screen.
    pub fn render(&mut self, cpu: &CPU) {
        let (rows, cols) = cpu.get_row_col();
        let (width, height) = self.canvas.window().size();
        let buffer = cpu.get_video_buffer();

        let (row_scale, col_scale) = if cpu.is_highres {
            (width / 128, height / 64)
        } else {
            (width / 64, height / 32)
        };

        for row in 0..rows {
            let offset = row as usize * cols as usize;

            for col in 0..cols {
                let color = if buffer[offset + col as usize] == 0 {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(255, 255, 255)
                };

                self.canvas.set_draw_color(color);

                let x = (col as i32) * row_scale as i32;
                let y = (row as i32) * col_scale as i32;

                let rect = Rect::new(x, y, row_scale, col_scale);
                self.canvas.fill_rect(rect).unwrap();
            }
        }

        self.canvas.present();
    }
}
