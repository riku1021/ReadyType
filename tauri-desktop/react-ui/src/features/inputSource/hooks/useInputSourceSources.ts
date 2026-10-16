import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAtomValue, useSetAtom } from "jotai";
import { useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

import { getInputSourceSettings, INPUT_SOURCES_CHANGED_EVENT, listInputSources } from "../api";
import { inputSourceSourcesAtom } from "../atoms/inputSourceSourcesAtom";
import { inputSourceStateAtom } from "../atoms/inputSourceStateAtom";

const FOCUS_REFRESH_DEBOUNCE_MS = 400;

/**
 * `list_input_sources` の共有キャッシュを維持する（bootstrap の Promise.all には載せない）。
 * MainApp で 1 回だけ呼ぶ。
 */
export function useInputSourceSources() {
  const { i18n } = useTranslation();
  const ready = useAtomValue(inputSourceStateAtom).ready;
  const { status, locale } = useAtomValue(inputSourceSourcesAtom);
  const setSources = useSetAtom(inputSourceSourcesAtom);
  const setFeature = useSetAtom(inputSourceStateAtom);
  const language = i18n.language;
  // 最後に発行したリクエストの結果だけ採用する
  const latestRequestRef = useRef(0);

  const load = useCallback(
    (refresh: boolean, requestedLanguage: string) => {
      latestRequestRef.current += 1;
      const requestId = latestRequestRef.current;
      void listInputSources(refresh)
        .then(async (sources) => {
          if (requestId !== latestRequestRef.current) {
            return;
          }
          setSources({
            sources,
            locale: requestedLanguage,
            status: "ready",
            fetchedAt: Date.now(),
          });
          // list 成功時に Native が display names を永続化するので settings を取り直す
          try {
            const settings = await getInputSourceSettings();
            if (requestId !== latestRequestRef.current) {
              return;
            }
            setFeature((current) => ({ ...current, settings }));
          } catch {
            // 表示名の同期失敗は一覧表示自体は妨げない
          }
        })
        .catch(() => {
          if (requestId !== latestRequestRef.current) {
            return;
          }
          // 取得済みなら維持する。locale も更新し、失敗時の再取得ループを避ける。
          setSources((current) =>
            current.sources.length > 0
              ? { ...current, locale: requestedLanguage, status: "ready" }
              : { ...current, status: "error" }
          );
        });
    },
    [setFeature, setSources]
  );

  // A: 初回 / B: UI 言語変更
  useEffect(() => {
    if (!ready) {
      return;
    }
    if (status === "idle") {
      setSources((current) => ({ ...current, status: "loading" }));
      load(false, language);
      return;
    }
    if (status === "ready" && locale !== language) {
      load(true, language);
    }
  }, [ready, status, locale, language, load, setSources]);

  // D: OS の enabled 入力ソース増減（Native 遅延再 list 後。FE も refresh で揃える）
  useEffect(() => {
    if (!ready || !isTauri()) {
      return;
    }
    let cancelled = false;
    const unlistenPromise = listen(INPUT_SOURCES_CHANGED_EVENT, () => {
      if (cancelled) {
        return;
      }
      load(true, i18n.language);
    });
    return () => {
      cancelled = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [ready, load, i18n]);

  // C: メイン窓フォーカス復帰（イベント取りこぼしの安全網）
  useEffect(() => {
    if (!ready) {
      return;
    }
    let timer: ReturnType<typeof setTimeout> | undefined;
    const unlistenFocusPromise = isTauri()
      ? getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (!focused) {
            return;
          }
          clearTimeout(timer);
          timer = setTimeout(() => load(true, i18n.language), FOCUS_REFRESH_DEBOUNCE_MS);
        })
      : undefined;

    return () => {
      clearTimeout(timer);
      // `.then` 代入前に unmount しても解除できるよう Promise を待つ。
      void unlistenFocusPromise?.then((unlisten) => unlisten());
    };
  }, [ready, load, i18n]);
}
