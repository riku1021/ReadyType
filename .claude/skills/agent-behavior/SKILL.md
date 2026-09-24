---
name: agent-behavior
description: >-
  実装・レビュー時の行動指針（前提明示、最小コード、外科的 diff、検証可能な成功条件）。
  コーディング開始前やリファクタ判断時に使う。
---

# Agent Behavior Skill

正本ルール: `.claude/rules/agent-behavior.md`（alwaysApply）。

**Tradeoff:** 慎重さを速度より優先。trivial な typo 修正では簡略化してよい。

## 1. Think Before Coding

- 前提を明示。不確かなら質問
- 解釈が複数なら提示（黙って 1 つ選ばない）
- より単純な方法があれば言う
- 不明点を名指しして止まる

## 2. Simplicity First

- 依頼されていない機能・抽象・設定を足さない
- 起き得ないエラー処理を書かない
- 過剰なら単純化する

## 3. Surgical Changes

- 必要な行だけ触る
- 隣接の「ついで改善」をしない
- 自分の変更で orphan になった import 等だけ削除
- 既存デッドコードは依頼なしに消さない

## 4. Goal-Driven Execution

曖昧指示 → 検証可能な目標:

- バリデーション追加 → 不正入力テストが通る
- バグ修正 → 再現テストが通る

```text
1. [手順] → verify: [確認]
2. [手順] → verify: [確認]
```
