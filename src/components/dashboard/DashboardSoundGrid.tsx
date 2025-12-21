import { Sound } from "../../types";
import SoundButton from "./SoundButton";

interface DashboardSoundGridProps {
  favoriteSounds: Sound[];
  regularSounds: Sound[];
  showFavoritesOnly: boolean;
  playingSoundIds: Set<string>;
  onPlay: (sound: Sound) => void;
  onEdit: (sound: Sound) => void;
  onDelete: (sound: Sound) => void;
  onToggleFavorite: (sound: Sound) => void;
  onTrim: (sound: Sound) => void;
  onAddSound: () => void;
  openContextMenu: { type: "sound" | "category"; id: string } | null;
  onContextMenuChange: (menu: { type: "sound"; id: string } | null) => void;
  hotkeyMappings: { mappings: Record<string, string> };
  onHotkeyChanged: () => void;
}

export default function DashboardSoundGrid({
  favoriteSounds,
  regularSounds,
  showFavoritesOnly,
  playingSoundIds,
  onPlay,
  onEdit,
  onDelete,
  onToggleFavorite,
  onTrim,
  onAddSound,
  openContextMenu,
  onContextMenuChange,
  hotkeyMappings,
  onHotkeyChanged,
}: DashboardSoundGridProps) {
  const filteredSounds = showFavoritesOnly
    ? favoriteSounds
    : [...favoriteSounds, ...regularSounds];

  if (filteredSounds.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-discord-text-muted">
        <div className="text-6xl mb-4">🔇</div>
        <p className="text-lg mb-2">No sounds in this category</p>
        <p className="text-sm mb-4">
          Click "Add Sound" or drag & drop an audio file to get started
        </p>
        <button
          onClick={onAddSound}
          className="px-4 py-2 bg-discord-primary hover:bg-discord-primary-hover
                     rounded-lg text-white font-medium transition-colors"
        >
          + Add Sound
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Favorites Section */}
      {!showFavoritesOnly && favoriteSounds.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-discord-text mb-3 flex items-center gap-2">
            <span className="text-yellow-400">⭐</span>
            Favorites
          </h3>
          <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-3">
            {favoriteSounds.map((sound) => (
              <SoundButton
                key={sound.id}
                sound={sound}
                isPlaying={playingSoundIds.has(sound.id)}
                onPlay={onPlay}
                onEdit={onEdit}
                onDelete={onDelete}
                onToggleFavorite={onToggleFavorite}
                onTrim={onTrim}
                showMenu={
                  openContextMenu?.type === "sound" &&
                  openContextMenu.id === sound.id
                }
                onMenuChange={(show: boolean) =>
                  onContextMenuChange(
                    show ? { type: "sound", id: sound.id } : null
                  )
                }
                hotkeyMappings={hotkeyMappings}
                onHotkeyChanged={onHotkeyChanged}
              />
            ))}
          </div>
        </div>
      )}

      {/* Divider */}
      {!showFavoritesOnly &&
        favoriteSounds.length > 0 &&
        regularSounds.length > 0 && (
          <div className="border-t border-discord-dark"></div>
        )}

      {/* Regular Sounds Section */}
      {!showFavoritesOnly && regularSounds.length > 0 && (
        <div>
          {favoriteSounds.length > 0 && (
            <h3 className="text-sm font-semibold text-discord-text-muted mb-3">
              All Sounds
            </h3>
          )}
          <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-3">
            {regularSounds.map((sound) => (
              <SoundButton
                key={sound.id}
                sound={sound}
                isPlaying={playingSoundIds.has(sound.id)}
                onPlay={onPlay}
                onEdit={onEdit}
                onDelete={onDelete}
                onToggleFavorite={onToggleFavorite}
                onTrim={onTrim}
                showMenu={
                  openContextMenu?.type === "sound" &&
                  openContextMenu.id === sound.id
                }
                onMenuChange={(show: boolean) =>
                  onContextMenuChange(
                    show ? { type: "sound", id: sound.id } : null
                  )
                }
                hotkeyMappings={hotkeyMappings}
                onHotkeyChanged={onHotkeyChanged}
              />
            ))}

            {/* Add Sound Button */}
            <button
              onClick={onAddSound}
              className="h-24 rounded-lg border-2 border-dashed border-discord-dark
                         text-discord-text-muted hover:border-discord-primary
                         hover:text-discord-primary transition-colors
                         flex flex-col items-center justify-center gap-1"
            >
              <span className="text-2xl">+</span>
              <span className="text-xs">Add Sound</span>
            </button>
          </div>
        </div>
      )}

      {/* Favorites Only View */}
      {showFavoritesOnly && (
        <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-3">
          {favoriteSounds.map((sound) => (
            <SoundButton
              key={sound.id}
              sound={sound}
              isPlaying={playingSoundIds.has(sound.id)}
              onPlay={onPlay}
              onEdit={onEdit}
              onDelete={onDelete}
              onToggleFavorite={onToggleFavorite}
              onTrim={onTrim}
              showMenu={
                openContextMenu?.type === "sound" &&
                openContextMenu.id === sound.id
              }
              onMenuChange={(show: boolean) =>
                onContextMenuChange(
                  show ? { type: "sound", id: sound.id } : null
                )
              }
              hotkeyMappings={hotkeyMappings}
              onHotkeyChanged={onHotkeyChanged}
            />
          ))}
        </div>
      )}
    </div>
  );
}
