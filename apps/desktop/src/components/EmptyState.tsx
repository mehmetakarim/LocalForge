import React from "react";
import { FolderPlus, Plus } from "lucide-react";

export interface EmptyStateProps {
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  title,
  description,
  actionLabel,
  onAction,
}) => {
  return (
    <div className="flex flex-col items-center justify-center p-12 text-center rounded-2xl lf-material-card border border-white/10 my-8">
      <div className="flex items-center justify-center w-14 h-14 rounded-2xl bg-indigo-600/15 border border-indigo-400/30 text-indigo-400 mb-4 shadow-[0_0_24px_rgba(79,110,247,0.2)]">
        <FolderPlus className="w-7 h-7" />
      </div>
      <h3 className="text-lg font-semibold text-white tracking-tight mb-2">
        {title}
      </h3>
      <p className="text-sm text-zinc-400 max-w-md mb-6 leading-relaxed">
        {description}
      </p>
      {actionLabel && onAction && (
        <button
          type="button"
          onClick={onAction}
          className="flex items-center gap-2 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs transition-all shadow-[0_0_16px_rgba(79,110,247,0.4)] border border-indigo-400/40"
        >
          <Plus className="w-4 h-4" />
          <span>{actionLabel}</span>
        </button>
      )}
    </div>
  );
};
