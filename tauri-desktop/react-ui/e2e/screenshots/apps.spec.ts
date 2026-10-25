import path from "node:path";
import { expect, test } from "@playwright/test";

import { installTauriMocks } from "../fixtures/mockTauri";

const screenshotDir = path.join("tmp", "screenshots");

test.describe("アプリごとの設定", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page);
  });

  test("一覧から詳細へ遷移し診断スクショを保存する", async ({ page }) => {
    await page.goto("/apps");

    await expect(page.getByRole("navigation", { name: "メインナビゲーション" })).toBeVisible();
    await expect(page.locator("header").getByText("アプリごとの設定")).toBeVisible();
    const search = page.getByPlaceholder("アプリを検索...");
    await expect(search).toBeEnabled();
    await expect(page.getByRole("button", { name: "アプリを追加" })).toHaveCount(0);
    await expect(page.getByText("Cursor", { exact: true })).toBeVisible();
    await expect(page.getByText("Terminal", { exact: true })).toBeVisible();
    await expect(page.getByRole("switch")).toHaveCount(0);
    await expect(page.getByText(/\d+個/)).toHaveCount(0);

    await search.fill("Cursor");
    await expect(page.getByText("Cursor", { exact: true })).toBeVisible();
    await expect(page.getByText("Terminal", { exact: true })).toHaveCount(0);
    await page.getByRole("button", { name: "検索をクリア" }).click();
    await expect(search).toHaveValue("");
    await expect(page.getByText("Terminal", { exact: true })).toBeVisible();

    const rows = page.locator("tbody tr");
    const sortOrder = page.getByRole("combobox", { name: "並び替え" });
    await sortOrder.focus();
    await expect(sortOrder).toHaveCSS("outline-style", "none");
    await expect(sortOrder.locator("xpath=..")).toHaveCSS("outline-style", "none");
    await expect(rows.nth(0)).toContainText("Cursor");
    await expect(rows.nth(1)).toContainText("Terminal");

    await sortOrder.selectOption({ label: "追加順" });
    await expect(rows.nth(0)).toContainText("Terminal");
    await expect(rows.nth(1)).toContainText("Cursor");

    await search.fill("Cursor");
    await expect(page.getByText("Terminal", { exact: true })).toHaveCount(0);
    await page.getByRole("button", { name: "検索をクリア" }).click();
    await expect(rows.nth(0)).toContainText("Terminal");

    await sortOrder.selectOption({ label: "ルール数が多い順" });
    await expect(rows.nth(0)).toContainText("Cursor");
    await expect(rows.nth(1)).toContainText("Terminal");

    await sortOrder.selectOption({ label: "名前順" });
    await expect(rows.nth(0)).toContainText("Cursor");

    await page.screenshot({
      path: path.join(screenshotDir, "apps.png"),
      fullPage: true,
    });

    await page.getByText("Cursor", { exact: true }).click();

    await expect(page.getByRole("link", { name: "一覧に戻る" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Cursor" })).toBeVisible();
    await expect(page.getByText("Cursor Composer", { exact: true })).toBeVisible();
    await expect(page.getByText("Cursor Editor", { exact: true })).toBeVisible();
    await expect(page.getByText("Cursor Terminal", { exact: true })).toBeVisible();

    await page.screenshot({
      path: path.join(screenshotDir, "apps-detail.png"),
      fullPage: true,
    });

    await page.getByRole("link", { name: "一覧に戻る" }).click();
    await page.getByRole("combobox", { name: "並び替え" }).selectOption({ label: "追加順" });
    await page.getByText("Terminal", { exact: true }).click();
    await page.getByRole("link", { name: "一覧に戻る" }).click();
    await expect(page.locator("tbody tr").nth(0)).toContainText("Terminal");
  });
});
