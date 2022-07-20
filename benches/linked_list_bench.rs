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
    use db3::base::linked_list::LinkedList;
    use test::Bencher;
    #[bench]
    fn bench_linked_list_push(b: &mut Bencher) {
        let ll: LinkedList<i32> = LinkedList::new();
        b.iter(|| {
            // Inner closure, the actual test
            for i in 1..1000 {
                ll.push_front(i);
            }
        });
    }

    #[bench]
    fn bench_linked_list_it(b: &mut Bencher) {
        let ll: LinkedList<i32> = LinkedList::new();
        for i in 1..1000 {
            ll.push_front(i);
        }
        b.iter(|| {
            // Inner closure, the actual test
            for i in 1..1000 {
                for j in ll.iter() {
                    let _ = i + j;
                }
            }
        });
    }
}
