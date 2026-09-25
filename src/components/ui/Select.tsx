import {
  Children,
  isValidElement,
  useEffect,
  useRef,
  useState,
  type OptionHTMLAttributes,
  type SelectHTMLAttributes,
} from "react";

type SelectProps = Omit<SelectHTMLAttributes<HTMLSelectElement>, "className"> & {
  className?: string;
  filterable?: boolean;
  filterPlaceholder?: string;
  filterAriaLabel?: string;
  filterEmptyLabel?: string;
};

export type SelectOption = {
  disabled: boolean;
  label: string;
  title?: string;
  value: string;
};

export function filterSelectOptions(options: SelectOption[], searchText: string): SelectOption[] {
  const normalizedSearch = searchText.trim().toLocaleLowerCase();
  if (!normalizedSearch) return options;
  return options.filter((option) =>
    `${option.label} ${option.value}`.toLocaleLowerCase().includes(normalizedSearch),
  );
}

function getOptions(children: SelectProps["children"]): SelectOption[] {
  return Children.toArray(children).flatMap((child) => {
    if (!isValidElement<OptionHTMLAttributes<HTMLOptionElement>>(child) || child.type !== "option") return [];

    return [{
      disabled: child.props.disabled ?? false,
      label: String(child.props.children ?? ""),
      title: child.props.title?.toString(),
      value: String(child.props.value ?? child.props.children ?? ""),
    }];
  });
}

export function Select({
  className,
  children,
  disabled = false,
  id,
  value,
  defaultValue,
  "aria-label": ariaLabel,
  "aria-labelledby": ariaLabelledBy,
  filterable = false,
  filterPlaceholder = "",
  filterAriaLabel = "",
  filterEmptyLabel = "",
  ...props
}: SelectProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const selectRef = useRef<HTMLSelectElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [filterText, setFilterText] = useState("");
  const options = getOptions(children);
  const selectedValues = Array.isArray(value)
    ? value.map(String)
    : value === undefined
      ? (Array.isArray(defaultValue) ? defaultValue.map(String) : defaultValue === undefined ? [] : [String(defaultValue)])
      : [String(value)];
  const displayValues = selectedValues.length === 0 && props.multiple
    ? [options[0]?.value ?? ""]
    : selectedValues;
  const selectedValue = String(value ?? defaultValue ?? options[0]?.value ?? "");
  const selectedOption = options.find((option) => option.value === selectedValue) ?? options[0];
  const selectedLabel = props.multiple && displayValues.some((item) => item !== "")
    ? options.filter((option) => displayValues.includes(option.value) && option.value !== "").map((option) => option.label).join(", ")
    : selectedOption?.label;
  const visibleOptions = filterable ? filterSelectOptions(options, filterText) : options;

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

  useEffect(() => {
    if (!isOpen) setFilterText("");
  }, [isOpen]);

  const selectOption = (nextValue: string) => {
    if (selectRef.current) {
      if (props.multiple) {
        const currentValues = displayValues.filter((item) => item !== "");
        const nextValues = nextValue === ""
          ? []
          : currentValues.includes(nextValue)
            ? currentValues.filter((item) => item !== nextValue)
            : [...currentValues, nextValue];
        Array.from(selectRef.current.options).forEach((option) => {
          option.selected = nextValues.length === 0
            ? option.value === ""
            : nextValues.includes(option.value);
        });
      } else {
        selectRef.current.value = nextValue;
      }
      selectRef.current.dispatchEvent(new Event("change", { bubbles: true }));
    }
    if (!props.multiple) {
      setIsOpen(false);
      setFilterText("");
    }
  };

  return (
    <div ref={containerRef} className={`ui-select${className ? ` ${className}` : ""}`}>
      <select
        ref={selectRef}
        {...props}
        aria-label={ariaLabel}
        aria-labelledby={ariaLabelledBy}
        value={value}
        defaultValue={defaultValue}
        disabled={disabled}
        className="ui-select-native"
        tabIndex={-1}
        aria-hidden="true"
      >
        {children}
      </select>
      <button
        type="button"
        id={id}
        aria-label={ariaLabel}
        aria-labelledby={ariaLabelledBy}
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
        <span className="ui-select-trigger-label">{selectedLabel}</span>
        <span className="ui-select-chevron" aria-hidden="true" />
      </button>
      {isOpen ? (
        <div className="ui-select-menu">
          {filterable ? (
            <input
              className="ui-select-filter"
              type="search"
              value={filterText}
              placeholder={filterPlaceholder}
              aria-label={filterAriaLabel || ariaLabel}
              onChange={(event) => setFilterText(event.currentTarget.value)}
              onClick={(event) => event.stopPropagation()}
              onKeyDown={(event) => {
                if (event.key === "Escape") {
                  event.preventDefault();
                  setIsOpen(false);
                }
              }}
              autoFocus
            />
          ) : null}
          <div role="listbox" aria-label={ariaLabel}>
            {visibleOptions.map((option) => (
              <button
                key={option.value}
                type="button"
                role="option"
                aria-selected={displayValues.includes(option.value)}
                className={`ui-select-option${displayValues.includes(option.value) ? " ui-select-option--selected" : ""}`}
                title={option.title}
                disabled={option.disabled}
                onClick={() => selectOption(option.value)}
              >
                {option.label}
              </button>
            ))}
            {filterable && filterText.trim() && visibleOptions.length === 0 && filterEmptyLabel ? (
              <span className="ui-select-empty">{filterEmptyLabel}</span>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  );
}
