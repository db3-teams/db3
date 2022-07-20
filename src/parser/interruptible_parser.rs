//
//
// interruptible_parser.rs
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
use sqlparser::ast::Statement;
use sqlparser::dialect::Dialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::*;
use sqlparser::tokenizer::{Token, Tokenizer};

pub struct InterruptibleParser<'a> {
    // sql parser
    parser: Parser<'a>,
    // keyword to show what's the type of sql, Some types like
    // * Create , create table or create database
    // * Insert , insert data
    // * Select , query data from table or system variables
    // * Set, update system variables
    // * Show, show system variables
    // More go to https://github.com/sqlparser-rs/sqlparser-rs/blob/main/src/parser.rs#L156
    keyword: Option<Keyword>,
    sql: &'a str,
}

impl<'a> InterruptibleParser<'a> {

    pub fn new(dialect: &'a dyn Dialect, sql: &'a str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(dialect, sql);
        let tokens = tokenizer.tokenize()?;
        Ok(Self {
            parser: Parser::new(tokens, dialect),
            keyword: None,
            sql,
        })
    }

    pub fn next_keyword(&mut self) -> Result<Keyword> {
        match self.keyword {
            Some(k) => Ok(k),
            None => match self.parser.next_token() {
                Token::Word(w) => {
                    let cloned_kw = w.keyword;
                    self.keyword = Some(w.keyword);
                    Ok(cloned_kw)
                }
                _ => Result::Err(DB3Error::SQLParseError(format!(
                    "fail to parse {}",
                    self.sql
                ))),
            },
        }
    }

    pub fn prev_token(&mut self) {
        self.parser.prev_token();
    }

    pub fn parse_left(&mut self) -> Result<Statement> {
        let statement = self.parser.parse_statement()?;
        Ok(statement)
    }
}
