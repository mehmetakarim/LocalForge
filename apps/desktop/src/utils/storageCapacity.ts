export interface StorageCapacityMetrics {
  hasValidCapacity: boolean;
  usedBytes: number;
  usagePercent: number | null;
  statusLabel: string;
  isWarning: boolean;
}

export function computeStorageCapacity(vol: {
  total_bytes: number;
  available_bytes: number;
  is_internal?: boolean;
  is_removable?: boolean;
}): StorageCapacityMetrics {
  const hasValidCapacity =
    typeof vol.total_bytes === "number" &&
    typeof vol.available_bytes === "number" &&
    vol.total_bytes > 0 &&
    vol.available_bytes <= vol.total_bytes &&
    vol.available_bytes >= 0;

  const usedBytes = hasValidCapacity
    ? Math.max(0, vol.total_bytes - vol.available_bytes)
    : 0;

  const usagePercent = hasValidCapacity
    ? Math.min(100, Math.round((usedBytes / vol.total_bytes) * 100))
    : null;

  const isExternal = !vol.is_internal || Boolean(vol.is_removable);

  const statusLabel = !hasValidCapacity
    ? "BAĞLANTI KESİK / BİLİNMİYOR"
    : isExternal
    ? "BAĞLI (APFS)"
    : "SİSTEM (APFS)";

  return {
    hasValidCapacity,
    usedBytes,
    usagePercent,
    statusLabel,
    isWarning: !hasValidCapacity,
  };
}
