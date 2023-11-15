use std::io;
use std::io::{stdin, BufRead, BufReader};
use std::process::exit;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use anyhow::anyhow;
use bip38::{Decrypt, EncryptWif};
use bitcoin::key::constants::SECRET_KEY_SIZE;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, Network, PrivateKey};
use rand::rngs::OsRng;
use rand::RngCore;
use rayon::prelude::*;

use crate::cli::{AddressType, GenerateAddressArgs, ValidateAddressArgs};
use crate::{input_password, public_to_address, truncate_sensitive, wif_to_public};

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

pub fn validate_address(args: ValidateAddressArgs) -> anyhow::Result<()> {
    let bip38_password = if args.bip38 {
        Some(rpassword::prompt_password("Enter password:")?)
    } else {
        None
    };

    fn worker(line: io::Result<String>, bip38_password: &Option<String>) -> anyhow::Result<()> {
        let line = line?;
        let mut split = line.split_whitespace();
        let Some((pk, addr)): Option<(_, _)> = (try {
            let x1 = split.next()?;
            let x2 = split.next()?;
            (x1, x2)
        }) else {
            eprintln!("WARN: malformed line: {}", line);
            return Ok(());
        };

        let address_type = match addr {
            _ if addr.starts_with('1') => AddressType::P2pkh,
            _ if addr.starts_with("bc1q") => AddressType::P2wpkh,
            _ => {
                return Err(anyhow!("Cannot infer address type: {}", addr));
            }
        };

        let wif = if let Some(p) = &bip38_password {
            pk.decrypt_to_wif(p)?
        } else {
            String::from(pk)
        };
        let public_key = wif_to_public(&wif)?;
        let derived_addr = public_to_address(&public_key, address_type)?;
        if derived_addr == addr {
            println!("Check OK: {}", derived_addr);
        } else {
            return Err(anyhow!(
                "Derived address mismatched! {} vs {}, from {}",
                addr,
                derived_addr,
                truncate_sensitive(&wif)
            ));
        }
        Ok(())
    }

    if let Some(j) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(j)
            .build_global()
            .unwrap();
    }

    let reader = BufReader::new(stdin());
    reader.lines().par_bridge().for_each(|line| {
        let result = worker(line, &bip38_password);
        if let Err(e) = result {
            eprintln!("Error: {e}");
            exit(1);
        };
    });
    Ok(())
}
