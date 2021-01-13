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

//! Contains helper functions and the main entry point for the frontend.

use glium::glutin::ContextBuilder;
use glium::glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};
use glium::{Display, Surface};

mod fps_limiter;
mod gui;

use fps_limiter::FpsLimiter;

/// Raw RGBA data of unmodified Rust logo.
const LOGO_DATA: &[u8] = include_bytes!("assets/Rust Logo.raw");

/// Initialize the window, and then glium's
/// display.
fn initialize_display(event_loop: &EventLoop<()>) -> Display {
    // Interpreter the raw data as a window icon.
    let icon_result = Icon::from_rgba(LOGO_DATA.to_vec(), 512, 512);

    if icon_result.is_err() {
        eprintln!("Failed to initialize window icon.");
    }

    // Create a GL context, and a window.
    let cb = ContextBuilder::new();
    let wb = WindowBuilder::new()
        .with_decorations(true)
        .with_window_icon(icon_result.ok())
        .with_title("Ferrous Chip-8")
        .with_min_inner_size(LogicalSize::new(128, 64))
        .with_inner_size(LogicalSize::new(1152, 576));

    // Create the glium display, and clear it.
    let display = Display::new(wb, cb, &event_loop)
        .expect("Failed to initialize the display.");

    let mut frame = display.draw();
    frame.clear_color_srgb(0.0, 0.0, 0.0, 1.0);
    frame.finish().expect("Failed to swap buffers.");

    display
}

/// Render Chip-8 display, onto the screen.
fn render_display(display: &Display) -> glium::Frame {
    let frame = display.draw();

    frame
}

/// Start the emulator, and run until
/// the user requests quitting.
pub fn start() {
    // Create the event loop and initialize the glium display.
    let event_loop = EventLoop::new();
    let display = initialize_display(&event_loop);
    let mut user_interface = gui::UserInterface::new(&display);
    let mut cpu = ferrous_core::CPU::new();
    let mut fps_limiter = FpsLimiter::new();

    event_loop.run(move |event, _, control_flow| {
        user_interface.handle_event(&display, &event);

        match event {
            Event::NewEvents(_) => {
                let delta = fps_limiter.update();
                user_interface.update_delta(delta);
            }

            Event::MainEventsCleared => {
                user_interface.prepare_frame(&display);
            }

            Event::RedrawRequested(_) => {
                use gui::EmulatorState::*;

                match user_interface.state.emulator_state {
                    Running => {
                        for _ in 0..user_interface.state.cycles_per_frame {
                            if cpu.execute_cycle().is_none() {
                                eprintln!(
                                    "[WARN] invalid or unknown opcode encountered."
                                );
                            }
                        }

                        cpu.step_timers();
                    }

                    Quit => *control_flow = ControlFlow::Exit,

                    _ => {}
                }

                let frame = render_display(&display);
                user_interface.render_ui(&display, frame, &mut cpu);
            }

            Event::RedrawEventsCleared => {
                fps_limiter.limit();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit;
                }

                _ => {}
            },

            _ => {}
        }
    });
}
