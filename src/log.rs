use std::{
    fs::{self, OpenOptions},
    io::Write,
};

pub fn init_log() {
    match fs::File::create("log.txt") {
        Ok(_) => {}
        // Potential errors are that the file already exists, so just ignore it.
        Err(_) => {}
    };
}

/// Log for when stdout is taken.
pub fn log(text: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")
        .expect("Unable to open file");

    file.write_all(text.as_bytes())
        .expect("Failed to write file");

    file.write_all("\n".as_bytes())
        .expect("Failed to write file");
}
