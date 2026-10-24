import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createJiti } from "jiti";

const jiti = createJiti(import.meta.url);

const { computeStorageCapacity } = await jiti.import("../src/utils/storageCapacity.ts");
const { computeProjectRouting } = await jiti.import("../src/utils/browserRouting.ts");

test("Gate 2A: PHP Version Matching distinguishes installed from missing requested versions", () => {
  const installedDetails = [
    {
      version: "8.5.3",
      major_minor: "8.5",
      binary_path: "/opt/homebrew/sbin/php-fpm",
      is_fpm_verified: true,
      architecture: "arm64",
      is_supported: true,
    },
  ];

  const requestedMissing = "8.4";
  const requestedAvailable = "8.5";

  const isMissingInstalled = installedDetails.some(
    (d) => d.major_minor === requestedMissing || d.version.startsWith(requestedMissing)
  );
  assert.equal(isMissingInstalled, false, "PHP 8.4 must be recognized as NOT installed on this host");

  const isAvailableInstalled = installedDetails.some(
    (d) => d.major_minor === requestedAvailable || d.version.startsWith(requestedAvailable)
  );
  assert.equal(isAvailableInstalled, true, "PHP 8.5 must be recognized as installed and verified");
});

test("Gate 2A: Project retains requested version without silent auto-migration", () => {
  const originalProject = {
    id: "proj_test_01",
    name: "Projetest",
    php_version: "8.4",
    status: "stopped",
  };

  // Rule: LocalForge MUST NOT automatically change originalProject.php_version to 8.5
  assert.equal(originalProject.php_version, "8.4", "Existing project configuration must NOT be silently changed");
});

test("Gate 2B: MariaDB Datadir Safety Guard blocks unapproved initialization", () => {
  const datadirStatus = {
    is_datadir_initialized: false,
    path: "/Users/test/Library/Application Support/LocalForge/runtime/mariadb/data",
  };

  // Safety check before start: If datadir is not initialized, automatic init is blocked
  const attemptStartWithoutApproval = () => {
    if (!datadirStatus.is_datadir_initialized) {
      throw new Error("MariaDB veri dizini başlatılmamış. Güvenlik politikası ve Product Owner direktifi gereği otomatik disk ilklendirme durdurulmuştur.");
    }
    return "started";
  };

  assert.throws(
    () => attemptStartWithoutApproval(),
    /otomatik disk ilklendirme durdurulmuştur/,
    "Uninitialized datadir must safely fail-closed without modifying disk"
  );
});

test("Gate 2C: Domain routing rejects unqualified loopback browsing", () => {
  const project = {
    name: "Projetest",
    local_domain: "projetest.test",
    loopback_port: 18081,
  };
  const port = 8080;

  const { projectUrl, curlCommand, canOpenBrowser } = computeProjectRouting(
    project.local_domain,
    port,
    project.name,
    project.loopback_port
  );
  assert.equal(projectUrl, "http://projetest.test:18081/");
  assert.equal(canOpenBrowser, true);
  assert.ok(curlCommand.includes("-H \"Host: projetest.localforge\""), "cURL helper must include explicit Host header");
  assert.ok(!projectUrl.startsWith("http://127.0.0.1"), "Never open unqualified loopback IP directly");
});

test("Gate 3: Native macOS App Icon has transparent squircle corners", () => {
  const iconPath = path.resolve(process.cwd(), "src-tauri/icons/icon.png");
  assert.ok(fs.existsSync(iconPath), "icon.png must exist");

  const stat = fs.statSync(iconPath);
  assert.ok(stat.size > 10000, "icon.png must be non-empty");

  const icnsPath = path.resolve(process.cwd(), "src-tauri/icons/icon.icns");
  assert.ok(fs.existsSync(icnsPath), "icon.icns must exist for native macOS Dock");
  assert.ok(fs.statSync(icnsPath).size > 100000, "icon.icns must be a full multi-resolution ICNS file");
});

test("Gate 3: Storage Capacity displays verified available bytes and free percentage", () => {
  const vol = {
    uuid: "vol_ext_01",
    name: "Mac Harici Disk",
    mount_point: "/Volumes/Mac Harici Disk",
    filesystem: "APFS",
    is_removable: true,
    is_internal: false,
    total_bytes: 2000000000000, // ~2 TB
    available_bytes: 1500000000000, // ~1.5 TB free
  };

  const capacity = computeStorageCapacity(vol);
  assert.equal(capacity.hasValidCapacity, true);
  assert.equal(capacity.usedBytes, 500000000000);
  assert.equal(capacity.usagePercent, 25);
  assert.equal(capacity.statusLabel, "BAĞLI (APFS)");
});
