use crate::cli::BrainWalletArgs;
use crate::{ec_hex_to_wif, ec_to_wif, input_password, wif_to_address, wif_to_public};
use anyhow::anyhow;
use bitcoin::{Address, Network, PrivateKey};
use static_assertions::const_assert_eq;
use std::fs::File;
use std::io::{BufRead, BufReader};
use unicode_normalization::UnicodeNormalization;

const_assert_eq!(blake3::OUT_LEN, 32);
const_assert_eq!(bitcoin::secp256k1::constants::SECRET_KEY_SIZE, 32);

pub fn main(args: BrainWalletArgs) -> anyhow::Result<()> {
    let passphrase = match args.pass_file {
        None => input_password()?,
        Some(f) => {
            let reader = BufReader::new(File::open(f)?);
            // use `lines` iterator to trim the trailing EOL mark
            reader
                .lines()
                .next()
                .ok_or_else(|| anyhow!("No line to read"))??
        }
    };

    let ec = derive_secret(&passphrase, args.iter_num);

    let wif = ec_to_wif(&ec, args.compressed)?;
    let address = wif_to_address(&wif, args.r#type.r#type)?;
    println!("{wif} {address}");

    Ok(())
}

fn derive_secret(passphrase: &str, iter_num: usize) -> [u8; 32] {
    let nfc_passphrase = passphrase.nfc().to_string();
    let initial_hash = blake3::hash(nfc_passphrase.as_bytes());

    let mut slot = [0_u8; 32 + 8];

    slot[..32].copy_from_slice(initial_hash.as_bytes());

    for i in 1..=iter_num {
        // increase the tail counter
        slot[32..].copy_from_slice(&i.to_le_bytes());
        // hash the whole, and fill back to the slot
        let hash = blake3::hash(&slot);
        slot[..32].copy_from_slice(hash.as_bytes());
    }

    // take the hash part as the secret key
    (&slot[..32]).try_into().unwrap()
}
