# LLM_RULES.md — NES Emulator (Rust → WASM) Strict Implementation Rules

## 0. Goal and Non-Goals

### Goal
Implement a correct NES emulator with a **Rust-first reference implementation** and a later **WASM (browser) build**.
Development order is strict:
1) CPU (6502 variant used in NES, i.e., 2A03 CPU core without decimal mode)
2) Memory bus + cartridges (mappers start with NROM)
3) PPU
4) APU (can be stubbed initially but must have deterministic timing hooks)
5) Integration + automated test ROM validation
6) WASM packaging + web UI (only after correctness gates pass)

### Non-Goals
- No speed hacks, no “close enough” behavior, no undocumented shortcuts.
- No UI work until core correctness gates pass.
- No “manual testing as primary validation”. Humans only check demos after automated tests pass.

---

## 1. Hard Safety & Legal Rules (MUST NOT VIOLATE)

1. **No copyrighted ROM distribution.**
   - Do NOT include commercial game ROMs, BIOS dumps, or copyrighted assets in the repo or tests.
   - Only use public-domain or permissively licensed test ROMs, or user-supplied ROM files loaded at runtime.

2. **No network calls for downloading ROMs or assets.**
   - The emulator must not fetch ROMs from the internet.
   - Tests must use locally provided test ROM binaries in a `tests/roms/` directory (not committed unless license permits).

3. **No shady “auto acquisition”.**
   - No instructions or code that helps users obtain copyrighted ROMs.

---

## 2. Project Structure Rules (Rust-first, WASM later)

### Mandatory Crate Layout
- `/crates/nes-core/` — pure Rust core, no WASM/web dependencies.
- `/crates/nes-cli/` — native CLI runner for testing and automation.
- `/crates/nes-wasm/` — WASM wrapper only (depends on `nes-core`).

### Dependency Rules
- `nes-core` must be `#![no_std]`-friendly as much as practical (allowed: `alloc` if needed).
- No `wasm-bindgen` in `nes-core`.
- Avoid heavy frameworks; keep dependencies minimal and stable.

### Forbidden
- Mixing web UI code into the core crate.
- Using JavaScript to implement emulator logic.
- Implementing “just enough for this ROM” special cases.

---

## 3. Correctness Gatekeeping (No Exceptions)

### Gate A — CPU-only Gates
CPU core is considered “OK” only if ALL pass:
- Deterministic step execution (instruction-by-instruction).
- Known CPU test suites pass (e.g., official/public 6502/NES CPU test ROMs).
- Cycle counting is correct (including page-cross penalties and branch timing rules).
- Decimal mode is disabled (NES 2A03 behavior).

### Gate B — Bus/Cartridge Gates
- NROM mapper implemented first.
- Correct CPU memory map mirroring and I/O mapping.
- Tests for read/write mapping and mirroring behavior.

### Gate C — PPU Gates
- PPU timing (scanlines/dots) is correct enough to pass PPU test ROMs.
- Correct register side effects (PPUSTATUS read, write toggle, etc.).
- Correct OAM/VRAM addressing and mirroring.

### Gate D — Integration Gates
- A known set of test ROMs must pass automatically in CI before “demo mode” is allowed.
- Human testing is allowed **only after** CI passes.

---

## 4. Testing Rules (Automated First)

### Test ROM Policy
- Use public test ROMs with known pass/fail criteria.
- Test results MUST be machine-checkable:
  - Prefer ROMs that write status codes to memory, or
  - ROMs that render a known “PASS” pattern to framebuffer, which can be hash-checked.

### No Manual Guessing
- Never accept “it looks right to me” as evidence.
- If a test ROM fails, fix the emulator. Do not patch the test.

### Required Test Harness Capabilities
- Headless mode: run N frames / cycles, capture:
  - CPU state snapshot(s)
  - Memory region dumps
  - Optional framebuffer dump
- Deterministic outputs: results must match bit-for-bit across runs.

### CI Requirement (If CI is set up)
- All tests must run in CI on every PR.
- No merging without green tests.

---

## 5. Emulator Architecture Rules

### 5.1 CPU (2A03 / 6502 variant)
- Implement as a pure state machine:
  - Registers: A, X, Y, P, SP, PC
  - Cycle counter / remaining cycles
- Use a structured decode table:
  - Prefer data-driven opcode table for legality/attributes (addr mode, base cycles).
- Do NOT implement undocumented opcodes unless explicitly planned; if unknown opcodes appear in tests, handle as per NES expectations (usually treat as NOP variants only if verified by test suite requirements).

### 5.2 Bus / Memory Map
- Separate “device” traits/modules:
  - RAM (2KB + mirroring)
  - PPU registers
  - APU/IO registers (stub initially but must not break bus timing)
  - Cartridge PRG/CHR
- No direct memory array hacks inside CPU. CPU reads/writes go through bus methods.

### 5.3 PPU
- Model PPU timing explicitly (scanline, dot).
- Framebuffer is produced deterministically.
- Keep PPU logic in core; only presentation is in CLI/WASM.

### 5.4 APU
- Allowed to stub initially, but must:
  - Implement register map reads/writes safely
  - Provide deterministic timing hooks so CPU/PPU sync remains correct

### 5.5 Timing / Sync
- Use a master clocking policy:
  - For every CPU cycle, step PPU 3 cycles (NES ratio).
- No “run PPU per frame and hope it matches”.

---

## 6. Code Style and Quality Rules

### MUST
- Rust 2021+ edition.
- `clippy` clean for core logic (allow very limited exceptions with justification comments).
- Small, reviewable commits/PRs.

### MUST NOT
- Copy-paste large code blobs from random sources.
- Introduce unsafe code unless absolutely required and reviewed; default is `#![forbid(unsafe_code)]` in `nes-core`.

### Documentation
- Every subsystem must have a short `README.md` explaining:
  - responsibilities
  - public APIs
  - timing model
  - known limitations (temporary)

---

## 7. WASM Rules (Only After Core Passes)

### Preconditions
- All core tests and integration tests pass on native builds.
- A minimal CLI demo works indicated by automated validation.

### WASM Implementation Constraints
- `nes-wasm` is a thin wrapper:
  - load ROM bytes from browser file input
  - call into `nes-core`
  - render framebuffer to `<canvas>`
  - audio optional later
- No behavior differences vs native.
- No “WASM-only hacks”.

---

## 8. LLM Interaction Rules (How You Must Work)

1. Always start by proposing a **minimal, testable increment**.
2. For each change, provide:
   - file list
   - rationale
   - what tests should pass after change
3. If a test fails, do NOT add hacks. Fix the root cause.
4. Do NOT invent specs. If uncertain, state uncertainty and implement in a way that is consistent with well-known NES behavior and test ROM expectations.

---

## 9. Definition of Done (for each milestone)

### Milestone: CPU
- CPU test ROM suite passes in headless mode.

### Milestone: Bus + NROM
- CPU tests still pass, plus bus mapping tests.

### Milestone: PPU
- PPU test ROM suite passes, and framebuffer hash tests are stable.

### Milestone: APU (basic)
- Deterministic stepping + no timing regressions; basic audio tests if included.

### Milestone: WASM
- Same ROMs that pass natively also pass in WASM build (validated by framebuffer hashes).

---

## 10. Absolute Forbidden List (Quick Reference)

- Shipping or downloading copyrighted ROMs
- Manual testing as primary validation
- WASM-first development
- Special-casing per-ROM behavior without a verified hardware/test rationale
- Implementing emulator logic in JavaScript
- Unsafe code in core (unless explicitly approved)

