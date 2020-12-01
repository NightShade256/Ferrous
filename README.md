# Oxidized Chip8

A simple, full featured Chip-8 interpreter implementation in Rust.

## Building

You can use the `ch8-sdl2` crate, to play ROM(s) with ease.

To build the crate, first install the SDL2 development libraries for your respective platform
by following the instructions on [this](https://crates.io/crates/sdl2) page.

Then execute the following command,

```bash
cargo build -p ch8-sdl2 --release
```

The executable will be stored in the directory `target/release`, copy that executable
to the top-level directory and also copy the `SDL2.dll` file from the `ch8-sdl2/bin` directory.

**You should only copy the DLL if you are on 64-bit Windows.**
(It will not cut it on 32-bit Win)

You will not require the shared library if you are on a \*nix platform.

## Usage

See the help section in CLI interface,

```bash
./ch8-sdl2 -h
```

## Structure

The 'interpreter' is divided into two parts,

1. The `ch8-core` crate, as the name suggests, is the core interpreter
   backend. It must used to power a frontend.

2. The frontend using SDL2, `ch8-sdl2` crate.

This results in a resuable core.

The core library also has a `cargo` feature called `wasm` which
allows it to target WASM running in the browsers.

## Implementation Details

The `ch8-core` passes the following test ROM(s),

1. https://github.com/corax89/chip8-test-rom
2. https://github.com/metteo/chip8-test-rom
3. https://github.com/Skosulor/c8int/tree/master/test
4. BestCoder's test ROM (Need to tweak quirks to pass)

There are options in the core crate to toggle behaviour (quirks) regarding the,
load/store and shift instructions as described [here](https://chip-8.github.io/database/#options).

By default though,

1. Shift instructions place value of Vy into Vx and then shift.

2. Load/Store instructions increment `I`.

## License

The project (all the crates in this repository) is licensed under the terms of the Apache-2.0 license.
See `LICENSE` for more.
