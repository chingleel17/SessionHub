import type { ButtonHTMLAttributes, ReactNode } from "react";

type IconButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  label: string;
  children: ReactNode;
  danger?: boolean;
};

export function IconButton({ label, danger = false, className, children, ...props }: IconButtonProps) {
  return (
    <button
      {...props}
      type={props.type ?? "button"}
      className={`ui-icon-button${danger ? " ui-icon-button--danger" : ""}${className ? ` ${className}` : ""}`}
      aria-label={label}
      title={label}
    >
      {children}
    </button>
  );
}
