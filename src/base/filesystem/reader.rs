//
//
// reader.rs
// Copyright (C) 2022 db3.network Author imotai <codego.me@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

// Copyright https://github.com/rust-lib-project/calibur/blob/main/src/common/file_system/reader.rs

use super::SequentialFile;
use crate::error::Result;

pub struct SequentialFileReader {
    file: Box<dyn SequentialFile>,
    filename: String,
}

impl SequentialFileReader {
    pub fn new(file: Box<dyn SequentialFile>, filename: String) -> Self {
        Self { file, filename }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read_sequential(buf)
    }

    pub fn name(&self) -> &str {
        self.filename.as_str()
    }

    pub fn use_direct_io(&self) -> bool {
        false
    }

    pub fn file_size(&self) -> usize {
        self.file.get_file_size()
    }
}
