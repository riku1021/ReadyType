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
    await expect(page.getByRole("button", { name: "アプリを追加" })).toBeDisabled();
    await expect(page.getByRole("columnheader", { name: "アプリ" })).toBeVisible();
    await expect(page.getByText("Cursor", { exact: true })).toBeVisible();
    await expect(page.getByText("Terminal", { exact: true })).toBeVisible();

    await search.fill("Cursor");
    await expect(page.getByText("Cursor", { exact: true })).toBeVisible();
    await expect(page.getByText("Terminal", { exact: true })).toHaveCount(0);
    await page.getByRole("button", { name: "検索をクリア" }).click();
    await expect(search).toHaveValue("");
    await expect(page.getByText("Terminal", { exact: true })).toBeVisible();

    await page.screenshot({
      path: path.join(screenshotDir, "apps.png"),
      fullPage: true,
    });

    await page.getByText("Cursor", { exact: true }).click();

    await expect(page.getByRole("link", { name: "一覧に戻る" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Cursor" })).toBeVisible();
    await expect(page.getByText("Chat（AIチャット欄）")).toBeVisible();
    await expect(page.getByText("Editor（コードエディタ）")).toBeVisible();
    await expect(page.getByText("Terminal（内蔵ターミナル）")).toBeVisible();

    await page.screenshot({
      path: path.join(screenshotDir, "apps-detail.png"),
      fullPage: true,
    });
  });
});
