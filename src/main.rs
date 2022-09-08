extern crate bitcoin;
extern crate bitcoincore_rpc;
extern crate jsonrpc;
extern crate log;
extern crate serde_json;
extern crate ticker;
extern crate tokio;

use bitcoin::schnorr::TweakedPublicKey;
//use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::{ RpcApi};
//use std::time::Duration;
//use ticker::Ticker;

use warp::Filter;

use std::str::FromStr;

use bip0039::{Count, Mnemonic};
use bitcoin::secp256k1::ffi::types::AlignedType;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Address;
use bitcoin::PublicKey;
use bitcoin::XOnlyPublicKey;

pub struct WebServer;

impl WebServer {
    pub async fn run() {
        println!("Starting webserver...");

        // GET /hello/warp => 200 OK with body "Hello, warp!"
        let add_address = warp::path!("address").map(|| {
            let address = generate_address();
            warp::reply::json(&address)
        });

        warp::serve(add_address).bind(([127, 0, 0, 1], 3030)).await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // run webserver
    // let tr = tokio::runtime::Runtime::new().unwrap();
    // tr.spawn(async {
    //     WebServer::run().await;
    // });
    // println!("Everything working good!");

    // let ticker = Ticker::new(0.., Duration::from_secs(5));

    // for _ in ticker {
    //     println!("Work !!!");
    // }

    generate_taproot_address();

/*    
    let rpc = Client::new(
        "http://10.60.9.67:10003",
        Auth::UserPass("user".to_string(), "user".to_string()),
    )
    .unwrap();

    let ticker = Ticker::new(0.., Duration::from_secs(5));

    let block_count = rpc.get_block_count().unwrap();
    println!("block count: {}", block_count);

    let mut next_block = rpc.get_block_hash(block_count).unwrap();
    let mut latest_scanned_block: String = "".to_string();

    for _ in ticker {
        println!("We are on the block {}", next_block);

        let block = rpc.get_block_info(&next_block).unwrap();

        if latest_scanned_block != block.hash.to_string() {
            for tx in &block.tx {
                scan_transaction(&tx, &rpc);
                latest_scanned_block = block.hash.to_string();
            }
        }
        // Lets got to next transaction only after 2 confirmations ?
        if block.nextblockhash != None && block.confirmations >= 2 {
            println!("{}", serde_json::to_string_pretty(&block).unwrap());
            next_block = block.nextblockhash.unwrap();
        } else {
            println!("No more blocks");
        }
    }
*/ 
}

#[allow(dead_code)]
fn scan_transaction(tx: &bitcoincore_rpc::bitcoin::Txid, rpc: &bitcoincore_rpc::Client) {
    let raw_transaction_hex = rpc.get_raw_transaction_hex(&tx, None).unwrap();
    let args = [
        serde_json::to_value(&raw_transaction_hex).unwrap(),
        true.into(),
    ];

    let result: serde_json::Value = rpc.call("decoderawtransaction", &args).unwrap();
    let json_object = result.as_object().unwrap();

    for index in json_object["vout"].as_array().unwrap() {
        println!("Result : {:#?}", index["scriptPubKey"]["addresses"][0]);
    }
}

#[allow(dead_code)]
fn generate_address() -> String {
    println!("generate_address()");

    let network = bitcoin::Network::Bitcoin;
    println!("Network: {:?}", network);

    // Generates an English mnemonic with 12 words randomly
    let mnemonic = Mnemonic::generate(Count::Words12);
    // Gets the phrase
    let _phrase = mnemonic.phrase();

    println!("Phrase generated: {}", _phrase);

    // Generates the HD wallet seed from the mnemonic and the passphrase.
    let seed = mnemonic.to_seed("");

    // we need secp256k1 context for key derivation
    let mut buf: Vec<AlignedType> = Vec::new();
    buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
    let secp = Secp256k1::preallocated_new(buf.as_mut_slice()).unwrap();

    // calculate root key from seed
    let root = ExtendedPrivKey::new_master(network, &seed).unwrap();
    println!("Root key: {}", root);

    // derive child xpub
    let path = DerivationPath::from_str("m/84h/0h/0h").unwrap();
    let child = root.derive_priv(&secp, &path).unwrap();
    println!("Child at {}: {}", path, child);

    let xpub = ExtendedPubKey::from_priv(&secp, &child);
    println!("Public key at {}: {}", path, xpub);

    // generate first receiving address at m/0/0
    // manually creating indexes this time
    let zero = ChildNumber::from_normal_idx(0).unwrap();
    let public_key = xpub
        .derive_pub(&secp, &vec![zero, zero])
        .unwrap()
        .public_key;
    let address = Address::p2wpkh(&PublicKey::new(public_key), network).unwrap();
    println!("First receiving address: {}", address);

    return address.to_string();
}


fn generate_taproot_address() -> String {
    //https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki

    println!("-------------- generate_taproot_address() -----------------");

    let network = bitcoin::Network::Bitcoin;
    println!("Network: {:?}", network);

    // // Generates an English mnemonic with 12 words randomly
    // let mnemonic = Mnemonic::generate(Count::Words12);
    // // Gets the phrase
    // let _phrase = mnemonic.phrase();

    // println!("Phrase generated: {}", _phrase);

    let mnemonic_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase).unwrap();

    // Generates the HD wallet seed from the mnemonic and the passphrase.
    let seed = mnemonic.to_seed("");

    // calculate root key from seed
    let rootpriv = ExtendedPrivKey::new_master(network, &seed).unwrap();
    println!("rootpriv: {}", rootpriv);

    // we need secp256k1 context for key derivation
    let mut buf: Vec<AlignedType> = Vec::new();
    buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
    let secp = Secp256k1::preallocated_new(buf.as_mut_slice()).unwrap();


    let rootpub  = ExtendedPubKey::from_priv(&secp, &rootpriv);
    println!("rootpub: {}", rootpub);

    // Account 0, root = m/86'/0'/0'
    let path = DerivationPath::from_str("m/86h/0h/0h").unwrap();
    let account0_xprv = rootpriv.derive_priv(&secp, &path).unwrap();
    println!("Account0_xprv at {}: {}", path, account0_xprv);
    let account0_xpub = ExtendedPubKey::from_priv(&secp, &account0_xprv);
    println!("Account0_xpub at {}: {}", path, account0_xpub);

    // generate first receiving address at m/0/0
    // manually creating indexes this time
    let zero = ChildNumber::from_normal_idx(0).unwrap();
    let public_key = account0_xpub
        .derive_pub(&secp, &vec![zero, zero])
        .unwrap();
    println!("Public_key idx_0 : {}", public_key);


    let private_key = account0_xprv.derive_priv(&secp, &vec![zero, zero] ).unwrap();
    println!("private_key idx_0 : {}", private_key);

    let internal_key = XOnlyPublicKey::from(public_key.public_key);
    println!("internal_key idx_0 : {}", internal_key);
    let secp = Secp256k1::verification_only();
    let address = Address::p2tr(&secp, internal_key, None, network);

    println!("First receiving address: {}", address);


    return "".to_string();
}
