use std::fs::{read_dir, DirEntry};
use std::io::ErrorKind;
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
    fn traversal_dir<F>(&self, callback: F) -> std::io::Result<()>
    where
        F: Fn(std::io::Result<&DirEntry>) + Clone;
}

fn recursive_traversal_dir<T, F>(path: &T, callback: F) -> std::io::Result<()>
where
    T: AsRef<Path> + ?Sized,
    F: Fn(std::io::Result<&DirEntry>) + Clone,
{
    let dir = read_dir(path)?;
    for entry in dir {
        let entry = entry;
        if let Ok(ref d) = entry {
            callback(Ok(d));
            let file_type = d.file_type();
            if let Ok(t) = file_type {
                if t.is_dir() {
                    recursive_traversal_dir(&d.path(), callback.clone());
                }
            } else if let Err(e) = file_type {
                callback(Err(e));
            }
        } else {
            callback(Ok(&entry.unwrap()));
        }
    }

    Ok(())
}

impl ForeachDir for Path {
    fn traversal_dir<F>(&self, callback: F) -> std::io::Result<()>
    where
        F: Fn(std::io::Result<&DirEntry>) + Clone,
    {
        recursive_traversal_dir(self, callback)
    }
}
