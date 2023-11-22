use std::io::{stdin, BufRead, BufReader};
use std::process::exit;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::{io, mem};

use anyhow::anyhow;
use bip38::{Decrypt, EncryptWif};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PrivateKey};
use rand::rngs::OsRng;
use rand::RngCore;
use rayon::prelude::*;

use crate::cli::{AddressType, GenerateAddressArgs, ValidateAddressArgs};
use crate::{input_password, public_to_address, truncate_sensitive, wif_to_public};

pub fn main(args: GenerateAddressArgs) -> anyhow::Result<()> {
    let bip38_password = if args.bip38 {
        Some(input_password()?)
    } else {
        None
    };

    let (sender, receiver) = channel();
    let sender_arc = Arc::new(Mutex::new(sender));
    for _ in 0..args.jobs.unwrap_or_else(num_cpus::get) {
        let substring = args.substring.clone();
        let sender = Arc::clone(&sender_arc);
        spawn(move || {
            let mut ec = [0_u8; 32];
            let k1 = Secp256k1::new();

            loop {
                // fill the first 28 bytes with random
                OsRng.fill_bytes(&mut ec[..(32 - 4)]);
                let mut address_string_buf =
                    [0_u8; 42 /* p2wpkh addresses should be 42 characters */];
                for inc_num in 0..u32::MAX {
                    // and then fill the last 4 bytes with an incremental number
                    // in order to reduce RNG calls
                    ec[(32 - 4)..]
                        .copy_from_slice(unsafe { mem::transmute::<_, &[u8; 4]>(&inc_num) });
                    let private_key =
                        PrivateKey::from_slice/* compressed */(&ec, Network::Bitcoin).unwrap();
                    let public_key = private_key.public_key(&k1);
                    let address = Address::p2wpkh(&public_key, Network::Bitcoin).unwrap();
                    unsafe {
                        use io::Write;
                        write!(&mut address_string_buf.as_mut_slice(), "{address}")
                            .unwrap_unchecked();
                    }

                    if address_string_buf
                        .windows(substring.len())
                        .any(|c| c == substring.as_bytes())
                    {
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

    fn worker(
        line: io::Result<String>,
        bip38_password: &Option<String>,
        args: &ValidateAddressArgs,
    ) -> anyhow::Result<()> {
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
        let derived_addr = public_to_address(&public_key, address_type)?.to_string();
        if derived_addr == addr {
            if args.decode {
                println!("{wif} {addr}");
            } else {
                println!("Check OK: {}", derived_addr);
            }
        } else {
            return Err(anyhow!(
                "Derived address mismatched! {} vs {}, from {}",
                addr,
                derived_addr,
                truncate_sensitive(pk)
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
        let result = worker(line, &bip38_password, &args);
        if let Err(e) = result {
            eprintln!("Error: {e}");
            exit(1);
        };
    });
    Ok(())
}
