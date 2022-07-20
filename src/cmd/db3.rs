//
//
// db3.rs
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
//
//
use shadow_rs::shadow;
shadow!(build);

#[macro_use(uselog)]
extern crate uselog_rs;
use msql_srv::*;
use tokio::net::TcpListener;

use db3::compute_node::compute_node_impl::{ComputeNodeConfig, ComputeNodeImpl};
use db3::frontend_node::mysql::mysql_handler;
use db3::memory_node::memory_node_impl::{MemoryNodeConfig, MemoryNodeImpl};
use db3::meta_node::meta_server::{MetaConfig, MetaServiceImpl};
use db3::proto::db3_base_proto::{Db3Node, Db3NodeType};
use db3::proto::db3_compute_proto::compute_node_server::ComputeNodeServer;
use db3::proto::db3_memory_proto::memory_node_server::MemoryNodeServer;
use db3::proto::db3_meta_proto::meta_server::MetaServer;
use db3::sdk::{build_compute_node_sdk, build_memory_node_sdk, build_meta_node_sdk};
use db3::store::{
    build_meta_store, build_readonly_meta_store, meta_store::MetaStoreType,
    object_store::build_region,
};
use std::sync::Arc;
use tonic::transport::Server;
extern crate pretty_env_logger;
uselog!(debug, info, warn);
use clap::{Parser, Subcommand};

const ABOUT: &str = "
██████╗ ██████╗ ██████╗ 
██╔══██╗██╔══██╗╚════██╗
██║  ██║██████╔╝ █████╔╝
██║  ██║██╔══██╗ ╚═══██╗
██████╔╝██████╔╝██████╔╝
╚═════╝ ╚═════╝ ╚═════╝ 
@db3.network🚀🚀🚀";

#[derive(Debug, Parser)]
#[clap(name = "db3")]
#[clap(about = ABOUT, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start Meta Node Server
    #[clap(arg_required_else_help = true)]
    Meta {
        #[clap(required = true)]
        port: i32,
        #[clap(required = true)]
        etcd_cluster: String,
        #[clap(required = true)]
        etcd_root_path: String,
        #[clap(required = true)]
        ns: String,
        #[clap(required = true)]
        region: String,
    },

    /// Start Compute Node Server
    #[clap(arg_required_else_help = true)]
    ComputeNode {
        #[clap(required = true)]
        port: i32,
        #[clap(required = true)]
        etcd_cluster: String,
        #[clap(required = true)]
        etcd_root_path: String,
        #[clap(required = true)]
        ns: String,
        #[clap(required = true)]
        region: String,
    },

    /// Start Memory Node Server
    #[clap(arg_required_else_help = true)]
    MemoryNode {
        #[clap(required = true)]
        port: i32,
        #[clap(required = true)]
        binlog_root_dir: String,
        #[clap(required = true)]
        tmp_root_dir: String,
        #[clap(required = true)]
        etcd_cluster: String,
        #[clap(required = true)]
        etcd_root_path: String,
        #[clap(required = true)]
        ns: String,
    },
    /// Start Frontend Node Server
    #[clap(arg_required_else_help = true)]
    FrontendNode {
        #[clap(required = true)]
        port: i32,
        #[clap(required = true)]
        etcd_cluster: String,
        #[clap(required = true)]
        etcd_root_path: String,
        #[clap(required = true)]
        ns: String,
        #[clap(required = true)]
        var_config_path: String,
    },
    Version,
}

fn setup_log() {
    pretty_env_logger::init_timed();
}

async fn start_memory_node(memory_node: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    if let Commands::MemoryNode {
        port,
        binlog_root_dir,
        tmp_root_dir,
        etcd_cluster,
        etcd_root_path,
        ns,
    } = memory_node
    {
        if let Ok(meta_store) = build_readonly_meta_store(etcd_cluster, etcd_root_path).await {
            let bind_addr = format!("{}:{}", ns, port);
            let node = Db3Node {
                endpoint: format!("http://{}", bind_addr),
                node_type: Db3NodeType::KMemoryNode as i32,
                ns: ns.to_string(),
                port: *port,
            };
            let config = MemoryNodeConfig {
                binlog_root_dir: binlog_root_dir.to_string(),
                tmp_store_root_dir: tmp_root_dir.to_string(),
                etcd_cluster: etcd_cluster.to_string(),
                etcd_root_path: etcd_root_path.to_string(),
                node,
            };
            let memory_node_impl = MemoryNodeImpl::new(config, Arc::new(meta_store));
            if let Err(e) = memory_node_impl.init().await {
                warn!("fail to connect to meta {} with err {}", etcd_cluster, e);
                return Ok(());
            }
            info!("start memory node server on addr {}", bind_addr);
            Server::builder()
                .add_service(MemoryNodeServer::new(memory_node_impl))
                .serve(bind_addr.parse().unwrap())
                .await?;
        }
    }
    Ok(())
}

