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

use std::time::{Duration, Instant};

use ferrous_core::CPU;

use glium::glutin::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::{app, audio};

/// Start the emulator, and run until the user quits.
pub fn start() {
    // Create the event loop.
    let events_loop = glium::glutin::event_loop::EventLoop::new();

    // Initialize the window, the renderer and audio system.
    let audio_system = audio::Audio::new();
    let mut app = app::Application::new(&events_loop);

    let mut last_time = Instant::now();
    let mut next_time = Instant::now() + Duration::from_secs_f64(1.0 / 60.0);

    events_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                next_time += Duration::from_secs_f64(1.0 / 60.0);

                app.imgui.io_mut().update_delta_time(now - last_time);
                last_time = now;
            }

            Event::MainEventsCleared => {
                let gl_window = app.display.gl_window();

                app.platform
                    .prepare_frame(app.imgui.io_mut(), gl_window.window())
                    .unwrap();

                gl_window.window().request_redraw();
            }

            Event::RedrawRequested(_) => {
                // Exit if the CPU encountered a Super Chip exit instruction.
                if app.cpu.is_halted && app.running {
                    app.running = false;
                    app.cpu.reset();
                }

                if app.running {
                    // Step timers, and execute the required cycles.
                    for _ in 0..app.cycles_per_frame {
                        app.cpu.execute_cycle().unwrap();
                    }
                }

                if app.cpu.st > 0 {
                    audio_system.start_beep();
                } else {
                    audio_system.pause_beep();
                }

                app.cpu.step_timers();

                // Render the framebuffer.
                app.render_frame();
            }

            // Limit framerate to 60 frames per second.
            Event::RedrawEventsCleared => {
                let now = Instant::now();

                if now < next_time {
                    std::thread::sleep(next_time - now);
                }
            }

            // Handle keyboard events, and quit requests.
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::KeyboardInput { input, .. } if app.running => {
                    handle_keyboard_event(&mut app.cpu, input)
                }
                _ => {}
            },

            _ => {}
        }

        let gl_window = app.display.gl_window();
        app.platform.handle_event(
            app.imgui.io_mut(),
            gl_window.window(),
            &event,
        );
    });
}

/// Handle events provided by the OS.
fn handle_keyboard_event(cpu: &mut CPU, event: &KeyboardInput) {
    if let KeyboardInput {
        virtual_keycode: Some(keycode),
        state,
        ..
    } = event
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
