//
//
// query_logical_plan.rs
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

use crate::catalog::catalog::Catalog;
use crate::error::{DB3Error, Result};
use crate::proto::substrait::{Expression, Rel, NamedStruct};
use crate::proto::substrait::expression::mask_expression::StructItem;
use tree_sitter::{Node, Tree, TreeCursor};
use std::sync::Arc;
use substring::Substring;
/// transform from query statement to query logical plan
///              QueryStatment
///                   |
///                QueryExpr
///                   |
///                 select
///               /   |    \
///              /    |      \
///         SELECT select_list from_clause
///                   |
///                   |
///                   V
///                RootRel
///                   |
///               ProjectRel
///                   |
///               ReadRel
pub fn transform(sql: &str, tree: &Tree, catalog: Arc<Catalog>, default_db: &str) -> Option<Rel> {
    None
}

///
///transform select node to substrait plan like
///
///1. select data on table, ReadRel -> ProjectRel
///           or
///2. select data on subquery, ProjectRel -> ProjectRel
///3. group , AggregateRel -> ProjectRel
///4. join , JoinRel -> ProjectRel
///
//pub fn transform_select_node(sql:&str, node:&Node)-> Result<Rel> {
//    // the node type must be select node
//    if node.kind() != "select" && node.has_name() {
//        reutrn Err(DB3Error::SQLTransformError(format!("{}", node)));
//    }
//
//    // the first child must be keyword 'SELECT'
//    // the second child must be select_list node
//    // the third child is from_clause node and it can be optional
//    if node.kind() != "SELECT"  {
//        reutrn Err(DB3Error::SQLTransformError(format!("expect keyword SELECT but {}", node)));
//    }
//}

///
///  transform variable e.g. `a`, `t1.a`, `db.t1`, `db.t1.event` or `t1.a.b`  to `StructItem`, the
///  variable can be a `column` or a `table` because it can come from `select cause` or `from cause`
///
pub fn transform_dotted_name_node(
    sql: &str,
    node: &Node,
    schema_tree: &NamedStruct,
) -> Result<StructItem> {
    if node.kind().eq("dotted_name") && node.is_named() {
        //TODO resolve `t1.a.b` and `db.t1.event`
        assert!(node.child_count() == 3);
        let names = vec![0, 2];
        let mut index = 0;
        for (i, name) in schema_tree.names.iter().enumerate() {
            if let Some(child) = node.child(names[index]) {
                let field_name = sql.substring(child.start_byte(), child.end_byte());
                if name.as_str().eq(field_name) {
                    if index == names.len() - 1 {
                        let item = StructItem {
                            field: i as i32,
                            child: None,
                        };
                        return Ok(item);
                    } else {
                        index += 1;
                    }
                }
            }
        }
    }
    Err(DB3Error::SQLTransformError(format!(
        "fail to transform node {:?} to select item",
        node
    )))
}
