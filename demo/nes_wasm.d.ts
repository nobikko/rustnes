/* tslint:disable */
/* eslint-disable */

/**
 * NES Emulator wrapper for WASM
 */
export class NesEmulator {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get CPU cycles
     */
    cpu_cycles(): number;
    /**
     * Get current PPU dot
     */
    dot(): number;
    /**
     * Get the current frame count
     */
    frame_count(): number;
    /**
     * Load a ROM from bytes
     */
    load_rom(rom_data: Uint8Array): void;
    /**
     * Create a new NES emulator
     */
    constructor();
    /**
     * Reset the emulator
     */
    reset(): void;
    /**
     * Run for N frames
     */
    run_frames(frames: number): void;
    /**
     * Get current PPU scanline
     */
    scanline(): number;
    /**
     * Step the emulator once
     */
    step(): boolean;
    /**
     * Check if VBLANK is active
     */
    vblank(): boolean;
    /**
     * Get PPU framebuffer (256x240 RGB pixels)
     * Returns raw RGB data (76800 bytes: 256 * 240 * 3)
     */
    readonly framebuffer_rgb: Uint8Array;
}

export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_nesemulator_free: (a: number, b: number) => void;
    readonly nesemulator_cpu_cycles: (a: number) => number;
    readonly nesemulator_dot: (a: number) => number;
    readonly nesemulator_frame_count: (a: number) => number;
    readonly nesemulator_framebuffer_rgb: (a: number) => [number, number];
    readonly nesemulator_load_rom: (a: number, b: number, c: number) => [number, number];
    readonly nesemulator_new: () => number;
    readonly nesemulator_reset: (a: number) => void;
    readonly nesemulator_run_frames: (a: number, b: number) => void;
    readonly nesemulator_scanline: (a: number) => number;
    readonly nesemulator_step: (a: number) => number;
    readonly nesemulator_vblank: (a: number) => number;
    readonly version: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
