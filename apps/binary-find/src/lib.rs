use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
/// Search binary in a file
pub struct Args {
    pub hex_string: String,
    /// Search in this file. If no file is given, stdin will be used
    pub file: Option<PathBuf>,
}

pub struct Windows<T, I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    buf: Vec<MaybeUninit<T>>,
    iter: I,
}

impl<T, I> Windows<T, I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    fn new(size: usize, iter: I) -> Self {
        Self {
            buf: (0..size).map(|_| MaybeUninit::uninit()).collect(),
            iter,
        }
    }

    pub fn iter(&mut self) -> WindowsIter<'_, T, I> {
        let window_size = self.buf.len();
        WindowsIter {
            inner: self,
            window_size,
            n: 0,
        }
    }
}

pub trait IterWindow<T, I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    fn windows(self, size: usize) -> Windows<T, I>;
}

impl<T, I> IterWindow<T, I> for I
where
    I: Iterator<Item = T>,
    T: Clone,
{
    fn windows(self, size: usize) -> Windows<T, I> {
        Windows::new(size, self)
    }
}

pub struct WindowsIter<'a, T, I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    inner: &'a mut Windows<T, I>,
    window_size: usize,
    n: usize,
}

impl<'a, T, I> Iterator for WindowsIter<'a, T, I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    // TODO: I haven't figured out the lifetime issue
    // type Item = &'a [T];
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 {
            // init
            for i in 0..self.window_size {
                let Some(item) = self.inner.iter.next() else {
                    return None;
                };
                self.inner.buf[i] = MaybeUninit::new(item);
            }
            self.n += 1;
            unsafe {
                return Some(slice_assume_init(&self.inner.buf).to_vec());
            }
        }

        self.inner.buf.rotate_left(1);

        let Some(item) = self.inner.iter.next() else {
            return None;
        };
        self.inner.buf[self.window_size - 1] = MaybeUninit::new(item);

        self.n += 1;

        unsafe { Some(slice_assume_init(&self.inner.buf).to_vec()) }
    }
}

unsafe fn slice_assume_init<T>(buf: &[MaybeUninit<T>]) -> &[T] {
    std::mem::transmute(buf)
}

#[cfg(test)]
mod test {
    use crate::IterWindow;

    #[test]
    fn iter_windows() {
        let array = [1, 2, 3, 4, 5];
        for size in 1..8 {
            let mut windows = array.into_iter().windows(size);
            let iter = windows.iter();
            assert_eq!(
                iter.collect::<Vec<_>>(),
                array.windows(size).collect::<Vec<_>>()
            );
        }
    }
}
