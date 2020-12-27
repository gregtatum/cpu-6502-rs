# NES Emulator written in Rust (WIP)

This project is an attempt to create a NES emulator for fun. Right now I'm only attempting to emulate the MOS 6502 CPU. See [assets/notes.txt](assets/notes.txt) for my notes as I work on this.

## How to run

The CPU visualizer is the most interactive way to look at the work so far.

See all of the asm examples.

```
ls src/bin/cpu-visualizer/asm
```

Run the `cpu-visualizer` binary with a path to the `.asm` file.

```
cargo run --bin cpu-visualizer src/bin/cpu-visualizer/asm/fill-zero-page.asm
```

To view the logs of the visualizer append the following:

```
2> error.log; clear; cat error.log
```
