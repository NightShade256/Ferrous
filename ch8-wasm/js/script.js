import init, * as core from "../pkg/ch8_core.js";
import { AudioHandler } from "./audio.js";

const canvas = document.querySelector("canvas");
const context = canvas.getContext("2d");

let CPU = null;

// Is the interpreter currently running?
let isRunning = false;

// The handle provided by the first call to `setInterval`.
let intervalHandle = null;

let audioHandler = new AudioHandler();

// Maps scancodes to chip8 key indices.
const SCANCODES = new Map([
    ["Digit1", 0x1],
    ["Digit2", 0x2],
    ["Digit3", 0x3],
    ["Digit4", 0xc],
    ["KeyQ", 0x4],
    ["KeyW", 0x5],
    ["KeyE", 0x6],
    ["KeyR", 0xd],
    ["KeyA", 0x7],
    ["KeyS", 0x8],
    ["KeyD", 0x9],
    ["KeyF", 0xe],
    ["KeyZ", 0xa],
    ["KeyX", 0x0],
    ["KeyC", 0xb],
    ["KeyV", 0xf],
]);

// Clear the canvas to all black background.
function clearCanvas() {
    context.fillStyle = "black";
    context.fillRect(0, 0, 1152, 576);
}

/// Fired when the Start button is clicked.
function onStart(_) {
    if (isRunning) {
        return;
    }

    // Fetch the romInput interface.
    let input = document.getElementById("romInput");

    if (input.files.length !== 0) {
        // Read the first selected file, as `File`.
        let file = input.files[0];

        // Create a new reader, and set the `onload` event,
        // which is fired when the file is fully read.
        let reader = new FileReader();

        reader.addEventListener("load", (_) => {
            // The Vec<u8> of the bytes of the file.
            let buffer = new Uint8Array(reader.result);

            // Load the ROM into the interpreter memory.
            CPU.load_rom(buffer);

            // Start the execution of the ROM.
            isRunning = true;
            intervalHandle = setInterval(mainLoop, 1 / 60);
        });

        // Initiate reading the file.
        reader.readAsArrayBuffer(file);
    }
}

/// Fired when a key is pressed down.
function onKeyPress(event) {
    if (!SCANCODES.has(event.code) || !isRunning) {
        return;
    }

    // Set the key to be pressed.
    let index = SCANCODES.get(event.code);
    CPU.set_key_at_index(index, true);
}

/// Fired when a key is released.
function onKeyUp(event) {
    if (!SCANCODES.has(event.code) || !isRunning) {
        return;
    }

    // Set the key to be not pressed.
    let index = SCANCODES.get(event.code);
    CPU.set_key_at_index(index, false);
}

// Setup listeners for buttons, keyboard and more.
function setupListeners() {
    // hook open div to the actual input tag.
    document.getElementById("openButton").addEventListener("click", (_) => {
        document.getElementById("romInput").click();
    });

    document.getElementById("startButton").addEventListener("click", onStart);
    document.getElementById("resetButton").addEventListener("click", (_) => {
        if (!isRunning) {
            return;
        }

        clearInterval(intervalHandle);
        intervalHandle = null;

        clearCanvas();

        let shift = CPU.shift_quirk;
        let load = CPU.load_store_quirk;

        CPU.reset();
        CPU.set_shift(shift);
        CPU.set_load_store(load);

        isRunning = false;
    });

    let input = document.getElementById("romInput");

    if (input.files.length !== 0) {
        document.getElementById(
            "currentROM"
        ).innerHTML = `ROM: ${input.files[0].name}`;
    }

    input.addEventListener("change", (_) => {
        document.getElementById(
            "currentROM"
        ).innerHTML = `ROM: ${input.files[0].name}`;
    });

    let shift = document.getElementById("shiftButton");
    let load = document.getElementById("loadButton");

    shift.addEventListener("click", (_) => {
        CPU.set_shift(!CPU.shift_quirk);

        if (CPU.shift_quirk) {
            shift.innerHTML = "Shift Quirk: ON";
        } else {
            shift.innerHTML = "Shift Quirk: OFF";
        }
    });

    load.addEventListener("click", (_) => {
        CPU.set_load_store(!CPU.load_store_quirk);

        if (CPU.load_store_quirk) {
            load.innerHTML = "Load Quirk: ON";
        } else {
            load.innerHTML = "Load Quirk: OFF";
        }
    });

    document.addEventListener("keydown", onKeyPress);
    document.addEventListener("keyup", onKeyUp);
}

/// Render the current frame onto the canvas.
function renderFrame() {
    let buffer = CPU.clone_video_buffer();

    for (let row = 0; row < 32; row++) {
        for (let col = 0; col < 64; col++) {
            if (buffer[row * 64 + col] == 0) {
                context.fillStyle = "black";
            } else {
                context.fillStyle = "white";
            }

            context.fillRect(col * 18, row * 18, 18, 18);
        }
    }
}

/// Main interpreter loop.
function mainLoop() {
    /// 10 cycles per frame = 600 cycles per second
    for (let i = 0; i < 10; i++) {
        CPU.execute_cycle();
    }

    CPU.step_timers();

    if (CPU.st > 0) {
        audioHandler.try_start();
    } else {
        audioHandler.try_stop();
    }

    renderFrame();
}

// Main entry point.
async function run() {
    clearCanvas();

    // Initialize the WASM module for usage.
    await init();

    CPU = core.CPU.new();

    setupListeners();
}

run();
