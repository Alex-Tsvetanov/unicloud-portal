import { cx } from "../cx";

export interface SeatsMeterProps {
  taken: number;
  capacity: number;
  /** Word after the numbers. Defaults to "места". */
  unit?: string;
  className?: string;
}

const WARN_RATIO = 0.8;

export function SeatsMeter({ taken, capacity, unit = "места", className }: SeatsMeterProps) {
  const cap = Math.max(0, capacity);
  const ratio = cap === 0 ? 1 : Math.min(1, Math.max(0, taken / cap));
  const level = ratio >= 1 ? "full" : ratio >= WARN_RATIO ? "warn" : "ok";
  return (
    <div className={cx("uc-seats", className)} data-level={level}>
      <span className="uc-seats-label">
        {taken} / {cap} {unit}
      </span>
      <div
        className="uc-seats-track"
        role="meter"
        aria-valuemin={0}
        aria-valuemax={cap}
        aria-valuenow={Math.min(taken, cap)}
        aria-label={`${taken} от ${cap} ${unit}`}
      >
        <span className="uc-seats-fill" style={{ transform: `scaleX(${ratio})` }} />
      </div>
    </div>
  );
}
