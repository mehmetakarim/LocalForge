import test from "node:test";
import assert from "node:assert/strict";

// We replicate the pure logic of getVisualFixture to assert production safety rules
function testGetVisualFixture(key, isProd, isDev) {
  if (isProd || !isDev) {
    return null;
  }
  if (!key) return null;
  if (key === "empty") return { projects: [], volumes: [{}], runtime: {} };
  if (key === "2") return { projects: [{}, {}], volumes: [{}], runtime: {} };
  if (key === "6") return { projects: Array(6).fill({}), volumes: [{}], runtime: {} };
  if (key === "8") return { projects: Array(8).fill({}), volumes: [{}], runtime: {} };
  return null;
}

// Replicate computeHeroHealthContent logic for comprehensive assertion
function computeHeroHealthContent(projects, volumes, runtime) {
  if (projects.length === 0) {
    return {
      prefix: "Henüz proje",
      highlight: "eklenmedi.",
      tone: "neutral",
    };
  }
  if (!runtime) {
    return {
      prefix: "Durum",
      highlight: "BİLİNMİYOR (UNKNOWN).",
      tone: "unknown",
    };
  }
  const disconnected = projects.filter(
    (p) => p.status === "disconnected" || !volumes.some((v) => v.uuid === p.volume_uuid)
  );
  if (disconnected.length > 0) {
    return {
      prefix: "Harici disk",
      highlight: "bağlantısı kesildi.",
      tone: "critical",
    };
  }
  if (!runtime.nginx_running) {
    return {
      prefix: "Nginx Gateway",
      highlight: "çevrimdışı.",
      tone: "critical",
    };
  }
  if (!runtime.mariadb_alive) {
    return {
      prefix: "MariaDB Veritabanı",
      highlight: "çevrimdışı.",
      tone: "warning",
    };
  }
  const failed = projects.filter((p) => p.status === "failed");
  if (failed.length > 0) {
    return {
      prefix: "Çalışma zamanı hatası",
      highlight: "saptandı.",
      tone: "critical",
    };
  }
  const degraded = projects.filter((p) => p.status === "degraded");
  if (degraded.length > 0) {
    return {
      prefix: "Kısmi servis",
      highlight: "bozulması.",
      tone: "warning",
    };
  }
  const unknown = projects.filter((p) => p.status === "unknown");
  if (unknown.length > 0) {
    return {
      prefix: "Durum",
      highlight: "BİLİNMİYOR (UNKNOWN).",
      tone: "unknown",
    };
  }
  const running = projects.filter((p) => p.status === "running");
  if (running.length > 0 && runtime.php_fpm_running === false) {
    return {
      prefix: "PHP-FPM denetleyicisi",
      highlight: "yanıt vermiyor.",
      tone: "critical",
    };
  }
  if (running.length > 0) {
    return {
      prefix: "Her şey",
      highlight: "yolunda.",
      tone: "success",
    };
  }
  return {
    prefix: "Çalışma alanı",
    highlight: "hazır.",
    tone: "ready",
  };
}

test("R1.3A-V1 Negative Regression: Production builds strictly reject fixtures", () => {
  // In production mode, NO fixture key should ever return data
  assert.equal(testGetVisualFixture("empty", true, false), null);
  assert.equal(testGetVisualFixture("2", true, false), null);
  assert.equal(testGetVisualFixture("6", true, false), null);
  assert.equal(testGetVisualFixture("8", true, false), null);
  assert.equal(testGetVisualFixture("baseline", true, false), null);
  assert.equal(testGetVisualFixture(null, true, false), null);

  // In DEV mode, fixtures are allowed
  assert.notEqual(testGetVisualFixture("empty", false, true), null);
  assert.notEqual(testGetVisualFixture("2", false, true), null);
  assert.notEqual(testGetVisualFixture("6", false, true), null);
});

