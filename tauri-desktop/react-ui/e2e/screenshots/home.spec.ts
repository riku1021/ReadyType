import path from "node:path";
import { expect, test } from "@playwright/test";

import { installTauriMocks } from "../fixtures/mockTauri";

const screenshotDir = path.join("tmp", "screenshots");

test.describe("ホーム", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page);
  });

  test("起動画面の診断スクショを保存する", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("navigation", { name: "メインナビゲーション" })).toBeVisible();
    await expect(page.getByRole("link", { name: "ホーム" })).toBeVisible();

    await page.screenshot({
      path: path.join(screenshotDir, "home.png"),
      fullPage: true,
    });
  });
});
