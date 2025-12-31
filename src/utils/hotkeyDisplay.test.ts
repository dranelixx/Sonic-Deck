import { describe, it, expect } from "vitest";
import { formatHotkeyForDisplay, parseDisplayHotkey } from "./hotkeyDisplay";

describe("formatHotkeyForDisplay", () => {
  describe("modifier keys", () => {
    it("should format Ctrl key", () => {
      expect(formatHotkeyForDisplay("Ctrl")).toBe("Ctrl");
    });

    it("should format Alt key", () => {
      expect(formatHotkeyForDisplay("Alt")).toBe("Alt");
    });

    it("should format Shift key", () => {
      expect(formatHotkeyForDisplay("Shift")).toBe("Shift");
    });

    it("should format Super as Win", () => {
      expect(formatHotkeyForDisplay("Super")).toBe("Win");
    });

    it("should format Meta as Cmd", () => {
      expect(formatHotkeyForDisplay("Meta")).toBe("Cmd");
    });
  });

  describe("combined hotkeys", () => {
    it("should format Ctrl+A", () => {
      expect(formatHotkeyForDisplay("Ctrl+A")).toBe("Ctrl + A");
    });

    it("should format Ctrl+Shift+A", () => {
      expect(formatHotkeyForDisplay("Ctrl+Shift+A")).toBe("Ctrl + Shift + A");
    });

    it("should format Alt+F4", () => {
      expect(formatHotkeyForDisplay("Alt+F4")).toBe("Alt + F4");
    });
  });

  describe("numpad keys", () => {
    it("should format NumPad0", () => {
      expect(formatHotkeyForDisplay("NumPad0")).toBe("Numpad 0");
    });

    it("should format NumPad9", () => {
      expect(formatHotkeyForDisplay("NumPad9")).toBe("Numpad 9");
    });

    it("should format NumPadDecimal", () => {
      expect(formatHotkeyForDisplay("NumPadDecimal")).toBe("Numpad ,");
    });

    it("should format NumPadEnter", () => {
      expect(formatHotkeyForDisplay("NumPadEnter")).toBe("Numpad Enter");
    });

    it("should format NumPadAdd", () => {
      expect(formatHotkeyForDisplay("NumPadAdd")).toBe("Numpad +");
    });

    it("should format NumPadSubtract", () => {
      expect(formatHotkeyForDisplay("NumPadSubtract")).toBe("Numpad -");
    });

    it("should format NumPadMultiply", () => {
      expect(formatHotkeyForDisplay("NumPadMultiply")).toBe("Numpad *");
    });

    it("should format NumPadDivide", () => {
      expect(formatHotkeyForDisplay("NumPadDivide")).toBe("Numpad /");
    });
  });

  describe("arrow keys", () => {
    it("should format ArrowUp as ↑", () => {
      expect(formatHotkeyForDisplay("ArrowUp")).toBe("↑");
    });

    it("should format ArrowDown as ↓", () => {
      expect(formatHotkeyForDisplay("ArrowDown")).toBe("↓");
    });

    it("should format ArrowLeft as ←", () => {
      expect(formatHotkeyForDisplay("ArrowLeft")).toBe("←");
    });

    it("should format ArrowRight as →", () => {
      expect(formatHotkeyForDisplay("ArrowRight")).toBe("→");
    });
  });

  describe("special keys", () => {
    it("should format Escape as Esc", () => {
      expect(formatHotkeyForDisplay("Escape")).toBe("Esc");
    });

    it("should format PageUp as Page Up", () => {
      expect(formatHotkeyForDisplay("PageUp")).toBe("Page Up");
    });

    it("should format PageDown as Page Down", () => {
      expect(formatHotkeyForDisplay("PageDown")).toBe("Page Down");
    });
  });

  describe("letters", () => {
    it("should capitalize lowercase letters", () => {
      expect(formatHotkeyForDisplay("a")).toBe("A");
      expect(formatHotkeyForDisplay("z")).toBe("Z");
    });

    it("should keep uppercase letters", () => {
      expect(formatHotkeyForDisplay("A")).toBe("A");
    });
  });

  describe("function keys", () => {
    it("should keep F1-F12 as-is", () => {
      expect(formatHotkeyForDisplay("F1")).toBe("F1");
      expect(formatHotkeyForDisplay("F12")).toBe("F12");
    });
  });
});

describe("parseDisplayHotkey", () => {
  describe("roundtrip", () => {
    it("should roundtrip Ctrl+A", () => {
      const original = "Ctrl+A";
      const display = formatHotkeyForDisplay(original);
      const parsed = parseDisplayHotkey(display);
      expect(parsed).toBe(original);
    });

    it("should roundtrip Ctrl+Shift+F5", () => {
      const original = "Ctrl+Shift+F5";
      const display = formatHotkeyForDisplay(original);
      const parsed = parseDisplayHotkey(display);
      expect(parsed).toBe(original);
    });

    it("should roundtrip Super key", () => {
      const original = "Super+A";
      const display = formatHotkeyForDisplay(original);
      expect(display).toBe("Win + A");
      const parsed = parseDisplayHotkey(display);
      expect(parsed).toBe(original);
    });

    it("should roundtrip NumPad5", () => {
      const original = "NumPad5";
      const display = formatHotkeyForDisplay(original);
      const parsed = parseDisplayHotkey(display);
      expect(parsed).toBe(original);
    });

    it("should roundtrip arrow keys", () => {
      const original = "Ctrl+ArrowUp";
      const display = formatHotkeyForDisplay(original);
      expect(display).toBe("Ctrl + ↑");
      const parsed = parseDisplayHotkey(display);
      expect(parsed).toBe(original);
    });
  });

  describe("special parsing", () => {
    it("should parse Win as Super", () => {
      expect(parseDisplayHotkey("Win + A")).toBe("Super+A");
    });

    it("should parse Cmd as Meta", () => {
      expect(parseDisplayHotkey("Cmd + A")).toBe("Meta+A");
    });

    it("should parse Esc as Escape", () => {
      expect(parseDisplayHotkey("Esc")).toBe("Escape");
    });

    it("should parse Page Up as PageUp", () => {
      expect(parseDisplayHotkey("Page Up")).toBe("PageUp");
    });

    it("should parse Page Down as PageDown", () => {
      expect(parseDisplayHotkey("Page Down")).toBe("PageDown");
    });
  });
});
