import { useEffect, useState } from "react";
import { ANIMATION_DURATIONS } from "../../constants";

interface ToastProps {
  message: string;
  onClose: () => void;
  duration?: number;
}

export default function Toast({
  message,
  onClose,
  duration = ANIMATION_DURATIONS.TOAST_DURATION,
}: ToastProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isExiting, setIsExiting] = useState(false);

  useEffect(() => {
    // Trigger enter animation after mount
    requestAnimationFrame(() => {
      setIsVisible(true);
    });

    // Start exit animation before removing
    const exitTimer = setTimeout(() => {
      setIsExiting(true);
    }, duration - ANIMATION_DURATIONS.TOAST_EXIT_START);

    // Remove from DOM after animation completes
    const removeTimer = setTimeout(() => {
      onClose();
    }, duration);

    return () => {
      clearTimeout(exitTimer);
      clearTimeout(removeTimer);
    };
  }, [onClose, duration]);

  return (
    <div
      className={`fixed bottom-6 left-1/2 -translate-x-1/2 z-50 pointer-events-none transition-all ease-in-out ${
        isExiting
          ? "opacity-0 translate-y-24 duration-500"
          : isVisible
            ? "opacity-100 translate-y-0 duration-300"
            : "opacity-0 translate-y-24 duration-0"
      }`}
    >
      <div className="bg-discord-darker border border-discord-dark rounded-full px-6 py-3 shadow-lg">
        <p className="text-sm text-discord-text whitespace-nowrap">{message}</p>
      </div>
    </div>
  );
}
