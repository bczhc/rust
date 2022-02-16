use binary_input::*;

fn main() -> Result<()> {
    #[cfg(unix)]
    {
        binary_input::unix::main()
    }
    #[cfg(windows)]
    {
        println!("Windows not supported :)");
        Ok(())
    }
}
