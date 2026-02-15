//! Rust NES Emulator
//!
//! A complete NES emulator implementation in Rust, including:
//! - 6502 CPU emulator
//! - PPU (Picture Processing Unit)
//! - APU (Audio Processing Unit)
//! - Controller input handling
//! - ROM loading and mapper support

pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod rom;
pub mod controller;
pub mod nes;
pub mod testing;

pub use cpu::{CPU, StatusFlags, Registers, IrqRequest, AddressingMode, Opcode, InstructionInfo};
pub use ppu::{PPU, STATUS_VBLANK, STATUS_SPRITE0HIT, STATUS_SPRITEOVERFLOW};
pub use apu::{APU, SquareChannel, TriangleChannel, NoiseChannel, DmcChannel, FrameCounter};
pub use rom::{Rom, RomHeader, Mapper, Mirroring, create_mapper, MapperInterface};
pub use controller::{StandardController, ZapperController, ControllerPorts, ControllerType, BUTTON_A, BUTTON_B, BUTTON_SELECT, BUTTON_START, BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT};
pub use nes::NES;