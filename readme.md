

# RustyBoy
A functioning **Game Boy** emulator.

Written in Rust, utilizing SDL2 for display rendering.


## Getting Started

### Installation and Compilation
1.  Follow the installation of Rust here: https://www.rust-lang.org/tools/install
2.  Clone or download the repository
3. Open into the RustyBoy folder and run the command `cargo build --release` to build the application
4. Run the emulator with your desired ROM with `cargo run --release ./path-to-rom`
> Make sure that the rom is placed in the correct path relative to the RustyBoy folder

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

| Test | RustyBoy |
|--|--|
| cgb sound | N/A* |
|cpu instrs|👍 |
|dmg sound |❌ |
|instr timing |👍|
|interrupt time |N/A*|
|mem timing|👍|
|mem timing 2|👍|

\* need GBC support, RustyBoy is DMG only.

## Planned Features

- Sound
- Saving and loading
- Game Boy Color support 