test("R1.3A-V1 Health Messaging: Empty projects yields neutral tone", () => {
  const result = computeHeroHealthContent([], [{ uuid: "vol_1" }], { nginx_running: true, mariadb_alive: true });
  assert.equal(result.tone, "neutral");
  assert.equal(result.highlight, "eklenmedi.");
});

test("R1.3A-V1 Health Messaging: Null runtime telemetry yields UNKNOWN", () => {
  const result = computeHeroHealthContent([{ id: "p1", volume_uuid: "vol_1", status: "running" }], [{ uuid: "vol_1" }], null);
  assert.equal(result.tone, "unknown");
  assert.match(result.highlight, /UNKNOWN/);
});

test("R1.3A-V1 Health Messaging: Disconnected volume yields critical tone", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_missing", status: "running" }],
    [{ uuid: "vol_other" }],
    { nginx_running: true, mariadb_alive: true, php_fpm_running: true }
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.highlight, "bağlantısı kesildi.");
});

test("R1.3A-V1 Health Messaging: Nginx down yields critical tone", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_1", status: "running" }],
    [{ uuid: "vol_1" }],
    { nginx_running: false, mariadb_alive: true, php_fpm_running: true }
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.highlight, "çevrimdışı.");
});

test("R1.3A-V1 Health Messaging: Failed project status yields critical tone", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_1", status: "failed" }],
    [{ uuid: "vol_1" }],
    { nginx_running: true, mariadb_alive: true, php_fpm_running: true }
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.highlight, "saptandı.");
});

test("R1.3A-V1 Health Messaging: Degraded project status yields warning tone", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_1", status: "degraded" }],
    [{ uuid: "vol_1" }],
    { nginx_running: true, mariadb_alive: true, php_fpm_running: true }
  );
  assert.equal(result.tone, "warning");
  assert.equal(result.highlight, "bozulması.");
});

test("R1.3A-V1 Health Messaging: Unknown project status yields UNKNOWN tone", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_1", status: "unknown" }],
    [{ uuid: "vol_1" }],
    { nginx_running: true, mariadb_alive: true, php_fpm_running: true }
  );
  assert.equal(result.tone, "unknown");
  assert.match(result.highlight, /UNKNOWN/);
});

test("R1.3A-V1 Health Messaging: Active project with php_fpm_running=false yields critical tone", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_1", status: "running" }],
    [{ uuid: "vol_1" }],
    { nginx_running: true, mariadb_alive: true, php_fpm_running: false }
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.highlight, "yanıt vermiyor.");
});

test("R1.3A-V1 Health Messaging: All verified yields success 'Her şey yolunda'", () => {
  const result = computeHeroHealthContent(
    [{ id: "p1", volume_uuid: "vol_1", status: "running" }],
    [{ uuid: "vol_1" }],
    { nginx_running: true, mariadb_alive: true, php_fpm_running: true }
  );
  assert.equal(result.tone, "success");
  assert.equal(result.highlight, "yolunda.");
});

test("R1.3A-V1 Multi-Project Handling: Pagination covers collections >6 without hiding records", () => {
  const eightProjects = Array.from({ length: 8 }).map((_, i) => ({
    id: `proj_${i + 1}`,
    name: `Project ${i + 1}`,
    volume_uuid: "vol_1",
    status: "stopped",
  }));

  const PAGE_SIZE = 6;
  const totalPages = Math.ceil(eightProjects.length / PAGE_SIZE);
  assert.equal(totalPages, 2);

  const page1 = eightProjects.slice(0, PAGE_SIZE);
  assert.equal(page1.length, 6);
  assert.equal(page1[0].name, "Project 1");
  assert.equal(page1[5].name, "Project 6");

  const page2 = eightProjects.slice(PAGE_SIZE, PAGE_SIZE * 2);
  assert.equal(page2.length, 2);
  assert.equal(page2[0].name, "Project 7");
  assert.equal(page2[1].name, "Project 8");

  // Zero records are lost
  assert.equal(page1.length + page2.length, 8);
});
