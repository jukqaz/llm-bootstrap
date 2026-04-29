use std::env;
use std::process::{self, Command};

fn main() {
    let target = env::current_exe()
        .ok()
        .map(|path| path.with_file_name("stack-pilot"))
        .filter(|path| path.exists())
        .map(|path| path.into_os_string())
        .unwrap_or_else(|| "stack-pilot".into());

    let status = Command::new(target).args(env::args_os().skip(1)).status();

    match status {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("sp: failed to execute stack-pilot: {error}");
            process::exit(1);
        }
    }
}
