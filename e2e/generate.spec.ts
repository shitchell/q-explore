import { test, expect } from '@playwright/test';

test.describe('Generate Functionality', () => {
  test('clicking generate without location shows alert', async ({ page }) => {
    await page.goto('/');

    // Listen for dialog
    page.on('dialog', async dialog => {
      expect(dialog.message()).toContain('select a location');
      await dialog.accept();
    });

    await page.click('#generate-btn');
  });

  test('can set location by right-clicking map', async ({ page }) => {
    await page.goto('/');

    // Wait for map to be ready
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Right-click on map to set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });

    // Check that coordinates are displayed
    const latDisplay = page.locator('#lat-display');
    const lngDisplay = page.locator('#lng-display');

    await expect(latDisplay).not.toHaveText('--');
    await expect(lngDisplay).not.toHaveText('--');
  });

  test('can set location by typing coordinates and pressing Enter', async ({ page }) => {
    await page.goto('/');

    // Wait for map to be ready
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Type coordinates in the location input
    await page.fill('#location-input', '40.7128, -74.0060');
    await page.press('#location-input', 'Enter');

    // Check that coordinates are displayed
    await expect(page.locator('#lat-display')).toHaveText('40.712800');
    await expect(page.locator('#lng-display')).toHaveText('-74.006000');
  });

  test('generate button works after setting location', async ({ page }) => {
    await page.goto('/');

    // Wait for map to be ready
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Right-click on map to set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });

    // Wait for location to be set
    await expect(page.locator('#lat-display')).not.toHaveText('--');

    // Click generate
    await page.click('#generate-btn');

    // Button should show loading state
    await expect(page.locator('#generate-btn')).toHaveText('Generating...');

    // Wait for result panel to appear (or error)
    await page.waitForSelector('#result-panel:not(.hidden), .error-message', { timeout: 15000 });

    // Result panel should be visible
    await expect(page.locator('#result-panel')).toBeVisible();
  });

  test('result panel shows coordinates after generation', async ({ page }) => {
    await page.goto('/');

    // Wait for map to be ready
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Right-click on map to set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });

    // Wait for location to be set
    await expect(page.locator('#lat-display')).not.toHaveText('--');

    // Click generate
    await page.click('#generate-btn');

    // Wait for result
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Check result content has coordinates
    const resultContent = page.locator('#result-content');
    await expect(resultContent).toContainText(/\d+\.\d+/); // Should contain decimal number
  });

  test('can switch anomaly type and regenerate', async ({ page }) => {
    await page.goto('/');

    // Wait for map to be ready
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });
    await expect(page.locator('#lat-display')).not.toHaveText('--');

    // Generate with attractor (default)
    await page.click('#generate-btn');
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Switch to void
    await page.click('[data-type="void"]');
    await expect(page.locator('[data-type="void"]')).toHaveClass(/active/);

    // Result should update to show void result
    await expect(page.locator('#result-panel')).toBeVisible();
  });
});

test.describe('Generate with Different Modes', () => {
  test('can generate with flower power mode', async ({ page }) => {
    await page.goto('/');

    // Wait for map
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });
    await expect(page.locator('#lat-display')).not.toHaveText('--');

    // Select flower power mode
    await page.selectOption('#mode-select', 'flower_power');

    // Generate
    await page.click('#generate-btn');
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Result should be visible
    await expect(page.locator('#result-panel')).toBeVisible();
  });
});
