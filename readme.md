

# RustyBoy
A functioning **Game Boy** emulator. Fairly accurate gameboy emulation with sound.

Written in **Rust**, utilizing **SDL2** for display rendering, **egui** for interface, and **rodio** for sound streaming.


## Getting Started

### Requirements 
1. Follow the instructions for the installation of Rust here: https://www.rust-lang.org/tools/install
2. Follow the instructions for the installation of CMake here: https://cmake.org/download/
3. Follow the instructions for the installation of SDL2 here: https://wiki.libsdl.org/SDL2/Installation

### Installation and Compilation
1.  Clone or download the repository
2. Open into the RustyBoy folder and run the command `cargo build --release` to build the application
3. Run the emulator with `cargo run --release`
4. Load the desired ROM using the "Load ROM" button

### Supported MBC Types
The emulator currently supports GameBoy games that use:
1. MBC0
2. MBC1
3. MBC3

### Keybinds
| Joypad | Keyboard |
|--------|----------|
| A      | K        |
| B      | L        |
| Select | I        |
| Start  | O        |
| Up     | W        |
| Down   | S        |
| Left   | A        |
| Right  | D        | 
## Testing

### Blargg Tests

| Test           | RustyBoy |
|----------------|----------|
| cgb sound      | N/A*     |
| cpu instrs     | 👍       |
| dmg sound      | 6/12     |
| instr timing   | 👍       |
| interrupt time | N/A*     |
| mem timing     | 👍       |
| mem timing 2   | 👍       |

\* need GBC support, RustyBoy is DMG only.

### Dmg-acid2 👍
![image](https://github.com/user-attachments/assets/6669e4a2-b36b-4f9f-be84-066817ae03d5)

## Screenshots

<img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/e61f9462-7724-4378-86bf-f3241bd108dd" />
<img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/301fb8e8-5fea-49ff-a86e-5e2d322f9af2" />


## Planned Features
- Saving and loading
- Game Boy Color support 

