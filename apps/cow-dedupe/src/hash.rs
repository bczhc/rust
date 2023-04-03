use std::hash::Hasher;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::{io, ptr, slice};

use digest::Digest;
use sha2::{Sha256, Sha512};

#[derive(Default)]
pub struct Sha256Digest {
    hasher: Sha256,
}

#[derive(Default)]
pub struct Sha512Digest {
    hasher: Sha512,
}

impl Sha256Digest {
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }
}

impl Sha512Digest {
    pub fn new() -> Self {
        Self {
            hasher: Sha512::new(),
        }
    }
}

pub struct B3Digest<const L: usize> {
    hasher: blake3::Hasher,
}

impl<const L: usize> B3Digest<L> {
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }
}

impl<const L: usize> Default for B3Digest<L> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait LengthDigest<const L: usize> {
    fn update(&mut self, data: &[u8]);

    fn finalize_len(self) -> [u8; L];
}

impl LengthDigest<32> for Sha256Digest {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn finalize_len(self) -> [u8; 32] {
        self.hasher.finalize().try_into().unwrap()
    }
}

impl LengthDigest<64> for Sha512Digest {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn finalize_len(self) -> [u8; 64] {
        self.hasher.finalize().try_into().unwrap()
    }
}

impl<const L: usize> LengthDigest<L> for B3Digest<L> {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn finalize_len(self) -> [u8; L] {
        let mut buf = [0_u8; L];
        let mut output_reader = self.hasher.finalize_xof();
        output_reader.fill(&mut buf);
        buf
    }
}

pub type B3_256Digest = B3Digest<32>;
pub type B3_512Digest = B3Digest<64>;

struct LengthDigestWrapper<D, const L: usize>(D)
where
    D: LengthDigest<L>;

impl<D, const L: usize> Write for LengthDigestWrapper<D, L>
where
    D: LengthDigest<L>,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// for `Hash` small vec, in bytes
const HASH_RESULT_MAX_LENGTH: usize = 64;

/// small vector on stack
pub struct Hash {
    container: [MaybeUninit<u8>; HASH_RESULT_MAX_LENGTH],
    size: usize,
}

impl Hash {
    pub fn new(data: &[u8]) -> Self {
        let mut container = [MaybeUninit::uninit(); HASH_RESULT_MAX_LENGTH];
        unsafe {
            ptr::copy(data.as_ptr(), container.as_mut_ptr() as *mut u8, data.len());
        }
        Self {
            container,
            size: data.len(),
        }
    }
}

impl std::hash::Hash for Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { slice::from_raw_parts(self.container.as_ptr() as *const u8, self.size) }
            .hash(state);
    }
}

pub trait DynHash {
    fn digest(&self, reader: &mut dyn Read, output: &mut [u8]);

    fn length(&self) -> usize;
}

struct DynSha256Hasher;
struct DynSha512Hasher;
struct DynB3_256Hasher;
struct DynB3_512Hasher;
struct DynB3_1024Hasher;

fn digest_reader<const L: usize>(
    mut reader: impl Read,
    digest: impl LengthDigest<L>,
    output: &mut [u8],
) {
    let mut wrapper = LengthDigestWrapper(digest);
    io::copy(&mut reader, &mut wrapper).unwrap();
    let output_len = output.len();
    if output_len < L {
        panic!("output array is too small to hold the digest output");
    }
    output[..L].copy_from_slice(&wrapper.0.finalize_len());
}

impl DynHash for DynSha256Hasher {
    fn digest(&self, reader: &mut dyn Read, output: &mut [u8]) {
        digest_reader(reader, Sha256Digest::new(), output);
    }

    fn length(&self) -> usize {
        256 / 8
    }
}

impl DynHash for DynSha512Hasher {
    fn digest(&self, reader: &mut dyn Read, output: &mut [u8]) {
        digest_reader(reader, Sha512Digest::new(), output);
    }

    fn length(&self) -> usize {
        512 / 8
    }
}

impl DynHash for DynB3_256Hasher {
    fn digest(&self, reader: &mut dyn Read, output: &mut [u8]) {
        digest_reader(reader, B3_256Digest::new(), output);
    }

    fn length(&self) -> usize {
        256 / 8
    }
}

impl DynHash for DynB3_512Hasher {
    fn digest(&self, reader: &mut dyn Read, output: &mut [u8]) {
        digest_reader(reader, B3_512Digest::new(), output);
    }

    fn length(&self) -> usize {
        512 / 8
    }
}

impl DynHash for DynB3_1024Hasher {
    fn digest(&self, reader: &mut dyn Read, output: &mut [u8]) {
        const L: usize = 1024 / 8;
        digest_reader(reader, B3Digest::<L>::new(), output);
    }

    fn length(&self) -> usize {
        1024 / 8
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::io::Cursor;

    use crate::hash::{
        B3Digest, DynB3_1024Hasher, DynB3_256Hasher, DynB3_512Hasher, DynHash, DynSha256Hasher,
        DynSha512Hasher, LengthDigest, LengthDigestWrapper, Sha256Digest, Sha512Digest,
    };

    fn digest<const L: usize>(digest: impl LengthDigest<L>, data: &[u8]) -> [u8; L] {
        let mut wrapper = LengthDigestWrapper(digest);
        let mut reader = Cursor::new(data);
        io::copy(&mut reader, &mut wrapper).unwrap();
        wrapper.0.finalize_len()
    }

    #[test]
    fn test() {
        let data = *b"hello, world";
        println!("{:?}", digest(Sha256Digest::new(), &data));
        println!("{:?}", digest(Sha512Digest::new(), &data));
        println!("{:?}", digest(B3Digest::<1>::new(), &data));
        println!("{:?}", digest(B3Digest::<32>::new(), &data));
        println!("{:?}", digest(B3Digest::<64>::new(), &data));
    }

    #[test]
    fn test2() {
        let data = b"hello, world";
        let hasher: [&dyn DynHash; 5] = [
            &DynB3_512Hasher,
            &DynB3_256Hasher,
            &DynSha512Hasher,
            &DynSha256Hasher,
            &DynB3_1024Hasher,
        ];
        let max_length = hasher.iter().map(|x| x.length()).max().unwrap();
        for h in hasher {
            let mut reader = Cursor::new(data);
            let mut output = vec![0_u8; max_length];
            h.digest(&mut reader, &mut output);
            println!("{:?}", &output[..h.length()]);
        }
    }
}
