import { test, expect } from "@playwright/test";

test("built palette shell loads", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByLabel("Command input")).toBeVisible();
});
