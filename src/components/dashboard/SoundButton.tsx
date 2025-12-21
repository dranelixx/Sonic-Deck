import { useEffect, useState } from "react";
import { Sound, HotkeyMapping } from "../../types";
import HotkeyManager from "../modals/HotkeyManager";
import { formatHotkeyForDisplay } from "../../utils/hotkeyDisplay";

interface SoundButtonProps {
  sound: Sound;
  isPlaying: boolean;
  onPlay: (sound: Sound) => void;
  onEdit: (sound: Sound) => void;
  onDelete: (sound: Sound) => void;
  onToggleFavorite: (sound: Sound) => void;
  onTrim: (sound: Sound) => void;
  showMenu: boolean;
  onMenuChange: (show: boolean) => void;
  hotkeyMappings: HotkeyMapping;
  onHotkeyChanged: () => void;
}

export default function SoundButton({
  sound,
  isPlaying,
  onPlay,
  onEdit,
  onDelete,
  onToggleFavorite,
  onTrim,
  showMenu,
  onMenuChange,
  hotkeyMappings,
  onHotkeyChanged,
}: SoundButtonProps) {
  const [showHotkeyManager, setShowHotkeyManager] = useState(false);

  const handleClick = () => {
    // Always call onPlay - let Dashboard handle restart logic
    onPlay(sound);
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation(); // Prevent document click listener
    onMenuChange(true);
  };

  // Close menu when clicking outside
  useEffect(() => {
    const handleClick = () => onMenuChange(false);
    if (showMenu) {
      document.addEventListener("click", handleClick);
      return () => document.removeEventListener("click", handleClick);
    }
  }, [showMenu, onMenuChange]);

  const handleEdit = (e: React.MouseEvent) => {
    e.stopPropagation();
    onMenuChange(false);
    onEdit(sound);
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    onMenuChange(false);
    onDelete(sound);
  };

  const handleToggleFavorite = (e: React.MouseEvent) => {
    e.stopPropagation();
    onMenuChange(false);
    onToggleFavorite(sound);
  };

  const handleTrim = (e: React.MouseEvent) => {
    e.stopPropagation();
    onMenuChange(false);
    onTrim(sound);
  };

  const handleAssignHotkey = (e: React.MouseEvent) => {
    e.stopPropagation();
    onMenuChange(false);
    setShowHotkeyManager(true);
  };

  // Get the hotkey assigned to this sound
  const assignedHotkey = Object.entries(hotkeyMappings.mappings).find(
    ([_, soundId]) => soundId === sound.id
  )?.[0];

  return (
    <div className="relative">
      <button
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        className={`w-full h-24 rounded-lg font-medium transition-all transform
                   flex flex-col items-center justify-center gap-1 p-2
                   ${
                     isPlaying
                       ? "bg-discord-success scale-95 shadow-lg shadow-discord-success/30"
                       : "bg-discord-dark hover:bg-discord-darker hover:scale-[1.02]"
                   }
                   text-discord-text border border-discord-darker
                   focus:outline-none focus:ring-2 focus:ring-discord-primary`}
        title={`Play: ${sound.name}\nPath: ${sound.file_path}\nRight-click for options`}
      >
        {/* Icon or default */}
        <span className="text-xl">
          {sound.icon || (isPlaying ? "🔊" : "🔈")}
        </span>

        {/* Name - truncated */}
        <span className="text-xs truncate w-full px-1 text-center">
          {sound.name}
        </span>

        {/* Hotkey display */}
        {assignedHotkey && (
          <span className="text-[10px] text-discord-text-muted font-mono truncate w-full px-1 text-center">
            {formatHotkeyForDisplay(assignedHotkey)}
          </span>
        )}

        {/* Favorite star */}
        {sound.is_favorite && (
          <div className="absolute top-1 left-1 text-yellow-400 text-lg">
            ⭐
          </div>
        )}

        {/* Playing indicator */}
        {isPlaying && (
          <div className="absolute top-1 right-1 flex gap-0.5">
            <span className="w-1 h-3 bg-white rounded-full animate-pulse"></span>
            <span
              className="w-1 h-3 bg-white rounded-full animate-pulse"
              style={{ animationDelay: "0.2s" }}
            ></span>
            <span
              className="w-1 h-3 bg-white rounded-full animate-pulse"
              style={{ animationDelay: "0.4s" }}
            ></span>
          </div>
        )}
      </button>

      {/* Context Menu */}
      {showMenu && (
        <div
          className="absolute top-full left-0 mt-1 z-50 bg-discord-darker
                     border border-discord-dark rounded-lg shadow-lg py-1 min-w-40"
          onClick={(e) => e.stopPropagation()} // Prevent closing when clicking inside menu
        >
          <button
            onClick={handleToggleFavorite}
            className="w-full px-4 py-2 text-left text-sm text-discord-text
                     hover:bg-discord-primary hover:text-white transition-colors"
          >
            {sound.is_favorite ? "Remove Favorite" : "Add to Favorites"}
          </button>
          <button
            onClick={handleEdit}
            className="w-full px-4 py-2 text-left text-sm text-discord-text
                     hover:bg-discord-primary hover:text-white transition-colors"
          >
            Edit Sound
          </button>
          <button
            onClick={handleTrim}
            className="w-full px-4 py-2 text-left text-sm text-discord-text
                     hover:bg-discord-primary hover:text-white transition-colors"
          >
            Trim Audio
          </button>
          <button
            onClick={handleAssignHotkey}
            className="w-full px-4 py-2 text-left text-sm text-discord-text
                     hover:bg-discord-primary hover:text-white transition-colors"
          >
            Assign Hotkey
          </button>
          <div className="border-t border-discord-dark my-1"></div>
          <button
            onClick={handleDelete}
            className="w-full px-4 py-2 text-left text-sm text-discord-danger
                     hover:bg-discord-danger hover:text-white transition-colors"
          >
            Delete Sound
          </button>
        </div>
      )}

      {/* Hotkey Manager Modal */}
      {showHotkeyManager && (
        <HotkeyManager
          sound={sound}
          hotkeyMappings={hotkeyMappings}
          onClose={() => setShowHotkeyManager(false)}
          onHotkeyAssigned={onHotkeyChanged}
        />
      )}
    </div>
  );
}
