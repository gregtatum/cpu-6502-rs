# (WIP) NES Emulator in Rust

This is a side project to build an NES emulator. It's mostly me learning some low level fundamentals and having fun building some tooling with a debugger, and visualizer. I've built an ASM compiler, which takes in an .asm file, and can either output the binary machine code, or load it directly in memory and run it. The PPU isn't built yet, nor the sound, so the emulator doesn't really do anything yet.

![screenshot of debugger](screenshot.png)

## How to run

> The WIP PPU code requires SLD2 to be installed. On macOS `brew install sdl2 sdl2_tff`. You may need to adjust `.cargo/config.toml`.

The CPU debugger and visualizer can visualize the CPU running, and let you step through the code.

See all of the [asm examples](src/bin/cpu-visualizer/asm).

```
tree src/bin/cpu-visualizer/asm

src/bin/cpu-visualizer/asm
├── add-with-carry.asm
├── addressing-modes.asm
├── branching.asm
├── compare.asm
├── fibonacci-u16.asm
├── fibonacci-u8.asm
├── fill-zero-page.asm
├── labels-with-jumps.asm
├── logical-operators.asm
├── register-a-modes.asm
├── stack.asm
└── status-flags.asm
```

Run the `cpu-visualizer` binary with a path to the `.asm` file.

```
cargo run -p cpu-visualizer -- cpu-visualizer/src/asm/fill-zero-page.asm
```

The shortcuts for the program can be viewed by hitting `?` while using the program.

```
┌Help─────────────────────────────────────┐
│   n - next instruction                  │
│ 1-9 - next instructions exponentionally │
│ h/? - show help                         │
│   q - quit                              │
│   a - add a page of memory              │
│   r - remove a page of memory           │
└─────────────────────────────────────────┘
```

To view the logs of the visualizer append the following:

```
2> error.log; clear; cat error.log
```

## Simple Game

I also built a simple game visualizer which can run the snake game from the [Easy 6502 tutorial](https://skilldrick.github.io/easy6502/).

```
cargo run -p simple-game -- crates/simple-game/asm/snake.asm
```
