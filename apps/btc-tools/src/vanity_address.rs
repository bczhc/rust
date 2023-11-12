use bip38::EncryptWif;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use bitcoin::key::constants::SECRET_KEY_SIZE;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, Network, PrivateKey};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::cli::GenerateAddressArgs;
use crate::input_password;

const RANDOM_BUF_SIZE: usize = 65536;

pub fn main(args: GenerateAddressArgs) -> anyhow::Result<()> {
    let bip38_password = if args.bip38 {
        Some(input_password()?)
    } else {
        None
    };

    let (sender, receiver) = channel();
    let sender_arc = Arc::new(Mutex::new(sender));
    for _ in 0..num_cpus::get() {
        let substring = args.substring.clone();
        let sender = Arc::clone(&sender_arc);
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
                        let guard = sender.lock().unwrap();
                        guard.send((private_key, address)).unwrap();
                    }
                }
            }
        });
    }

    for (private_key, address) in receiver {
        match bip38_password {
            None => {
                println!("{} {}", private_key, address);
            }
            Some(ref p) => {
                let encrypted_wif = private_key.to_wif().encrypt_wif(p)?;
                println!("{} {}", encrypted_wif, address);
            }
        }
    }

    Ok(())
}
