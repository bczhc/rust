use std::io;
use std::io::Write;

use digest::consts::{U128, U64};
use digest::{ExtendableOutput, Output, Update};

pub struct HashWriter<H>(pub H)
where
    H: Update;

impl<H> Write for HashWriter<H>
where
    H: Update,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub type B3_256 = blake3::Hasher;

macro_rules! impl_b3_xof {
    ($name:ident, $size:ty) => {
        #[derive(Default)]
        pub struct $name {
            inner: blake3::Hasher,
        }

        impl $name {
            pub fn new() -> Self {
                Self::default()
            }
        }

        impl Update for $name {
            fn update(&mut self, data: &[u8]) {
                Update::update(&mut self.inner, data);
            }
        }

        impl digest::OutputSizeUser for $name {
            type OutputSize = $size;
        }

        impl digest::FixedOutput for $name {
            fn finalize_into(self, out: &mut Output<Self>) {
                self.inner.finalize_xof_into(out);
            }
        }
    };
}

impl_b3_xof!(B3_512, U64);
impl_b3_xof!(B3_1024, U128);
