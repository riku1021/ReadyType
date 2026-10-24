import type { KeyboardEvent, MouseEvent } from "react";

import { cx } from "@/styled-system/css";

import * as styles from "./DataTable.styles";
import type { DataTableProps } from "./types";

function stopRowSelect(event: MouseEvent | KeyboardEvent) {
  event.stopPropagation();
}

export function DataTable<T>(props: DataTableProps<T>) {
  const {
    columns,
    getRowId,
    selectedRowId,
    onRowSelect,
    rowClassName,
    emptyMessage,
    footer,
    className,
    hideHeader = false,
  } = props;

  const isGrouped = "groups" in props && props.groups !== undefined;
  const isEmpty = isGrouped ? props.groups.length === 0 : props.rows.length === 0;

  const colgroup = (
    <colgroup>
      {columns.map((column) => (
        <col key={column.id} style={column.width ? { width: column.width } : undefined} />
      ))}
    </colgroup>
  );

  const renderRow = (row: T) => {
    const rowId = getRowId(row);
    const selected = rowId === selectedRowId;
    return (
      <tr
        key={rowId}
        className={cx(
          styles.dataRow,
          onRowSelect ? styles.interactiveRow : undefined,
          rowClassName?.(row, selected)
        )}
        aria-selected={selected}
        tabIndex={0}
        onClick={() => onRowSelect?.(rowId)}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            onRowSelect?.(rowId);
          }
        }}
      >
        {columns.map((column) => {
          const cellClassName =
            typeof column.cellClassName === "function"
              ? column.cellClassName(row, selected)
              : column.cellClassName;
          const handlers = column.stopPropagation
            ? { onClick: stopRowSelect, onKeyDown: stopRowSelect }
            : {};
          return (
            <td
              key={column.id}
              className={cx(onRowSelect ? styles.interactiveCell : undefined, cellClassName)}
              {...handlers}
            >
              {column.cell(row)}
            </td>
          );
        })}
      </tr>
    );
  };

  return (
    <section className={cx(styles.panel, className)}>
      {isEmpty ? (
        <p className={styles.empty}>{emptyMessage}</p>
      ) : (
        <>
          {hideHeader ? null : (
            <div className={styles.tableHeader}>
              <table className={styles.table}>
                {colgroup}
                <thead>
                  <tr>
                    {columns.map((column) => (
                      <th
                        key={column.id}
                        scope="col"
                        className={column.headerClassName}
                        aria-label={column.headerAriaLabel}
                      >
                        {column.header}
                      </th>
                    ))}
                  </tr>
                </thead>
              </table>
            </div>
          )}
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              {colgroup}
              {isGrouped ? (
                props.groups.map((group) => (
                  <tbody key={group.id}>
                    <tr>
                      <th
                        scope="colgroup"
                        colSpan={columns.length}
                        className={group.headerClassName}
                      >
                        {group.label}
                      </th>
                    </tr>
                    {group.rows.map(renderRow)}
                  </tbody>
                ))
              ) : (
                <tbody>{props.rows.map(renderRow)}</tbody>
              )}
            </table>
          </div>
          {footer}
        </>
      )}
    </section>
  );
}
