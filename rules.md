# Rust NES Emulator - Rules and Guidelines

## Development Practices

### 1. Document All Code Changes in DEVELOPMENT_HISTORY.md
Every time code is modified, the changes should be documented in the `DEVELOPMENT_HISTORY.md` file with:
- Date of change
- File(s) modified
- Description of the fix or modification
- Any relevant context or issues encountered

### 2. Create rules.md for User Feedback
This file (`rules.md`) should document:
- User-specified requirements and instructions
- Feedback from the user about code behavior or design
- Specific constraints or preferences given by the user

## User Feedback and Instructions

### egui 0.28 API Changes
The user encountered significant breaking changes when upgrading to egui 0.28. The following API changes were required:

- **RawInput Structure**: Changed from having `keys` and `pointer` fields to using `raw.events` for all input events
- **Texture Handling**: `TextureHandle` now requires a texture options parameter and uses `ColorImage::from_rgba_unmultiplied()`
- **Image Creation**: `Image::from_texture()` now takes a reference to the texture handle
- **Viewport Management**: Window title updates now use `send_viewport_cmd(egui::ViewportCommand::Title(...))`

### Rust Naming Conventions
Opcode variants should follow Rust naming conventions:
- Use camelCase (e.g., `AslA`, `LsrA`, `RolA`, `RorA`) instead of SCREAMING_SNAKE_CASE

### Borrow Checker Patterns
When dealing with multiple mutable borrows of `self` in Rust:
- Store intermediate values in local variables before passing them to other methods
- Separate `effective_address()` calls from `load()` calls to avoid borrow conflicts

## Project Structure

```
src/
├── main.rs        # Desktop app using eframe/egui
├── lib.rs         # Library exports
├── cpu.rs         # 6502 CPU emulator
├── ppu.rs         # Picture Processing Unit
├── apu.rs         # Audio Processing Unit
├── rom.rs         # ROM loading and mapper support
├── controller.rs  # Controller input handling
└── nes.rs         # Main NES struct combining all components
```

## Build and Run

```bash
# Build the project
cargo build

# Run the emulator
cargo run
```

## Controls

- **A**: S key
- **B**: A Key
- **Select**: Space Key
- **Start**: Enter Key
- **Up**: Arrow Up
- **Down**: Arrow Down
- **Left**: Arrow Left
- **Right**: Arrow Right

## Notes

- The emulator uses a 256x240 pixel frame buffer
- Audio is configured at 44100 Hz sample rate (requires libasound2-dev on Linux)

## Audio Configuration

### Current Status
The ALSA audio dependencies (cpal, alsa-sys) have been temporarily removed from Cargo.toml to avoid system library requirements. The emulator runs without audio output.

### To Enable Audio
1. Install the ALSA development library:
```bash
sudo apt-get update && sudo apt-get install -y libasound2-dev
```

2. Add the following to Cargo.toml:
```toml
[dependencies]
cpal = "0.15"
alsa-sys = "0.3"
```

3. Rebuild: `cargo build`

## Recent Fixes (2026-02-14)

### Rust Compilation Errors

1. **Mapper enum explicit discriminants** - Rust doesn't allow explicit discriminants with non-unit variants. Changed:
```rust
pub enum Mapper {
    NoMapper = 0,           // ERROR: explicit discriminant
    Other(u8),              // non-unit variant
}
// Fix: Remove explicit values
pub enum Mapper {
    NoMapper,
    Other(u8),
}
```

2. **Missing std::io::Write import** - Added `use std::io::Write;` to rom.rs for `write_all()` method

3. **Bool bit shift error** - `bool << 4` is invalid. Changed to:
```rust
if self.light_sensor { 1 << 4 } else { 0 }
```

4. **Debug trait on trait objects** - Cannot derive Debug on structs containing trait objects like `Box<dyn MapperInterface>` or `Box<dyn Fn()>`

5. **Missing opcode variants** - Added: JMP, DEY, INY, DEX, INX, DEC, INC, ANE, TAS, LXA, AslA, LsrA, RolA, RorA

6. **IrqRequest match exhaustiveness** - Must handle all variants including `IrqRequest::None`

7. **Type casting in comparisons** - Casts in comparisons need parentheses to avoid generic argument interpretation

8. **u16 vs usize indexing** - Slice indices must be `usize`, not `u16`