use colored::*;
use std::{env, process::exit};

fn parse_cli_args() -> String {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(filename) => filename.clone(),
        None => {
            eprintln!(
                "The nametable file viewer expects the first argument to be a path to a .nam file."
            );
            eprintln!(
                "cargo run --bin nam-viewer src/bin/cpu-visualizer/asm/add-with-carry.asm"
            );
            exit(1);
        }
    }
}

fn print_byte(byte: u8) {
    print!("{}", {
        match byte {
            0..=31 => format!("{:02x}", byte).magenta().dimmed(),
            32..=63 => format!("{:02x}", byte).magenta(),
            64..=95 => format!("{:02x}", byte).blue().dimmed(),
            96..=127 => format!("{:02x}", byte).blue(),
            128..=159 => format!("{:02x}", byte).cyan().dimmed(),
            160..=191 => format!("{:02x}", byte).cyan(),
            192..=223 => format!("{:02x}", byte).green().dimmed(),
            224..=255 => format!("{:02x}", byte).green(),
        }
    })
}

fn print_attribute(attribute: u8) {
    print!("{}", {
        match attribute {
            0 => format!("{:x}", attribute).magenta(),
            1 => format!("{:x}", attribute).blue(),
            2 => format!("{:x}", attribute).cyan(),
            3 => format!("{:x}", attribute).green(),
            _ => unreachable!("Unexpected attribute value"),
        }
    })
}

fn visualize_nametable(slice: &[u8]) {
    println!(
        "{}",
        "\n\n┣━━━━┫ Nametable ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫\n"
            .cyan()
    );

    println!("The nametable is the tilemap data that is stored in the PPU memory.");
    println!("It is used to draw the background on the NES. Each byte references");
    println!("a tile in the character data.");
    println!("");
    println!(
        "{}",
        "https://www.nesdev.org/wiki/PPU_nametables".underline()
    );
    println!("");
    println!("PPU Nametable 0 = $2000 - $20bf");
    println!("PPU Nametable 1 = $2400 - $24bf");
    println!("PPU Nametable 2 = $2800 - $28bf");
    println!("PPU Nametable 3 = $2c00 - $2cbf");
    println!("");

    println!(
        "{}",
        "     00  10  20  30  40  50  60  70  80  90  a0  b0  c0  d0  e0  f0    "
            .dimmed()
    );
    println!(
        "{}",
        "   ┌──────────────────────────────────────────────────────────────────┐"
            .dimmed()
    );
    for (i, window) in slice.chunks(32).enumerate() {
        print!("{}", format!("{:02x} │ ", i * 8).dimmed());

        for byte in window {
            print_byte(*byte);
        }
        println!("{}", " │".dimmed());
    }
    println!(
        "{}",
        "   └──────────────────────────────────────────────────────────────────┘"
            .dimmed()
    );
}

fn visualize_attributes(slice: &[u8]) {
    println!(
        "{}",
        "\n\n┣━━━━┫ Attributes ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫\n"
            .cyan()
    );

    println!("The attributes are stored in the nametable packed into the last");
    println!("64 bytes. The attribute picks palettes 0, 1, 2, or 3. Each attribute");
    println!("affects a 2x2 tile group in the nametable.");
    println!("");
    println!(
        "{}",
        "https://www.nesdev.org/wiki/PPU_attribute_tables".underline()
    );
    println!("");
    println!("PPU Nametable 0 = $20c0 - $20ff");
    println!("PPU Nametable 1 = $24c0 - $24ff");
    println!("PPU Nametable 2 = $28c0 - $28ff");
    println!("PPU Nametable 3 = $2cc0 - $2cff");
    println!("");
    println!("{}", "       Byte View".yellow());
    println!("{}", "     x0  x2  x4  x6".dimmed());
    println!("{}", "   ┌──────────────────┐".dimmed());
    for (i, window) in slice.chunks(8).enumerate() {
        let index = i as u8;
        print!("{}", format!("{:02x} │ ", index * 8 + 0xC0).dimmed());
        for byte in window {
            print_byte(*byte);
        }
        println!("{}", " │".dimmed());
    }
    println!("{}", "   └──────────────────┘".dimmed());
    println!("");
    println!("{}", "         Unpacked Attribute View".yellow());
    println!("{}", "      x0  x1  x2  x3  x4  x5  x6  x7".dimmed());
    println!("{}", "   ┌─────────────────────────────────┐".dimmed());
    for (i, window) in slice.chunks(8).enumerate() {
        // 7654 3210
        // |||| ||++- Color bits 3-2 for top left quadrant of this byte
        // |||| ++--- Color bits 3-2 for top right quadrant of this byte
        // ||++------ Color bits 3-2 for bottom left quadrant of this byte
        // ++-------- Color bits 3-2 for bottom right quadrant of this byte
        let br_mask = 0b1100_0000;
        let bl_mask = 0b0011_0000;
        let tr_mask = 0b0000_1100;
        let tl_mask = 0b0000_0011;

        print!("{}", format!("{:02x} │ ", (i as u8) * 8 + 0xC0).dimmed());
        for byte in window {
            let tr = (byte & tr_mask) >> 2;
            let tl = byte & tl_mask;
            print_attribute(tr);
            print!(" ");
            print_attribute(tl);
            print!(" ");
        }
        println!("{}", "│".dimmed());

        print!("{}", "   │ ".dimmed());
        for byte in window {
            let br = (byte & br_mask) >> 6;
            let bl = (byte & bl_mask) >> 4;
            print_attribute(br);
            print!(" ");
            print_attribute(bl);
            print!(" ");
        }
        println!("{}", "│".dimmed());
    }
    println!("{}", "   └─────────────────────────────────┘".dimmed());
}

/// Visualize nametable byte data.
fn main() {
    let filename = parse_cli_args();
    println!("Loading file {}", filename);

    let data = match std::fs::read(filename.clone()) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to read the file: {}", err);
            exit(1);
        }
    };

    if data.len() != 1024 {
        eprintln!(
            "Expected the nametable file to contain 1024 bytes. Instead {} were found.",
            data.len()
        );
        exit(1);
    }

    visualize_nametable(&data[0..960]);
    visualize_attributes(&data[960..]);
}
