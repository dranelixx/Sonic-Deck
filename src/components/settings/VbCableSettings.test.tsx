import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import VbCableSettings from "./VbCableSettings";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock contexts
const mockSaveSettings = vi.fn();
const mockRefreshDevices = vi.fn();

vi.mock("../../contexts/SettingsContext", () => ({
  useSettings: () => ({
    settings: {
      microphone_routing_device_id: null,
      microphone_routing_enabled: false,
      broadcast_device_id: null,
    },
    saveSettings: mockSaveSettings,
  }),
}));

vi.mock("../../contexts/AudioContext", () => ({
  useAudio: () => ({
    refreshDevices: mockRefreshDevices,
  }),
}));

describe("VbCableSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("when VB-Cable is not installed", () => {
    beforeEach(() => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({ status: "not_installed" });
        }
        return Promise.resolve(null);
      });
    });

    it("renders install button", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Install VB-Cable")).toBeInTheDocument();
      });
    });

    it("renders manual download button", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Manual Download")).toBeInTheDocument();
      });
    });

    it("renders donationware notice", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(
          screen.getByText(/VB-Cable is donationware/)
        ).toBeInTheDocument();
      });
    });

    it("calls check_vb_cable_status on mount", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("check_vb_cable_status");
      });
    });
  });

  describe("when VB-Cable is installed", () => {
    beforeEach(() => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({
            status: "installed",
            info: { output_device: "CABLE Input" },
          });
        }
        if (cmd === "list_microphones") {
          return Promise.resolve([
            ["mic-1", "Microphone 1"],
            ["mic-2", "Microphone 2"],
          ]);
        }
        if (cmd === "get_microphone_routing_status") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("shows installed status", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("VB-Cable is installed")).toBeInTheDocument();
      });
    });

    it("shows device name", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText(/CABLE Input/)).toBeInTheDocument();
      });
    });

    it("renders microphone dropdown", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Select microphone...")).toBeInTheDocument();
      });
    });

    it("renders uninstall button", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        const buttons = screen.getAllByText("Uninstall VB-Cable");
        // Should have heading and button
        expect(buttons.length).toBe(2);
        // One should be a button
        expect(
          buttons.some((el) => el.tagName.toLowerCase() === "button")
        ).toBe(true);
      });
    });

    it("renders CABLE In 16 Ch help guide", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(
          screen.getByText('Tip: Hide "CABLE In 16 Ch"')
        ).toBeInTheDocument();
      });
    });

    it("lists available microphones", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Microphone 1")).toBeInTheDocument();
        expect(screen.getByText("Microphone 2")).toBeInTheDocument();
      });
    });
  });

  describe("microphone routing", () => {
    beforeEach(() => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({
            status: "installed",
            info: { output_device: "CABLE Input" },
          });
        }
        if (cmd === "list_microphones") {
          return Promise.resolve([["mic-1", "Test Microphone"]]);
        }
        if (cmd === "get_microphone_routing_status") {
          return Promise.resolve(null);
        }
        if (cmd === "enable_microphone_routing") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("enable button is disabled when no microphone selected", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        const enableButton = screen.getByRole("button", { name: "Enable" });
        expect(enableButton).toBeDisabled();
      });
    });

    it("enable button is enabled when microphone is selected", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Test Microphone")).toBeInTheDocument();
      });

      const select = screen.getByRole("combobox");
      fireEvent.change(select, { target: { value: "mic-1" } });

      const enableButton = screen.getByRole("button", { name: "Enable" });
      expect(enableButton).not.toBeDisabled();
    });

    it("calls enable_microphone_routing when enable clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Test Microphone")).toBeInTheDocument();
      });

      const select = screen.getByRole("combobox");
      fireEvent.change(select, { target: { value: "mic-1" } });

      const enableButton = screen.getByRole("button", { name: "Enable" });
      fireEvent.click(enableButton);

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("enable_microphone_routing", {
          microphoneId: "mic-1",
        });
      });
    });
  });

  describe("open sound settings", () => {
    beforeEach(() => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({
            status: "installed",
            info: { output_device: "CABLE Input" },
          });
        }
        if (cmd === "list_microphones") {
          return Promise.resolve([]);
        }
        if (cmd === "get_microphone_routing_status") {
          return Promise.resolve(null);
        }
        if (cmd === "open_sound_settings") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("calls open_sound_settings when link clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Open Sound Settings")).toBeInTheDocument();
      });

      const link = screen.getByText("Open Sound Settings");
      fireEvent.click(link);

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("open_sound_settings");
      });
    });
  });
});
