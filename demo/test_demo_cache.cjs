const { chromium } = require('@playwright/test');
const fs = require('fs');
const path = require('path');

async function runTest() {
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
        storageState: { cookies: [], origins: [] }
    });
    const page = await context.newPage();

    let allPassed = true;

    try {
        console.log('=== NES Emulator Demo Test (Clean Cache) ===\n');

        // Test 1: Load page and verify canvas exists
        console.log('Test 1: Loading demo page with clean cache...');
        await page.goto('http://localhost:8081');
        await page.waitForSelector('#nes-screen');
        console.log('  PASS: Page loaded, canvas found\n');

        // Test 2: Check for console errors during load
        console.log('Test 2: Checking for console errors...');
        const consoleErrors = [];
        page.on('console', msg => {
            if (msg.type() === 'error') {
                consoleErrors.push(msg.text());
            }
        });
        await page.waitForTimeout(2000);

        if (consoleErrors.length === 0) {
            console.log('  PASS: No console errors\n');
        } else {
            console.log('  FAIL: Console errors found:');
            consoleErrors.forEach(e => console.log('    - ' + e));
            console.log();
            allPassed = false;
        }

        // Test 3: Test emulator creation using module import
        console.log('Test 3: Testing emulator creation...');
        const result = await page.evaluate(async () => {
            try {
                const mod = await import('./nes_wasm.js');
                const emulator = new mod.NesEmulator();
                return { success: true, frame: emulator.frame_count() };
            } catch (e) {
                return { success: false, error: e.message };
            }
        });

        if (result.success) {
            console.log('  PASS: Emulator created successfully (frame_count=' + result.frame + ')\n');
        } else {
            console.log('  FAIL: Emulator creation failed - ' + (result.error || 'unknown') + '\n');
            allPassed = false;
        }

        // Test 4: Test loading ROM
        console.log('Test 4: Loading test ROM...');
        const romResult = await page.evaluate(async () => {
            const mod = await import('./nes_wasm.js');
            try {
                const response = await fetch('working_test.nes');
                if (!response.ok) throw new Error('Failed to fetch ROM');
                const romData = new Uint8Array(await response.arrayBuffer());

                const emulator = new mod.NesEmulator();
                emulator.load_rom(romData);
                return { success: true, frame: emulator.frame_count() };
            } catch (e) {
                return { success: false, error: e.message };
            }
        });

        if (romResult.success) {
            console.log('  PASS: ROM loaded successfully\n');
        } else {
            console.log('  FAIL: ROM loading failed - ' + romResult.error + '\n');
            allPassed = false;
        }

        // Test 5: Test framebuffer access
        console.log('Test 5: Testing framebuffer access...');
        const fbResult = await page.evaluate(async () => {
            const mod = await import('./nes_wasm.js');
            try {
                const response = await fetch('working_test.nes');
                const romData = new Uint8Array(await response.arrayBuffer());
                const emulator = new mod.NesEmulator();
                emulator.load_rom(romData);

                const framebuffer = emulator.framebuffer_rgb;
                if (framebuffer && framebuffer.length >= 256 * 240 * 3) {
                    return { success: true, length: framebuffer.length };
                }
                return { success: false, error: 'Invalid framebuffer length' };
            } catch (e) {
                return { success: false, error: e.message };
            }
        });

        if (fbResult.success) {
            console.log('  PASS: Framebuffer accessed (length=' + fbResult.length + ')\n');
        } else {
            console.log('  FAIL: Framebuffer access failed - ' + fbResult.error + '\n');
            allPassed = false;
        }

        // Test 6: Run frames
        console.log('Test 6: Running frames...');
        const runResult = await page.evaluate(async () => {
            const mod = await import('./nes_wasm.js');
            try {
                const response = await fetch('working_test.nes');
                const romData = new Uint8Array(await response.arrayBuffer());
                const emulator = new mod.NesEmulator();
                emulator.load_rom(romData);

                emulator.run_frames(1);
                return { success: true, frame: emulator.frame_count() };
            } catch (e) {
                return { success: false, error: e.message };
            }
        });

        if (runResult.success) {
            console.log('  PASS: Frames run successfully (frame_count=' + runResult.frame + ')\n');
        } else {
            console.log('  FAIL: Frame run failed - ' + runResult.error + '\n');
            allPassed = false;
        }

        // Final result
        console.log('=== Test Summary ===');
        if (allPassed) {
            console.log('ALL TESTS PASSED!');
        } else {
            console.log('SOME TESTS FAILED');
        }

    } catch (error) {
        console.error('Test error:', error);
        allPassed = false;
    } finally {
        await browser.close();
    }

    process.exit(allPassed ? 0 : 1);
}

runTest();