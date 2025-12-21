import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Sound, Category } from "../../types";
import EmojiPicker from "../common/EmojiPicker";

interface SoundModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: () => Promise<void>;
  categories: Category[];
  sound?: Sound | null; // null = add mode, Sound = edit mode
  defaultCategoryId?: string;
  defaultFilePath?: string; // For drag & drop
}

export default function SoundModal({
  isOpen,
  onClose,
  onSave,
  categories,
  sound,
  defaultCategoryId,
  defaultFilePath,
}: SoundModalProps) {
  const [name, setName] = useState("");
  const [filePath, setFilePath] = useState("");
  const [categoryId, setCategoryId] = useState("");
  const [icon, setIcon] = useState<string | null>(null);
  const [volume, setVolume] = useState<number | null>(null);
  const [useCustomVolume, setUseCustomVolume] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isEditMode = !!sound;

  // Initialize form when modal opens
  useEffect(() => {
    if (isOpen) {
      if (sound) {
        // Edit mode - populate with existing values
        setName(sound.name);
        setFilePath(sound.file_path);
        setCategoryId(sound.category_id);
        setIcon(sound.icon);
        setVolume(sound.volume);
        setUseCustomVolume(sound.volume !== null);
      } else {
        // Add mode - use defaults
        const path = defaultFilePath || "";
        setFilePath(path);
        setCategoryId(defaultCategoryId || categories[0]?.id || "");
        setIcon(null);
        setVolume(null);
        setUseCustomVolume(false);

        // Auto-generate name from defaultFilePath if provided
        if (path) {
          const fileName = path.split(/[/\\]/).pop() || "";
          const formattedName = formatFileName(fileName);
          setName(formattedName);
        } else {
          setName("");
        }
      }
      setError(null);
    }
  }, [isOpen, sound, defaultCategoryId, defaultFilePath, categories]);

  // Smart filename formatter: kebab-case/snake_case to Title Case + Umlauts
  const formatFileName = (fileName: string): string => {
    // Remove extension
    const nameWithoutExtension = fileName.replace(/\.[^/.]+$/, "");

    // Replace dashes and underscores with spaces
    let formatted = nameWithoutExtension.replace(/[-_]/g, " ");

    // IMPORTANT: Capitalize BEFORE umlaut conversion to avoid "VerrüCkter"
    // Capitalize first letter of each word
    formatted = formatted.replace(/\b\w/g, (char) => char.toUpperCase());

    // Convert common umlaut patterns AFTER capitalization
    // Handle capitalized versions first
    formatted = formatted.replace(/Ae/g, "Ä");
    formatted = formatted.replace(/Oe/g, "Ö");
    formatted = formatted.replace(/Ue/g, "Ü");
    // Then lowercase versions
    formatted = formatted.replace(/ae/g, "ä");
    formatted = formatted.replace(/oe/g, "ö");
    formatted = formatted.replace(/ue/g, "ü");

    return formatted;
  };

  // Auto-generate name from file path
  const handleFilePathChange = (newPath: string) => {
    setFilePath(newPath);

    // Auto-fill name if empty
    if (!name && newPath) {
      const fileName = newPath.split(/[/\\]/).pop() || "";
      const formattedName = formatFileName(fileName);
      setName(formattedName);
    }
  };

  // Handle drag & drop on file path input
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const files = Array.from(e.dataTransfer.files);
    const audioFile = files.find(
      (f) =>
        f.type.startsWith("audio/") || /\.(mp3|wav|ogg|m4a|flac)$/i.test(f.name)
    );

    if (audioFile) {
      // In Tauri, we need the full path - try to get it from the path property
      const path = (audioFile as { path?: string }).path || audioFile.name;
      if ((path && path.includes("/")) || path.includes("\\")) {
        handleFilePathChange(path);
      }
    }
  };

  // Open file browser dialog
  const handleBrowseFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Audio Files",
            extensions: ["mp3", "wav", "ogg", "m4a", "flac"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        handleFilePathChange(selected);
      }
    } catch (error) {
      console.error("File dialog error:", error);
    }
  };

  const handleSubmit = async () => {
    // Validation
    if (!name.trim()) {
      setError("Please enter a name for the sound");
      return;
    }
    if (!filePath.trim()) {
      setError("Please enter the file path");
      return;
    }
    if (!categoryId) {
      setError("Please select a category");
      return;
    }

    try {
      setIsSubmitting(true);
      setError(null);

      if (isEditMode && sound) {
        // Update existing sound
        // Always send volume when editing (even if null) to support clearing
        const volumeValue = useCustomVolume ? volume : null;
        await invoke("update_sound", {
          soundId: sound.id,
          name: name.trim(),
          filePath: filePath.trim(),
          categoryId: categoryId,
          icon: icon,
          volume: volumeValue, // Always included in edit mode
        });
      } else {
        // Add new sound
        await invoke("add_sound", {
          name: name.trim(),
          filePath: filePath.trim(),
          categoryId: categoryId,
          icon: icon,
          volume: useCustomVolume ? volume : null,
        });
      }

      await onSave();
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === "Escape") {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />

      {/* Modal */}
      <div
        className="relative bg-discord-dark rounded-lg shadow-xl w-full max-w-md
                    border border-discord-darker p-6 mx-4"
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-discord-text">
            {isEditMode ? "Edit Sound" : "Add Sound"}
          </h2>
          <button
            onClick={onClose}
            className="text-discord-text-muted hover:text-discord-text
                     text-2xl leading-none transition-colors"
          >
            &times;
          </button>
        </div>

        {/* Error */}
        {error && (
          <div
            className="mb-4 p-3 bg-discord-danger/20 border border-discord-danger
                        rounded-lg text-discord-danger text-sm"
          >
            {error}
          </div>
        )}

        {/* Form */}
        <div className="space-y-4" onKeyDown={handleKeyDown}>
          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-discord-text-muted mb-2">
              Name *
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Airhorn"
              autoFocus
              className="w-full bg-discord-darker border border-discord-dark rounded
                       px-3 py-2 text-discord-text focus:outline-none
                       focus:ring-2 focus:ring-discord-primary"
            />
          </div>

          {/* File Path */}
          <div>
            <label className="block text-sm font-medium text-discord-text-muted mb-2">
              File Path * (drag & drop supported)
            </label>
            <div className="relative">
              <input
                type="text"
                value={filePath}
                onChange={(e) => handleFilePathChange(e.target.value)}
                onDragOver={handleDragOver}
                onDrop={handleDrop}
                placeholder="e.g., C:/Sounds/airhorn.mp3 (or drag file here)"
                className="w-full bg-discord-darker border border-discord-dark rounded
                         pl-3 pr-12 py-2 text-discord-text focus:outline-none
                         focus:ring-2 focus:ring-discord-primary"
              />
              <button
                type="button"
                onClick={handleBrowseFile}
                className="absolute right-1 top-1/2 -translate-y-1/2
                         w-10 h-[calc(100%-8px)] bg-discord-dark hover:bg-discord-darker
                         rounded text-xl transition-colors flex items-center justify-center"
                title="Browse for audio file"
              >
                📁
              </button>
            </div>
            <p className="text-xs text-discord-text-muted mt-1">
              Supports MP3, WAV, OGG, M4A - Click 📁 or drag file here
            </p>
          </div>

          {/* Category */}
          <div>
            <label className="block text-sm font-medium text-discord-text-muted mb-2">
              Category *
            </label>
            <select
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value)}
              className="w-full bg-discord-darker border border-discord-dark rounded
                       px-3 py-2 text-discord-text focus:outline-none
                       focus:ring-2 focus:ring-discord-primary"
            >
              {categories.map((cat) => (
                <option key={cat.id} value={cat.id}>
                  {cat.icon && `${cat.icon} `}
                  {cat.name}
                </option>
              ))}
            </select>
          </div>

          {/* Icon */}
          <EmojiPicker selectedIcon={icon} onIconSelect={setIcon} />

          {/* Custom Volume */}
          <div>
            <label className="flex items-center gap-2 text-sm text-discord-text-muted mb-2 cursor-pointer">
              <input
                type="checkbox"
                checked={useCustomVolume}
                onChange={(e) => {
                  setUseCustomVolume(e.target.checked);
                  if (!e.target.checked) setVolume(null);
                  else if (volume === null) setVolume(0.5);
                }}
                className="rounded border-discord-dark bg-discord-darker
                         text-discord-primary focus:ring-discord-primary"
              />
              Custom volume for this sound
            </label>
            {useCustomVolume && (
              <div className="flex items-center gap-3">
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={volume || 0.5}
                  onChange={(e) => setVolume(parseFloat(e.target.value))}
                  className="flex-1"
                  style={{
                    accentColor:
                      (volume || 0.5) >= 0.75 ? "#ef4444" : "#5865f2",
                  }}
                />
                <span
                  className={`text-sm w-12 text-right ${(volume || 0.5) >= 0.75 ? "text-red-400" : "text-discord-text"}`}
                >
                  {Math.round((volume || 0.5) * 100)}%
                </span>
              </div>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3 mt-6">
          <button
            onClick={onClose}
            disabled={isSubmitting}
            className="flex-1 px-4 py-2 bg-discord-dark hover:bg-discord-darker
                     rounded-lg text-discord-text font-medium transition-colors
                     disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isSubmitting}
            className="flex-1 px-4 py-2 bg-discord-success hover:bg-green-600
                     disabled:bg-gray-600 disabled:cursor-not-allowed
                     rounded-lg text-white font-medium transition-colors"
          >
            {isSubmitting
              ? "Saving..."
              : isEditMode
                ? "Save Changes"
                : "Add Sound"}
          </button>
        </div>
      </div>
    </div>
  );
}
