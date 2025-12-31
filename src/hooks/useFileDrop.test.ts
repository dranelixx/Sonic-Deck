import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { useFileDrop } from "./useFileDrop";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe("useFileDrop", () => {
  const mockShowToast = vi.fn();
  const mockOnFilesDropped = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should initialize with isDragging false", () => {
    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );
    expect(result.current.isDragging).toBe(false);
  });

  it("should setup event listeners on mount", () => {
    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );

    act(() => {
      result.current.setupFileDropListeners();
    });

    expect(listen).toHaveBeenCalledWith(
      "tauri://drag-drop",
      expect.any(Function)
    );
    expect(listen).toHaveBeenCalledWith("tauri://drag", expect.any(Function));
    expect(listen).toHaveBeenCalledWith(
      "tauri://drag-cancelled",
      expect.any(Function)
    );
  });

  it("should return drag event handlers", () => {
    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );

    expect(typeof result.current.handleDragOver).toBe("function");
    expect(typeof result.current.handleDragLeave).toBe("function");
    expect(typeof result.current.handleDrop).toBe("function");
  });

  it("should prevent default on drag over event", () => {
    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );

    const mockEvent = {
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
    } as unknown as React.DragEvent;

    act(() => {
      result.current.handleDragOver(mockEvent);
    });

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(mockEvent.stopPropagation).toHaveBeenCalled();
  });

  it("should prevent default on drag leave event", () => {
    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );

    const mockEvent = {
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
    } as unknown as React.DragEvent;

    act(() => {
      result.current.handleDragLeave(mockEvent);
    });

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(mockEvent.stopPropagation).toHaveBeenCalled();
  });

  it("should prevent default on drop event", () => {
    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );

    const mockEvent = {
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
    } as unknown as React.DragEvent;

    act(() => {
      result.current.handleDrop(mockEvent);
    });

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(mockEvent.stopPropagation).toHaveBeenCalled();
  });

  it("should return cleanup function from setupFileDropListeners", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(mockUnlisten);

    const { result } = renderHook(() =>
      useFileDrop({
        showToast: mockShowToast,
        onFilesDropped: mockOnFilesDropped,
      })
    );

    let cleanup: (() => void) | undefined;
    act(() => {
      cleanup = result.current.setupFileDropListeners();
    });

    expect(cleanup).toBeDefined();
    expect(typeof cleanup).toBe("function");
  });
});
