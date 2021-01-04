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

use std::sync::mpsc::{channel, Receiver};

use nfd2::Response;

pub enum DialogResult {
    None,
    RomFile(std::path::PathBuf),
}

/// Utility for creating native file dialogs.
pub struct FileDialog {
    /// Is a dialog currently open?
    pub is_open: bool,

    /// Result reciever.
    rx: Option<Receiver<DialogResult>>,
}

impl FileDialog {
    /// Create a new `FileDialog` instance.
    pub fn new() -> Self {
        Self {
            is_open: false,
            rx: None,
        }
    }

    /// Create a file dialog for selecting a ROM.
    pub fn create_rom_dialog(&mut self) {
        self.is_open = true;

        let (tx, rx) = channel();
        self.rx = Some(rx);

        std::thread::spawn(move || {
            let response = nfd2::open_file_dialog(None, None);

            let result = match response.unwrap_or(Response::Cancel) {
                Response::Okay(path) => DialogResult::RomFile(path),
                _ => DialogResult::None,
            };

            tx.send(result).unwrap();
        });
    }

    /// Query the result for last dialog.
    pub fn query_result(&mut self) -> DialogResult {
        if let Some(rx) = &self.rx {
            if let Ok(res) = rx.try_recv() {
                self.is_open = false;
                return res;
            }
        }

        DialogResult::None
    }
}
