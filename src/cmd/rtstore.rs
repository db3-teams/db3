//
//
// rtstore.rs
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
//
//
#[macro_use(uselog)]
extern crate uselog_rs;
use rtstore::meta_node::meta_server::MetaServiceImpl;
use rtstore::proto::rtstore_meta_proto::meta_client::MetaClient;
use rtstore::proto::rtstore_meta_proto::meta_server::MetaServer;
use rtstore::proto::rtstore_meta_proto::PingRequest;
use tonic::transport::Server;
extern crate pretty_env_logger;
uselog!(debug, info, warn);
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "rtstore")]
#[clap(about = "a table store engine for realtime ingesting and analytics", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start MetaServer
    #[clap(arg_required_else_help = true)]
    Meta {
        #[clap(required = true)]
        port: i32,
    },

    /// Start Client Cli
    #[clap(arg_required_else_help = true)]
    Client {
        #[clap(required = true)]
        port: i32,
    },
}

fn setup_log() {
    pretty_env_logger::init_timed();
}

async fn start_metaserver(port: i32) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port).parse().unwrap();
    let meta_service = MetaServiceImpl::new();
    info!("start metaserver on port {}", port);
    Server::builder()
        .add_service(MetaServer::new(meta_service))
        .serve(addr)
        .await?;
    Ok(())
}

async fn start_client(port: i32) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{}", port);
    let mut client = MetaClient::connect(addr).await?;
    let request = tonic::Request::new(PingRequest {});
    let response = client.ping(request).await?;
    println!("{:?}", response);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();
    let args = Cli::parse();
    match args.command {
        Commands::Meta { port } => start_metaserver(port).await,
        Commands::Client { port } => start_client(port).await,
    }
}
