//
//
// strings.rs
// Copyright (C) 2022 db3.network Author imrtstore <rtstore_dev@outlook.com>
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

use crate::error::{DB3Error, Result};
uselog!(debug);
const NUM_LABELS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const STORAGE_LABELS: [char; 7] = [' ', 'K', 'M', 'G', 'T', 'P', 'E'];

pub fn to_readable_num_str(input: usize, len: usize) -> String {
    let mut name = vec!['0'; len];
    let mut shift = 0;
    let mut target = input;
    while shift < len {
        name[len - shift - 1] = NUM_LABELS[target % 10];
        target /= 10;
        shift += 1;
    }
    name.iter().collect::<String>()
}

pub fn bytes_to_readable_num_str(bytes_size: u64) -> String {
    let max_shift = 7;
    let mut shift = 0;
    let mut local_bytes_size = bytes_size;
    let mut value: f64 = bytes_size as f64;
    local_bytes_size >>= 10;
    while local_bytes_size > 0 && shift < max_shift {
        value /= 1024.0;
        shift += 1;
        debug!(
            "input byte size {} local_bytes_size {}",
            bytes_size, local_bytes_size
        );
        local_bytes_size >>= 10;
    }
    format!("{0:.2}{1}", value, STORAGE_LABELS[shift])
}

#[inline]
pub fn hex_string_to_u64(number: &str) -> Result<u64> {
    let without_prefix = number.trim_start_matches("0x");
    let result = u64::from_str_radix(without_prefix, 16)
        .map_err(|e| DB3Error::ParseNumberError(e, number.to_string()))?;
    Ok(result)
}

#[inline]
pub fn gen_s3_url(bucket: &str, prefix: &[&str], filename: &str) -> String {
    format!("s3://{}/{}/{}", bucket, prefix.join("/"), filename)
}

#[inline]
pub fn parse_s3_url(url: &str) -> Result<(String, String)> {
    let (bucket, key) = url
        .split_once('/')
        .ok_or_else(|| DB3Error::FSInvalidFileError {
            path: url.to_string(),
        })?;
    Ok((bucket.to_owned(), key.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_s3_url() {
        let url = gen_s3_url("test_bk", &["test"], "test.parquet");
        assert_eq!("s3://test_bk/test/test.parquet", &url);

        let url2 = "test_bk/test/test.parquet";
        if let Ok((bucket, key)) = parse_s3_url(&url2) {
            assert_eq!(&bucket, "test_bk");
            assert_eq!("test/test.parquet", &key);
        } else {
            panic!("should not be here");
        }
    }

    #[test]
    fn test_it_readable_num_str_normal() {
        let label = to_readable_num_str(10, 3);
        assert_eq!("010", label);
        let label = to_readable_num_str(10, 0);
        assert_eq!("", label);
    }

    #[test]
    fn test_bytes_to_readable_num_str() {
        let less_1k = 1023;
        let label = bytes_to_readable_num_str(less_1k);
        assert_eq!("1023.00 ", label);
    }

    #[test]
    fn test_hex_to_u64() {
        let number: &str = "0xA";
        let result = hex_string_to_u64(number);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }
}
