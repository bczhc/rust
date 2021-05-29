use std::fs::{read_dir, DirEntry};
use std::path::Path;

pub trait ForeachDir {
    /// Traversal a directory recursively
    ///
    /// # Examples
    /// ```no_run
    /// use std::path::Path;
    /// use lib::fs::ForeachDir;
    ///
    /// let path = Path::new("/tmp/zhc");
    /// path.traversal_dir(|entry| {
    ///     println!("{:?}", entry);
    /// });
    /// ```
    fn traversal_dir<F>(&self, callback: F)
    where
        F: Fn(&DirEntry) + Clone;
}

fn recursive_traversal_dir<T, F>(path: &T, callback: F)
where
    T: AsRef<Path> + ?Sized,
    F: Fn(&DirEntry) + Clone,
{
    let dir = read_dir(path).unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        callback(&entry);
        if file_type.is_dir() {
            recursive_traversal_dir(&entry.path(), callback.clone());
        }
    }
}

impl ForeachDir for Path {
    fn traversal_dir<F>(&self, callback: F)
    where
        F: Fn(&DirEntry) + Clone,
    {
        recursive_traversal_dir(self, callback);
    }
}
