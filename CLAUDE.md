# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NES Emulator with Rust-first reference implementation followed by WASM (browser) build.

### Development Order (Strict)
1. CPU (6502 variant - 2A03 core, no decimal mode)
2. Memory bus + cartridges (NROM mapper first)
3. PPU
4. APU (stub initially with deterministic timing hooks)
5. Integration + automated test ROM validation
6. WASM packaging + web UI

### Crate Layout
- `/crates/nes-core/` — pure Rust core, no WASM/web dependencies
- `/crates/nes-cli/` — native CLI runner for testing
- `/crates/nes-wasm/` — WASM wrapper only (depends on `nes-core`)

### Hard Rules
- No copyrighted ROM distribution (use only public-domain test ROMs)
- No network calls for ROM downloads
- No "close enough" behavior - must pass automated test suites
- No WASM code in `nes-core` (use `wasm-bindgen` only in `nes-wasm`)

### Gates (Must Pass in Order)
- **Gate A**: CPU test ROMs pass
- **Gate B**: Bus mapping + NROM tests pass
- **Gate C**: PPU test ROMs pass
- **Gate D**: Integration tests pass before demo mode

### Code Style
- Rust 2021+ edition
- `clippy` clean for core logic
- Small, reviewable commits
- No unsafe code in `nes-core` unless explicitly approved

### Git Workflow
- Repository: `github.com/nobikko/rustnes`
- Push changes to GitHub after any code or documentation modifications
- Use `git push` to sync with origin