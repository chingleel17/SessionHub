import { Children, isValidElement, useEffect, useRef, useState, type OptionHTMLAttributes, type SelectHTMLAttributes } from "react";

type SelectProps = SelectHTMLAttributes<HTMLSelectElement>;

type SelectOption = {
  disabled: boolean;
  label: string;
  value: string;
};

function getOptions(children: SelectProps["children"]): SelectOption[] {
  return Children.toArray(children).flatMap((child) => {
    if (!isValidElement<OptionHTMLAttributes<HTMLOptionElement>>(child) || child.type !== "option") return [];

    return [{
      disabled: child.props.disabled ?? false,
      label: String(child.props.children ?? ""),
      value: String(child.props.value ?? child.props.children ?? ""),
    }];
  });
}

export function Select({ className, children, disabled = false, id, value, defaultValue, ...props }: SelectProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const selectRef = useRef<HTMLSelectElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  const options = getOptions(children);
  const selectedValue = String(value ?? defaultValue ?? options[0]?.value ?? "");
  const selectedOption = options.find((option) => option.value === selectedValue) ?? options[0];

  useEffect(() => {
    const handlePointerDown = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) setIsOpen(false);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setIsOpen(false);
    };

    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const selectOption = (nextValue: string) => {
    if (selectRef.current) {
      selectRef.current.value = nextValue;
      selectRef.current.dispatchEvent(new Event("change", { bubbles: true }));
    }
    setIsOpen(false);
  };

  return (
    <div ref={containerRef} className={`ui-select${className ? ` ${className}` : ""}`}>
      <select ref={selectRef} {...props} value={value} defaultValue={defaultValue} disabled={disabled} className="ui-select-native" tabIndex={-1} aria-hidden="true">
        {children}
      </select>
      <button
        type="button"
        id={id}
        className="ui-select-trigger"
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        onClick={() => setIsOpen((open) => !open)}
        onKeyDown={(event) => {
          if (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            setIsOpen(true);
          }
        }}
      >
        <span>{selectedOption?.label}</span>
        <span className="ui-select-chevron" aria-hidden="true" />
      </button>
      {isOpen ? (
        <div className="ui-select-menu" role="listbox">
          {options.map((option) => (
            <button
              key={option.value}
              type="button"
              role="option"
              aria-selected={option.value === selectedValue}
              className={`ui-select-option${option.value === selectedValue ? " ui-select-option--selected" : ""}`}
              disabled={option.disabled}
              onClick={() => selectOption(option.value)}
            >
              {option.label}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
