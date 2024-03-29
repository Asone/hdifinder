
mod models;

use assert_cmd::Command;
use bip39::{Error, Mnemonic};
use bitcoin::{
    network::constants::Network,
    util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey},
    Address,
};
use clap::{App, Arg, ArgMatches};
use hdpath::{AccountHDPath, Purpose, StandardHDPath};
use models::{SearchConfig, ExecutionConf, WalletConf };
use rayon::prelude::*;
use secp256k1::Secp256k1;
use std::{process::exit, str::from_utf8};
use std::{convert::TryInto, usize};

/**
 * Retrieves a private key derived from a seed
 */
fn get_private_key(seed: [u8; 64], hd_path: &StandardHDPath) -> ExtendedPrivKey {
    let secp = Secp256k1::new();
    ExtendedPrivKey::new_master(Network::Bitcoin, &seed)
        // we convert HD Path to bitcoin lib format (DerivationPath)
        .and_then(|k| k.derive_priv(&secp, &DerivationPath::from(hd_path)))
        .unwrap()
}

#[test]
fn test_get_private_key(){
    let test_mnemonic_phrase: &str = "erupt quit sphere taxi air decade vote mixed life elevator mammal search empower rabbit barely indoor crush grid slide correct scatter deal tenant verb";
    let test_seed = self::get_mnemonic(test_mnemonic_phrase).unwrap().to_seed("");

    let hd_path= AccountHDPath::new(Purpose::Pubkey, 0, 0).address_at(0,5).unwrap();
    let private_key = get_private_key(test_seed, &hd_path);

    assert_eq!(private_key.private_key.to_string(),"L1TmQPcEkfoxHh6pJdbVASwiq18BpF3waAKf9LaannZWvLr4p2DF")
}

/**
 * Retrieves a public key derived from a private key
 */
fn get_public_key(private_key: ExtendedPrivKey) -> ExtendedPubKey {
    let secp = Secp256k1::new();
    ExtendedPubKey::from_private(&secp, &private_key)
}

#[test]
fn test_get_public_key(){
    let test_mnemonic_phrase: &str = "erupt quit sphere taxi air decade vote mixed life elevator mammal search empower rabbit barely indoor crush grid slide correct scatter deal tenant verb";
    let test_seed = self::get_mnemonic(test_mnemonic_phrase).unwrap().to_seed("");

    let hd_path= AccountHDPath::new(Purpose::Pubkey, 0, 0).address_at(0,5).unwrap();
    let private_key = get_private_key(test_seed, &hd_path);

    let public_key = get_public_key(private_key);
    assert_eq!("02016653fa405f3ecedb3dc88a378dabf7cd4c1c1acf1430515e854a630254cbbe",public_key.public_key.to_string());

}

/**
 * Computes the different addresses types of a public key
 */
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

#[test]
fn test_address_compute(){
let test_mnemonic_phrase: &str = "erupt quit sphere taxi air decade vote mixed life elevator mammal search empower rabbit barely indoor crush grid slide correct scatter deal tenant verb";
    let test_seed = self::get_mnemonic(test_mnemonic_phrase).unwrap().to_seed("");

    let hd_path= AccountHDPath::new(Purpose::Pubkey, 0, 0).address_at(0,5).unwrap();
    let private_key = get_private_key(test_seed, &hd_path);

    let public_key = get_public_key(private_key);
    
    let expected_results = [
        [
            "p2pkh",
            "14odE5c1eXuphR24fXMtzDfsXMLCmFTFgK"
        ],
        [
            "p2wpkh",
            "bc1q9xuuqjdz920rkcs0kvnmqh0t4anmgtk5u60h0y"
        ],
        [
            "p2shwpkh",
            "39gFyg2s6bp5AwwqtCrH7iNqRBh664LnZg"

        ]
    ];
    let addresses = address_compute(public_key);
    for (i,address) in IntoIterator::into_iter(addresses).enumerate(){
        assert_eq!(expected_results[i][0],address.0);
        assert_eq!(expected_results[i][1],address.1);
         
    }
}

/**
 * Executor for parallelized computing in order to find an address.
 */
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
        '_inner: for addr in &addresses {
            if addr.1.as_str() == address {
                return Some((i, addr.1.to_string(), addr.0.to_string()));
            }
        }
    }
    return None;
}

#[test]
fn test_executor(){

    let test_address = "14odE5c1eXuphR24fXMtzDfsXMLCmFTFgK";
    let test_mnemonic_phrase = "erupt quit sphere taxi air decade vote mixed life elevator mammal search empower rabbit barely indoor crush grid slide correct scatter deal tenant verb";
    let test_seed = self::get_mnemonic(test_mnemonic_phrase).unwrap().to_seed("");

    let test_wallet_config = WalletConf{
        seed: test_seed,
        account: AccountHDPath::new(Purpose::Pubkey, 0, 0)
    };

    let execution_config =  ExecutionConf{
        start: 10,
        end: 25
    };

    assert!(executor(test_address,&test_wallet_config,execution_config).is_none());

    let execution_config =  ExecutionConf{
        start: 0,
        end: 10
    };
    
    let result = executor(test_address,&test_wallet_config,execution_config.clone());

    assert!(&result.is_some());
    assert_eq!(result.clone().unwrap().0,5);
    assert_eq!(result.clone().unwrap().1,test_address);

}

/**
 * Configuration builder
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

    let mut end: usize = 10000000;
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

    let mut chunksize: usize = 2500;
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

/**
 * Builds a mnemonic object based on a mnemonic phrase
 */
fn get_mnemonic(mnemonic: &str) -> Result<Mnemonic, Error> {
    return Mnemonic::parse_normalized(&mnemonic);
}

/**
 * Executor configuration builder. Will be called before each
 * thread iteration.
 */
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

fn app() -> App<'static, 'static>{
    App::new("hdifinder")
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
}
fn main() {
    let matches = self::app().get_matches();

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

#[test]
fn test_main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic_test: &str = "erupt quit sphere taxi air decade vote mixed life elevator mammal search empower rabbit barely indoor crush grid slide correct scatter deal tenant verb";
    let address_test: &str = "15Wbvv7V9yWLCr3pxmPSFsAS3NSyQyqeA3";
    let mut cmd = Command::cargo_bin("hdifinder")?;
    cmd.arg(mnemonic_test);
    cmd.arg(address_test);
    let result = cmd.assert().success();
    let stdout = &result.get_output().stdout;
    let output = from_utf8(&stdout).unwrap();
    assert!(output.contains("15Wbvv7V9yWLCr3pxmPSFsAS3NSyQyqeA3 "));
    assert!(output.contains("15"));
    assert!(output.contains("p2pkh"));
    Ok(())
}