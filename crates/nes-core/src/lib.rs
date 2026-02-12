//! NES Core - Pure Rust NES emulator library
//!
//! This crate provides the core emulation logic for a Nintendo Entertainment System (NES).
//! It is designed to be `no_std`-friendly and contains no WASM or web dependencies.

#![forbid(unsafe_code)]

/// CPU module containing the 2A03 (6502 variant) implementation
pub mod cpu;
/// Memory bus and mapping
pub mod bus;
/// PPU (Picture Processing Unit) implementation
pub mod ppu;
/// APU (Audio Processing Unit) stub with timing hooks
pub mod apu;
/// Cartridge and mapper support
pub mod cartridge;
/// Integration module for complete NES system
pub mod system;

