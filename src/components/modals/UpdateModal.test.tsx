import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import UpdateModal from "./UpdateModal";
import { UseUpdateCheckReturn } from "../../hooks/useUpdateCheck";

const createMockUpdateState = (
  overrides: Partial<UseUpdateCheckReturn> = {}
): UseUpdateCheckReturn => ({
  available: {
    version: "2.0.0",
    body: "## Changes\n- feat: New feature\n- fix: Bug fix",
    downloadAndInstall: vi.fn(),
  },
  checking: false,
  downloading: false,
  progress: 0,
  error: null,
  checkForUpdates: vi.fn(),
  installUpdate: vi.fn(),
  dismissError: vi.fn(),
  ...overrides,
});

describe("UpdateModal", () => {
  it("should not render when closed", () => {
    const updateState = createMockUpdateState();

    render(
      <UpdateModal isOpen={false} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.queryByText("Update Available")).not.toBeInTheDocument();
  });

  it("should not render without available update", () => {
    const updateState = createMockUpdateState({ available: null });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.queryByText("Update Available")).not.toBeInTheDocument();
  });

  it("should render when open with available update", () => {
    const updateState = createMockUpdateState();

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("Update Available")).toBeInTheDocument();
    expect(screen.getByText("Version 2.0.0")).toBeInTheDocument();
  });

  it("should display version info", () => {
    const updateState = createMockUpdateState({
      available: {
        version: "3.5.0",
        body: "Some changes",
        downloadAndInstall: vi.fn(),
      },
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("Version 3.5.0")).toBeInTheDocument();
  });

  it("should display changelog body in pre element", () => {
    const updateState = createMockUpdateState({
      available: {
        version: "2.0.0",
        body: "This is the changelog content",
        downloadAndInstall: vi.fn(),
      },
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    // The body appears in a pre element
    const preElement = screen.getByText("This is the changelog content", {
      selector: "pre",
    });
    expect(preElement).toBeInTheDocument();
  });

  it("should parse changelog summary - features and fixes", () => {
    const updateState = createMockUpdateState({
      available: {
        version: "2.0.0",
        body: "- feat: Feature 1\n- feat: Feature 2\n- fix: Bug fix",
        downloadAndInstall: vi.fn(),
      },
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("2 new features, 1 bug fix")).toBeInTheDocument();
  });

  it("should parse changelog summary - only features", () => {
    const updateState = createMockUpdateState({
      available: {
        version: "2.0.0",
        body: "- feat: Feature 1",
        downloadAndInstall: vi.fn(),
      },
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("1 new feature")).toBeInTheDocument();
  });

  it("should parse changelog summary - only fixes", () => {
    const updateState = createMockUpdateState({
      available: {
        version: "2.0.0",
        body: "- fix: Bug 1\n- fix: Bug 2",
        downloadAndInstall: vi.fn(),
      },
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("2 bug fixes")).toBeInTheDocument();
  });

  it("should show default summary when no features or fixes detected", () => {
    const updateState = createMockUpdateState({
      available: {
        version: "2.0.0",
        body: undefined,
        downloadAndInstall: vi.fn(),
      },
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("New version available")).toBeInTheDocument();
  });

  it("should call onClose when Later button is clicked", () => {
    const onClose = vi.fn();
    const updateState = createMockUpdateState();

    render(
      <UpdateModal isOpen={true} onClose={onClose} updateState={updateState} />
    );

    fireEvent.click(screen.getByText("Later"));

    expect(onClose).toHaveBeenCalled();
  });

  it("should call onClose when X button is clicked", () => {
    const onClose = vi.fn();
    const updateState = createMockUpdateState();

    render(
      <UpdateModal isOpen={true} onClose={onClose} updateState={updateState} />
    );

    fireEvent.click(screen.getByText("×"));

    expect(onClose).toHaveBeenCalled();
  });

  it("should call onClose when backdrop is clicked", () => {
    const onClose = vi.fn();
    const updateState = createMockUpdateState();

    render(
      <UpdateModal isOpen={true} onClose={onClose} updateState={updateState} />
    );

    // Click the backdrop (first div with bg-black/60)
    const backdrop = document.querySelector(".bg-black\\/60");
    if (backdrop) {
      fireEvent.click(backdrop);
    }

    expect(onClose).toHaveBeenCalled();
  });

  it("should call installUpdate when Install button is clicked", () => {
    const installUpdate = vi.fn();
    const updateState = createMockUpdateState({ installUpdate });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    fireEvent.click(screen.getByText("Install Update"));

    expect(installUpdate).toHaveBeenCalled();
  });

  it("should show progress bar during download", () => {
    const updateState = createMockUpdateState({
      downloading: true,
      progress: 45,
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.getByText("Downloading update...")).toBeInTheDocument();
    expect(screen.getByText("45%")).toBeInTheDocument();
  });

  it("should show installing message at 100% progress", () => {
    const updateState = createMockUpdateState({
      downloading: true,
      progress: 100,
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(
      screen.getByText("Installing... App will restart shortly.")
    ).toBeInTheDocument();
  });

  it("should hide action buttons during download", () => {
    const updateState = createMockUpdateState({
      downloading: true,
      progress: 50,
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.queryByText("Later")).not.toBeInTheDocument();
    expect(screen.queryByText("Install Update")).not.toBeInTheDocument();
  });

  it("should hide close button during download", () => {
    const updateState = createMockUpdateState({
      downloading: true,
      progress: 50,
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(screen.queryByText("×")).not.toBeInTheDocument();
  });

  it("should display error message when error exists", () => {
    const updateState = createMockUpdateState({
      error: "Download failed: Network error",
    });

    render(
      <UpdateModal isOpen={true} onClose={vi.fn()} updateState={updateState} />
    );

    expect(
      screen.getByText("Download failed: Network error")
    ).toBeInTheDocument();
  });

  it("should not close on backdrop click during download", () => {
    const onClose = vi.fn();
    const updateState = createMockUpdateState({
      downloading: true,
      progress: 50,
    });

    render(
      <UpdateModal isOpen={true} onClose={onClose} updateState={updateState} />
    );

    const backdrop = document.querySelector(".bg-black\\/60");
    if (backdrop) {
      fireEvent.click(backdrop);
    }

    expect(onClose).not.toHaveBeenCalled();
  });

  it("should close on Escape key when not downloading", () => {
    const onClose = vi.fn();
    const updateState = createMockUpdateState();

    render(
      <UpdateModal isOpen={true} onClose={onClose} updateState={updateState} />
    );

    fireEvent.keyDown(
      screen.getByText("Update Available").parentElement!.parentElement!,
      {
        key: "Escape",
      }
    );

    expect(onClose).toHaveBeenCalled();
  });

  it("should not close on Escape key during download", () => {
    const onClose = vi.fn();
    const updateState = createMockUpdateState({
      downloading: true,
      progress: 50,
    });

    render(
      <UpdateModal isOpen={true} onClose={onClose} updateState={updateState} />
    );

    fireEvent.keyDown(
      screen.getByText("Update Available").parentElement!.parentElement!,
      {
        key: "Escape",
      }
    );

    expect(onClose).not.toHaveBeenCalled();
  });
});
