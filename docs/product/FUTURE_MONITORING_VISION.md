# LocalForge — Future Monitoring Subsystem Vision

**Document Version**: 1.0.0  
**Status**: ROADMAP SPECIFICATION (Not Implemented in R0 / R1)  
**Target Milestone**: Phase R2 / Phase 3 Extensions  

---

## 1. Executive Concept

Modern developers need visibility beyond their local workstation. Once a project is deployed or running on remote staging/production infrastructure, developers frequently switch between disparate monitoring tools (Pingdom, Datadog, UptimeRobot, Cloudflare).

The **LocalForge Monitoring Subsystem** envisions an integrated, calm monitoring experience designed into the Spatial Studio interface, bridging local runtime health with remote fleet observability.

---

## 2. Three Distinct Monitoring Tiers (Architectural Honesty)

To preserve architectural truth and avoid misleading claims:

```
+-----------------------------------------------------------------------------------------+
|                                  MONITORING TIERS                                       |
+-----------------------------------------------------------------------------------------+
| Tier 1: Local In-App Polling                                                            |
| -> Active ONLY while the LocalForge desktop application is running.                     |
| -> Performs HTTP probes to local loopback domains (e.g. http://myproject.test:8080).    |
| -> Zero cloud dependencies; completely stops when app is closed.                       |
+-----------------------------------------------------------------------------------------+
| Tier 2: macOS Background Daemon (Launchd Agent - Future Optional Service)               |
| -> Lightweight unprivileged Launchd daemon running under user space.                    |
| -> Periodically polls local loopback or local MariaDB/PHP-FPM sockets.                 |
| -> Clearly identified in UI as "Local Background Agent (Active / Inactive)".            |
+-----------------------------------------------------------------------------------------+
| Tier 3: Remote Server Agent & Cloud Heartbeat (Future Hosted / Self-Hosted)             |
| -> Lightweight headless binary installed on remote production/staging servers.          |
| -> Pushes encrypted metrics (CPU, RAM, Disk, SSL expiry) to LocalForge or webhook.      |
| -> Never assumed or simulated unless explicitly connected and authenticated.            |
+-----------------------------------------------------------------------------------------+
```

---

## 3. Potential Future Capabilities

### 3.1 Website & HTTP Health Monitoring
- **Periodic HTTP/HTTPS Probes**: Configurable interval (1m, 5m, 15m) monitoring status codes (200 OK, 301, 500).
- **SSL Certificate Expiration Watchdog**: Warns when TLS certificates are within 30, 14, or 7 days of expiration.
- **Response Time & Latency Metrics**: Rolling average response time tracked on micro-sparkline charts.

### 3.2 Remote Infrastructure Observability
- **CPU & Memory Utilization**: Live telemetry from monitored remote servers.
- **Disk Space Saturation Alerts**: Proactive notifications before remote storage exhausts (80%, 90%, 95% thresholds).
- **Service Process Status**: Verification that remote Nginx, PHP-FPM, or MySQL services are actively listening.

### 3.3 Deployment Correlation & Incident Timeline
- **Correlation Engine**: Automatically overlays deployment events (from the LocalForge deployment ledger) onto the monitoring timeline.
  *Example: "Latency spiked from 85ms to 1,420ms immediately following Deployment prev_49f82 at 14:22 UTC."*
- **Incident Ledger**: Chronological log of degraded states, failed probes, and auto-recovery events.

### 3.4 AI-Assisted Diagnostics
- **Context-Aware Failure Analysis**: When an HTTP 502 Bad Gateway or MariaDB connection failure occurs, the AI command bar can synthesize recent deployment diffs, Nginx error logs, and PHP-FPM pool socket status to suggest probable root causes.

---

## 4. Operational Invariants & Safeguards

1. **No Simulated Health**: If the desktop application is closed and no daemon is running, the UI MUST NOT display green "Monitored 24/7" indicators.
2. **Bandwidth & Resource Conservation**: Polling intervals must back off exponentially during sleep or metered connections.
3. **Privacy & Data Sovereignty**: Telemetry remains strictly local or sent directly to developer-owned endpoints. No user project telemetry is harvested by external servers.
