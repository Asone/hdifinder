pub struct WalletConf{
    seed: [u8; 64],
    account: AccountHDPath,
    address: &'static str
}

pub struct ExecutionConf{
    start_index: usize,
    chunk_size: usize
}