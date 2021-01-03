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

use clap::{App, Arg};
use ferrous_core::CPU;

mod audio;
mod emulator;
mod graphics;

fn main() {
    let matches = App::new("Ferrous Chip-8")
        .version("0.3.0")
        .about("A simple, accurate (super) Chip-8 emulator written in Rust.")
        .arg(
            Arg::with_name("file")
                .help("The ROM file to execute")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("load_store_quirk")
                .help("Leave `I` unchanged in load/store instructions")
                .short("l")
                .long("load-store-quirk"),
        )
        .arg(
            Arg::with_name("shift_quirk")
                .help("Ignore Vy in shift instructions")
                .short("s")
                .long("shift-quirk"),
        )
        .arg(
            Arg::with_name("cycles")
                .help("Number of cycles to execute per frame")
                .short("c")
                .help("cycles")
                .takes_value(true),
        )
        .get_matches();

    // Parse CLI input.
    let rom_path = matches.value_of("file").unwrap();
    let lq = matches.is_present("load_store_quirk");
    let sq = matches.is_present("shift_quirk");

    let cycles = matches
        .value_of("cycles")
        .unwrap_or("10")
        .parse::<i32>()
        .unwrap();

    // Read the ROM to an in memory buffer.
    let rom = std::fs::read(rom_path).unwrap();

    // Create the CPU.
    let mut cpu = CPU::new();

    // Configure quirks.
    cpu.set_load_store(lq);
    cpu.set_shift(sq);

    match cpu.load_rom(&rom) {
        Ok(_) => {}

        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    }

    emulator::start(cpu, cycles);
}
