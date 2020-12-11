# Oxidized Chip-8

A simple, full featured (super) Chip-8 interpreter written in pure Rust.

## Structure

The project is divided into three parts,

1. The `ch8-core` backend crate which is a `no_std` crate
   providing an implementation on the (super) Chip-8 interpreter.

2. The `ch8-sdl2` frontend crate which augments the core crate by constructing
   a frontend with the help of SDL 2.

3. The `ch8-wasm` frontend which is built with HTML/CSS/JS to work in a Web browser.
   You can find a live deployed version for this frontend [here](https://nightshade256.github.io/oxidized-ch8/).

You can use the core crate in your own interpreters and build a frontend. It is fully
documented and you shouldn't have a problem.

## Building

You can build the `ch8-sdl2` crate and play some ROM(s).

To do so, you will require a `Rust` toolchain for your platform, CMake and
a working `C` compiler.

```bash
cd ch8-sdl2
...

# This will build SDL 2 along with the interpreter.
# If you have SDL 2 development headers/library already
# installed you can drop the 'bundled' feature.
cargo build --release --features "bundled"
```

The binary will be stored in `target/release` copy that to a suitable location.
If you are on Windows, copy the `SDL2.dll` from `ch8-sdl2/bin` to the same directory
as above.

## Usage

See the CLI help page.

```bash
./ch8-sdl2 -h
```

## WASM

You can optionally use the Web frontend instead.
To do so please install `wasm-pack` first.

```bash
cd ch8-core
...

wasm-pack build --target web -- --features "wasm"
```

Copy the resulting `pkg/` directory to `ch8-wasm` folder.
cd to the `ch8-wasm` folder and start an HTTP server, and hop on over to localhost
to see the Web frontend.

You can find a live deployed version for this frontend [here](https://nightshade256.github.io/oxidized-ch8/). 

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

The project is licensed under the terms of the Apache-2.0 license.
