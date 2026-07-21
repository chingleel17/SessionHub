import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "ghost" | "danger";
  loading?: boolean;
  children: ReactNode;
};

export function Button({ variant = "secondary", loading = false, disabled = false, className, children, ...props }: ButtonProps) {
  return (
    <button
      {...props}
      type={props.type ?? "button"}
      className={`ui-button ui-button--${variant}${className ? ` ${className}` : ""}`}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
    >
      {loading ? <span className="ui-button-spinner" aria-hidden="true" /> : null}
      {children}
    </button>
  );
}
