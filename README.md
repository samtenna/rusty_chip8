# Rusty CHIP-8

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![SDL2](https://img.shields.io/badge/SDL2-225699?style=for-the-badge&logo=sdl2&logoColor=white)
[![Tests](https://img.shields.io/github/actions/workflow/status/samtenna/rusty_chip8/ci.yml?style=for-the-badge&label=Tests)](https://github.com/samtenna/rusty_chip8/actions/workflows/ci.yml)

A fully featured CHIP8 emulator written in Rust. This project was built to explore CPU emulation, bitwise operations, and memory management in a low level systems language.

## Features

- **CPU Emulation**: Implements all standard CHIP8 opcodes, including stack management, index registers, and timer functions.
- **Robust Test Coverage**: Suite of unit tests rigorously validating the CPU lifecycle.
- **Display**: Uses SDL2 for screen emulation and input.
- **Timers**: Accurate emulation of both delay and sound timers.

## Getting Started

### Prerequisites

You will need the Rust toolchain and SDL2 development libraries installed on your machine.

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/samtenna/rusty_chip8.git
   cd rusty_chip8
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```

### Running a Game

Provide the path to a CHIP8 ROM file as a command line argument. The repository comes with a few test ROMs in the `roms/` directory.

```bash
cargo run --release -- roms/rom.ch8
```
## Controls

This emulator maps the CHIP8 keys to your keyboard as follows:

| CHIP-8 Keypad | Emulated Keypad |
|---------------|-----------------|
| `1` `2` `3` `C` | `1` `2` `3` `4` |
| `4` `5` `6` `D` | `Q` `W` `E` `R` |
| `7` `8` `9` `E` | `A` `S` `D` `F` |
| `A` `0` `B` `F` | `Z` `X` `C` `V` |

Press `ESC` at any time to exit the emulator.

## Testing

Run the test suite via Cargo:
```bash
cargo test
```
