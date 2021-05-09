pub mod pool;
pub mod util;
pub mod worker;

use std::path::PathBuf;

use hex_literal::hex;
use structopt::StructOpt;
use tracing::info;
use web3::types::Address;

use crate::pool::{Pool, PoolConfig};

#[derive(Debug, StructOpt)]
pub struct Command {
    /// The pool's address, ex: eth-us-west.flexpool.io:4444
    #[structopt(long)]
    pool: String,
    #[structopt(long)]
    worker: String,
    #[structopt(long)]
    wallet: Address,
    #[structopt(long)]
    web3: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().pretty().init();

    let opt = Command::from_args();
    info!(?opt);

    let pool = Pool::new(PoolConfig {
        upstream_url: opt.pool,
        wallet_address: opt.wallet,
        worker_name: opt.worker,
        web3: opt.web3,
    });

    loop {}
}
