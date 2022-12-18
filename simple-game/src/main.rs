mod load_cpu;
mod system;

use std::{env, error::Error};
use system::{SimpleGame, System};

fn parse_cli_args() -> String {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(filename) => filename.clone(),
        None => {
            eprintln!(
                "The simple game expects the first argument to be a path to a raw .asm file."
            );
            eprintln!("cargo run -p simple-game crates/simple-game/asm/snake.asm");
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the CPU first, as this can exit the process.
    let filename = parse_cli_args();
    let (cpu, _) = load_cpu::load_cpu(&filename);
    let mut system = System::new();
    let mut game = SimpleGame::new(cpu, &mut system);
    game.run_loop()?;

    Ok(())
}
