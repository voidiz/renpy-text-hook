use arboard::Clipboard;
use clap::Parser;
use intruducer::intruduce;
use std::collections::HashSet;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::{
    fs,
    io::Read,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
};

const SOCKET_PATH: &'static str = "/tmp/renpy-text-hook";

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Ren'Py process ID
    #[clap(short, value_parser)]
    pid: u32,

    /// Shared library path
    #[clap(short, parse(from_os_str), default_value = "librenpy_text_hook.so")]
    lib: PathBuf,
}

/// Handle the next incoming connection
fn handle_stream(
    listener: &UnixListener,
    seen_lines: &mut HashSet<String>,
    clipboard: &mut Clipboard,
) -> anyhow::Result<()> {
    let (mut stream, _) = listener.accept()?;
    let mut line = String::new();
    stream.read_to_string(&mut line)?;

    if !seen_lines.contains(&line) {
        clipboard.set_text(line.to_string())?;
        println!("{}", &line);
        seen_lines.insert(line);
    }

    Ok(())
}

/// Create a socket for the shared library to connect and write to
fn create_socket() -> anyhow::Result<UnixListener> {
    let listener = UnixListener::bind(SOCKET_PATH)?;
    fs::set_permissions(SOCKET_PATH, Permissions::from_mode(0o777))?;
    Ok(listener)
}

fn main() {
    let args: Args = Args::parse();
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH).expect("Could not remove existing socket");
    }

    let listener = create_socket().expect("Could not create socket");

    match intruduce(args.pid, args.lib) {
        Ok(_) => println!("Injected shared object"),
        Err(e) => println!(
            "Hooking failed, make sure the PID and shared library path is correct.\nError: {:?}",
            e
        ),
    }

    let mut clipboard = Clipboard::new().expect("Could not access the clipboard");
    let mut seen_lines: HashSet<String> = HashSet::new();
    loop {
        if let Err(e) = handle_stream(&listener, &mut seen_lines, &mut clipboard) {
            println!("Error while reading from socket: {:?}", e);
        }
    }
}
