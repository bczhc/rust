use anyhow::anyhow;
use crate::cli::AddressType;
use bitcoin::key::Secp256k1;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{Address, Network, PrivateKey, PublicKey};

pub mod cli;
pub mod vanity_address;

pub type KeyError = bitcoin::key::Error;

pub fn public_to_address(
    k: &PublicKey,
    address_type: AddressType,
) -> Result<String, bitcoin::address::Error> {
    let address = match address_type {
        AddressType::P2pkh => Address::p2pkh(k, Network::Bitcoin).to_string(),
        AddressType::P2wpkh => Address::p2wpkh(k, Network::Bitcoin)?.to_string(),
    };
    Ok(address)
}

pub fn wif_to_public(wif: &str) -> Result<PublicKey, KeyError> {
    let private_key = PrivateKey::from_wif(wif)?;
    let k1 = Secp256k1::new();
    let public_key = private_key.public_key(&k1);
    Ok(public_key)
}

pub fn ec_to_wif(hex: &str, compressed: bool) -> anyhow::Result<String> {
    let secret_key = SecretKey::from_slice(&hex::decode(hex)?)?;
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
