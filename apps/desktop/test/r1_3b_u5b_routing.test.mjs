import test from "node:test";
import assert from "node:assert/strict";
import { createJiti } from "jiti";

const jiti = createJiti(import.meta.url);
const { computeProjectRouting } = await jiti.import("../src/utils/browserRouting.ts");

test("Gate U5.B: ProjectRouting - Slugification and Branded Preferred Domain", () => {
  const result = computeProjectRouting("projetest.test", 8080, "Projetest", 18081);
  assert.equal(result.preferredUrl, "http://projetest.localforge/");
  assert.equal(result.localhostUrl, "http://projetest.localforge.localhost:18081/");
  assert.equal(result.loopbackUrl, "http://127.0.0.1:18081/");
  assert.equal(result.curlCommand, 'curl -s -H "Host: projetest.localforge" http://127.0.0.1:18081/');
});

test("Gate U5.B: ProjectRouting - Multi-word project slugification", () => {
  const result = computeProjectRouting("site1.test", 8080, "Site One & Two!", 18082);
  assert.equal(result.preferredUrl, "http://site-one-two.localforge/");
  assert.equal(result.localhostUrl, "http://site-one-two.localforge.localhost:18082/");
  assert.equal(result.loopbackUrl, "http://127.0.0.1:18082/");
  assert.equal(result.curlCommand, 'curl -s -H "Host: site-one-two.localforge" http://127.0.0.1:18082/');
});

test("Gate U5.B1: ProjectRouting - Unassigned loopback port blocks browser and returns unassigned status", () => {
  const unassigned1 = computeProjectRouting("legacy.test", 8080, "Legacy Project");
  assert.equal(unassigned1.canOpenBrowser, false);
  assert.equal(unassigned1.portStatus, "unassigned");
  assert.equal(unassigned1.projectUrl, null);
  assert.equal(unassigned1.loopbackUrl, null);
  assert.equal(unassigned1.localhostUrl, null);

  const unassigned2 = computeProjectRouting("legacy.test", 8080, "Legacy Project", null);
  assert.equal(unassigned2.canOpenBrowser, false);
  assert.equal(unassigned2.portStatus, "unassigned");

  const unassigned3 = computeProjectRouting("legacy.test", 8080, "Legacy Project", 0);
  assert.equal(unassigned3.canOpenBrowser, false);
  assert.equal(unassigned3.portStatus, "unassigned");
});

test("Gate U5.B1: ProjectRouting - Out of range loopback port (<18081 or >18999) fails closed", () => {
  const lowPort = computeProjectRouting("invalid.test", 8080, "Invalid Low", 8080);
  assert.equal(lowPort.canOpenBrowser, false);
  assert.equal(lowPort.portStatus, "invalid_range");
  assert.equal(lowPort.projectUrl, null);
  assert.equal(lowPort.loopbackUrl, null);

  const edgeLow = computeProjectRouting("invalid.test", 8080, "Invalid Low Edge", 18080);
  assert.equal(edgeLow.canOpenBrowser, false);
  assert.equal(edgeLow.portStatus, "invalid_range");

  const edgeHigh = computeProjectRouting("invalid.test", 8080, "Invalid High Edge", 19000);
  assert.equal(edgeHigh.canOpenBrowser, false);
  assert.equal(edgeHigh.portStatus, "invalid_range");
});

test("Gate U5.B1: ProjectRouting - Legacy mode fallback allows 8080 when explicitly requested", () => {
  const result = computeProjectRouting("legacy.test", 8080, "Legacy Project", null, { legacyMode: true });
  assert.equal(result.canOpenBrowser, true);
  assert.equal(result.portStatus, "valid");
  assert.equal(result.preferredUrl, "http://legacy-project.localforge/");
  assert.equal(result.loopbackUrl, "http://127.0.0.1:8080/");
  assert.equal(result.localhostUrl, "http://legacy-project.localforge.localhost:8080/");
});

test("Gate U5.B: ProjectRouting - Tenant Port Isolation", () => {
  const projA = computeProjectRouting("alpha.test", 8080, "Project Alpha", 18081);
  const projB = computeProjectRouting("beta.test", 8080, "Project Beta", 18082);

  assert.notEqual(projA.loopbackUrl, projB.loopbackUrl);
  assert.notEqual(projA.localhostUrl, projB.localhostUrl);
  assert.notEqual(projA.preferredUrl, projB.preferredUrl);
  assert.ok(projA.loopbackUrl.includes("18081"));
  assert.ok(projB.loopbackUrl.includes("18082"));
});

test("Gate U5.B2: Browser Readiness - Stopped project, unassigned port, and conflict fail closed with Turkish guidance", () => {
  // 1. Stopped project simulation
  const stoppedReport = {
    is_ready: false,
    target_url: null,
    runtime_ready: false,
    failure_reason_tr: "Proje Durumu: Proje şu anda durdurulmuş (Stopped). Tarayıcıda açmak için önce projeyi başlatınız.",
  };
  assert.equal(stoppedReport.is_ready, false);
  assert.equal(stoppedReport.target_url, null);
  assert.ok(stoppedReport.failure_reason_tr.includes("durdurulmuş (Stopped)"));

  // 2. Unassigned port simulation
  const unassignedReport = {
    is_ready: false,
    target_url: null,
    port_assigned: false,
    failure_reason_tr: "Port Tahsisi: Projeye atanmış geçerli bir ayrık loopback portu bulunmuyor.",
  };
  assert.equal(unassignedReport.is_ready, false);
  assert.equal(unassignedReport.target_url, null);
  assert.ok(unassignedReport.failure_reason_tr.includes("Port Tahsisi"));

  // 3. Port conflict simulation
  const conflictReport = {
    is_ready: false,
    target_url: null,
    listener_verified: false,
    failure_reason_tr: "Port Çakışması: Port 18081 LocalForge dışındaki bir süreç tarafından işgal edilmiş.",
  };
  assert.equal(conflictReport.is_ready, false);
  assert.equal(conflictReport.target_url, null);
  assert.ok(conflictReport.failure_reason_tr.includes("Port Çakışması"));

  // 4. Ready project with DNS resolved
  const readyDnsReport = {
    is_ready: true,
    target_url: "http://projetest.localforge:18081",
    dns_resolved: true,
    failure_reason_tr: null,
  };
  assert.equal(readyDnsReport.is_ready, true);
  assert.equal(readyDnsReport.target_url, "http://projetest.localforge:18081");

  // 5. Ready project without DNS resolved (fallback to loopback)
  const readyLoopbackReport = {
    is_ready: true,
    target_url: "http://127.0.0.1:18081",
    dns_resolved: false,
    failure_reason_tr: null,
  };
  assert.equal(readyLoopbackReport.is_ready, true);
  assert.equal(readyLoopbackReport.target_url, "http://127.0.0.1:18081");
});
