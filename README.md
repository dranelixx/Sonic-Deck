# SonicDeck 🚀

![SonicDeck Banner](httpsse://example.com/sonicdeck-banner.png) <!-- Placeholder for a future banner -->

**SonicDeck is a high-performance, open-source desktop soundboard application for Windows, macOS, and Linux, built for gamers, streamers, and audio enthusiasts.**

Inspired by the seamless experience of Discord, SonicDeck provides a powerful toolset for managing and broadcasting audio with minimal latency. It's built with a modern stack including **Tauri v2, Rust, React, and TypeScript**.

---

## ✨ Features

- **🎧 Dual-Audio Routing**: Play sounds to two separate audio devices simultaneously (e.g., your headphones and a virtual audio cable for your stream).
- **🎛️ Sound Management**: Easily organize your sounds with categories. Drag and drop `MP3`, `WAV`, and `OGG` files directly into the app.
- **✂️ Audio Editing**: Visualize and trim your audio clips with an integrated waveform editor powered by `wavesurfer.js`.
- **Hotkey Support**: Assign global hotkeys to your sounds for instant playback, even when the app is minimized.
- **Modern UI**: A sleek, Discord-inspired dark theme that's easy on the eyes.
- **High Performance & Low Latency**: Built with Rust on the backend for optimal performance and near-instant audio playback.

## 🗺️ Project Roadmap

This project is being developed in phases:

- ✅ **Phase 1 (Audio Foundation)**: Implement the core Rust backend for device listing and dual-sink playback.
- 🔲 **Phase 2 (Settings & Routing UI)**: Build the user interface for selecting monitor and broadcast audio devices.
- 🔲 **Phase 3 (Main Dashboard)**: Develop the main soundboard grid and category sidebar.
- 🔲 **Phase 4 (Editing & Sound Management)**: Integrate the waveform editor, drag-and-drop functionality, and sound persistence.
- 🔲 **Phase 5 (System Logic)**: Implement global hotkeys, a "stop-all" feature, and configuration management.

## 🚀 Getting Started

> **Note**: The project is currently in early development. The following instructions are for building from source.

### Prerequisites

- [Node.js](https://nodejs.org/en/)
- [Yarn Package Manager](https://yarnpkg.com/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation & Running

1. **Clone the repository:**

   ```sh
   git clone https://github.com/DraneLixX/SonicDeck.git
   cd SonicDeck
   ```

2. **Install frontend dependencies:**

   ```sh
   yarn install
   ```

3. **Run the development server:**

   ```sh
   yarn tauri dev
   ```

## 🤝 Contributing

Contributions are welcome! If you have ideas for new features, improvements, or bug fixes, please feel free to:

1. Fork the repository.
2. Create a new feature branch (`git checkout -b feature/AmazingFeature`).
3. Commit your changes (`git commit -m 'feat: Add some AmazingFeature'`).
4. Push to the branch (`git push origin feature/AmazingFeature`).
5. Open a Pull Request.

Please make sure your code adheres to the project's conventions and includes tests where applicable.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Built with ❤️ by [Adrian Konopczynski (DraneLixX)](https://github.com/DraneLixX)
