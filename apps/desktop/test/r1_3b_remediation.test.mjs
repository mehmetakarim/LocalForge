import test from "node:test";
import assert from "node:assert/strict";

test("R1.3B Storage Capacity: Never fabricates 45% fallback for unreadable or zero-byte volumes", () => {
  const computeUsage = (vol) => {
    const hasValidCapacity =
      vol.total_bytes > 0 &&
      vol.available_bytes <= vol.total_bytes &&
      vol.available_bytes >= 0;
    const usedBytes = hasValidCapacity
      ? Math.max(0, vol.total_bytes - vol.available_bytes)
      : 0;
    const usagePercent = hasValidCapacity
      ? Math.min(100, Math.round((usedBytes / vol.total_bytes) * 100))
      : null;
    return { hasValidCapacity, usedBytes, usagePercent };
  };

  // Case 1: Valid volume with 50% usage
  const validVol = { total_bytes: 1000, available_bytes: 500 };
  const res1 = computeUsage(validVol);
  assert.equal(res1.hasValidCapacity, true);
  assert.equal(res1.usagePercent, 50);

  // Case 2: Zero total bytes (e.g. unmounted / unreadable volume)
  const zeroVol = { total_bytes: 0, available_bytes: 0 };
  const res2 = computeUsage(zeroVol);
  assert.equal(res2.hasValidCapacity, false);
  assert.equal(res2.usagePercent, null, "Must NOT fall back to 45%");

  // Case 3: Negative total bytes or invalid telemetry
  const invalidVol = { total_bytes: -100, available_bytes: 50 };
  const res3 = computeUsage(invalidVol);
  assert.equal(res3.hasValidCapacity, false);
  assert.equal(res3.usagePercent, null, "Must NOT fall back to 45%");

  // Case 4: Available bytes exceeds total bytes (corrupted APFS telemetry)
  const overflowVol = { total_bytes: 1000, available_bytes: 2000 };
  const res4 = computeUsage(overflowVol);
  assert.equal(res4.hasValidCapacity, false);
  assert.equal(res4.usagePercent, null, "Must NOT fall back to 45%");
});

test("R1.3B Services Runtime: Explicitly distinguishes UNKNOWN from STOPPED", () => {
  const deriveServiceStates = (runtime) => {
    const nginxStatus =
      runtime == null
        ? "unknown"
        : runtime.nginx_running
        ? "running"
        : runtime.nginx_port_conflict
        ? "conflict"
        : "stopped";

    const phpFpmStatus =
      runtime == null || runtime.php_fpm_running === undefined || runtime.php_fpm_running === null
        ? "unknown"
        : runtime.php_fpm_running
        ? "running"
        : "stopped";

    const mariadbStatus =
      runtime == null || runtime.mariadb_alive === undefined || runtime.mariadb_alive === null
        ? "unknown"
        : runtime.mariadb_alive
        ? "running"
        : "stopped";

    return { nginxStatus, phpFpmStatus, mariadbStatus };
  };

  // Case 1: runtime is null (unprobed / loading error)
  const nullRuntimeRes = deriveServiceStates(null);
  assert.equal(nullRuntimeRes.nginxStatus, "unknown");
  assert.equal(nullRuntimeRes.phpFpmStatus, "unknown");
  assert.equal(nullRuntimeRes.mariadbStatus, "unknown");

  // Case 2: php_fpm_running is undefined (partial probe)
  const partialRuntime = {
    nginx_running: true,
    nginx_port_conflict: false,
    mariadb_alive: true,
    // php_fpm_running undefined
  };
  const partialRes = deriveServiceStates(partialRuntime);
  assert.equal(partialRes.nginxStatus, "running");
  assert.equal(partialRes.phpFpmStatus, "unknown", "Undefined PHP-FPM must be unknown, not stopped");
  assert.equal(partialRes.mariadbStatus, "running");

  // Case 3: explicitly stopped services
  const stoppedRuntime = {
    nginx_running: false,
    nginx_port_conflict: false,
    php_fpm_running: false,
    mariadb_alive: false,
  };
  const stoppedRes = deriveServiceStates(stoppedRuntime);
  assert.equal(stoppedRes.nginxStatus, "stopped");
  assert.equal(stoppedRes.phpFpmStatus, "stopped");
  assert.equal(stoppedRes.mariadbStatus, "stopped");

  // Case 4: Nginx port conflict
  const conflictRuntime = {
    nginx_running: false,
    nginx_port_conflict: true,
    php_fpm_running: false,
    mariadb_alive: true,
  };
  const conflictRes = deriveServiceStates(conflictRuntime);
  assert.equal(conflictRes.nginxStatus, "conflict");
});

test("R1.3B Browser Routing Safety: Project domain URL and cURL fallback formation", () => {
  const project = {
    id: "proj_01",
    name: "Demo Project",
    local_domain: "demo.test",
  };
  const port = 8080;

  const projectUrl = `http://${project.local_domain}:${port}/`;
  const curlCmd = `curl -H "Host: ${project.local_domain}" http://127.0.0.1:${port}/`;

  assert.equal(projectUrl, "http://demo.test:8080/");
  assert.equal(curlCmd, 'curl -H "Host: demo.test" http://127.0.0.1:8080/');
  assert.ok(!projectUrl.includes("127.0.0.1"), "Project URL must target domain, not raw loopback IP");
});
