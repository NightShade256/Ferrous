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

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

/// Represents a square wave.
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    // Repurposed from SDL2 Doc Examples.
    fn callback(&mut self, out: &mut [Self::Channel]) {
        // Generate a square wave.
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

/// Handles the audio output (a single beep).
pub struct Audio {
    device: AudioDevice<SquareWave>,
    is_playing: bool,
}

impl Audio {
    /// Create a new `Audio` instance.
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let system = sdl_context.audio().unwrap();

        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = system
            .open_playback(None, &spec, |asn| SquareWave {
                phase_inc: 360.0 / asn.freq as f32,
                phase: 0.0,
                volume: 0.40,
            })
            .unwrap();

        Self {
            device,
            is_playing: false,
        }
    }

    // Resume paused beep.
    pub fn start_beep(&mut self) {
        if !self.is_playing {
            self.device.resume();
            self.is_playing = true;
        }
    }

    // Pause the playing beep.
    pub fn stop_beep(&mut self) {
        if self.is_playing {
            self.device.pause();
            self.is_playing = false;
        }
    }
}
