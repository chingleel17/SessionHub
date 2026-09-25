import type { InputHTMLAttributes, ReactNode } from "react";

type CheckboxProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & {
  children: ReactNode;
  className?: string;
};

export function Checkbox({ children, className, ...inputProps }: CheckboxProps) {
  return (
    <label className={`ui-checkbox${className ? ` ${className}` : ""}`}>
      <input {...inputProps} type="checkbox" />
      <span>{children}</span>
    </label>
  );
}
