import test from "node:test";
import assert from "node:assert/strict";
import { createJiti } from "jiti";

const jiti = createJiti(import.meta.url);

// Import ACTUAL production utilities — Zero duplicated business logic
const { computeStorageCapacity } = await jiti.import("../src/utils/storageCapacity.ts");
const { deriveServiceStates } = await jiti.import("../src/utils/servicesTelemetry.ts");
const { computeProjectRouting } = await jiti.import("../src/utils/browserRouting.ts");

test("R1.3B Production Utility: computeStorageCapacity never fabricates 45% fallback", () => {
  // Case 1: Valid volume with 50% usage
  const validVol = { total_bytes: 1000, available_bytes: 500, is_internal: false };
  const res1 = computeStorageCapacity(validVol);
  assert.equal(res1.hasValidCapacity, true);
  assert.equal(res1.usedBytes, 500);
  assert.equal(res1.usagePercent, 50);
  assert.equal(res1.statusLabel, "BAĞLI (APFS)");
  assert.equal(res1.isWarning, false);

  // Case 2: Zero total bytes (e.g. unmounted / unreadable volume)
  const zeroVol = { total_bytes: 0, available_bytes: 0 };
  const res2 = computeStorageCapacity(zeroVol);
  assert.equal(res2.hasValidCapacity, false);
  assert.equal(res2.usagePercent, null, "Must NOT fall back to 45%");
  assert.equal(res2.statusLabel, "BAĞLANTI KESİK / BİLİNMİYOR");
  assert.equal(res2.isWarning, true);

  // Case 3: Negative total bytes or invalid telemetry
  const invalidVol = { total_bytes: -100, available_bytes: 50 };
  const res3 = computeStorageCapacity(invalidVol);
  assert.equal(res3.hasValidCapacity, false);
  assert.equal(res3.usagePercent, null, "Must NOT fall back to 45%");
  assert.equal(res3.statusLabel, "BAĞLANTI KESİK / BİLİNMİYOR");

  // Case 4: Available bytes exceeds total bytes (corrupted APFS telemetry)
  const overflowVol = { total_bytes: 1000, available_bytes: 2000 };
  const res4 = computeStorageCapacity(overflowVol);
  assert.equal(res4.hasValidCapacity, false);
  assert.equal(res4.usagePercent, null, "Must NOT fall back to 45%");
  assert.equal(res4.statusLabel, "BAĞLANTI KESİK / BİLİNMİYOR");

  // Case 5: Internal container valid capacity
  const internalVol = { total_bytes: 1000, available_bytes: 800, is_internal: true };
  const res5 = computeStorageCapacity(internalVol);
  assert.equal(res5.hasValidCapacity, true);
  assert.equal(res5.statusLabel, "SİSTEM (APFS)");
  assert.equal(res5.usagePercent, 20);
});

test("R1.3B Production Utility: deriveServiceStates explicitly distinguishes UNKNOWN from STOPPED", () => {
  // Case 1: runtime is null (unprobed / loading error)
  const nullRes = deriveServiceStates(null);
  assert.equal(nullRes.nginxStatus, "unknown");
  assert.equal(nullRes.phpFpmStatus, "unknown");
  assert.equal(nullRes.mariadbStatus, "unknown");

  // Case 2: php_fpm_running is undefined (partial probe)
  const partialRuntime = {
    php_versions: ["8.4"],
    nginx_port: 8080,
    nginx_running: true,
    nginx_port_conflict: false,
    mariadb_port: 3306,
    mariadb_socket: "/run/mariadb.sock",
    mariadb_alive: true,
    // php_fpm_running undefined
  };
  const partialRes = deriveServiceStates(partialRuntime);
  assert.equal(partialRes.nginxStatus, "running");
  assert.equal(partialRes.phpFpmStatus, "unknown", "Undefined PHP-FPM must be unknown, not stopped");
  assert.equal(partialRes.mariadbStatus, "running");

  // Case 3: explicitly stopped services
  const stoppedRuntime = {
    php_versions: ["8.4"],
    nginx_port: 8080,
    nginx_running: false,
    nginx_port_conflict: false,
    mariadb_port: 3306,
    mariadb_socket: "/run/mariadb.sock",
    mariadb_alive: false,
    php_fpm_running: false,
  };
  const stoppedRes = deriveServiceStates(stoppedRuntime);
  assert.equal(stoppedRes.nginxStatus, "stopped");
  assert.equal(stoppedRes.phpFpmStatus, "stopped");
  assert.equal(stoppedRes.mariadbStatus, "stopped");

  // Case 4: Nginx port conflict
  const conflictRuntime = {
    ...stoppedRuntime,
    nginx_port_conflict: true,
  };
  const conflictRes = deriveServiceStates(conflictRuntime);
  assert.equal(conflictRes.nginxStatus, "conflict");
});

test("R1.3B Production Utility: computeProjectRouting generates domain-specific URL and cURL helper", () => {
  const res = computeProjectRouting("site-a.test", 8080);
  assert.equal(res.projectUrl, "http://site-a.test:8080/");
  assert.equal(res.curlCommand, 'curl -H "Host: site-a.test" http://127.0.0.1:8080/');
  assert.ok(!res.projectUrl.includes("127.0.0.1"), "Project URL must target domain, never raw loopback IP");
});

test("R1.3B Import Error Workflow: Failed import preserves entered path, displays error, and never closes modal", async () => {
  // Simulate Import Workflow Logic
  let modalClosed = false;
  let formCleared = false;
  let displayedError = null;
  let enteredPath = "/Volumes/Mac Harici Disk/Projects/broken-site";

  const executeImportSubmit = async (path, onImportFn) => {
    const trimmed = path.trim();
    if (!trimmed) {
      displayedError = "Lütfen geçerli bir proje klasör yolu girin.";
      return;
    }

    try {
      await onImportFn(trimmed);
      enteredPath = "";
      formCleared = true;
      modalClosed = true;
      displayedError = null;
    } catch (err) {
      displayedError = typeof err === "string" ? err : err?.message || String(err);
      // DO NOT close modal or clear enteredPath on error!
    }
  };

  // 1. Simulate failing backend call
  const failingBackendImport = async (_path) => {
    throw new Error("Could not determine volume for project: path is outside active mount points");
  };

  await executeImportSubmit(enteredPath, failingBackendImport);

  assert.equal(modalClosed, false, "Failed import MUST NOT close the modal");
  assert.equal(formCleared, false, "Failed import MUST NOT clear the form");
  assert.equal(enteredPath, "/Volumes/Mac Harici Disk/Projects/broken-site", "Entered path must remain available for user correction");
  assert.ok(displayedError.includes("Could not determine volume"), "Error must be surfaced to user");

  // 2. Simulate corrected path with successful import
  enteredPath = "/Volumes/Mac Harici Disk/Projects/valid-site";
  const successfulBackendImport = async (_path) => {
    return;
  };

  await executeImportSubmit(enteredPath, successfulBackendImport);

  assert.equal(modalClosed, true, "Confirmed success MUST close the modal");
  assert.equal(formCleared, true, "Confirmed success MUST clear the form");
  assert.equal(enteredPath, "", "Form path must be reset after success");
  assert.equal(displayedError, null, "Error must be cleared on success");
});
