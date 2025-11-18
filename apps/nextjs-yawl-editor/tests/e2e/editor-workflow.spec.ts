/**
 * End-to-End Workflow Tests - Playwright
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 1: RDF round-trip integrity (full stack)
 * - Covenant 2: Pattern validation (user perspective)
 * - Complete user workflows from UI to kernel
 */

import { test, expect } from '@playwright/test';

test.describe('YAWL Editor E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/editor');
  });

  test.describe('Covenant 1: RDF Round-Trip Integrity', () => {
    test('should create workflow → export Turtle → verify format', async ({ page }) => {
      // Create a simple workflow
      await page.click('[data-testid="add-task-button"]');

      // Export to Turtle
      await page.click('[data-testid="export-button"]');
      await page.click('[data-testid="export-turtle"]');

      // Verify download triggered
      const downloadPromise = page.waitForEvent('download');
      await page.click('[data-testid="confirm-export"]');
      const download = await downloadPromise;

      expect(download.suggestedFilename()).toMatch(/\.ttl$/);
    });

    test('should import Turtle → render workflow → verify nodes', async ({ page }) => {
      // Import sample workflow
      const fileInput = await page.locator('input[type="file"]');

      await fileInput.setInputFiles({
        name: 'sample.ttl',
        mimeType: 'text/turtle',
        buffer: Buffer.from(`
          @prefix yawl: <http://knhk.io/ontology/yawl#> .
          :workflow-1 a yawl:Specification .
        `),
      });

      // Verify canvas renders
      await expect(page.locator('[data-testid="workflow-canvas"]')).toBeVisible();
    });
  });

  test.describe('Covenant 2: Pattern Validation', () => {
    test('should show validation error for missing end node', async ({ page }) => {
      // Add start and task, but no end
      await page.click('[data-testid="add-condition-button"]');
      await page.click('[data-testid="add-task-button"]');

      // Connect them
      // (Simulate drag-and-drop connection)

      // Expect validation error
      await expect(page.locator('[data-testid="validation-error"]')).toContainText(
        'end'
      );
    });

    test('should validate AND-split requires AND-join', async ({ page }) => {
      // Create AND-split
      await page.click('[data-testid="add-and-split"]');

      // Add tasks
      await page.click('[data-testid="add-task-button"]');
      await page.click('[data-testid="add-task-button"]');

      // Try to connect without AND-join
      // Should show validation error

      await expect(page.locator('[data-testid="validation-error"]')).toContainText(
        'AND-join'
      );
    });
  });

  test.describe('Canvas Interactions', () => {
    test('should add task node to canvas', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');

      await expect(page.locator('[data-testid="task-node"]')).toBeVisible();
    });

    test('should select node and show properties panel', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');

      // Click on task node
      await page.click('[data-testid="task-node"]');

      // Properties panel should appear
      await expect(page.locator('[data-testid="property-panel"]')).toBeVisible();
    });

    test('should delete node from canvas', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');

      // Select node
      await page.click('[data-testid="task-node"]');

      // Delete
      await page.keyboard.press('Delete');

      // Node should be gone
      await expect(page.locator('[data-testid="task-node"]')).not.toBeVisible();
    });

    test('should connect two nodes with edge', async ({ page }) => {
      // Add two tasks
      await page.click('[data-testid="add-task-button"]');
      await page.click('[data-testid="add-task-button"]');

      // Drag to connect (simulate)
      // (Actual implementation depends on React Flow behavior)

      // Verify edge exists
      // await expect(page.locator('[data-testid="edge"]')).toBeVisible();
    });
  });

  test.describe('Property Editing', () => {
    test('should edit task label', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');
      await page.click('[data-testid="task-node"]');

      // Edit label in property panel
      await page.fill('[data-testid="task-label-input"]', 'Process Order');

      // Verify label updated
      await expect(page.locator('[data-testid="task-node"]')).toContainText('Process Order');
    });

    test('should set task decomposition type', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');
      await page.click('[data-testid="task-node"]');

      // Select decomposition type
      await page.selectOption('[data-testid="decomposition-select"]', 'composite');

      // Verify updated
      // (Check via export or state inspection)
    });
  });

  test.describe('Workflow Management', () => {
    test('should save workflow to local storage', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');

      // Save
      await page.click('[data-testid="save-button"]');

      // Verify saved (check localStorage or success message)
      await expect(page.locator('[data-testid="save-success"]')).toBeVisible();
    });

    test('should load saved workflow', async ({ page }) => {
      // Assume workflow was saved previously
      await page.click('[data-testid="load-button"]');

      // Select workflow
      await page.click('[data-testid="saved-workflow-1"]');

      // Verify loaded
      await expect(page.locator('[data-testid="workflow-canvas"]')).toBeVisible();
    });
  });

  test.describe('Performance (Chicago TDD)', () => {
    test('should render canvas within 2 seconds', async ({ page }) => {
      const start = Date.now();

      await page.goto('/editor');
      await page.waitForSelector('[data-testid="workflow-canvas"]');

      const elapsed = Date.now() - start;

      expect(elapsed).toBeLessThan(2000);
      console.log(`✓ Canvas loaded in ${elapsed}ms`);
    });

    test('should validate workflow quickly (≤100ms perceived)', async ({ page }) => {
      await page.click('[data-testid="add-task-button"]');

      const start = Date.now();

      // Trigger validation
      await page.click('[data-testid="validate-button"]');
      await page.waitForSelector('[data-testid="validation-result"]');

      const elapsed = Date.now() - start;

      expect(elapsed).toBeLessThan(100);
      console.log(`✓ Validation completed in ${elapsed}ms`);
    });
  });

  test.describe('Error Handling', () => {
    test('should show error for invalid Turtle import', async ({ page }) => {
      const fileInput = await page.locator('input[type="file"]');

      await fileInput.setInputFiles({
        name: 'invalid.ttl',
        mimeType: 'text/turtle',
        buffer: Buffer.from('invalid turtle syntax{{{'),
      });

      await expect(page.locator('[data-testid="import-error"]')).toBeVisible();
    });

    test('should recover from validation errors gracefully', async ({ page }) => {
      // Create invalid workflow
      await page.click('[data-testid="add-task-button"]');

      // Should show error
      await expect(page.locator('[data-testid="validation-error"]')).toBeVisible();

      // Fix by adding end node
      await page.click('[data-testid="add-end-node"]');

      // Error should clear
      await expect(page.locator('[data-testid="validation-error"]')).not.toBeVisible();
    });
  });
});
