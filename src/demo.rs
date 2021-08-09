use bczhc_lib::fs::ForeachDir;
use std::path::{Path, PathBuf};

fn main() {
    let path = Path::new("/home/bczhc/upload");

    let dir_name = path.file_name().unwrap().to_str().unwrap();
    println!("{}", dir_name);

    let abs_path = path.canonicalize().unwrap();

    path.traversal_dir(|d| {
        let d = d.unwrap();
        let d = d.path();
        let d = d.canonicalize().unwrap();
        let path_diff = d.strip_prefix(abs_path.to_str().unwrap());
        println!("{:?}", path_diff);
    })
    .unwrap();
}
