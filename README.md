# Oxidized Chip8

A simple, full featured Chip-8 interpreter implementation.

## Structure

The 'interpreter' is divided into two parts,

1. The `ch8-core` crate, as the name suggests, is the core interpreter
   backend. It must used to power a frontend.

2. The frontend. (WIP)

## Implementation Details

The `ch8-core` passes the following test ROM(s),

1. https://github.com/corax89/chip8-test-rom
2. https://github.com/metteo/chip8-test-rom
3. https://github.com/Skosulor/c8int/tree/master/test
4. BestCoder's test ROM (no main page).

There are options in the core crate to toggle behaviour (quirks) regarding the,
load/store and shift instructions as described [here](https://chip-8.github.io/database/#options).

By default though,

1. Shift instructions shift Vx inplace, and leave Vy as it is.
2. Load/Store instructions leave `I` unchanged.

## License

The project is licensed under the terms of the Apache-2.0 license.
See `LICENSE` for more.
