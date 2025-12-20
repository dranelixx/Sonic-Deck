/**
 * Convert technical hotkey strings to user-friendly display format
 */
export function formatHotkeyForDisplay(hotkey: string): string {
  return hotkey
    .split("+")
    .map((part) => {
      const trimmed = part.trim();

      // Format modifier keys
      switch (trimmed) {
        case "Ctrl":
          return "Ctrl";
        case "Alt":
          return "Alt";
        case "Shift":
          return "Shift";
        case "Super":
          return "Win";
        case "Meta":
          return "Cmd";
      }

      // Format NumPad keys
      if (trimmed.startsWith("NumPad")) {
        const number = trimmed.replace("NumPad", "");
        switch (number) {
          case "0":
            return "Numpad 0";
          case "1":
            return "Numpad 1";
          case "2":
            return "Numpad 2";
          case "3":
            return "Numpad 3";
          case "4":
            return "Numpad 4";
          case "5":
            return "Numpad 5";
          case "6":
            return "Numpad 6";
          case "7":
            return "Numpad 7";
          case "8":
            return "Numpad 8";
          case "9":
            return "Numpad 9";
          case "Decimal":
            return "Numpad ,";
          case "Enter":
            return "Numpad Enter";
          case "Add":
            return "Numpad +";
          case "Subtract":
            return "Numpad -";
          case "Multiply":
            return "Numpad *";
          case "Divide":
            return "Numpad /";
          default:
            return `Numpad ${number}`;
        }
      }

      // Format function keys
      if (trimmed.match(/^F\d+$/)) {
        return trimmed; // F1, F2, etc. stay as-is
      }

      // Format arrow keys
      switch (trimmed) {
        case "ArrowUp":
          return "↑";
        case "ArrowDown":
          return "↓";
        case "ArrowLeft":
          return "←";
        case "ArrowRight":
          return "→";
      }

      // Format special keys
      switch (trimmed) {
        case "Space":
          return "Space";
        case "Enter":
          return "Enter";
        case "Tab":
          return "Tab";
        case "Escape":
          return "Esc";
        case "Backspace":
          return "Backspace";
        case "Delete":
          return "Delete";
        case "Home":
          return "Home";
        case "End":
          return "End";
        case "PageUp":
          return "Page Up";
        case "PageDown":
          return "Page Down";
        case "Insert":
          return "Insert";
      }

      // Capitalize single letters (a -> A, k -> K, etc.)
      if (trimmed.length === 1 && trimmed.match(/[a-z]/i)) {
        return trimmed.toUpperCase();
      }

      // Everything else stays as-is (numbers, etc.)
      return trimmed;
    })
    .join(" + ");
}

/**
 * Convert display format back to technical format (for saving)
 */
export function parseDisplayHotkey(displayHotkey: string): string {
  return displayHotkey
    .split(" + ")
    .map((part) => {
      const trimmed = part.trim();

      // Reverse format modifier keys
      switch (trimmed) {
        case "Win":
          return "Super";
        case "Cmd":
          return "Meta";
      }

      // Reverse format NumPad keys
      if (trimmed.startsWith("Numpad ")) {
        const suffix = trimmed.replace("Numpad ", "");
        switch (suffix) {
          case "0":
            return "NumPad0";
          case "1":
            return "NumPad1";
          case "2":
            return "NumPad2";
          case "3":
            return "NumPad3";
          case "4":
            return "NumPad4";
          case "5":
            return "NumPad5";
          case "6":
            return "NumPad6";
          case "7":
            return "NumPad7";
          case "8":
            return "NumPad8";
          case "9":
            return "NumPad9";
          case ",":
            return "NumPadDecimal";
          case "Enter":
            return "NumPadEnter";
          case "+":
            return "NumPadAdd";
          case "-":
            return "NumPadSubtract";
          case "*":
            return "NumPadMultiply";
          case "/":
            return "NumPadDivide";
          default:
            return `NumPad${suffix}`;
        }
      }

      // Reverse format arrow keys
      switch (trimmed) {
        case "↑":
          return "ArrowUp";
        case "↓":
          return "ArrowDown";
        case "←":
          return "ArrowLeft";
        case "→":
          return "ArrowRight";
      }

      // Reverse format special keys
      switch (trimmed) {
        case "Space":
          return "Space";
        case "Esc":
          return "Escape";
        case "Backspace":
          return "Backspace";
        case "Delete":
          return "Delete";
        case "Home":
          return "Home";
        case "End":
          return "End";
        case "Page Up":
          return "PageUp";
        case "Page Down":
          return "PageDown";
        case "Insert":
          return "Insert";
      }

      // Everything else stays as-is
      return trimmed;
    })
    .join("+");
}
