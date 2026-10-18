import React from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";

export interface ErrorStateProps {
  title?: string;
  message: string;
  onRetry?: () => void;
}

export const ErrorState: React.FC<ErrorStateProps> = ({
  title = "System Operation Error",
  message,
  onRetry,
}) => {
  return (
    <div className="flex flex-col items-center justify-center p-8 text-center rounded-2xl bg-rose-950/20 border border-rose-500/30 my-6">
      <div className="flex items-center justify-center w-12 h-12 rounded-xl bg-rose-500/15 border border-rose-500/30 text-rose-400 mb-3 shadow-[0_0_20px_rgba(244,63,94,0.25)]">
        <AlertTriangle className="w-6 h-6" />
      </div>
      <h3 className="text-base font-semibold text-rose-200 mb-1">
        {title}
      </h3>
      <p className="text-xs text-rose-300/80 max-w-lg mb-4 font-mono select-all bg-black/40 p-2 rounded">
        {message}
      </p>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/15 text-white font-medium text-xs transition-colors border border-white/10"
        >
          <RefreshCw className="w-3.5 h-3.5" />
          <span>Retry Operation</span>
        </button>
      )}
    </div>
  );
};
