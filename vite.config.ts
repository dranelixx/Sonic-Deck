import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import fs from "fs";
import path from "path";

// Load version from version.json (in project root, same directory as vite.config.ts)
const versionFile = path.join(__dirname, "version.json");
let versionData = { version: "0.0.0-dev", channel: "unknown" };
try {
  versionData = JSON.parse(fs.readFileSync(versionFile, "utf-8"));
} catch (err) {
  console.warn(`⚠️  Warning: Could not read version.json, using fallback version (0.0.0-unknown)`);
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  define: {
    "import.meta.env.VITE_APP_VERSION": JSON.stringify(versionData.version),
    "import.meta.env.VITE_APP_CHANNEL": JSON.stringify(versionData.channel),
  },

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
