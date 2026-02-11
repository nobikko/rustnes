const { chromium } = require('@playwright/test');
const http = require('http');
const fs = require('fs');
const path = require('path');

async function startStaticServer(port) {
    const server = http.createServer((req, res) => {
        let filePath = path.join(__dirname, req.url === '/' ? 'index.html' : req.url);

        const ext = path.extname(filePath);
        const contentTypes = {
            '.html': 'text/html',
            '.js': 'application/javascript',
            '.wasm': 'application/wasm',
            '.css': 'text/css',
            '.json': 'application/json',
            '.wasm.d.ts': 'text/plain'
        };

        const contentType = contentTypes[ext] || 'application/octet-stream';

        fs.readFile(filePath, (err, data) => {
            if (err) {
                res.writeHead(404);
                res.end('Not found');
                return;
            }
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(data);
        });
    });

    return new Promise((resolve) => {
        server.listen(port, '127.0.0.1', () => {
            console.log(`Server running at http://127.0.0.1:${port}`);
            resolve(server);
        });
    });
}

async function runTest() {
    const port = 8765;
    const server = await startStaticServer(port);

    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
        console.log('Navigating to http://127.0.0.1:' + port);

        // Set up console handler before navigation
        const consoleLogs = [];
        page.on('console', (msg) => {
            const text = msg.text();
            if (msg.type() === 'error') {
                console.log('Console ERROR:', text);
            } else {
                console.log('Console:', text);
                consoleLogs.push(text);
            }
        });

        await page.goto(`http://127.0.0.1:${port}`);

        // Wait for page to load
        await page.waitForSelector('#nes-screen');
        console.log('Page loaded successfully');

        // Check if canvas exists
        const canvas = await page.$('#nes-screen');
        if (canvas) {
            console.log('Canvas element found');
        }

        // Wait for emulator to be ready (check window.emulator exists)
        console.log('Waiting for emulator to be ready...');
        let maxAttempts = 10;
        let emulatorReady = false;
        for (let i = 0; i < maxAttempts; i++) {
            await page.waitForTimeout(500);
            const isReady = await page.evaluate(() => {
                return window.emulator !== undefined && window.emulator !== null;
            });
            if (isReady) {
                console.log('Emulator is ready!');
                emulatorReady = true;
                break;
            }
            console.log(`Waiting for emulator... attempt ${i + 1}/${maxAttempts}`);
        }

        if (!emulatorReady) {
            console.log('Emulator not ready after timeout');
        }

        // Try to click the load test button
        const loadBtn = await page.$('#load-test-btn');
        if (loadBtn) {
            console.log('Load Test ROM button found, clicking...');
            await loadBtn.click();
        }

        // Wait for any updates
        await page.waitForTimeout(1000);

        // Try running frames if emulator is available
        const result = await page.evaluate(async () => {
            if (window.emulator) {
                window.emulator.run_frames(1);
                return { success: true, frame: window.emulator.frame_count() };
            }
            return { success: false, error: 'No emulator available' };
        });
        console.log('Frame run result:', result);

    } catch (error) {
        console.error('Test error:', error);
    } finally {
        await browser.close();
        server.close();
    }
}

runTest();