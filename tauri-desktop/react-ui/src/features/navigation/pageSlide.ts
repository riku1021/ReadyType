import { pageSlideBackType, pageSlideForwardType } from "@/styles/bases/pageSlide";

type PageSlideDirection = "forward" | "back";

const pageSlideTypeByDirection = {
  forward: pageSlideForwardType,
  back: pageSlideBackType,
} as const;

/**
 * 一覧から詳細へ進むときは右から、詳細から一覧へ戻るときは左から入る。
 * 動きを減らす設定ではスライドしない。
 */
export function pageSlideTypes(
  direction: PageSlideDirection,
  reduceMotion: boolean
): string[] | false {
  if (reduceMotion) {
    return false;
  }
  return [pageSlideTypeByDirection[direction]];
}

type PageSlideChange = {
  fromLocation?: { pathname: string };
  toLocation: { pathname: string };
};

function isDirectDetailPath(pathname: string, listPath: string): boolean {
  const prefix = `${listPath}/`;
  if (!pathname.startsWith(prefix)) {
    return false;
  }
  const detailId = pathname.slice(prefix.length);
  return detailId.length > 0 && !detailId.includes("/");
}

/**
 * 一覧の直接の詳細から、その一覧へ戻るときだけ左スライドにする。
 * 別セクションからの移動ではスライドしない。
 */
export function pageSlideBackToListTypes(
  fromPathname: string | undefined,
  toPathname: string,
  listPath: string,
  reduceMotion: boolean
): string[] | false {
  if (
    reduceMotion ||
    fromPathname === undefined ||
    toPathname !== listPath ||
    !isDirectDetailPath(fromPathname, listPath)
  ) {
    return false;
  }
  return pageSlideTypes("back", false);
}

function supportsDirectionalViewTransition(): boolean {
  if (typeof document === "undefined" || typeof CSS === "undefined") {
    return false;
  }
  return (
    "startViewTransition" in document && CSS.supports("selector(:active-view-transition-type(a))")
  );
}

type PageSlideOption = {
  types: (change: PageSlideChange) => string[] | false;
};

/** 未対応の環境では全画面のクロスフェードに落ちないよう false を返す。 */
function pageSlideOption(
  resolve: (reduceMotion: boolean, change: PageSlideChange) => string[] | false
): false | PageSlideOption {
  if (!supportsDirectionalViewTransition()) {
    return false;
  }
  return {
    types: (change) =>
      resolve(window.matchMedia("(prefers-reduced-motion: reduce)").matches, change),
  };
}

/** `navigate` / `Link` の `viewTransition` に渡す。 */
export function pageSlide(direction: PageSlideDirection): false | PageSlideOption {
  return pageSlideOption((reduceMotion) => pageSlideTypes(direction, reduceMotion));
}

/**
 * ドロワーなど、一覧へのリンクに渡す。
 * その一覧の詳細にいるときだけ、一覧へ戻るスライドになる。
 */
export function pageSlideBackTo(listPath: string): false | PageSlideOption {
  return pageSlideOption((reduceMotion, change) =>
    pageSlideBackToListTypes(
      change.fromLocation?.pathname,
      change.toLocation.pathname,
      listPath,
      reduceMotion
    )
  );
}
