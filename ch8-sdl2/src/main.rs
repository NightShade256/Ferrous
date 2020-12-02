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

use std::{fs, thread::sleep, time::Duration};

use ch8_core::CPU;
use clap::{App, Arg};
use sdl2::{event::Event, keyboard::Keycode, EventPump};

mod audio;
mod graphics;

/// Main entrypoint.
fn main() {
    let matches = App::new("Oxidized Chip8")
        .version("0.1.1")
        .about("A simple, accurate Chip8 emulator written in Rust.")
        .arg(
            Arg::with_name("file")
                .help("The ROM file to execute")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("lsq")
                .help("Leave `I` unchanged in load/store instructions")
                .short("l")
                .long("load-store-quirk"),
        )
        .arg(
            Arg::with_name("sfq")
                .help("Ignore Vy in shift instructions")
                .short("s")
                .long("shift-quirk"),
        )
        .get_matches();

    let path = matches.value_of("file").unwrap();
    let lsq = matches.is_present("lsq");
    let sfq = matches.is_present("sfq");
    let rom = fs::read(path).unwrap();

    // Initialize SDL.
    let context = sdl2::init().unwrap();
    let mut event_pump = context.event_pump().unwrap();

    // Create a new renderer and CPU.
    let mut renderer = graphics::Renderer::new(&context);
    let mut audio_handler = audio::Audio::new(&context);
    let mut cpu = CPU::new();

    // Enable/Disable quirks.
    cpu.set_load_store(lsq);
    cpu.set_shift(sfq);

    // Load the Chip8 ROM into memory.
    match cpu.load_rom(&rom) {
        None => {}
        Some(_) => {
            eprintln!("The ROM is greater than 3584 bytes in size.");
            return;
        }
    }

    'main: loop {
        // This gets called 10 times per frame,
        // thus yielding 600 cycles per second.
        for _ in 0..10 {
            cpu.execute_cycle();
        }

        // Step the sound and delay timers.
        cpu.step_timers();

        // Handle input, and events.
        match handle_events(&mut event_pump, &mut cpu) {
            Ok(_) => {}
            Err(_) => break 'main,
        }

        // Start/Stop beep.
        if cpu.st > 0 {
            audio_handler.start_beep();
        } else {
            audio_handler.stop_beep();
        }

        // Render the current frame.
        renderer.render(cpu.get_video_buffer());

        // Sleep for 16.67 ms.
        sleep(Duration::from_secs_f64(1.0 / 60.0));
    }
}

/// Handle keyboard input, and Window quit events.
fn handle_events(event_pump: &mut EventPump, cpu: &mut CPU) -> Result<(), ()> {
    for event in event_pump.poll_iter() {
        if let Event::Quit { .. } = event {
            return Err(());
        }
    }

    cpu.reset_keys();

    let keys: Vec<Keycode> = event_pump
        .keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect();

    for key in keys {
        let index = match key {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        };

        if let Some(i) = index {
            cpu.set_key_at_index(i, true);
        }
    }

    Ok(())
}
