import { Link } from "@tanstack/react-router";
import { IoHome } from "react-icons/io5";
import NotFoundAnimation from "@/animations/NotFoundAnimation";
import { Button, button } from "@/components/Button";
import { cx } from "@/styled-system/css";

import * as styles from "./NotFoundPage.styles";

export function NotFoundPage() {
  return (
    <div className={styles.notFoundContainer}>
      <NotFoundAnimation className={styles.animation} />
      <h2 className={styles.heading}>ページが見つかりません</h2>
      <p className={styles.paragraph}>
        お探しのページは存在しないか、URLが間違っている可能性があります。
      </p>
      <Link to="/" className={cx(button({ variant: "secondary" }), styles.homeLink)}>
        <IoHome className={styles.homeIcon} /> ホームへ
      </Link>
      <div className={styles.backButtonWrap}>
        <Button variant="text" onClick={() => window.history.back()}>
          戻る
        </Button>
      </div>
    </div>
  );
}
