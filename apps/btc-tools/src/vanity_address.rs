use std::fs::File;
use std::io;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use bitcoin::key::constants::SECRET_KEY_SIZE;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, Network, PrivateKey};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::cli::GenerateAddressArgs;

const RANDOM_BUF_SIZE: usize = 65536;

pub fn main(args: GenerateAddressArgs) -> anyhow::Result<()> {
    let mut address_output_file = match args.address_output_file {
        None => None,
        Some(f) => Some(File::create(f)?),
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
        println!("{} {}", private_key, address);
        if let Some(f) = &mut address_output_file {
            use io::Write;
            writeln!(f, "{}", address)?;
        }
    }

    Ok(())
}
