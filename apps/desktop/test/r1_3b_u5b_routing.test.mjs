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

test("Gate U5.B: ProjectRouting - Fallback to gateway port when loopback_port is unassigned", () => {
  const result = computeProjectRouting("legacy.test", 8080, "Legacy Project");
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
