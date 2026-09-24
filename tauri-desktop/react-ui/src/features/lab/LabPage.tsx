import { useState } from "react";

import { Button } from "@/components/Button";
import { AlertDialog, ErrorDialog, LoadingDialog, SuccessDialog } from "@/components/Dialog";
import { FormModal } from "@/components/FormModal";

import * as styles from "./LabPage.styles";

export function LabPage() {
  const [infoResult, setInfoResult] = useState("");
  const [isSuccessDialogOpen, setIsSuccessDialogOpen] = useState(false);
  const [isErrorDialogOpen, setIsErrorDialogOpen] = useState(false);
  const [isAlertDialogOpen, setIsAlertDialogOpen] = useState(false);
  const [isLoadingDialogOpen, setIsLoadingDialogOpen] = useState(false);
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);

  const handleLoadingDialog = () => {
    setIsLoadingDialogOpen(true);
    window.setTimeout(() => {
      setIsLoadingDialogOpen(false);
    }, 3000);
  };

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>開発確認</h1>
      <p className={styles.lead}>
        ダイアログやモーダルなど、共通コンポーネントの動作確認用ページです。
      </p>

      <h2 className={styles.sectionTitle}>ダイアログ / モーダル確認</h2>
      <div className={styles.buttonGroup}>
        <Button variant="primary" onClick={() => setIsSuccessDialogOpen(true)}>
          成功
        </Button>
        <Button variant="primary" onClick={() => setIsErrorDialogOpen(true)}>
          エラー
        </Button>
        <Button variant="secondary" onClick={() => setIsAlertDialogOpen(true)}>
          確認
        </Button>
        <Button variant="primary" onClick={handleLoadingDialog}>
          ローディング
        </Button>
        <Button variant="secondary" onClick={() => setIsFormModalOpen(true)}>
          フォームモーダル
        </Button>
      </div>
      {infoResult ? <p className={styles.result}>結果: {infoResult}</p> : null}

      <SuccessDialog
        isOpen={isSuccessDialogOpen}
        onClose={() => setIsSuccessDialogOpen(false)}
        title="成功"
        text="操作が完了しました"
      />
      <ErrorDialog
        isOpen={isErrorDialogOpen}
        onClose={() => setIsErrorDialogOpen(false)}
        title="エラー"
        text="エラーが発生しました"
      />
      <AlertDialog
        isOpen={isAlertDialogOpen}
        onClose={() => setIsAlertDialogOpen(false)}
        title="確認"
        text="この操作を実行しますか？"
        confirmButtonText="実行する"
        cancelButtonText="キャンセル"
        onConfirm={() => setInfoResult("操作が確認されました")}
      />
      <LoadingDialog
        isOpen={isLoadingDialogOpen}
        onClose={() => setIsLoadingDialogOpen(false)}
        title="処理中..."
        text="データを読み込んでいます"
      />
      <FormModal
        isOpen={isFormModalOpen}
        onClose={() => setIsFormModalOpen(false)}
        title="サンプル"
      >
        <div className={styles.formBody}>
          <p>FormModal の中身サンプルです。実際のフォームは今後ここに載せます。</p>
          <Button variant="primary" onClick={() => setIsFormModalOpen(false)}>
            閉じる
          </Button>
        </div>
      </FormModal>
    </div>
  );
}
