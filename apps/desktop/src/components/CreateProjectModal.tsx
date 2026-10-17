import React, { useState } from "react";
import { X, FolderPlus } from "lucide-react";
import { VolumeInfo } from "../types";

interface CreateProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  volumes: VolumeInfo[];
  onCreate: (name: string, parentDir: string, docroot: string, phpVersion: string) => Promise<void>;
}

export const CreateProjectModal: React.FC<CreateProjectModalProps> = ({
  isOpen,
  onClose,
  volumes,
  onCreate,
}) => {
  const [name, setName] = useState("");
  const [parentDir, setParentDir] = useState(
    volumes.find((v) => v.mount_point.startsWith("/Volumes"))?.mount_point || "/Volumes"
  );
  const [docroot, setDocroot] = useState("public");
  const [phpVersion, setPhpVersion] = useState("8.2");
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !parentDir.trim()) return;

    setLoading(true);
    try {
      await onCreate(name, parentDir, docroot, phpVersion);
      setName("");
      onClose();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-xl w-full max-w-md p-6 shadow-2xl relative">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-slate-400 hover:text-white"
        >
          <X className="w-5 h-5" />
        </button>

        <div className="flex items-center gap-2 mb-4">
          <FolderPlus className="w-5 h-5 text-indigo-400" />
          <h2 className="text-lg font-semibold text-white">Create New PHP Project</h2>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">Project Name</label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. client-store"
              className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-sm text-white focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1">Storage Location</label>
            <select
              value={parentDir}
              onChange={(e) => setParentDir(e.target.value)}
              className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-sm text-white focus:outline-none focus:border-indigo-500"
            >
              {volumes.map((v) => (
                <option key={v.uuid} value={v.mount_point}>
                  {v.name} ({v.mount_point})
                </option>
              ))}
            </select>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-medium text-slate-300 mb-1">Document Root</label>
              <input
                type="text"
                value={docroot}
                onChange={(e) => setDocroot(e.target.value)}
                placeholder="public"
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-sm text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-slate-300 mb-1">PHP Version</label>
              <select
                value={phpVersion}
                onChange={(e) => setPhpVersion(e.target.value)}
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-sm text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="8.4">PHP 8.4</option>
                <option value="8.3">PHP 8.3</option>
                <option value="8.2">PHP 8.2</option>
                <option value="8.1">PHP 8.1</option>
              </select>
            </div>
          </div>

          <div className="pt-2 flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-lg text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-300"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 rounded-lg text-xs font-medium bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50"
            >
              {loading ? "Creating..." : "Create Project"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
