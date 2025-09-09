"use client";

import * as React from "react";
import { X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Command,
  CommandGroup,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Command as CommandPrimitive } from "cmdk";

type Option = Record<"value" | "label", string>;

export function MultiSelect({
  options,
  selected,
  onChange,
  placeholder = "Select options...",
  ...props
}: {
  options: Option[];
  selected: string[];
  onChange: React.Dispatch<React.SetStateAction<string[]>>;
  placeholder?: string;
  className?: string;
}) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const [isOpen, setIsOpen] = React.useState(false);
  const [query, setQuery] = React.useState("");

  // Memoize the selected options for performance
  const selectedOptions = React.useMemo(
    () => options.filter((option) => selected.includes(option.value)),
    [options, selected]
  );

  // Handle unselecting an option
  const unSelect = React.useCallback(
    (value: string) => {
      onChange(selected.filter((s) => s !== value));
    },
    [onChange, selected]
  );

  // Handle keyboard events
  const handleKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      const input = inputRef.current;
      if (input) {
        if (e.key === "Delete" || e.key === "Backspace") {
          if (input.value === "" && selected.length > 0) {
            unSelect(selected[selected.length - 1]);
          }
        }
        if (e.key === "Escape") {
          input.blur();
        }
      }
    },
    [selected, unSelect]
  );

  const filteredOptions = options.filter(
    (option) =>
      !selected.includes(option.value) &&
      option.label.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <CommandPrimitive
      onKeyDown={handleKeyDown}
      className="overflow-visible bg-transparent"
    >
      <div className="group rounded-md border border-input px-3 py-2 text-sm ring-offset-background focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2">
        <div className="flex flex-wrap gap-1">
          {selectedOptions.map(({ value, label }) => {
            return (
              <Badge key={value} variant="secondary">
                {label}
                <button
                  className="ml-1 rounded-full outline-none ring-offset-background focus:ring-2 focus:ring-ring focus:ring-offset-2"
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      unSelect(value);
                    }
                  }}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                  }}
                  onClick={() => unSelect(value)}
                >
                  <X className="h-3 w-3 text-muted-foreground hover:text-foreground" />
                </button>
              </Badge>
            );
          })}
          <CommandPrimitive.Input
            ref={inputRef}
            value={query}
            onValueChange={setQuery}
            onBlur={() => setIsOpen(false)}
            onFocus={() => setIsOpen(true)}
            placeholder={placeholder}
            className="ml-2 flex-1 bg-transparent outline-none placeholder:text-muted-foreground"
          />
        </div>
      </div>
      <div className="relative mt-2">
        {isOpen && filteredOptions.length > 0 ? (
          <div className="absolute top-0 z-10 w-full rounded-md border bg-popover text-popover-foreground shadow-md outline-none animate-in">
            <CommandList>
              <CommandGroup>
                {filteredOptions.map((option) => {
                  return (
                    <CommandItem
                      key={option.value}
                      onMouseDown={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                      }}
                      onSelect={() => {
                        setQuery("");
                        onChange((prev) => [...prev, option.value]);
                      }}
                      className={"cursor-pointer"}
                    >
                      {option.label}
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            </CommandList>
          </div>
        ) : null}
      </div>
    </CommandPrimitive>
  );
}
