//
//
// row_codec_bench.rs
// Copyright (C) 2022 rtstore.io Author imotai <codego.me@gmail.com>
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

#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
    use db3::codec::row_codec::{Data, RowRecordBatch};
    use test::Bencher;
    #[bench]
    fn bench_encode(b: &mut Bencher) {
        let batch = vec![
            vec![Data::Bool(true), Data::Int32(12)],
            vec![Data::Bool(false), Data::Int32(11)],
        ];

        let row_batch = RowRecordBatch {
            batch,
            schema_version: 1,
        };

        b.iter(|| {
            // Inner closure, the actual test
            for _i in 1..1000 {
                bincode::serialize(&row_batch).unwrap();
            }
        });
    }
}
