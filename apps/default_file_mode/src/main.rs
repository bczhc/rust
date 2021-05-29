use lib::fs::ForeachDir;
use lib::libc::ToCString;
use lib::utils::get_args_without_self_path;
use magic::{Cookie, CookieFlags, MagicError};
use std::env::current_dir;
use std::path::Path;

fn main() -> Result<(), String> {
    let mut m = Main::new();
    m.run()
}

struct Main {
    arguments: Arguments,
    file_magic_detector: FileMagicDetector,
}

struct Arguments {
    path: Option<String>,
}

impl Main {
    #[inline]
    fn new() -> Main {
        Self {
            arguments: Arguments { path: None },
            file_magic_detector: FileMagicDetector::new(),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        let file_mode = u32::from_str_radix("644", 8).unwrap();
        let dir_mode = u32::from_str_radix("755", 8).unwrap();
        let executable_mode = u32::from_str_radix("755", 8).unwrap();

        let args = get_args_without_self_path();
        if args.len() == 0 {
            let current_path = String::from(current_dir().unwrap().to_str().unwrap());
            self.arguments.path = Some(current_path);
        } else if args.len() == 1 {
            self.arguments.path = Some(args[0].clone())
        }

        let path = Path::new(self.arguments.path.as_ref().unwrap());

        path.traversal_dir(|entry| unsafe {
            let path_buf = entry.path();
            let path_str = path_buf.to_str().unwrap();
            let path_c_string = path_str.to_c_string();
            let path_c_str_ptr = path_c_string.as_ptr();
            let file_type = entry.file_type().unwrap();

            if file_type.is_file() {
                let is_executable = self.file_magic_detector.is_executable(&path_buf);
                if is_executable {
                    // executables
                    if libc::chmod(path_c_str_ptr, executable_mode) != 0 {
                        eprintln!("Failed to change executable mode: {}", path_str);
                    } else {
                        println!("Executable ok: {}", path_str);
                    }
                } else {
                    // normal files
                    if libc::chmod(path_c_str_ptr, file_mode) != 0 {
                        eprintln!("{}", format!("Failed to change file mode: {}", path_str));
                    }
                }
            } else if file_type.is_dir() {
                if libc::chmod(path_c_str_ptr, dir_mode as libc::mode_t) != 0 {
                    eprintln!(
                        "{}",
                        format!("Failed to change directory mode: {}", path_str)
                    );
                }
            }
        });

        Ok(())
    }
}

trait IsExecutable {
    fn is_executable<P>(&self, filename: P) -> bool
    where
        P: AsRef<Path>;
}

struct FileMagicDetector {
    cookie: Cookie,
}

impl FileMagicDetector {
    #[inline]
    fn new() -> Self {
        let cookie = Cookie::open(CookieFlags::default()).unwrap();
        let databases: &Vec<&str> = &vec![];
        cookie.load(databases).unwrap();
        Self { cookie }
    }

    fn file<P>(&self, filename: P) -> Result<String, MagicError>
    where
        P: AsRef<Path>,
    {
        self.cookie.file(filename)
    }
}

impl IsExecutable for FileMagicDetector {
    fn is_executable<P>(&self, filename: P) -> bool
    where
        P: AsRef<Path>,
    {
        let result = self.file(filename).unwrap();

        return if result.contains("shell script") {
            true
        } else if result.contains("ELF") {
            true
        } else {
            false
        };
    }
}
