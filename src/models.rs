use hdpath::{AccountHDPath};

pub struct WalletConf {
    pub seed: [u8; 64],
    pub account: AccountHDPath,
}

#[derive(Clone)]
pub struct ExecutionConf {
    pub start: usize,
    pub end: usize,
}

pub struct SearchConfig {
    pub start: usize,
    pub end: usize,
    pub chunksize: usize,
    pub passphrase: String,
    pub address: String,
}