mod load_cpu;
mod system;

use std::{env, error::Error};
use system::System;


fn parse_cli_args() -> String {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(filename) => filename.clone(),
        None => {
            eprintln!(
                "The CPU visualizer expects the first argument to be a path to a raw .asm file."
            );
            eprintln!(
                "cargo run --bin cpu-visualizer src/bin/cpu-visualizer/asm/add-with-carry.asm"
            );
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the CPU first, as this can exit the process.
    let filename = parse_cli_args();
    let (cpu, _) = load_cpu::load_cpu(&filename);
    let mut system = System::new(cpu);
    system.run_loop()?;

    Ok(())
}
