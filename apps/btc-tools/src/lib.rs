#![feature(try_blocks)]

use crate::cli::AddressType;
use anyhow::anyhow;
use bitcoin::key::Secp256k1;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{Address, Network, PrivateKey, PublicKey};
use hash_tools::{hash, sha256};

pub mod brain_wallet;
pub mod cli;
pub mod vanity_address;

pub type KeyError = bitcoin::key::Error;

pub fn public_to_address(
    k: &PublicKey,
    address_type: AddressType,
) -> Result<Address, bitcoin::address::Error> {
    let address = match address_type {
        AddressType::P2pkh => Address::p2pkh(k, Network::Bitcoin),
        AddressType::P2wpkh => Address::p2wpkh(k, Network::Bitcoin)?,
    };
    Ok(address)
}

pub fn wif_to_public(wif: &str) -> Result<PublicKey, KeyError> {
    let private_key = PrivateKey::from_wif(wif)?;
    let k1 = Secp256k1::new();
    let public_key = private_key.public_key(&k1);
    Ok(public_key)
}

pub fn wif_to_address(wif: &str, addr_type: AddressType) -> anyhow::Result<Address> {
    let puk = wif_to_public(wif)?;
    Ok(public_to_address(&puk, addr_type)?)
}

pub fn ec_hex_to_wif(hex: &str, compressed: bool) -> anyhow::Result<String> {
    ec_to_wif(&hex::decode(hex)?, compressed)
}

pub fn ec_to_wif(ec: &[u8], compressed: bool) -> anyhow::Result<String> {
    let secret_key = SecretKey::from_slice(ec)?;
    let wif = if compressed {
        PrivateKey::new(secret_key, Network::Bitcoin)
    } else {
        PrivateKey::new_uncompressed(secret_key, Network::Bitcoin)
    }
    .to_wif();
    Ok(wif)
}

pub fn input_password() -> anyhow::Result<String> {
    let password = rpassword::prompt_password("Enter password: ")?;
    if rpassword::prompt_password("Enter again to verify: ")? != password {
        return Err(anyhow!("Password mismatched"));
    }
    Ok(password)
}

pub fn truncate_sensitive(s: &str) -> String {
    let chars = s.chars().collect::<Vec<_>>();
    if chars.len() <= 20 {
        unreachable!("Unexpected message length")
    }
    String::from_iter(&chars[0..10]) + "..."
}

pub fn hash160(data: &[u8]) -> [u8; 20] {
    hash!(ripemd::Ripemd160, &sha256(data))
}
