import { useState, useEffect } from "react";
import { fetchFromCDN, CompactEmoji } from "emojibase";

interface EmojiPickerProps {
  selectedIcon: string | null;
  onIconSelect: (icon: string | null) => void;
}

// Common emojis for sound icons - 3 rows of 8
const ICON_OPTIONS = [
  null, // No icon
  "🔊",
  "🎵",
  "🎶",
  "🎤",
  "🎸",
  "🥁",
  "🎹",
  "🎺",
  "🔔",
  "📢",
  "🗣️",
  "👏",
  "😂",
  "😱",
  "🎉",
  "💥",
  "🚀",
  "✨",
  "🔥",
  "💀",
  "👻",
  "🤖",
  "🎮",
];

export default function EmojiPicker({
  selectedIcon,
  onIconSelect,
}: EmojiPickerProps) {
  const [emojiSearch, setEmojiSearch] = useState("");
  const [emojiData, setEmojiData] = useState<CompactEmoji[]>([]);
  const [filteredEmojis, setFilteredEmojis] = useState<CompactEmoji[]>([]);

  // Load emoji data once on mount
  useEffect(() => {
    fetchFromCDN("en/compact.json")
      .then((emojis) => {
        setEmojiData(emojis as CompactEmoji[]);
      })
      .catch((err) => {
        console.error("Failed to load emoji data:", err);
      });
  }, []);

  // Filter emojis based on search
  useEffect(() => {
    if (!emojiSearch.trim()) {
      setFilteredEmojis([]);
      return;
    }

    const search = emojiSearch.toLowerCase();
    const matches = emojiData
      .filter((emoji) => {
        // Search in label (name)
        if (emoji.label?.toLowerCase().includes(search)) return true;
        // Search in tags
        if (emoji.tags?.some((tag) => tag.toLowerCase().includes(search)))
          return true;
        return false;
      })
      .slice(0, 20); // Limit to 20 results

    setFilteredEmojis(matches);
  }, [emojiSearch, emojiData]);

  return (
    <div>
      <label className="block text-sm font-medium text-discord-text-muted mb-2">
        Icon (optional)
      </label>

      {/* Show selected emoji */}
      {selectedIcon && (
        <div className="mb-3 p-4 bg-discord-dark rounded-lg flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-4xl">{selectedIcon}</span>
            <span className="text-sm text-discord-text-muted">Selected</span>
          </div>
          <button
            type="button"
            onClick={() => {
              onIconSelect(null);
              setEmojiSearch("");
            }}
            className="px-3 py-1 bg-discord-darker hover:bg-discord-darkest rounded
                     text-discord-text-muted text-sm transition-colors"
          >
            Clear
          </button>
        </div>
      )}

      {/* Quick select buttons - 3 rows */}
      <div className="flex flex-wrap gap-2 mb-3">
        {ICON_OPTIONS.map((opt, idx) => (
          <button
            key={idx}
            type="button"
            onClick={() => {
              onIconSelect(opt);
              setEmojiSearch("");
            }}
            className={`w-10 h-10 rounded-lg flex items-center justify-center
                     text-lg transition-colors
                     ${
                       selectedIcon === opt
                         ? "bg-discord-primary text-white"
                         : "bg-discord-darker hover:bg-discord-dark text-discord-text"
                     }`}
          >
            {opt || "∅"}
          </button>
        ))}
      </div>

      {/* Combined search and paste input */}
      <div className="relative">
        <input
          type="text"
          value={emojiSearch}
          onChange={(e) => {
            const value = e.target.value;
            setEmojiSearch(value);
            // If user pastes emoji directly, select it
            if (value.length <= 4 && /\p{Emoji}/u.test(value)) {
              onIconSelect(value);
            }
          }}
          placeholder="Search or paste emoji: music, 🎸, fire..."
          maxLength={50}
          className="w-full bg-discord-darker border border-discord-dark rounded
                   px-3 py-2 text-discord-text text-sm focus:outline-none
                   focus:ring-2 focus:ring-discord-primary"
        />

        {/* Search results dropdown */}
        {filteredEmojis.length > 0 && (
          <div
            className="absolute z-10 w-full mt-1 bg-discord-darker border border-discord-dark
                        rounded-lg shadow-xl max-h-48 overflow-y-auto"
          >
            {filteredEmojis.map((emoji, idx) => (
              <button
                key={idx}
                type="button"
                onClick={() => {
                  onIconSelect(emoji.unicode);
                  setEmojiSearch("");
                }}
                className="w-full px-3 py-2 text-left hover:bg-discord-dark
                         transition-colors flex items-center gap-2"
              >
                <span className="text-xl">{emoji.unicode}</span>
                <span className="text-sm text-discord-text">{emoji.label}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
