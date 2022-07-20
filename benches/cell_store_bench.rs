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
    use arrow::datatypes::Schema;
    use arrow::datatypes::*;
    use db3::codec::row_codec::{Data, RowRecordBatch};
    use db3::store::cell_store::{CellStore, CellStoreConfig};
    use s3::bucket::Bucket;
    use s3::creds::Credentials;
    use std::sync::Arc;
    use tempdir::TempDir;
    use test::Bencher;

    //fn bench_cell_store(b: &mut Bencher) {
    //    let valid_schema = Arc::new(Schema::new(vec![Field::new("c1", DataType::Int64, true)]));
    //    let auth = Credentials::from_env_specific(
    //        Some("AWS_S3_ACCESS_KEY"),
    //        Some("AWS_S3_SECRET_KEY"),
    //        None,
    //        None,
    //    )
    //    .unwrap();
    //    let tmp_dir_path = TempDir::new("put_records_bench").expect("create temp dir");
    //    let bucket_name = "test_bk";
    //    let region = "http://127.0.0.1:9090";
    //    if let Some(tmp_dir_path_str) = tmp_dir_path.path().to_str() {
    //        let local_binlog_path_prefix = tmp_dir_path_str.to_string();
    //        let config = CellStoreConfig::new(
    //            bucket_name,
    //            region,
    //            &valid_schema,
    //            &local_binlog_path_prefix,
    //            auth,
    //        )
    //        .unwrap();
    //        if let Ok(c) = CellStore::new(config) {
    //            b.iter(|| {
    //                // Inner closure, the actual test
    //                for _i in 1..100 {
    //                    c.put_records(gen_sample_row_batch());
    //                }
    //            });
    //        }
    //    }
    //}

    //fn gen_sample_row_batch() -> RowRecordBatch {
    //    let batch = vec![
    //        vec![Data::Int32(12)],
    //        vec![Data::Int32(11)],
    //        vec![Data::Int32(10)],
    //    ];
    //    RowRecordBatch {
    //        batch,
    //        schema_version: 1,
    //        id: "eth.price".to_string(),
    //    }
    //}
}
