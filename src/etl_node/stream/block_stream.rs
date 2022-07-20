//
//
// block_stream.rs
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

use crate::base::strings;
use crate::error::{DB3Error, Result};
use futures::stream::Stream;
use futures::task::Context;
use futures::task::Poll;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::rpc_params;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::sleep;
uselog!(info, warn, debug);

pub struct BlockStreamConfig {
    // the start block number
    pub start_offset: u64,
    // the chain node url
    pub node_url: String,
    pub channel_buffer_size: usize,
}

pub struct BlockStream {
    // the config for block streaming
    config: BlockStreamConfig,
    // the current block number
    current_block_number: Arc<AtomicU64>,
    // the block node client
    client: HttpClient,
    // the latest block number
    latest_block_number: Arc<AtomicU64>,
    // the running state of stream
    running: Arc<AtomicBool>,
}

impl BlockStream {
    pub fn new(config: BlockStreamConfig) -> Result<Arc<BlockStream>> {
        let client = HttpClientBuilder::default().build(&config.node_url)?;
        let current_block_number = Arc::new(AtomicU64::new(config.start_offset));
        info!(
            "init block streaming with block offset {}",
            config.start_offset
        );
        let block_stream = Arc::new(Self {
            config,
            current_block_number,
            client,
            latest_block_number: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
        });
        Ok(block_stream)
    }

    pub fn start(block_stream: Arc<Self>) -> Result<Receiver<serde_json::Value>> {
        block_stream.running.store(true, Ordering::Relaxed);
        let (tx, rx): (Sender<serde_json::Value>, Receiver<serde_json::Value>) =
            mpsc::channel(block_stream.config.channel_buffer_size);
        Self::start_produce(tx, block_stream);
        Ok(rx)
    }

    pub fn close(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn start_produce(sender: Sender<serde_json::Value>, block_stream: Arc<Self>) {
        tokio::task::spawn(async move {
            loop {
                if !block_stream.running() {
                    break;
                }
                //TODO reduce call times from  remote node
                let result = block_stream.try_sync_latest_block_number().await;

                if let Err(e) = result {
                    warn!("fail to get latest block number for e {}", e);
                }
                let block = block_stream.try_get_a_block().await.inspect_err(|e| {
                    warn!("fail to get block for e {}", e);
                });
                if let Ok(Some(v)) = block {
                    if let Err(e) = sender.send(v).await {
                        //TODO retry and do not lost this block
                        warn!("fail to write block to channel");
                    }
                }
            }
        });
    }

    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let response: String = self
            .client
            .request("eth_blockNumber", rpc_params!())
            .await?;
        strings::hex_string_to_u64(&response)
    }

    pub async fn try_sync_latest_block_number(&self) -> Result<()> {
        if self.latest_block_number.load(Ordering::Relaxed)
            <= self.current_block_number.load(Ordering::Relaxed)
        {
            let number = self.get_latest_block_number().await?;
            debug!("sync latest block number {}", number);
            // the latest block is not ready to fetch
            if number <= self.latest_block_number.load(Ordering::Relaxed) {
                // sleep five seconds
                sleep(Duration::from_millis(1000 * 5)).await;
            }
            self.latest_block_number.store(number, Ordering::Relaxed);
        }
        Ok(())
    }

    /// try to get a block from chain node and return a json object
    /// return None if no more block to get
    pub async fn try_get_a_block(&self) -> Result<Option<serde_json::Value>> {
        let new_block_number = self.current_block_number.load(Ordering::Relaxed) + 1;
        if self.latest_block_number.load(Ordering::Relaxed) >= new_block_number {
            let value: serde_json::Value = self
                .client
                .request(
                    "eth_getBlockByNumber",
                    rpc_params!(format!("0x{:x}", new_block_number), false),
                )
                .await?;
            self.current_block_number
                .store(new_block_number, Ordering::Relaxed);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    //TODO add preview
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    #[tokio::test]
    async fn test_get_a_block() -> Result<()> {
        let config = BlockStreamConfig {
            start_offset: 15048327,
            node_url: format!(
                "{}",
                "https://mainnet.infura.io:443/v3/02b9d8695d254f0ebb23d6ff5bc7dbff"
            ),
            channel_buffer_size: 1,
        };
        let block_stream = BlockStream::new(config)?;
        let mut receiver = BlockStream::start(block_stream.clone())?;
        let block_opt = receiver.recv().await;
        if let Some(v) = block_opt {
            if let serde_json::Value::String(s) = &v["number"] {
                assert_eq!(strings::hex_string_to_u64(&s)?, 15048327 + 1);
            } else {
                panic!("should not be here");
            }
        } else {
            panic!("should not be here");
        }
        let block_opt = receiver.recv().await;
        if let Some(v) = block_opt {
            if let serde_json::Value::String(s) = &v["number"] {
                assert_eq!(strings::hex_string_to_u64(&s)?, 15048327 + 2);
            } else {
                panic!("should not be here");
            }
        } else {
            panic!("should not be here");
        }
        Ok(())
    }
}
