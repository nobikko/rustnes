/**
 * NES Emulator wrapper for WASM
 */
export class NesEmulator {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NesEmulatorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_nesemulator_free(ptr, 0);
    }
    /**
     * Get CPU cycles
     * @returns {number}
     */
    cpu_cycles() {
        const ret = wasm.nesemulator_cpu_cycles(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get current PPU dot
     * @returns {number}
     */
    dot() {
        const ret = wasm.nesemulator_dot(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the current frame count
     * @returns {number}
     */
    frame_count() {
        const ret = wasm.nesemulator_frame_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get PPU framebuffer length
     * @returns {number}
     */
    framebuffer_len() {
        const ret = wasm.nesemulator_framebuffer_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get PPU framebuffer (256x240 RGB pixels)
     * Returns raw RGB data (76800 bytes: 256 * 240 * 3)
     * @returns {Uint8Array}
     */
    get framebuffer_rgb() {
        const ret = wasm.nesemulator_framebuffer_rgb(this.__wbg_ptr);
        return ret;
    }
    /**
     * Load a ROM from bytes
     * Returns true on success, false on failure
     * @param {Uint8Array} rom_data
     * @returns {boolean}
     */
    load_rom(rom_data) {
        const ptr0 = passArray8ToWasm0(rom_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nesemulator_load_rom(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Create a new NES emulator
     */
    constructor() {
        const ret = wasm.nesemulator_new();
        this.__wbg_ptr = ret >>> 0;
        NesEmulatorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Reset the emulator
     */
    reset() {
        wasm.nesemulator_reset(this.__wbg_ptr);
    }
    /**
     * Run for N frames
     * @param {number} frames
     */
    run_frames(frames) {
        wasm.nesemulator_run_frames(this.__wbg_ptr, frames);
    }
    /**
     * Get current PPU scanline
     * @returns {number}
     */
    scanline() {
        const ret = wasm.nesemulator_scanline(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Step the emulator once
     * @returns {boolean}
     */
    step() {
        const ret = wasm.nesemulator_step(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Check if VBLANK is active
     * @returns {boolean}
     */
    vblank() {
        const ret = wasm.nesemulator_vblank(this.__wbg_ptr);
        return ret !== 0;
    }
}
if (Symbol.dispose) NesEmulator.prototype[Symbol.dispose] = NesEmulator.prototype.free;

/**
 * @returns {string}
 */
export function version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_length_32ed9a279acd054c: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_3664a4ca8662b61d: function(arg0) {
            const ret = new ArrayBuffer(arg0 >>> 0);
            return ret;
        },
        __wbg_new_dd2b680c8bf6ae29: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_set_cc56eefd2dd91957: function(arg0, arg1, arg2) {
            arg0.set(getArrayU8FromWasm0(arg1, arg2));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./nes_wasm_bg.js": import0,
    };
}

const NesEmulatorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_nesemulator_free(ptr >>> 0, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('nes_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
