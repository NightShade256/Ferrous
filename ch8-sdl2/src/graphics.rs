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

use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl};

/// VRAM -> SDL2 Window Scale.
const SCALE: i32 = 10;

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
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self { canvas }
    }

    /// Render the VRAM buffer onto the screen.
    pub fn render(&mut self, buffer: &[u8]) {
        for row in 0..32 {
            let offset = row * 64;

            for col in 0..64 {
                let color = if buffer[offset + col] == 0 {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(255, 255, 255)
                };

                self.canvas.set_draw_color(color);

                let x = (col as i32) * SCALE;
                let y = (row as i32) * SCALE;

                let rect = Rect::new(x, y, SCALE as u32, SCALE as u32);
                self.canvas.fill_rect(rect).unwrap();
            }
        }

        self.canvas.present();
    }
}
