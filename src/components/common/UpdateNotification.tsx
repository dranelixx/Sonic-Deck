import { useState } from "react";
import { UseUpdateCheckReturn } from "../../hooks/useUpdateCheck";
import UpdateModal from "../modals/UpdateModal";

interface UpdateNotificationProps {
  updateState: UseUpdateCheckReturn;
}

/**
 * Small header badge that shows when an update is available.
 * Clicking opens the UpdateModal with changelog and install option.
 */
export default function UpdateNotification({
  updateState,
}: UpdateNotificationProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Don't render if no update available
  if (!updateState.available) {
    return null;
  }

  return (
    <>
      {/* Update Badge */}
      <button
        onClick={() => setIsModalOpen(true)}
        className="relative flex items-center gap-1.5 px-2 py-1 rounded-md
                   bg-discord-primary/20 hover:bg-discord-primary/30
                   text-discord-primary text-sm font-medium
                   transition-colors cursor-pointer"
        title="Update available"
      >
        {/* Arrow/Download Icon */}
        <svg
          className="w-4 h-4"
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
        <span className="hidden sm:inline">Update</span>
        {/* Subtle pulse animation on new update */}
        <span className="absolute -top-0.5 -right-0.5 flex h-2 w-2">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-discord-primary opacity-75"></span>
          <span className="relative inline-flex rounded-full h-2 w-2 bg-discord-primary"></span>
        </span>
      </button>

      {/* Update Modal */}
      <UpdateModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        updateState={updateState}
      />
    </>
  );
}
