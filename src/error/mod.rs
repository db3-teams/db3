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

use arrow::error::ArrowError;
use datafusion::error::DataFusionError;
use parquet::errors::ParquetError;
use s3::error::S3Error;
use sqlparser::parser::ParserError;
use sqlparser::tokenizer::TokenizerError;
use std::io::{Error as IoError, ErrorKind};
use thiserror::Error;
use tonic::Status;

/// The error system for rtstore
#[derive(Debug, Error)]
pub enum DB3Error {
    #[error("db with name {0} was not found")]
    DBNotFoundError(String),
    #[error("db with name {0} exist")]
    DBNameExistError(String),
    #[error("invalid input for a new database")]
    DBInvalidInput,
    #[error("table with name {tname} was not found")]
    TableNotFoundError { tname: String },
    #[error("invalid table names for {error}")]
    TableInvalidNamesError { error: String },
    #[error("table with name {name} exists")]
    TableNamesExistError { name: String },
    #[error("table type mismatch left {left} and right {right}")]
    TableTypeMismatchError { left: String, right: String },
    #[error("table to arrow for error : {0}")]
    TableArrowError(ArrowError),
    #[error("table {table_id} encounter encoding or decoding error {err}")]
    TableCodecError { table_id: String, err: String },
    #[error("bad url (0) for table")]
    TableBadUrl(String),
    #[error("file with {path} is invalid")]
    FSInvalidFileError { path: String },
    #[error("filesystem io error:{0}")]
    FSIoError(IoError),
    #[error("reach the end of file")]
    FSIoEofError,
    #[error("fail to read log for {0}")]
    FSLogReaderError(String),
    #[error("parquet error: {0}")]
    FSParquetError(ParquetError),
    #[error("fail to convert {0} to rtstore column type")]
    TableSchemaConvertError(i32),
    #[error("the schema for table {name} is invalid, please check the input")]
    TableSchemaInvalidError { name: String },
    #[error("create table error for {err}")]
    MetaRpcCreateTableError { err: String },
    #[error("the {name} of cell store config is invalid for {err}")]
    CellStoreInvalidConfigError { name: String, err: String },
    #[error("the cell exist in memory node with tid {tid} and pid {pid}")]
    CellStoreExistError { tid: String, pid: i32 },
    #[error("the cell has not been found in memory node with tid {tid} and pid {pid}")]
    CellStoreNotFoundError { tid: String, pid: i32 },
    #[error("aws-s3: {0}")]
    StoreS3Error(String),
    #[error("row codec error : {0}")]
    RowCodecError(bincode::Error),
    #[error("system busy for error : {0}")]
    BaseBusyError(String),
    #[error("memory node with endpoint {0} exists")]
    MemoryNodeExistError(String),
    #[error("not enough memory node")]
    MemoryNodeNotEnoughError,
    #[error("fail to connect to {0}")]
    NodeRPCError(String),
    #[error("invalid endpoint for node {name}")]
    NodeRPCInvalidEndpointError { name: String },
    #[error("fail to decode data from etcd for err {0}")]
    EtcdCodecError(String),
    #[error("meta store type mismatch")]
    MetaStoreTypeMisatchErr,
    #[error("the {name} for {key} has exist")]
    MetaStoreExistErr { name: String, key: String },
    #[error("encounter some etcd error {0}")]
    MetaStoreEtcdErr(etcd_client::Error),
    #[error("no meta store found")]
    MetaStoreNotFoundErr,
    #[error("fail to parse sql for error {0}")]
    SQLParseError(String),
    #[error("fail to create credentials for s3")]
    S3AuthError,
    #[error("sql execution error for e {0}")]
    SQLEngineError(DataFusionError),
    #[error("fail to encode or decode RecordBatch for {0}")]
    RecordBatchCodecError(String),
    #[error("fail to call rpc for {0}")]
    RPCStatusError(Status),
    #[error("fail to connect to rpc node for {0}")]
    RPCConnectError(tonic::transport::Error),
    #[error("{0}")]
    RPCInternalError(String),
    #[error("fail to parse json with error {0}")]
    JSONParseError(serde_json::Error),
    #[error("fail to parse string {1} with error {0}")]
    ParseNumberError(std::num::ParseIntError, String),
    #[error("fail call json rpc for error {0}")]
    JSONRpcError(jsonrpsee::core::Error),
    #[error("fail to transform sql to plan for error {0}")]
    SQLTransformError(String),
}

/// convert io error to rtstore error
impl From<IoError> for DB3Error {
    fn from(error: IoError) -> Self {
        DB3Error::FSIoError(error)
    }
}

impl From<jsonrpsee::core::Error> for DB3Error {
    fn from(error: jsonrpsee::core::Error) -> Self {
        DB3Error::JSONRpcError(error)
    }
}

impl From<ParquetError> for DB3Error {
    fn from(error: ParquetError) -> Self {
        DB3Error::FSParquetError(error)
    }
}

impl From<serde_json::Error> for DB3Error {
    fn from(err: serde_json::Error) -> Self {
        DB3Error::JSONParseError(err)
    }
}

impl From<DataFusionError> for DB3Error {
    fn from(err: DataFusionError) -> Self {
        DB3Error::SQLEngineError(err)
    }
}

impl From<ParserError> for DB3Error {
    fn from(error: ParserError) -> Self {
        match error {
            ParserError::TokenizerError(e) => DB3Error::SQLParseError(e),
            ParserError::ParserError(e) => DB3Error::SQLParseError(e),
        }
    }
}

impl From<TokenizerError> for DB3Error {
    fn from(err: TokenizerError) -> Self {
        DB3Error::SQLParseError(err.message)
    }
}

impl From<S3Error> for DB3Error {
    fn from(error: S3Error) -> Self {
        DB3Error::StoreS3Error(format!("s3 error {}", error))
    }
}

impl From<ArrowError> for DB3Error {
    fn from(error: ArrowError) -> Self {
        DB3Error::TableArrowError(error)
    }
}

impl From<etcd_client::Error> for DB3Error {
    fn from(error: etcd_client::Error) -> Self {
        DB3Error::MetaStoreEtcdErr(error)
    }
}

impl From<DB3Error> for IoError {
    fn from(error: DB3Error) -> Self {
        match error {
            DB3Error::FSIoError(e) => e,
            _ => IoError::from(ErrorKind::Other),
        }
    }
}

impl From<DB3Error> for String {
    fn from(error: DB3Error) -> Self {
        format!("{}", error)
    }
}

impl From<Status> for DB3Error {
    fn from(err: Status) -> Self {
        DB3Error::RPCStatusError(err)
    }
}

impl From<DB3Error> for Status {
    fn from(error: DB3Error) -> Self {
        match error {
            DB3Error::TableInvalidNamesError { .. }
            | DB3Error::TableSchemaConvertError { .. }
            | DB3Error::TableSchemaInvalidError { .. }
            | DB3Error::MetaRpcCreateTableError { .. } => Status::invalid_argument(error),
            DB3Error::TableNotFoundError { .. } | DB3Error::CellStoreNotFoundError { .. } => {
                Status::not_found(error)
            }
            DB3Error::TableNamesExistError { .. } | DB3Error::CellStoreExistError { .. } => {
                Status::already_exists(error)
            }
            _ => Status::internal(error),
        }
    }
}

/// The Result for rtstore
pub type Result<T> = std::result::Result<T, DB3Error>;
