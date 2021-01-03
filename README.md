# Ferrous Chip-8

A simple, full featured (super) Chip-8 interpreter written in pure Rust.

## Structure

The project is divided into two parts,

1. The `ferrous-core` backend crate which provides an implementation
   of the (super) Chip-8 interpreter.

2. The `ferrous-ch8` frontend crate which augments the core crate by constructing
   a frontend with the help of OpenGL (glium).

You can use the core crate in your own interpreters and build a frontend. It is fully
documented and you shouldn't have a problem.

## Building

You can build the interpreter through `cargo`.

To do so, you will require a `Rust` toolchain for your platform.

```bash
cargo build --release
```

The binary will be stored in `target/release` copy that to a suitable location.

## Usage

See the CLI help page.

```bash
./ferrous-ch8 -h
```

## Implementation Details

The interpreter passes the following test ROM(s),

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
