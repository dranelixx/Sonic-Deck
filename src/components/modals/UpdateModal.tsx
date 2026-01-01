import { UseUpdateCheckReturn } from "../../hooks/useUpdateCheck";

interface UpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
  updateState: UseUpdateCheckReturn;
}

/**
 * Modal showing update information, changelog summary, and install option.
 * Shows download progress during installation.
 */
export default function UpdateModal({
  isOpen,
  onClose,
  updateState,
}: UpdateModalProps) {
  if (!isOpen || !updateState.available) return null;

  const { available, downloading, progress, error, installUpdate } =
    updateState;

  // Parse changelog for summary (count features and fixes)
  const parseChangelogSummary = (body: string | undefined): string => {
    if (!body) return "New version available";

    const lines = body.split("\n");
    let features = 0;
    let fixes = 0;

    lines.forEach((line) => {
      const lowerLine = line.toLowerCase();
      if (
        lowerLine.includes("feat:") ||
        lowerLine.includes("feature") ||
        lowerLine.includes("✨")
      ) {
        features++;
      }
      if (
        lowerLine.includes("fix:") ||
        lowerLine.includes("bug") ||
        lowerLine.includes("🐛")
      ) {
        fixes++;
      }
    });

    if (features === 0 && fixes === 0) {
      // Fallback: show first meaningful lines
      const meaningfulLines = lines
        .filter((line) => line.trim().length > 3 && !line.startsWith("#"))
        .slice(0, 2);
      if (meaningfulLines.length > 0) {
        return meaningfulLines.join(" • ");
      }
      return "New version available";
    }

    const parts: string[] = [];
    if (features > 0) {
      parts.push(`${features} new feature${features > 1 ? "s" : ""}`);
    }
    if (fixes > 0) {
      parts.push(`${fixes} bug fix${fixes > 1 ? "es" : ""}`);
    }
    return parts.join(", ");
  };

  const handleInstall = async () => {
    await installUpdate();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape" && !downloading) {
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onKeyDown={handleKeyDown}
    >
      {/* Backdrop */}
      <div
        className={`absolute inset-0 bg-black/60 ${downloading ? "" : "cursor-pointer"}`}
        onClick={downloading ? undefined : onClose}
      />

      {/* Modal */}
      <div
        className="relative bg-discord-dark rounded-lg shadow-xl w-full max-w-md
                    border border-discord-darker p-6 mx-4"
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold text-discord-text flex items-center gap-2">
            <svg
              className="w-6 h-6 text-discord-primary"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
              />
            </svg>
            Update Available
          </h2>
          {!downloading && (
            <button
              onClick={onClose}
              className="text-discord-text-muted hover:text-discord-text
                       text-2xl leading-none transition-colors"
            >
              &times;
            </button>
          )}
        </div>

        {/* Version Info */}
        <div className="mb-4">
          <div className="text-discord-text font-medium">
            Version {available.version}
          </div>
          <div className="text-discord-text-muted text-sm mt-1">
            {parseChangelogSummary(available.body)}
          </div>
        </div>

        {/* Changelog Preview (if body exists) */}
        {available.body && (
          <div className="mb-4 max-h-32 overflow-y-auto bg-discord-darker rounded p-3">
            <pre className="text-xs text-discord-text-muted whitespace-pre-wrap font-mono">
              {available.body}
            </pre>
          </div>
        )}

        {/* Error */}
        {error && (
          <div
            className="mb-4 p-3 bg-discord-danger/20 border border-discord-danger
                        rounded-lg text-discord-danger text-sm"
          >
            {error}
          </div>
        )}

        {/* Progress Bar (during download) */}
        {downloading && (
          <div className="mb-4">
            <div className="flex items-center justify-between text-sm text-discord-text-muted mb-2">
              <span>Downloading update...</span>
              <span>{Math.round(progress)}%</span>
            </div>
            <div className="w-full h-2 bg-discord-darker rounded-full overflow-hidden">
              <div
                className="h-full bg-discord-primary transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
            {progress >= 100 && (
              <div className="text-center text-discord-text-muted text-sm mt-3">
                Installing... App will restart shortly.
              </div>
            )}
          </div>
        )}

        {/* Actions */}
        {!downloading && (
          <div className="flex gap-3 mt-6">
            <button
              onClick={onClose}
              className="flex-1 px-4 py-2 bg-discord-dark hover:bg-discord-darker
                       border border-discord-darker
                       rounded-lg text-discord-text font-medium transition-colors"
            >
              Later
            </button>
            <button
              onClick={handleInstall}
              className="flex-1 px-4 py-2 bg-discord-primary hover:bg-discord-primary-hover
                       rounded-lg text-white font-medium transition-colors"
            >
              Install Update
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
