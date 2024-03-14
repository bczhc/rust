#![allow(incomplete_features, const_evaluatable_unchecked)]
#![feature(generic_const_exprs)]

use std::io;
use std::io::{Read, Write};

use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use digest::{Digest, ExtendableOutput, FixedOutput, OutputSizeUser};

pub mod cli;

pub fn fixed_output_hash<H, R>(
    mut reader: R,
    iter_num: u64,
) -> io::Result<[u8; H::OutputSize::USIZE]>
where
    H: Digest + FixedOutput + OutputSizeUser,
    R: Read,
    H: Write,
    [(); H::OutputSize::USIZE]:,
    GenericArray<u8, H::OutputSize>: From<[u8; H::OutputSize::USIZE]>,
    GenericArray<u8, H::OutputSize>: Into<[u8; H::OutputSize::USIZE]>,
{
    // initial hashing
    let mut hash = GenericArray::<u8, H::OutputSize>::from([0_u8; H::OutputSize::USIZE]);
    let mut hasher = H::new();
    io::copy(&mut reader, &mut hasher)?;
    FixedOutput::finalize_into(hasher, &mut hash);

    // last iterations
    for _ in 1..iter_num {
        let mut hasher = H::new();
        Digest::update(&mut hasher, &*hash);
        FixedOutput::finalize_into(hasher, &mut hash);
    }

    Ok(hash.into())
}

pub fn xof_output_hash<H, R>(mut reader: R, length: usize, iter_num: u64) -> io::Result<Vec<u8>>
where
    H: Digest + ExtendableOutput,
    R: Read,
    H: Write,
{
    let mut hash = vec![0_u8; length];

    // initial hashing
    let mut hasher = H::new();
    io::copy(&mut reader, &mut hasher)?;
    hasher.finalize_xof_into(&mut hash);

    // last iterations
    for _ in 1..iter_num {
        let mut hasher = H::new();
        Digest::update(&mut hasher, &hash);
        hasher.finalize_xof_into(&mut hash);
    }

    Ok(hash)
}

pub fn fixed_output_hash_bytes<H>(data: &[u8], iter_num: u64) -> [u8; H::OutputSize::USIZE]
where
    H: Digest + FixedOutput + OutputSizeUser + Write,
    [(); H::OutputSize::USIZE]:,
    GenericArray<u8, H::OutputSize>: From<[u8; H::OutputSize::USIZE]>,
    GenericArray<u8, H::OutputSize>: Into<[u8; H::OutputSize::USIZE]>,
{
    fixed_output_hash::<H, _>(&mut &*data, iter_num).unwrap()
}

#[macro_export]
macro_rules! hash {
    ($t:ty, $data:expr, $iter:expr) => {
        $crate::fixed_output_hash_bytes::<$t>($data, $iter)
    };
    ($t:ty, $data:expr) => {
        $crate::hash!($t, $data, 1)
    };
}

pub fn sha256(data: &[u8]) -> [u8; 256 / 8] {
    hash!(sha2::Sha256, data)
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    #[test]
    fn test() {
        assert_eq!(
            hash!(ripemd::Ripemd160, b"hello"),
            hex!("108f07b8382412612c048d07d13f814118445acd")
        );
        assert_eq!(
            hash!(ripemd::Ripemd160, b"hello", 100),
            hex!("a1c6aa5b9ec15af182eb06dc63ddfd2153c947b8")
        );
    }
}
