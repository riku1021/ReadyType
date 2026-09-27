import type { ReactNode } from "react";

export type DataTableColumn<T> = {
  id: string;
  /** ヘッダー表示。チェック列などラベルなしは省略可 */
  header?: ReactNode;
  /** ラベルを持たない列の読み上げ用 */
  headerAriaLabel?: string;
  /** colgroup に渡す列幅（例: "6.5rem" / "32%"） */
  width?: string;
  headerClassName?: string;
  /** セルの className。選択状態で変える場合は関数を渡す */
  cellClassName?: string | ((row: T, selected: boolean) => string);
  /** true のときセルクリックで行選択を発火させない */
  stopPropagation?: boolean;
  cell: (row: T) => ReactNode;
};

export type DataTableGroup<T> = {
  id: string;
  /** グループ帯の中身（ラベル + 件数など） */
  label: ReactNode;
  rows: T[];
  /** グループ帯 th の className（トーン等） */
  headerClassName?: string;
};

type DataTableSharedProps<T> = {
  columns: DataTableColumn<T>[];
  getRowId: (row: T) => string;
  selectedRowId?: string | null;
  onRowSelect?: (rowId: string) => void;
  /** 行の className。選択状態で変える場合に使う */
  rowClassName?: (row: T, selected: boolean) => string;
  emptyMessage: string;
  /** スクロール領域外に固定表示する要素（ページネーション等） */
  footer?: ReactNode;
  /** パネル殻への追加 className */
  className?: string;
};

/** フラット行（企業一覧など）。groups とは排他 */
type DataTableRowsProps<T> = DataTableSharedProps<T> & {
  rows: T[];
  groups?: never;
};

/** グループ行（タスク・予定など）。rows とは排他 */
type DataTableGroupsProps<T> = DataTableSharedProps<T> & {
  groups: DataTableGroup<T>[];
  rows?: never;
};

/** `rows` か `groups` のどちらか一方が必須（同時指定不可） */
export type DataTableProps<T> = DataTableRowsProps<T> | DataTableGroupsProps<T>;
