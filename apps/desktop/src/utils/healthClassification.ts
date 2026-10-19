import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";

export type HeroHealthTone = "success" | "warning" | "critical" | "unknown" | "neutral";

export interface HeroHealthContent {
  prefix: string;
  highlight: string;
  desc: string;
  tone: HeroHealthTone;
}

/**
 * Pure health classification function evaluating local infrastructure and runtime states.
 * Evaluates live signals: Nginx gateway, MariaDB socket, verified PHP-FPM pools,
 * APFS volume attachments, and individual project probe classifications.
 */
export function computeHeroHealthContent(
  projects: ProjectRecord[],
  volumes: VolumeInfo[],
  runtime: RuntimeOverview | null | undefined
): HeroHealthContent {
  if (projects.length === 0) {
    return {
      prefix: "Henüz proje",
      highlight: "eklenmedi.",
      desc: "Yerel PHP çalışma alanınızı başlatmak için harici veya dahili diskten bir proje bağlayın.",
      tone: "neutral",
    };
  }

  // Unavailable or incomplete backend telemetry (Case 4)
  if (!runtime) {
    return {
      prefix: "Durum",
      highlight: "BİLİNMİYOR (UNKNOWN).",
      desc: "Çalışma zamanı telemetrisi okunamadı veya arka plan servisine ulaşılamıyor.",
      tone: "unknown",
    };
  }

  // Disconnected workspace volumes check (Case 5)
  const disconnectedProjects = projects.filter(
    (p) =>
      p.status === "disconnected" ||
      !volumes.some((v) => v.uuid === p.volume_uuid)
  );
  if (disconnectedProjects.length > 0) {
    return {
      prefix: "Harici disk",
      highlight: "bağlantısı kesildi.",
      desc: `${disconnectedProjects.length} projenin APFS depolama birimi bağlı değil veya unmount edildi.`,
      tone: "critical",
    };
  }

  // Nginx Gateway infrastructure health
  if (!runtime.nginx_running) {
    return {
      prefix: "Nginx Gateway",
      highlight: "çevrimdışı.",
      desc: "Yerel 8080 ters proxy portu yanıt vermiyor. Servis denetleyicisini kontrol edin.",
      tone: "critical",
    };
  }

  // MariaDB database health
  if (!runtime.mariadb_alive) {
    return {
      prefix: "MariaDB Veritabanı",
      highlight: "çevrimdışı.",
      desc: "Yerel MariaDB soketi yanıt vermiyor. Veritabanı hizmeti beklemede.",
      tone: "warning",
    };
  }

  // Project probe failure classification
  const failedProjects = projects.filter((p) => p.status === "failed");
  if (failedProjects.length > 0) {
    return {
      prefix: "Çalışma zamanı hatası",
      highlight: "saptandı.",
      desc: `${failedProjects.length} projenin PHP-FPM havuzu veya soketi beklenmedik şekilde çöktü.`,
      tone: "critical",
    };
  }

  const degradedProjects = projects.filter((p) => p.status === "degraded");
  if (degradedProjects.length > 0) {
    return {
      prefix: "Kısmi servis",
      highlight: "bozulması.",
      desc: `${degradedProjects.length} projenin HTTP loopback sondasında 502/504 veya aktarım hatası alındı.`,
      tone: "warning",
    };
  }

  const unknownProjects = projects.filter((p) => p.status === "unknown");
  if (unknownProjects.length > 0) {
    return {
      prefix: "Durum",
      highlight: "BİLİNMİYOR (UNKNOWN).",
      desc: `${unknownProjects.length} projenin sağlık durumu arka plan denetleyicisi tarafından doğrulanamadı.`,
      tone: "unknown",
    };
  }

  // Indeterminate or missing PHP-FPM health telemetry (Case 3: strictly unknown)
  if (runtime.php_fpm_running === undefined) {
    return {
      prefix: "Durum",
      highlight: "BİLİNMİYOR (UNKNOWN).",
      desc: "PHP-FPM servis telemetrisi veya havuz sağlık verisi eksik/doğrulanamadı.",
      tone: "unknown",
    };
  }

  const runningProjects = projects.filter((p) => p.status === "running");

  // Verified PHP-FPM pool health check for active running projects (Case 2: failure)
  if (runningProjects.length > 0 && runtime.php_fpm_running === false) {
    return {
      prefix: "PHP-FPM denetleyicisi",
      highlight: "yanıt vermiyor.",
      desc: "Aktif projeler için beklenen PHP-FPM havuz soketleri doğrulanamadı.",
      tone: "critical",
    };
  }

  // Genuinely verified operational health (Case 1: success)
  if (runningProjects.length > 0 && runtime.php_fpm_running === true) {
    return {
      prefix: "Her şey",
      highlight: "yolunda.",
      desc: `${runningProjects.length} aktif projede Nginx 8080, MariaDB ve PHP-FPM servisleri doğrulandı.`,
      tone: "success",
    };
  }

  // Stopped projects must not automatically imply a malfunction (Case 6: ready/neutral)
  return {
    prefix: "Çalışma alanı",
    highlight: "hazır.",
    desc: "Tüm altyapı servisleri hazır. Projelerinizi başlatabilir veya yeni bir proje oluşturabilirsiniz.",
    tone: "neutral",
  };
}
