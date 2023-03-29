// use sha2::{digest, Sha256};
// use std::io::Write;
//
// use sha2::digest::{Digest, Update, XofReader};
// use static_assertions::const_assert_eq;
//
// const_assert_eq!(blake3::OUT_LEN, 32);
//
// pub struct HashWriter<D>
// where
//     D: Digest,
// {
//     digest: D,
// }
//
// impl<D> HashWriter<D>
// where
//     D: Digest,
// {
//     pub fn new(digest: D) -> Self {
//         Self { digest }
//     }
//
//     pub fn finalize(self) {
//         self.digest.finalize();
//     }
// }
//
// impl<D> Write for HashWriter<D>
// where
//     D: Digest,
// {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         self.digest.update(buf);
//         Ok(buf.len())
//     }
//
//     fn flush(&mut self) -> std::io::Result<()> {
//         Ok(())
//     }
// }
//
// pub trait ExtendableDigest: digest::ExtendableOutput + Update {}
//
// impl<T> ExtendableDigest for T where T: digest::ExtendableOutput + Update {}
//
// pub struct ExtendableOutputHashWriter<D, const L: usize>
// where
//     D: ExtendableDigest,
// {
//     digest: D,
// }
//
// impl<D, const L: usize> ExtendableOutputHashWriter<D, L>
// where
//     D: ExtendableDigest,
// {
//     pub fn new(digest: D) -> Self {
//         Self { digest }
//     }
//
//     pub fn finalize(self) -> [u8; L] {
//         let mut reader = self.digest.finalize_xof();
//         let mut buf = [0_u8; L];
//         reader.read(&mut buf);
//         buf
//     }
// }
//
// impl<D, const L: usize> Write for ExtendableOutputHashWriter<D, L>
// where
//     D: ExtendableDigest,
// {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         self.digest.update(buf);
//         Ok(buf.len())
//     }
//
//     fn flush(&mut self) -> std::io::Result<()> {
//         Ok(())
//     }
// }
//
// pub type Sha256Hasher = HashWriter<sha2::Sha256>;
// pub type Sha512Hasher = HashWriter<sha2::Sha512>;
// pub type Blake3_256Hasher = HashWriter<blake3::Hasher>;
// pub type Blake3_512Hasher = ExtendableOutputHashWriter<blake3::Hasher, 64>;

use std::io::{Read, Write};

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
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::io::Cursor;

    use crate::hash::{B3Digest, LengthDigest, LengthDigestWrapper, Sha256Digest, Sha512Digest};

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
}
