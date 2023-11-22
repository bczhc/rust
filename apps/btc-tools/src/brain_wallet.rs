use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::anyhow;
use static_assertions::const_assert_eq;
use unicode_normalization::UnicodeNormalization;

use crate::cli::BrainWalletArgs;
use crate::{ec_to_wif, input_password, wif_to_address};

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

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use crate::brain_wallet::derive_secret;

    #[test]
    fn derive_iters() {
        // secret: blake3('hello, world')
        let secret = derive_secret("hello, world", 0);
        assert_eq!(
            secret,
            hex!("a1a55887535397bf461902491c8779188a5dd1f8c3951b3d9cf6ecba194e87b0")
        );

        // note: `<>` is an infix concatenation notation
        // secret: blake3(blake3('hello, world') <> hex('01000000 00000000'))
        let secret = derive_secret("hello, world", 1);
        assert_eq!(
            secret,
            hex!("aaa291d0b236b164e3d2299a8d66908349f06c64f3ca166d61d3669ff9e2abde")
        );

        // secret: blake3(
        //   blake3(blake3('hello, world') <> hex('01000000 00000000')) <> hex('02000000 00000000')
        // )
        let secret = derive_secret("hello, world", 2);
        assert_eq!(
            secret,
            hex!("79305b69f4c14959d8668762902e5c77f125a05a2916c6896e3cd0c8fc163e15")
        );
    }
}
