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
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};
use glium::{Display, Surface};

mod audio;
mod fps_limiter;
mod gui;

use fps_limiter::FpsLimiter;

/// Raw RGBA data of unmodified Rust logo.
const LOGO_DATA: &[u8] = include_bytes!("assets/Rust Logo.raw");

/// Initialize the window, and then glium's
/// display.
fn initialize_display(event_loop: &EventLoop<()>) -> Display {
    // Interpreter the raw data as a window icon.
    let icon_result = Icon::from_rgba(LOGO_DATA.to_vec(), 64, 64);

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
    let display = Display::new(wb, cb, &event_loop).expect("Failed to initialize the display.");

    let mut frame = display.draw();
    frame.clear_color_srgb(0.0, 0.0, 0.0, 1.0);
    frame.finish().expect("Failed to swap buffers.");

    display
}

/// Handle events provided by the OS.
fn handle_keyboard_event(cpu: &mut ferrous::CPU, input: &KeyboardInput) {
    if let KeyboardInput {
        virtual_keycode: Some(keycode),
        state,
        ..
    } = input
    {
        let index = match keycode {
            VirtualKeyCode::Key1 => Some(0x1),
            VirtualKeyCode::Key2 => Some(0x2),
            VirtualKeyCode::Key3 => Some(0x3),
            VirtualKeyCode::Key4 => Some(0xC),
            VirtualKeyCode::Q => Some(0x4),
            VirtualKeyCode::W => Some(0x5),
            VirtualKeyCode::E => Some(0x6),
            VirtualKeyCode::R => Some(0xD),
            VirtualKeyCode::A => Some(0x7),
            VirtualKeyCode::S => Some(0x8),
            VirtualKeyCode::D => Some(0x9),
            VirtualKeyCode::F => Some(0xE),
            VirtualKeyCode::Z => Some(0xA),
            VirtualKeyCode::X => Some(0x0),
            VirtualKeyCode::C => Some(0xB),
            VirtualKeyCode::V => Some(0xF),
            _ => None,
        };

        if let Some(i) = index {
            cpu.set_key_at_index(i, *state == ElementState::Pressed);
        }
    }
}

/// Start the emulator, and run until
/// the user requests quitting.
pub fn start() {
    // Create the event loop and initialize the glium display.
    let event_loop = EventLoop::new();
    let audio = audio::Audio::new();
    let display = initialize_display(&event_loop);
    let mut user_interface = gui::UserInterface::new(&display);
    let mut cpu = ferrous::CPU::new();
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
                                eprintln!("[WARN] invalid or unknown opcode encountered.");
                            }
                        }

                        cpu.step_timers();
                    }

                    Quit => *control_flow = ControlFlow::Exit,

                    _ => {}
                }

                if cpu.st > 0 && user_interface.state.emulator_state == Running {
                    audio.play_beep();
                } else {
                    audio.pause_beep();
                }

                user_interface.update_framebuffer(&cpu);
                user_interface.render_ui(&display, &mut cpu);
            }

            Event::RedrawEventsCleared => {
                fps_limiter.limit();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::KeyboardInput { ref input, .. }
                    if user_interface.state.emulator_state == gui::EmulatorState::Running =>
                {
                    handle_keyboard_event(&mut cpu, input);
                }

                _ => {}
            },

            _ => {}
        }
    });
}
