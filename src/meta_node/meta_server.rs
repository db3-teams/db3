//
//
// meta_server.rs
// Copyright (C) 2022 rtstore.io Author imrtstore <rtstore_dev@outlook.com>
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
use super::table::Table;
use crate::error::{RTStoreError, Result};
use crate::proto::rtstore_base_proto::{RtStoreNodeType, RtStoreTableDesc};
use crate::proto::rtstore_meta_proto::meta_server::Meta;
use crate::proto::rtstore_meta_proto::{
    CreateTableRequest, CreateTableResponse, PingRequest, PingResponse, RegisterNodeRequest,
    RegisterNodeResponse,
};
use crate::sdk::memory_node_sdk::MemoryNodeSDK;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
uselog!(debug, info, warn);

pub struct MetaServiceState {
    // key is the id of table
    tables: HashMap<String, Table>,
    memory_nodes: HashMap<String, Arc<MemoryNodeSDK>>,
    table_to_nodes: HashMap<String, HashMap<i32, String>>,
}

impl MetaServiceState {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            memory_nodes: HashMap::new(),
            table_to_nodes: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, table_desc: &RtStoreTableDesc) -> Result<()> {
        // join the names of table desc
        let id = Table::gen_id(table_desc)?;
        debug!("create table with id {}", id);
        match self.tables.get(&id) {
            Some(_) => Err(RTStoreError::TableNamesExistError { name: id }),
            _ => {
                let table = Table::new(table_desc)?;
                info!("create a new table with id {} successfully", id);
                self.tables.insert(id, table);
                Ok(())
            }
        }
    }

    pub fn add_memory_node(&mut self, endpoint: &str, node: &Arc<MemoryNodeSDK>) -> Result<()> {
        match self.memory_nodes.get(endpoint) {
            Some(_) => Err(RTStoreError::MemoryNodeExistError(endpoint.to_string())),
            _ => {
                self.memory_nodes.insert(endpoint.to_string(), node.clone());
                info!("add a new memory node {}", endpoint);
                Ok(())
            }
        }
    }
}

impl Default for MetaServiceState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MetaServiceImpl {
    state: Arc<Mutex<MetaServiceState>>,
}

impl Default for MetaServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaServiceImpl {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MetaServiceState::new())),
        }
    }
}

#[tonic::async_trait]
impl Meta for MetaServiceImpl {
    async fn create_table(
        &self,
        request: Request<CreateTableRequest>,
    ) -> std::result::Result<Response<CreateTableResponse>, Status> {
        let create_request = request.into_inner();
        let table_desc = match &create_request.table_desc {
            Some(t) => Ok(t),
            _ => Err(RTStoreError::MetaRpcCreateTableError {
                err: "input is invalid for empty table description".to_string(),
            }),
        }?;
        let mut local_state = self.state.lock().unwrap();
        local_state.create_table(table_desc)?;
        Ok(Response::new(CreateTableResponse {}))
    }

    async fn ping(
        &self,
        _request: Request<PingRequest>,
    ) -> std::result::Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse {}))
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> std::result::Result<Response<RegisterNodeResponse>, Status> {
        let register_node_req = request.into_inner();
        if RtStoreNodeType::KMemoryNode as i32 == register_node_req.node_type {
            match MemoryNodeSDK::connect(&register_node_req.endpoint).await {
                Ok(node_sdk) => {
                    let node_sdk_arc = Arc::new(node_sdk);
                    let mut local_state = self.state.lock().unwrap();
                    local_state.add_memory_node(&register_node_req.endpoint, &node_sdk_arc)?;
                    return Ok(Response::new(RegisterNodeResponse {}));
                }
                Err(e) => {
                    warn!(
                        "fail to connect memory node {} with err {}",
                        &register_node_req.endpoint, e
                    );
                    return Err(Status::internal(RTStoreError::NodeRPCError(
                        register_node_req.endpoint,
                    )));
                }
            }
        } else {
            return Err(Status::invalid_argument(
                "memory node is required".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::rtstore_base_proto::{RtStoreColumnDesc, RtStoreSchemaDesc};
    #[tokio::test]
    async fn test_ping() {
        let meta = MetaServiceImpl::new();
        let req = Request::new(PingRequest {});
        let result = meta.ping(req).await;
        if result.is_err() {
            panic!("should go error");
        }
    }

    #[tokio::test]
    async fn test_create_table_empty_desc() {
        let meta = MetaServiceImpl::new();
        let req = Request::new(CreateTableRequest { table_desc: None });
        let result = meta.create_table(req).await;
        if result.is_ok() {
            panic!("should go error");
        }
    }

    fn create_simple_table_desc(tname: &str) -> RtStoreTableDesc {
        let col1 = RtStoreColumnDesc {
            name: "col1".to_string(),
            ctype: 0,
            null_allowed: true,
        };
        let schema = RtStoreSchemaDesc {
            columns: vec![col1],
            version: 1,
        };
        RtStoreTableDesc {
            names: vec![tname.to_string()],
            schema: Some(schema),
            partition_desc: None,
        }
    }

    #[tokio::test]
    async fn test_create_table() {
        let table_desc = Some(create_simple_table_desc("test.t1"));
        let meta = MetaServiceImpl::new();
        let req = Request::new(CreateTableRequest { table_desc });
        let result = meta.create_table(req).await;
        assert!(result.is_ok());
    }
}
