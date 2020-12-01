export class AudioHandler {
    constructor() {
        this.context = new AudioContext();

        // create new oscillator to play a Square Wave at 360 Hz.
        this.oscilator = this.context.createOscillator();
        this.oscilator.type = "square";
        this.oscilator.frequency.setValueAtTime(360, this.context.currentTime);

        // create a gain node, hook the oscillator to it,
        // and the gain node to the context.
        this.gain = this.context.createGain();
        this.oscilator.connect(this.gain);
        this.gain.connect(this.context.destination);

        this.gain.gain.value = 0;
        this.oscilator.start();
        this.isPlaying = false;
    }

    /// Start the beep if paused.
    try_start() {
        if (!this.isPlaying) {
            this.gain.gain.value = 1;
            this.isPlaying = true;
        }
    }

    /// Stop the beep if playing.
    try_stop() {
        if (this.isPlaying) {
            this.gain.gain.value = 0;
            this.isPlaying = false;
        }
    }
}
