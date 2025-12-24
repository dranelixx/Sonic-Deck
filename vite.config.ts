import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import fs from "fs";
import path from "path";

// Load version from version.json
const versionFile = path.join(__dirname, "../version.json");
let versionData = { version: "0.2.0-alpha" };
try {
  versionData = JSON.parse(fs.readFileSync(versionFile, "utf-8"));
} catch (err) {
  console.warn(`⚠️  Warning: Could not read version.json, using default version`);
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  define: {
    "import.meta.env.VITE_APP_VERSION": JSON.stringify(versionData.version),
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
