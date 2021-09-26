use bip39::{Error, Mnemonic};
use bitcoin::{
    network::constants::Network,
    util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey},
    Address,
};
use clap::{App, Arg, ArgMatches};
use hdpath::{AccountHDPath, Purpose, StandardHDPath};
use rayon::prelude::*;
use secp256k1::Secp256k1;
use std::process::exit;
use std::{convert::TryInto, usize};

fn get_private_key(seed: [u8; 64], hd_path: &StandardHDPath) -> ExtendedPrivKey {
    let secp = Secp256k1::new();
    ExtendedPrivKey::new_master(Network::Bitcoin, &seed)
        // we convert HD Path to bitcoin lib format (DerivationPath)
        .and_then(|k| k.derive_priv(&secp, &DerivationPath::from(hd_path)))
        .unwrap()
}

fn get_public_key(private_key: ExtendedPrivKey) -> ExtendedPubKey {
    let secp = Secp256k1::new();
    ExtendedPubKey::from_private(&secp, &private_key)
}

struct WalletConf {
    seed: [u8; 64],
    account: AccountHDPath,
}

struct ExecutionConf {
    start: usize,
    end: usize,
}

struct SearchConfig {
    start: usize,
    end: usize,
    chunksize: usize,
    passphrase: String,
    address: String,
}

fn address_compute(pubkey: ExtendedPubKey) -> [(&'static str, String); 3] {
    let p2pkh: String = Address::p2pkh(&pubkey.public_key, Network::Bitcoin).to_string();

    let p2wpkh: String = Address::p2wpkh(&pubkey.public_key, Network::Bitcoin)
        .unwrap()
        .to_string();

    let p2shwpkh: String = Address::p2shwpkh(&pubkey.public_key, Network::Bitcoin)
        .unwrap()
        .to_string();

    [("p2pkh", p2pkh), ("p2wpkh", p2wpkh), ("p2shwpkh", p2shwpkh)]
}

fn executor(
    address: &str,
    wallet_config: &WalletConf,
    execution_config: ExecutionConf,
) -> Option<(usize, String, String)> {
    let start = execution_config.start;
    let end = execution_config.end;

    '_outer: for i in start..end {
        let hd_path = wallet_config
            .account
            .address_at(0, i.try_into().unwrap())
            .unwrap();
        let private_key = self::get_private_key(wallet_config.seed, &hd_path);
        let public_key = self::get_public_key(private_key);
        let addresses = self::address_compute(public_key);
        '_inner: for addr in addresses {
            if addr.1.as_str() == address {
                return Some((i, addr.1, addr.0.to_string()));
            }
        }
    }
    return None;
}

/**
 * Configuration builder
 *
 */
fn load_config(args: &ArgMatches) -> SearchConfig {
    let mut passphrase = String::new();
    if args.is_present("passphrase") {
        match args.value_of("passphrase") {
            Some(r) => {
                passphrase = r.to_string();
            }
            None => {
                passphrase = "".to_string();
            }
        }
    }

    let mut start: usize = 0;
    if args.is_present("start") {
        match args.value_of("start") {
            Some(r) => {
                start = r.parse::<usize>().unwrap_or_default();
            }
            None => {
                start = 0;
            }
        }
    }

    let mut end: usize = 0;
    if args.is_present("end") {
        match args.value_of("end") {
            Some(r) => {
                end = r.parse::<usize>().unwrap_or_else(|_| 10000000);
            }
            None => {
                end = 10000000;
            }
        }
    }

    let mut chunksize: usize = 0;
    if args.is_present("chunksize") {
        match args.value_of("chunksize") {
            Some(r) => {
                chunksize = r.parse::<usize>().unwrap_or_else(|_| 2500);
            }
            None => {
                chunksize = 2500;
            }
        }
    }

    let address: String;
    match args.value_of("address") {
        Some(r) => address = r.to_string(),
        None => {
            println!("No address provided. Exiting");
            exit(1);
        }
    }

    return SearchConfig {
        passphrase,
        start,
        end,
        chunksize,
        address,
    };
}

fn get_mnemonic(mnemonic: &str) -> Result<Mnemonic, Error> {
    return Mnemonic::parse_normalized(&mnemonic);
}

fn get_executor_config(config: &SearchConfig, iteration: usize) -> ExecutionConf {
    let mut conf = ExecutionConf {
        start: iteration * config.chunksize,
        end: 0,
    };

    let remaining = config.end - (iteration * config.chunksize);

    if remaining < config.chunksize {
        conf.end = conf.start + remaining;
    } else {
        conf.end = conf.start + config.chunksize
    }

    return conf;
}

fn main() {
    let matches = App::new("hdifinder")
        .version("1.0")
        .author("Nelson Herbin <nelson@herbin.info>")
        .about("A small utility to find if a key is part of an HD scheme")
        .arg(
            Arg::with_name("passphrase")
                .short("p")
                .long("passphrase")
                .help("The mnemonic passphrase")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("start")
                .short("s")
                .long("start")
                .help("The start index for key index search")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("end")
                .short("e")
                .long("end")
                .help("The end index for key index search")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("chunksize")
                .short("c")
                .long("chunksize")
                .help("The chuncksize for index search threads")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("mnemonic")
                .index(1)
                .help("A 24 words seed (without passphrase)"),
        )
        .arg(
            Arg::with_name("address")
                .index(2)
                .help("The address to be found"),
        )
        .after_help(
            "Examples:\n
            
        ",
        )
        .get_matches();

    let config: SearchConfig = self::load_config(&matches);

    let slices = (config.end - config.start) / config.chunksize;

    let mnemonic;

    match matches.value_of("mnemonic") {
        Some(r) => mnemonic = r,
        None => {
            println!("No mnemonic found. Exiting");
            exit(1);
        }
    }

    match self::get_mnemonic(mnemonic) {
        Ok(mnemonic) => {
            let seed = mnemonic.to_seed(&config.passphrase);
            let wallet_config: WalletConf = WalletConf {
                seed,
                account: AccountHDPath::new(Purpose::Pubkey, 0, 0),
            };

            (0..slices).into_par_iter().for_each(|slice| {
                let execution_config = self::get_executor_config(&config, slice);

                match self::executor(&config.address, &wallet_config, execution_config) {
                    Some(result) => {
                        println!(
                            "address {} found at index {}. address type: {}",
                            result.1, result.0, result.2
                        );
                        // std::io::stdout().lock().write_all(&result.0.to_string().into_bytes()).unwrap();
                        exit(0);
                    }
                    None => {}
                }
            });
        }
        Err(_) => {
            exit(1);
        }
    }
}
