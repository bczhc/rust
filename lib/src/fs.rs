use std::ffi::OsString;
use std::fs::{read_dir, DirEntry, File};
use std::path::{Path, PathBuf};

pub trait ForeachDir {
    /// Traversal a directory recursively
    ///
    /// # Examples
    /// ```no_run
    /// use std::path::Path;
    /// use bczhc_lib::fs::ForeachDir;
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
                    recursive_traversal_dir(&d.path(), callback.clone())?;
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

/// create a file with unique filename for preventing from overwriting existing files
///
/// # Examples
/// ```no_run
/// use std::path::Path;
/// use bczhc_lib::fs::new_unique_file;
///
/// let path = Path::new("~/hello");
/// // now `path` doesn't exist; return without any changes
/// assert_eq!(new_unique_file(path), path);
///
/// let new_path = new_unique_file(path);
/// // the new path contains ".1" suffix
/// assert_eq!(new_path, Path::new("~/hello.1"))
/// ```
pub fn new_unique_file<P>(path: P) -> std::io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if !path.exists() {
        File::create(path)?;
        return Ok(path.into());
    }

    let mut counter = 1;
    loop {
        let mut string = OsString::from(path.as_os_str());
        string.push(format!(".{}", counter));
        let new_path = PathBuf::from(&string);
        if !new_path.exists() {
            File::create(&new_path)?;
            return Ok(new_path);
        }
        counter += 1;
    }
}
