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

use std::sync::mpsc::{channel, Sender};

use rodio::{source::SineWave, OutputStream, Sink};

/// Audio subsystem for the interpreter.
pub struct Audio {
    sender: Sender<bool>,
}

impl Audio {
    /// Create a new `Audio` instance.
    pub fn new() -> Self {
        let source = SineWave::new(420);
        let (tx, rx) = channel();

        // We are going for a multithreaded model due to a conflict with glium.
        // See [https://github.com/RustAudio/rodio/issues/214] for more.
        std::thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            sink.pause();
            sink.append(source);

            while let Ok(continue_beep) = rx.recv() {
                if continue_beep {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
        });

        Audio { sender: tx }
    }

    /// Start playing the beep, if not already playing.
    pub fn play_beep(&self) {
        self.sender.send(true).unwrap();
    }

    /// Pause the beep, if not already paused.
    pub fn pause_beep(&self) {
        self.sender.send(false).unwrap();
    }
}
