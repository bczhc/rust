use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PrivateKey};
use hex::ToHex;

use crate::cli::GetAddressArgs;

pub fn main(args: GetAddressArgs) -> anyhow::Result<()> {
    let k1 = Secp256k1::new();

    let private_key = PrivateKey::from_wif(&args.private_key)?;
    let public_key = private_key.public_key(&k1);
    println!(
        "Secret: {}",
        private_key.inner.secret_bytes().encode_hex::<String>()
    );
    println!("p2pkh: {}", Address::p2pkh(&public_key, Network::Bitcoin));
    println!(
        "p2wpkh: {}",
        Address::p2wpkh(&public_key, Network::Bitcoin)?
    );

    Ok(())
}
