import type { Key, ReactNode } from "react";
import { cx } from "../cx";

export interface Column<T> {
  key: string;
  header: ReactNode;
  render: (row: T) => ReactNode;
  numeric?: boolean;
}

export interface DataTableProps<T> {
  columns: Column<T>[];
  rows: T[];
  rowKey: (row: T) => Key;
  /** Shown instead of the table when there are no rows. */
  empty?: ReactNode;
  caption?: ReactNode;
  className?: string;
}

export function DataTable<T>({ columns, rows, rowKey, empty, caption, className }: DataTableProps<T>) {
  if (rows.length === 0 && empty != null) return <>{empty}</>;
  return (
    <div className={cx("uc-table-wrap", className)}>
      <table className="uc-table">
        {caption != null && <caption className="uc-muted uc-small">{caption}</caption>}
        <thead>
          <tr>
            {columns.map((c) => (
              <th key={c.key} scope="col" className={c.numeric ? "uc-num" : undefined}>
                {c.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={rowKey(row)}>
              {columns.map((c) => (
                <td key={c.key} className={c.numeric ? "uc-num" : undefined}>
                  {c.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
