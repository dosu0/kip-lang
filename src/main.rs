use kip::driver::main_loop;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    println!("kip v{}", version);

    if let Err(e) = main_loop() {
        eprintln!("[kip::driver] error: {}", e);
    };
}
