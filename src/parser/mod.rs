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

/// use tree sitter to produce syntax tree
pub fn parse_sql(sql: &str) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_sql::language()).unwrap();
    parser.parse(sql, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn visit_node(node: tree_sitter::Node, level: usize) {
        println!(
            "level {} kind {} id {}  has name {}",
            level,
            &node.kind(),
            &node.kind_id(),
            &node.is_named()
        );
        for i in 0..node.child_count() {
            if let Some(n) = node.child(i) {
                visit_node(n, level + 1);
            }
        }
    }
    #[test]
    fn test_parse_sql() {
        let sql = "select t1.a, t1.b, log(t1.c), t1.e + 1 from t1;select * from t3;";
        let tree = parse_sql(sql);
        if let Some(t) = tree {
            visit_node(t.root_node(), 0);
        }
    }
}
