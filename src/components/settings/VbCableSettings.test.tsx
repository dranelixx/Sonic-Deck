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

  describe("disable routing", () => {
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
          return Promise.resolve("mic-1"); // Routing is active
        }
        if (cmd === "disable_microphone_routing") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("shows disable button when routing is active", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Disable" })
        ).toBeInTheDocument();
      });
    });

    it("shows routing active message", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(
          screen.getByText("Microphone is being routed to CABLE Input")
        ).toBeInTheDocument();
      });
    });

    it("calls disable_microphone_routing when disable clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Disable" })
        ).toBeInTheDocument();
      });

      const disableButton = screen.getByRole("button", { name: "Disable" });
      fireEvent.click(disableButton);

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("disable_microphone_routing");
      });
    });

    it("saves settings after disabling", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Disable" })
        ).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole("button", { name: "Disable" }));

      await waitFor(() => {
        expect(mockSaveSettings).toHaveBeenCalledWith(
          expect.objectContaining({
            microphone_routing_enabled: false,
          })
        );
      });
    });
  });

  describe("install flow", () => {
    beforeEach(() => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({ status: "not_installed" });
        }
        if (cmd === "save_all_default_devices") {
          return Promise.resolve({
            render_console: "device-1",
            render_communications: null,
            capture_console: null,
            capture_communications: null,
          });
        }
        if (cmd === "start_vb_cable_install") {
          return Promise.resolve(null);
        }
        if (cmd === "wait_for_vb_cable_device") {
          return Promise.resolve("CABLE Input");
        }
        if (cmd === "restore_all_default_devices") {
          return Promise.resolve({
            restored_count: 4,
            failed_count: 0,
            failures: [],
          });
        }
        if (cmd === "list_audio_devices") {
          return Promise.resolve([{ id: "cable-1", name: "CABLE Input" }]);
        }
        if (cmd === "cleanup_vb_cable_install") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("shows installing state when install clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Install VB-Cable")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText("Install VB-Cable"));

      await waitFor(() => {
        expect(screen.getByText("Installing...")).toBeInTheDocument();
      });
    });

    it("calls save_all_default_devices before install", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Install VB-Cable")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText("Install VB-Cable"));

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("save_all_default_devices");
      });
    });

    it("calls start_vb_cable_install during install", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Install VB-Cable")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText("Install VB-Cable"));

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("start_vb_cable_install");
      });
    });
  });

  describe("uninstall flow", () => {
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
        if (cmd === "start_vb_cable_uninstall") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("calls start_vb_cable_uninstall when uninstall clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        const buttons = screen.getAllByText("Uninstall VB-Cable");
        expect(buttons.length).toBe(2);
      });

      const uninstallButton = screen
        .getAllByText("Uninstall VB-Cable")
        .find((el) => el.tagName.toLowerCase() === "button");
      fireEvent.click(uninstallButton!);

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("start_vb_cable_uninstall");
      });
    });

    it("shows uninstalling state", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        const buttons = screen.getAllByText("Uninstall VB-Cable");
        expect(buttons.length).toBe(2);
      });

      const uninstallButton = screen
        .getAllByText("Uninstall VB-Cable")
        .find((el) => el.tagName.toLowerCase() === "button");
      fireEvent.click(uninstallButton!);

      await waitFor(() => {
        expect(screen.getByText("Uninstalling...")).toBeInTheDocument();
      });
    });
  });

  describe("error handling", () => {
    it("shows error when status check fails", async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.reject("Network error");
        }
        return Promise.resolve(null);
      });

      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText(/Status check failed/)).toBeInTheDocument();
      });
    });

    it("shows error when routing fails", async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({
            status: "installed",
            info: { output_device: "CABLE Input" },
          });
        }
        if (cmd === "list_microphones") {
          return Promise.resolve([["mic-1", "Test Mic"]]);
        }
        if (cmd === "get_microphone_routing_status") {
          return Promise.resolve(null);
        }
        if (cmd === "enable_microphone_routing") {
          return Promise.reject("Device not found");
        }
        return Promise.resolve(null);
      });

      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Test Mic")).toBeInTheDocument();
      });

      const select = screen.getByRole("combobox");
      fireEvent.change(select, { target: { value: "mic-1" } });

      const enableButton = screen.getByRole("button", { name: "Enable" });
      fireEvent.click(enableButton);

      await waitFor(() => {
        expect(
          screen.getByText(/Microphone routing failed/)
        ).toBeInTheDocument();
      });
    });
  });

  describe("website link", () => {
    beforeEach(() => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_vb_cable_status") {
          return Promise.resolve({ status: "not_installed" });
        }
        if (cmd === "open_vb_audio_website") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("calls open_vb_audio_website when manual download clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("Manual Download")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText("Manual Download"));

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("open_vb_audio_website");
      });
    });

    it("calls open_vb_audio_website when donationware link clicked", async () => {
      render(<VbCableSettings />);

      await waitFor(() => {
        expect(screen.getByText("vb-audio.com")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText("vb-audio.com"));

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("open_vb_audio_website");
      });
    });
  });
});