async fn start_compute_node(cmd: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    if let Commands::ComputeNode {
        port,
        etcd_cluster,
        etcd_root_path,
        ns,
        region,
    } = cmd
    {
        if let Ok(meta_store) = build_meta_store(
            etcd_cluster,
            etcd_root_path,
            MetaStoreType::MutableMetaStore,
        )
        .await
        {
            let addr = format!("{}:{}", ns, port);
            let node = Db3Node {
                endpoint: format!("http://{}", addr),
                node_type: Db3NodeType::KComputeNode as i32,
                ns: ns.to_string(),
                port: *port,
            };

            let r = build_region(&region);
            let config = ComputeNodeConfig {
                node,
                etcd_cluster: etcd_cluster.to_string(),
                etcd_root_path: etcd_root_path.to_string(),
            };

            let compute_node = ComputeNodeImpl::new(r, config, Arc::new(meta_store))?;
            compute_node.init().await.unwrap();
            info!("start compute node server on addr {}", addr);
            Server::builder()
                .add_service(ComputeNodeServer::new(compute_node))
                .serve(addr.parse().unwrap())
                .await?;
        }
    } else {
        warn!("fail start meta node for bad args");
    }
    Ok(())
}

async fn start_metaserver(cmd: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    if let Commands::Meta {
        port,
        etcd_cluster,
        etcd_root_path,
        ns,
        region,
    } = cmd
    {
        if let Ok(meta_store) = build_meta_store(
            etcd_cluster,
            etcd_root_path,
            MetaStoreType::MutableMetaStore,
        )
        .await
        {
            let addr = format!("{}:{}", ns, port);
            let node = Db3Node {
                endpoint: format!("http://{}", addr),
                node_type: Db3NodeType::KMetaNode as i32,
                ns: ns.to_string(),
                port: *port,
            };
            let r = build_region(&region);
            let config = MetaConfig {
                node,
                etcd_cluster: etcd_cluster.to_string(),
                etcd_root_path: etcd_root_path.to_string(),
                region: r,
            };
            let meta_service = MetaServiceImpl::new(config, Arc::new(meta_store));
            meta_service.init().await?;
            info!("start metaserver on addr {}", addr);
            Server::builder()
                .add_service(MetaServer::new(meta_service))
                .serve(addr.parse().unwrap())
                .await?;
        }
    } else {
        warn!("fail start meta node for bad args");
    }
    Ok(())
}

async fn start_frontend_server(cmd: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    if let Commands::FrontendNode {
        port,
        etcd_cluster,
        etcd_root_path,
        ns,
        var_config_path,
    } = cmd
    {
        info!("start frontend node ...");
        if let Ok(meta_store) = build_readonly_meta_store(etcd_cluster, etcd_root_path).await {
            if let (Ok(meta_node_sdk), Ok(memory_node_sdk), Ok(compute_node_sdk)) = (
                build_meta_node_sdk(&meta_store).await,
                build_memory_node_sdk(&meta_store).await,
                build_compute_node_sdk(&meta_store).await,
            ) {
                let addr = format!("{}:{}", ns, port);
                info!("start frontend node on addr {}", addr);
                let listener = TcpListener::bind(addr).await.unwrap();
                let arc_store = Arc::new(meta_store);
                if let Ok(handler) = mysql_handler::MySQLHandler::new(
                    meta_node_sdk,
                    memory_node_sdk,
                    compute_node_sdk,
                    arc_store,
                    var_config_path,
                ) {
                    assert!(handler.init().await.is_ok());
                    loop {
                        let (socket, _) = listener.accept().await.unwrap();
                        let new_handler = handler.clone();
                        tokio::spawn(async move {
                            let result = AsyncMysqlIntermediary::run_on(new_handler, socket).await;
                            match result {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!("fail to process incoming connection with e {}", e);
                                }
                            }
                        });
                    }
                } else {
                    info!("fail to new mysql handler");
                }
            }
        } else {
            info!("fail to start frontend node");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();
    let args = Cli::parse();
    if let Err(e) = match args.command {
        Commands::Meta { .. } => start_metaserver(&args.command).await,
        Commands::MemoryNode { .. } => start_memory_node(&args.command).await,
        Commands::FrontendNode { .. } => start_frontend_server(&args.command).await,
        Commands::ComputeNode { .. } => start_compute_node(&args.command).await,
        Commands::Version => {
            println!("{}", build::VERSION);
            Ok(())
        }
    } {
        warn!("fail to start node for err {}", e);
    }
    Ok(())
}
