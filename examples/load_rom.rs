use nes::rom::ROM;
use std::process;
use std::{env, fs::File};
use std::{io::BufWriter, path::Path};

fn main() {
    let mut args = env::args();
    let _ = args.next(); // executable name

    let filename = match (args.next(), args.next()) {
        (Some(filename), None) => filename,
        _ => {
            eprintln!("Usage: cargo run --example load_rom -- path/to/filename.nes");
            process::exit(1);
        }
    };

    let path = Path::new(&filename);
    if !path.exists() {
        eprintln!("The file provided did not exist.");
        process::exit(1);
    }

    match ROM::load_ines_file(path) {
        Ok(rom) => {
            eprintln!("ROM Header: {:#?}", rom.header);
            print_characters(&rom);
        }
        Err(nes::rom::ROMLoadError::Message(string)) => {
            eprintln!("Error loading ROM: {:?}", string);
        }
        Err(nes::rom::ROMLoadError::IoError(err)) => {
            eprintln!("Error loading ROM: {:?}", err);
        }
    };
}

fn print_characters(rom: &ROM) {
    let tile_width = 8;
    let pixels_per_tile = tile_width * tile_width;
    let tiles_per_row = 8;
    let bytes_per_tile = pixels_per_tile / 4;
    let tile_count = rom.character_rom.len() / bytes_per_tile;
    let tiles_per_col = {
        // Round up division.
        let mut v = tile_count / 8;
        if v % 8 != 0 {
            v += 1;
        };
        v
    };
    let png_width = tiles_per_row * tile_width;
    let png_height = tiles_per_col * tile_width;

    let path = Path::new(r"image.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, png_width as u32, png_height as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    let mut png_data = Vec::new();
    let mut tiles = Vec::new();

    eprintln!("bytes_per_tile {:?}", bytes_per_tile);
    // Convert the packed data into tiles.
    for chunk in rom.character_rom.chunks(bytes_per_tile) {
        let mut single_tile = Vec::with_capacity(pixels_per_tile);

        for channels in chunk.chunks(2) {
            // A row of pixels in a sprite are composed of two bytes. These
            // bytes are combined to create pixel values ranged 0-3.
            // channel a: 0b1111_0000
            // channel b: 0b1010_1010
            // result   :   3131_1010
            // values   :   3, 1, 3, 1, 1, 0, 1, 0
            let channel_a = channels[0];
            let channel_b = channels[1];

            for i in 0..8 {
                let i = 7 - i;
                single_tile.push((channel_a >> i & 0b1) | (channel_b >> i & 0b1 << 1));
            }
        }
        assert_eq!(single_tile.len(), pixels_per_tile);
        tiles.push(single_tile);
    }

    // Fill in the last tile if needed.
    // let pixels_needed = pixels_per_tile - single_tile.len();
    // for _ in 0..pixels_needed {
    //     single_tile.push(0);
    // }

    // Fill in the last row of tiles.
    let tiles_needed = tiles.len() % tiles_per_row;
    eprintln!("tiles_needed {:?}", tiles_needed);
    for _ in 0..tiles_needed {
        tiles.push(vec![0; pixels_per_tile]);
    }

    // Go through each row, and blit out the pixels in the correct order.
    for chunk in tiles.chunks(tiles_per_row) {
        for y in 0..tile_width {
            for tile in chunk.iter() {
                for x in 0..tile_width {
                    let pixel = tile.get(y * tile_width + x).unwrap();
                    png_data.push(match pixel {
                        0 => 0,
                        1 => 85,
                        2 => 170,
                        3 => 255,
                        _ => panic!("Unexpected pixel value"),
                    })
                }
            }
        }
    }

    eprintln!(
        "Pixels: {:?}",
        pixels_per_tile * tiles_per_col * tiles_per_row
    );
    writer.write_image_data(&png_data).unwrap();
}
