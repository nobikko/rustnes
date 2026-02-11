// Playwright test script
const { chromium } = require('playwright');

async function testEmulator() {
    const browser = await chromium.launch({
        headless: false,
        slowMo: 100
    });

    const context = await browser.newContext();
    const page = await context.newPage();

    // Navigate to the demo
    await page.goto('http://localhost:8000');

    console.log('Page loaded, waiting for canvas...');

    // Wait for canvas to be rendered
    const canvas = page.locator('#nes-screen');
    await canvas.waitFor({ state: 'attached', timeout: 10000 });

    console.log('Canvas found!');

    // Check the status elements
    const frameCount = page.locator('#frame-count');
    const scanline = page.locator('#scanline');
    const dot = page.locator('#dot');
    const vblank = page.locator('#vblank');
    const cpuCycles = page.locator('#cpu-cycles');

    console.log('Initial status:');
    console.log('  Frame:', await frameCount.innerText());
    console.log('  Scanline:', await scanline.innerText());
    console.log('  Dot:', await dot.innerText());
    console.log('  VBLANK:', await vblank.innerText());
    console.log('  CPU Cycles:', await cpuCycles.innerText());

    // Test the buttons
    console.log('\nTesting buttons...');

    // Check if buttons exist and are enabled
    const runBtn = page.locator('#run-btn');
    const resetBtn = page.locator('#reset-btn');

    console.log('Run button enabled:', await runBtn.isDisabled() === false);
    console.log('Reset button enabled:', await resetBtn.isDisabled() === false);

    // Test run button
    await runBtn.click();
    console.log('Clicked run button');

    // Wait and check multiple times
    for (let i = 0; i < 5; i++) {
        await page.waitForTimeout(500);
        const frame = await frameCount.innerText();
        const sc = await scanline.innerText();
        const db = await vblank.innerText();
        const cc = await cpuCycles.innerText();
        console.log(`  After ${(i+1) * 500}ms: Frame=${frame}, Scanline=${sc}, Dot=${sc}, VBLANK=${db}, Cycles=${cc}`);
    }

    // Test reset button
    await resetBtn.click();
    console.log('Clicked reset button');

    console.log('\nFinal status:');
    console.log('  Frame:', await frameCount.innerText());
    console.log('  Scanline:', await scanline.innerText());

    console.log('\nTest completed!');

    await browser.close();
}

testEmulator().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});