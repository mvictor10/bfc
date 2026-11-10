#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod platform;

#[cfg(windows)]
fn main() {
    if let Err(error) = platform::run() {
        eprintln!("Falha ao iniciar Brazil Fight Combat: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    println!("Brazil Fight Combat Rust Engine: a interface grafica desta versao usa Win32/GDI e deve ser compilada no Windows.");
    println!("Para validar a logica portavel, execute: cargo test");
}
