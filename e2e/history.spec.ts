import { test, expect } from '@playwright/test';

test.describe('History Functionality', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
  });

  test('history tab shows empty state initially', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-tab="history"]');

    await expect(page.locator('#history-tab')).toHaveClass(/active/);
    await expect(page.locator('#history-list')).toContainText('No history yet');
  });

  test('generation adds entry to history', async ({ page }) => {
    await page.goto('/');

    // Wait for map
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });
    await expect(page.locator('#lat-display')).not.toHaveText('--');

    // Generate
    await page.click('#generate-btn');
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Switch to history tab
    await page.click('[data-tab="history"]');

    // Should have one entry
    const historyItems = page.locator('.history-item');
    await expect(historyItems).toHaveCount(1);
  });

  test('clicking history entry shows result on map', async ({ page }) => {
    await page.goto('/');

    // Wait for map
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Set location and generate
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });
    await expect(page.locator('#lat-display')).not.toHaveText('--');
    await page.click('#generate-btn');
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Hide result panel (switch tab then back)
    await page.click('[data-tab="history"]');
    await expect(page.locator('#history-tab')).toHaveClass(/active/);

    // Click on history entry
    await page.click('.history-item');

    // Should switch to generate tab and show result
    await expect(page.locator('[data-tab="generate"]')).toHaveClass(/active/);
    await expect(page.locator('#result-panel')).toBeVisible();
  });

  test('multiple generations create multiple history entries', async ({ page }) => {
    await page.goto('/');

    // Wait for map
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // First generation
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });
    await expect(page.locator('#lat-display')).not.toHaveText('--');
    await page.click('#generate-btn');
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Second generation (different location)
    await map.click({ button: 'right', position: { x: 400, y: 300 } });
    await page.click('#generate-btn');
    await page.waitForSelector('#generate-btn:has-text("Generate")', { timeout: 15000 });

    // Check history
    await page.click('[data-tab="history"]');
    const historyItems = page.locator('.history-item');
    await expect(historyItems).toHaveCount(2);
  });

  test('history displays correct mode (Standard/Flower)', async ({ page }) => {
    await page.goto('/');

    // Wait for map
    await page.waitForSelector('#map .leaflet-tile-loaded', { timeout: 10000 });

    // Set location
    const map = page.locator('#map');
    await map.click({ button: 'right', position: { x: 300, y: 200 } });
    await expect(page.locator('#lat-display')).not.toHaveText('--');

    // Generate with standard mode (default)
    await page.click('#generate-btn');
    await page.waitForSelector('#result-panel:not(.hidden)', { timeout: 15000 });

    // Check history shows Standard
    await page.click('[data-tab="history"]');
    await expect(page.locator('.history-item')).toContainText('Standard');
  });
});
