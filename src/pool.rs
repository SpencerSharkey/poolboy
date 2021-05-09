use std::{collections::HashMap, path::PathBuf, time::Duration};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::{net::TcpStream, sync::mpsc};
use tracing::{error, info};
use web3::types::{self, Block, CallRequest};

use crate::{
    util::{calls::EthGetWorkResponse, WrappedCall},
    worker::PoolWorker,
};

#[derive(Clone)]
pub struct Pool {
    sender: mpsc::UnboundedSender<PoolMessage>,
}

impl Pool {
    pub fn new(config: PoolConfig) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let _inner = PoolInner::new(config, receiver);
        Self { sender }
    }
}

pub struct PoolConfig {
    pub upstream_url: String,
    pub wallet_address: types::Address,
    pub worker_name: String,
    pub web3: String,
}

struct PoolInner {
    config: PoolConfig,
    state: PoolState,
}

#[derive(Default)]
struct PoolState {
    workers: HashMap<String, PoolWorker>,
}

impl PoolInner {
    pub fn new(config: PoolConfig, receiver: mpsc::UnboundedReceiver<PoolMessage>) {
        let inner = Self {
            config,
            state: Default::default(),
        };
        tokio::spawn(inner.watchdog(receiver));
    }

    async fn watchdog(mut self, mut receiver: mpsc::UnboundedReceiver<PoolMessage>) {
        info!(pool = ?self.config.upstream_url, "pool watchdog started");

        loop {
            let res = self.run(&mut receiver).await;
            error!(?res, "reconnecting in 5s");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn run(
        &mut self,
        receiver: &mut mpsc::UnboundedReceiver<PoolMessage>,
    ) -> Result<(), anyhow::Error> {
        let transport = web3::transports::WebSocket::new(&self.config.web3).await?;
        let web3 = web3::Web3::new(transport);

        let mut tcp_stream = TcpStream::connect(self.config.upstream_url.clone()).await?;
        let (tcp_read, mut tcp_write) = tcp_stream.split();
        let stream = BufReader::new(tcp_read);
        let mut lines = stream.lines();

        info!(pool = ?self.config.upstream_url, "conneced!");

        let login_call = WrappedCall::new(
            1,
            "eth_submitLogin",
            vec![serde_json::to_value(self.config.wallet_address)?],
        )
        .serialize(self.config.worker_name.clone());

        info!("submitting login");
        tcp_write.write(&login_call.unwrap()).await?;

        let mut latest_work: Option<EthGetWorkResponse> = None;

        loop {
            enum Op {
                RecvPoolMessage(PoolMessage),
                RecvRpcMessage(String),
            }

            let op = tokio::select! {
                Some(msg) = receiver.recv() => {
                    Op::RecvPoolMessage(msg)
                }
                Ok(Some(msg)) = lines.next_line() => {
                    Op::RecvRpcMessage(msg)
                }
            };

            match op {
                Op::RecvPoolMessage(msg) => {
                    dbg!("pool msg");
                }
                Op::RecvRpcMessage(msg) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&msg) {
                        let id = value.get("id").unwrap().as_i64().unwrap();
                        let mut result = value.get("result");
                        if id == 1 {
                            info!(?result, "login success");
                            continue;
                        } else if id == 0 && result.is_some() {
                            let work =
                                EthGetWorkResponse::from_rpc(result.take().unwrap()).unwrap();

                            latest_work = Some(work)
                        }
                    }
                }
            }
        }
    }
}

pub enum PoolMessage {}
