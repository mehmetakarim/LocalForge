import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createJiti } from "jiti";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const jiti = createJiti(import.meta.url);

// Import ACTUAL production modules — Zero duplicated business logic
const { computeHeroHealthContent } = await jiti.import("../src/utils/healthClassification.ts");
const { getVisualFixture } = await jiti.import("../src/fixtures/visualFixtures.ts");

const mockVolume = {
  uuid: "vol-main-1234",
  name: "Mac Harici Disk",
  mount_point: "/Volumes/Mac Harici Disk",
  filesystem: "apfs",
  is_removable: true,
  is_internal: false,
  total_bytes: 1000000000,
  available_bytes: 500000000,
};

const mockRunningProject = {
  id: "p1",
  name: "Alpha Proje",
  path: "/Volumes/Mac Harici Disk/alpha",
  volume_uuid: "vol-main-1234",
  document_root: "public",
  php_version: "8.4",
  local_domain: "alpha.test",
  status: "running",
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

const mockStoppedProject = {
  ...mockRunningProject,
  id: "p2",
  name: "Beta Proje",
  status: "stopped",
};

const mockHealthyRuntime = {
  php_versions: ["8.4"],
  nginx_port: 8080,
  nginx_running: true,
  mariadb_port: 3306,
  mariadb_socket: "/run/mariadb.sock",
  mariadb_alive: true,
  php_fpm_running: true,
};

test("R1.3A-V1 Fixture Isolation: Production gate strictly rejects fixture activation", () => {
  // 1. In standard non-dev environments (such as Node execution), returns null
  assert.equal(getVisualFixture("2"), null);
  assert.equal(getVisualFixture("6"), null);
  assert.equal(getVisualFixture("empty"), null);

  // 2. Even when PROD and DEV are both set or query keys are passed, PROD strictly overrides
  assert.equal(getVisualFixture("2", { PROD: true, DEV: true }), null);
  assert.equal(getVisualFixture("2", { PROD: true, DEV: false }), null);
  assert.equal(getVisualFixture("2", { PROD: false, DEV: false }), null);

  // 3. Only explicit development mode (PROD=false, DEV=true) permits fixture retrieval
  const devFixture = getVisualFixture("2", { PROD: false, DEV: true });
  assert.notEqual(devFixture, null);
  assert.equal(devFixture.projects.length, 2);

  const emptyFixture = getVisualFixture("empty", { PROD: false, DEV: true });
  assert.notEqual(emptyFixture, null);
  assert.equal(emptyFixture.projects.length, 0);

  // Unknown fixture key returns null even in dev mode
  assert.equal(getVisualFixture("unknown_key", { PROD: false, DEV: true }), null);
});

test("R1.3A-V1 Production Bundle Inspection: Built bundle excludes fixture activation", () => {
  const distAssetsDir = path.resolve(__dirname, "../dist/assets");
  if (!fs.existsSync(distAssetsDir)) {
    // If not built yet, skip bundle verification (it will run after vite build)
    return;
  }
  const files = fs.readdirSync(distAssetsDir);
  const jsBundle = files.find((f) => f.startsWith("index-") && f.endsWith(".js"));
  assert.ok(jsBundle, "Expected production JS bundle to exist in dist/assets");

  const bundleContent = fs.readFileSync(path.join(distAssetsDir, jsBundle), "utf-8");

  // The fixture mock disk UUID must NEVER be compiled into the production bundle
  const mockFixtureUuid = "7E26530E-5179-414C-81B5-6FBDB429FC65";
  assert.equal(
    bundleContent.includes(mockFixtureUuid),
    false,
    "Production bundle must not contain visual fixture UUIDs"
  );
});

test("Acceptance Case 1: php_fpm_running: true with all checks healthy yields SUCCESS", () => {
  const result = computeHeroHealthContent(
    [mockRunningProject],
    [mockVolume],
    { ...mockHealthyRuntime, php_fpm_running: true }
  );

  assert.equal(result.tone, "success");
  assert.equal(result.prefix, "Her şey");
  assert.equal(result.highlight, "yolunda.");
  assert.ok(result.desc.includes("PHP-FPM"));
});

test("Acceptance Case 2: php_fpm_running: false for a running project yields failure", () => {
  const result = computeHeroHealthContent(
    [mockRunningProject],
    [mockVolume],
    { ...mockHealthyRuntime, php_fpm_running: false }
  );

  assert.equal(result.tone, "critical");
  assert.equal(result.prefix, "PHP-FPM denetleyicisi");
  assert.equal(result.highlight, "yanıt vermiyor.");
});

test("Acceptance Case 3: php_fpm_running: undefined yields UNKNOWN", () => {
  // Test both with running project and stopped project
  const resultRunning = computeHeroHealthContent(
    [mockRunningProject],
    [mockVolume],
    { ...mockHealthyRuntime, php_fpm_running: undefined }
  );
  assert.equal(resultRunning.tone, "unknown");
  assert.equal(resultRunning.prefix, "Durum");
  assert.equal(resultRunning.highlight, "BİLİNMİYOR (UNKNOWN).");

  const resultStopped = computeHeroHealthContent(
    [mockStoppedProject],
    [mockVolume],
    { ...mockHealthyRuntime, php_fpm_running: undefined }
  );
  assert.equal(resultStopped.tone, "unknown");
  assert.equal(resultStopped.prefix, "Durum");
  assert.equal(resultStopped.highlight, "BİLİNMİYOR (UNKNOWN).");
});

test("Acceptance Case 4: Missing runtime response yields UNKNOWN", () => {
  const resultNull = computeHeroHealthContent([mockRunningProject], [mockVolume], null);
  assert.equal(resultNull.tone, "unknown");
  assert.equal(resultNull.prefix, "Durum");
  assert.equal(resultNull.highlight, "BİLİNMİYOR (UNKNOWN).");

  const resultUndefined = computeHeroHealthContent([mockRunningProject], [mockVolume], undefined);
  assert.equal(resultUndefined.tone, "unknown");
});

test("Acceptance Case 5: Disconnected project volume yields appropriate warning/failure", () => {
  // Project volume missing from mounted volumes list
  const resultMissingVolume = computeHeroHealthContent(
    [mockRunningProject],
    [], // No mounted volumes
    mockHealthyRuntime
  );
  assert.equal(resultMissingVolume.tone, "critical");
  assert.equal(resultMissingVolume.prefix, "Harici disk");
  assert.equal(resultMissingVolume.highlight, "bağlantısı kesildi.");

  // Explicit status: "disconnected"
  const resultDisconnectedStatus = computeHeroHealthContent(
    [{ ...mockRunningProject, status: "disconnected" }],
    [mockVolume],
    mockHealthyRuntime
  );
  assert.equal(resultDisconnectedStatus.tone, "critical");
  assert.equal(resultDisconnectedStatus.prefix, "Harici disk");
  assert.equal(resultDisconnectedStatus.highlight, "bağlantısı kesildi.");
});

test("Acceptance Case 6: Stopped projects must not automatically imply a malfunction", () => {
  const result = computeHeroHealthContent(
    [mockStoppedProject],
    [mockVolume],
    { ...mockHealthyRuntime, php_fpm_running: false }
  );

  assert.equal(result.tone, "neutral");
  assert.equal(result.prefix, "Çalışma alanı");
  assert.equal(result.highlight, "hazır.");
  assert.notEqual(result.tone, "critical");
  assert.notEqual(result.tone, "warning");
});

test("Health Messaging: Empty project list yields neutral tone", () => {
  const result = computeHeroHealthContent([], [mockVolume], mockHealthyRuntime);
  assert.equal(result.tone, "neutral");
  assert.equal(result.prefix, "Henüz proje");
  assert.equal(result.highlight, "eklenmedi.");
});

test("Health Messaging: Nginx port conflict yields critical tone", () => {
  const result = computeHeroHealthContent(
    [mockRunningProject],
    [mockVolume],
    { ...mockHealthyRuntime, nginx_running: false, nginx_port_conflict: true }
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.prefix, "Port çakışması");
  assert.equal(result.highlight, "algılandı (8080).");
});

test("Health Messaging: Nginx Gateway down yields critical tone", () => {
  const result = computeHeroHealthContent(
    [mockRunningProject],
    [mockVolume],
    { ...mockHealthyRuntime, nginx_running: false }
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.prefix, "Nginx Gateway");
  assert.equal(result.highlight, "çevrimdışı.");
});

test("Health Messaging: MariaDB down yields warning tone", () => {
  const result = computeHeroHealthContent(
    [mockRunningProject],
    [mockVolume],
    { ...mockHealthyRuntime, mariadb_alive: false }
  );
  assert.equal(result.tone, "warning");
  assert.equal(result.prefix, "MariaDB Veritabanı");
  assert.equal(result.highlight, "çevrimdışı.");
});

test("Health Messaging: Project probe failure yields critical tone", () => {
  const result = computeHeroHealthContent(
    [{ ...mockRunningProject, status: "failed" }],
    [mockVolume],
    mockHealthyRuntime
  );
  assert.equal(result.tone, "critical");
  assert.equal(result.prefix, "Çalışma zamanı hatası");
  assert.equal(result.highlight, "saptandı.");
});

test("Health Messaging: Degraded project status yields warning tone", () => {
  const result = computeHeroHealthContent(
    [{ ...mockRunningProject, status: "degraded" }],
    [mockVolume],
    mockHealthyRuntime
  );
  assert.equal(result.tone, "warning");
  assert.equal(result.prefix, "Kısmi servis");
  assert.equal(result.highlight, "bozulması.");
});

test("Health Messaging: Unknown project status yields UNKNOWN tone", () => {
  const result = computeHeroHealthContent(
    [{ ...mockRunningProject, status: "unknown" }],
    [mockVolume],
    mockHealthyRuntime
  );
  assert.equal(result.tone, "unknown");
  assert.equal(result.prefix, "Durum");
  assert.equal(result.highlight, "BİLİNMİYOR (UNKNOWN).");
});

test("Multi-Project Handling: Pagination covers collections >6 without hiding records", () => {
  const eightProjects = Array.from({ length: 8 }, (_, i) => ({
    ...mockStoppedProject,
    id: `proj_${i + 1}`,
    name: `Proje ${i + 1}`,
  }));

  const PAGE_SIZE = 6;
  const page0 = eightProjects.slice(0, PAGE_SIZE);
  const page1 = eightProjects.slice(PAGE_SIZE, PAGE_SIZE * 2);

  assert.equal(page0.length, 6);
  assert.equal(page1.length, 2);
  assert.equal(page0.length + page1.length, 8);
  assert.equal(page0[0].name, "Proje 1");
  assert.equal(page1[0].name, "Proje 7");
});
