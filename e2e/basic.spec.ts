import { test, expect } from '@playwright/test';

test.describe('Basic Page Load', () => {
  test('page loads with title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle('q-explore');
  });

  test('map container is visible', async ({ page }) => {
    await page.goto('/');
    const map = page.locator('#map');
    await expect(map).toBeVisible();
  });

  test('control panel is visible', async ({ page }) => {
    await page.goto('/');
    const panel = page.locator('#panel');
    await expect(panel).toBeVisible();
    await expect(page.locator('h1')).toHaveText('q-explore');
  });

  test('all tabs are present', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('[data-tab="generate"]')).toBeVisible();
    await expect(page.locator('[data-tab="history"]')).toBeVisible();
    await expect(page.locator('[data-tab="settings"]')).toBeVisible();
  });

  test('generate tab is active by default', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('[data-tab="generate"]')).toHaveClass(/active/);
    await expect(page.locator('#generate-tab')).toHaveClass(/active/);
  });
});

test.describe('Tab Navigation', () => {
  test('can switch to history tab', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-tab="history"]');
    await expect(page.locator('[data-tab="history"]')).toHaveClass(/active/);
    await expect(page.locator('#history-tab')).toHaveClass(/active/);
  });

  test('can switch to settings tab', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-tab="settings"]');
    await expect(page.locator('[data-tab="settings"]')).toHaveClass(/active/);
    await expect(page.locator('#settings-tab')).toHaveClass(/active/);
  });
});

test.describe('Generate Form Elements', () => {
  test('location input exists', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#location-input')).toBeVisible();
  });

  test('radius input has default value', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#radius-input')).toHaveValue('3000');
  });

  test('mode select has options', async ({ page }) => {
    await page.goto('/');
    const modeSelect = page.locator('#mode-select');
    await expect(modeSelect).toBeVisible();
    await expect(modeSelect.locator('option')).toHaveCount(2);
  });

  test('backend select has options', async ({ page }) => {
    await page.goto('/');
    const backendSelect = page.locator('#backend-select');
    await expect(backendSelect).toBeVisible();
    await expect(backendSelect.locator('option')).toHaveCount(2);
  });

  test('type buttons are present', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('[data-type="attractor"]')).toBeVisible();
    await expect(page.locator('[data-type="void"]')).toBeVisible();
    await expect(page.locator('[data-type="power"]')).toBeVisible();
    await expect(page.locator('[data-type="blind_spot"]')).toBeVisible();
  });

  test('attractor is selected by default', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('[data-type="attractor"]')).toHaveClass(/active/);
  });

  test('generate button exists', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#generate-btn')).toBeVisible();
    await expect(page.locator('#generate-btn')).toHaveText('Generate');
  });
});
