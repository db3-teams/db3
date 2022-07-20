//
//
// mod.rs
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

use crate::error::{DB3Error, Result};
use crate::proto::substrait::{Plan, PlanRel};
use tree_sitter::{Tree, TreeCursor};

pub mod query_logical_plan;

// convert syntax tree to substrait plan
//pub fn build_logical_plan(tree:Tree) -> Result<substrait::Plan> {
//    let mut tree_cursor = tree.walk();
//}
