use crate::cli::GenerateVanityWalletArgs;
use bitcoin::key::constants::SECRET_KEY_SIZE;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, Network, PrivateKey};
use rand::rngs::OsRng;
use rand::RngCore;
use std::sync::{Condvar, Mutex};
use std::thread::spawn;

const RANDOM_BUF_SIZE: usize = 65536;

pub fn main(args: GenerateVanityWalletArgs) -> anyhow::Result<()> {
    for _ in 0..num_cpus::get() {
        let substring = args.substring.clone();
        spawn(move || {
            let k1 = Secp256k1::new();
            let mut random_buf = [0_u8; RANDOM_BUF_SIZE];
            loop {
                OsRng.fill_bytes(&mut random_buf);
                for sk_bytes in random_buf.windows(SECRET_KEY_SIZE) {
                    let secret_key = SecretKey::from_slice(sk_bytes).unwrap();
                    let private_key = PrivateKey::new(secret_key, Network::Bitcoin);
                    let public_key = private_key.public_key(&k1);
                    let address = Address::p2wpkh(&public_key, Network::Bitcoin)
                        .unwrap()
                        .to_string();
                    if address.contains(&substring) {
                        println!("{} {}", private_key, address);
                    }
                }
            }
        });
    }

    let mutex = Mutex::new(());
    let _g = Condvar::new().wait(mutex.lock().unwrap()).unwrap();

    Ok(())
}
