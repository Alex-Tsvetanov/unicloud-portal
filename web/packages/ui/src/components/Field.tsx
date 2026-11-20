import {
  cloneElement,
  useId,
  type InputHTMLAttributes,
  type ReactElement,
  type ReactNode,
  type SelectHTMLAttributes,
  type TextareaHTMLAttributes,
} from "react";
import { cx } from "../cx";

type ControlProps = { id?: string; "aria-describedby"?: string; "aria-invalid"?: boolean };

export interface FieldProps {
  label: ReactNode;
  hint?: ReactNode;
  error?: ReactNode;
  /** One control. Field sets its id, aria-describedby and aria-invalid. */
  children: ReactElement<ControlProps>;
  className?: string;
}

export function Field({ label, hint, error, children, className }: FieldProps) {
  const auto = useId();
  const id = children.props.id ?? auto;
  const hintId = hint != null ? `${id}-hint` : undefined;
  const errId = error != null ? `${id}-err` : undefined;
  const describedBy = [hintId, errId].filter(Boolean).join(" ") || undefined;
  return (
    <div className={cx("uc-field", className)}>
      <label className="uc-label" htmlFor={id}>{label}</label>
      {cloneElement(children, {
        id,
        "aria-describedby": describedBy,
        "aria-invalid": error != null ? true : undefined,
      })}
      {hint != null && <span id={hintId} className="uc-hint">{hint}</span>}
      {error != null && <span id={errId} className="uc-field-error">{error}</span>}
    </div>
  );
}

export function TextInput({ className, ...rest }: InputHTMLAttributes<HTMLInputElement>) {
  return <input className={cx("uc-input", className)} {...rest} />;
}

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  options?: SelectOption[];
}

export function Select({ className, options, children, ...rest }: SelectProps) {
  return (
    <select className={cx("uc-select", className)} {...rest}>
      {options?.map((o) => (
        <option key={o.value} value={o.value}>{o.label}</option>
      ))}
      {children}
    </select>
  );
}

export function TextArea({ className, ...rest }: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return <textarea className={cx("uc-textarea", className)} {...rest} />;
}

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label: ReactNode;
}

export function Checkbox({ label, className, ...rest }: CheckboxProps) {
  return (
    <label className={cx("uc-check", className)}>
      <input type="checkbox" {...rest} />
      <span>{label}</span>
    </label>
  );
}
