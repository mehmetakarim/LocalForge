# LOCALFORGE — PHASE R1.1 KABUL VE KAPANIS RAPORU (RUNTIME ACCEPTANCE REPORT)

**Tarih:** 18 Ekim 2026  
**Aktif Dal (Branch):** `phase-r1/r1-1-runtime-implementation`  
**Temel Alınan Onaylı Commit (R1.0):** `80fe61c`  
**Mimari Kapsam:** Faz R1.1 — Gerçek PHP-FPM ve Nginx Çalışma Zamanı Entegrasyonu (Real PHP-FPM & Nginx Runtime Integration)  
**Doğrulama Sonucu:** **PASS (TAM BAŞARILI)**  

---

## 1. Yönetici Özeti ve Doğrulama Kararı (Executive Summary & Verification Verdict)

LocalForge projesinde simüle edilmiş çalışma zamanı (runtime mock/scaffolding) tamamen kaldırılarak yerine macOS Darwin çekirdeğiyle (`libc::proc_pidinfo`, `proc_bsdinfo`) doğrudan entegre, kimlik doğrulamalı (verified PID/process identity) ve yetkisiz (unprivileged) gerçek **PHP-FPM** ile paylaşımlı **Nginx loopback gateway** (`127.0.0.1:8080`) çalışma zamanı orkestrasyon servisi (`ProjectRuntimeService`) entegre edilmiştir.

Harici APFS depolama birimi (`/Volumes/Mac Harici Disk`) üzerindeki test projesi ile CLI ve Rust test harness üzerinden gerçekleştirilen canlı E2E testlerinde:
1. PHP-FPM havuzu unprivileged modda ve özel işlem grubunda (`setpgid(0, 0)`) başarıyla ayağa kaldırılmış,
2. Unix domain soket (`sockaddr_un`) sınırları gözetilerek FastCGI kanalı kurulmuş,
3. Nginx gateway `nginx -t` ile doğrulanıp atomik olarak projeye yönlendirilmiş,
4. Yapılan gerçek HTTP isteğinde **HTTP 200 OK** ve dinamik PHP çıktısı (`PHP 8.5.3`, `LocalForge`, `timestamp`) doğrulanmış,
5. Kapatma (`stop`) işleminde FPM proses grubu Darwin çekirdek zaman damgası doğrulamasıyla (`pbi_start_tvsec`) zarifçe sonlandırılmış, soket ve pid dosyaları silinmiş, arkada hiçbir yetim (orphan) proses bırakılmamıştır.

**Karar:** **PHASE R1.1 — AWAITING PRODUCT OWNER ACCEPTANCE.**

---

## 2. Sistem Ortamı ve Doğrulanan Önkoşullar (System Environment & Prerequisites)

* **İşletim Sistemi:** macOS (Darwin 24.x, Apple Silicon `aarch64`)
* **Harici APFS Volume:** `/Volumes/Mac Harici Disk` (UUID: `7E26530E-5179-414C-81B5-6FBDB429FC65`)
* **PHP Binary:** Homebrew PHP 8.5.3 (`/opt/homebrew/sbin/php-fpm`)
* **Nginx Binary:** Homebrew Nginx 1.31.7 (`/opt/homebrew/bin/nginx`)
* **Rust Toolchain:** `rustc 1.85+ (stable-aarch64-apple-darwin)`
* **Node & Frontend:** Vite 6.4.4 + React + TypeScript

---

## 3. Mimari ve Servis Katmanı Uygulaması (Architecture & Service Layer Implementation)

Tüm çalıştırma, durdurma, durum sorgulama ve süreç izleme mantığı paylaşımlı Rust çekirdek çalışma alanı olan `crates/runtime` altındaki `ProjectRuntimeService` etrafında birleştirilmiştir:
* **Tauri Desktop (`apps/desktop/src-tauri/src/lib.rs`):** UI doğrudan `ProjectRuntimeService` domain servisini tüketir. UI içinde hiçbir özel runtime mantığı veya shell çağrısı tekrarlanmamıştır.
* **CLI (`tools/cli/src/main.rs`):** `localforge start`, `localforge stop`, `localforge status` ve `localforge list` komutları aynı `ProjectRuntimeService` API'sini çalıştırır.
* **Süreç Sorumlusu (Supervisors):**
  * `PhpFpmSupervisor`: Bağımsız proje bazlı FPM havuzlarını yönetir (`crates/runtime/src/php.rs`).
  * `NginxSupervisor`: Paylaşımlı tekil loopback gateway'ini (`127.0.0.1:8080`) ve `conf.d/{project_id}.conf` sanal sunucu yapılandırmalarını yönetir (`crates/runtime/src/nginx.rs`).

---

## 4. PHP-FPM Süreç Yaşam Döngüsü ve Doğrulama Kanıtı (PHP-FPM Lifecycle)

* **Versiyon Doğrulama:** Çalıştırılmadan önce `php-fpm -v` çalıştırılarak istenen sürümle binary çıktısı semantik olarak karşılaştırılır.
* **İzole Havuz Yapılandırması:** Her proje için `run_dir` altında izole `[global]` ve `[{project_id}]` blokları içeren FPM yapılandırması üretilir (`pm = ondemand`, `pm.max_children = 5`).
* **Root Yetkisi İstememe:** `process_group(0)` ile geçerli kullanıcı altında arka planda çalıştırılır, `sudo` gereksinimi tamamen engellenmiştir.
* **Soket Hazırlığı:** Süreç ayağa kalktıktan sonra 3 saniyelik zaman aşımıyla Unix domain soketinin yanıt verip vermediği (`TcpStream`/`UnixStream`) aktif olarak yoklanır.

---

## 5. Süreç Kimliği ve PID Yeniden Kullanım Savunması (Process Identity & PID Reuse Defense)

Salt PID'ye güvenilerek yapılan `kill` işlemlerinin yaratacağı felaketleri (PID wrap-around / reuse yarış durumları) önlemek amacıyla Darwin çekirdeği düzeyinde doğrulama uygulanmıştır:
* `crates/platform/src/macos/process.rs`:
  * `libc::proc_pidinfo` ve `libc::proc_bsdinfo` yapısı kullanılarak hedef prosesin tam yürütülebilir dosya yolu (`proc_pidpath`) ile çekirdek düzeyindeki başlatılma zaman damgası (`pbi_start_tvsec` ve `pbi_start_tvusec`) sorgulanır.
  * Süreç sonlandırılmadan (`terminate_process_group_verified`) önce:
    1. Hedef PID'nin OS işlem tablosunda var olup olmadığı,
    2. Yürütülebilir dosya yolunun beklenen binary (`php-fpm` veya `nginx`) ile eşleşip eşleşmediği,
    3. Başlangıç zaman damgasının hafızada/disk kaydında saklanan zaman damgasıyla birebir uyuşup uyuşmadığı
    denetlenir. Eşleşmeyen durumlarda işlem derhal reddedilir (`fail-closed`).

---

## 6. FastCGI Unix Domain Socket ve 104-Bayt Sınırı Azaltımı (Socket Architecture)

macOS Darwin çekirdeğinde `sockaddr_un.sun_path` dizisi 104 bayt ile sınırlandırılmıştır. Derin kullanıcı dizinleri veya APFS mount yollarında bu sınır aşıldığında PHP-FPM soketi sessizce kırpılarak hata veriyordu:
* `PhpFpmSupervisor::socket_path()` fonksiyonunda soket yol uzunluğu denetlenir:
  * Eğer yol uzunluğu $\ge 90$ karakter ise otomatik olarak güvenli `/tmp/lf_{project_id}.sock` yoluna yönlendirilir.
  * Böylece hem yol taşması engellenmiş, hem de soket erişim izinleri (`0660`) korunmuştur.

---

## 7. Nginx Loopback Gateway ve Modüler Sanal Sunucu Yapısı (Nginx Modular Vhosts)

* **Tekil Gateway:** `127.0.0.1:8080` üzerinde çalışan tekil paylaşımlı Nginx master süreci.
* **Modüler Yapılandırma:** Her proje için `conf.d/{project_id}.conf` dosyası üretilir.
* **Atomik Doğrulama (`nginx -t`):** Yeni bir sanal sunucu dosyası yazıldığında gateway'e reload sinyali gönderilmeden önce mutlaka `nginx -t` çalıştırılır. Hata varsa dosya derhal geri alınır (rollback); çalışan diğer projelerin kesintiye uğraması engellenir.
* **Hatasız Reload:** Gateway çalışıyorsa `nginx -s reload` ile sıfır kesintiyle yeni sanal konak etkinleştirilir.

---

## 8. HTTP Loopback vs Tarayıcı Alan Adı Çözümlemesi (.test vs Host Header)

ADR-0010 ve R1.0 kararları gereğince sahte iddialardan kaçınılmıştır:
* R1.1 kapsamında işletim sistemi düzeyinde DNS/mDNS veya `/etc/hosts` yetkili manipülasyonu yapılmamıştır.
* Projeler HTTP düzleminde `Host: <project>.test` başlığı ile `127.0.0.1:8080` üzerinden tam olarak erişilebilir ve çalışır durumdadır.
* Tarayıcı üzerinden sıfır yapılandırmalı `.test` çözünürlüğü ayrı bir kabiliyet olarak Faz R1.4 / ağ katmanında ele alınacaktır.

---

## 9. Harici APFS Depolama Alanı ve Güvenli Ayrılma (Volume Barrier Safety)

* Proje kök dizini `/Volumes/Mac Harici Disk/...` üzerinde bulunurken veritabanı ve çalışma zamanı dosyaları `~/Library/Application Support/LocalForge` altında güvenle tutulur.
* `VolumeBarrier::verify_volume_available` kontrolü, harici disk unmount olduğunda veya bağlantısı kesildiğinde disk üzerinde hayalet klasörler açılmasını engeller ve projeyi derhal `Disconnected` durumuna geçirerek güvenli şekilde durdurur.

---

## 10. Çoklu Proje İzolasyonu Doğrulama Kanıtı (Multi-Project Isolation Evidence)

Entegrasyon testi `test_r1_1_06_multi_project_isolation` ile kanıtlanmıştır:
1. **Proje Alpha** (`alpha.test`) ve **Proje Beta** (`beta.test`) aynı anda başlatılmıştır.
2. Her iki proje de kendi FPM havuzlarında birbirinden bağımsız PID'lerle çalıştırılmıştır.
3. Proje Alpha durdurulduğunda (`service.stop_project(&proj_a.id)`):
   * Yalnızca Proje Alpha'nın FPM süreci sonlandırılmış,
   * Yalnızca Proje Alpha'nın Unix soketi silinmiş,
   * Proje Beta'nın FPM süreci ve soketi **kesintisiz çalışmaya devam etmiş**,
   * Paylaşımlı Nginx gateway Proje Beta'ya `HTTP 200` ile hizmet vermeyi sürdürmüştür.

---

## 11. Canlı Uçtan Uca Kabul Senaryosu Kanıtı (Live E2E Acceptance Evidence)

Harici APFS disk üzerinde `/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project` projesi ile terminal üzerinde canlı icra edilen adımlar:

### A. Proje İçe Aktarma (Import):
```bash
./target/debug/localforge import scratch/r1_acceptance_project
```
**Çıktı:**
```
✓ Successfully imported project 'r1_acceptance_project' (ID: proj_e92e56cd0e0c4c36952804a35b3587a3)
  Path:        /Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project
  PHP Version: 8.5
  Domain:      r1_acceptance_project.test
```

### B. Proje Başlatma (Start):
```bash
./target/debug/localforge start r1_acceptance_project
```
**Çıktı:**
```
Starting development runtime for project 'r1_acceptance_project'...
[18-Oct-2026 16:47:02] NOTICE: fpm is running, pid 66842
[18-Oct-2026 16:47:02] NOTICE: ready to handle connections
✓ Successfully started runtime for project 'r1_acceptance_project'
  PHP-FPM PID:      66842
  FastCGI Socket:   /tmp/lf_proj_e92e56cd0e0c4c36952804a35b3587a3.sock
  Nginx Master PID: 66845
  Local Gateway:    http://r1_acceptance_project.test:8080
  Diagnostics:      Runtime active and verified. HTTP response status: 200
```

### C. Canlı HTTP FastCGI İsteği (Curl):
```bash
curl -i -H "Host: r1_acceptance_project.test" http://127.0.0.1:8080/
```
**HTTP Yanıtı:**
```http
HTTP/1.1 200 OK
Server: nginx/1.31.7
Date: Sun, 18 Oct 2026 14:20:44 GMT
Content-Type: application/json
Transfer-Encoding: chunked
Connection: keep-alive
X-Powered-By: PHP/8.5.3

{"application":"LocalForge","runtime":"8.5.3","execution":"php-fpm","timestamp":1792333244}
```

### D. Durum ve Teşhis Sorgulama (Status):
```bash
./target/debug/localforge status r1_acceptance_project
```
**Çıktı:**
```
=== LocalForge System Diagnostics ===
MariaDB loopback (127.0.0.1:3306): Idle
Nginx Gateway (127.0.0.1:8080):    Active / Bound
Installed PHP Runtimes:            8.5.3

=== Project Details: r1_acceptance_project ===
ID:            proj_e92e56cd0e0c4c36952804a35b3587a3
Name:          r1_acceptance_project
Path:          /Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project
Volume UUID:   7E26530E-5179-414C-81B5-6FBDB429FC65
Document Root: public
PHP Version:   8.5
Local Domain:  r1_acceptance_project.test
Status:        Running
PHP-FPM PID:   Some(66842)
Socket Path:   Some("/tmp/lf_proj_e92e56cd0e0c4c36952804a35b3587a3.sock")
Nginx PID:     Some(66845)
Gateway Port:  8080
Local URL:     http://r1_acceptance_project.test:8080
Diagnostics:   Runtime active and healthy.
```

### E. Proje Durdurma ve Yetim Süreç Temizliği (Stop):
```bash
./target/debug/localforge stop r1_acceptance_project
```
**Çıktı:**
```
Stopping development runtime for project 'r1_acceptance_project'...
○ Successfully stopped runtime for project 'r1_acceptance_project'
  Status:           Stopped
  Diagnostics:      Project runtime stopped cleanly.
```
* Doğrulama: `ps -p 66842` çağrısında FPM sürecinin tamamen öldüğü, `/tmp/lf_proj_e92e56cd0e0c4c36952804a35b3587a3.sock` soketinin ve `nginx/conf.d/{id}.conf` dosyasının silindiği kesin olarak teyit edilmiştir.

---

## 12. Test Matrisi ve Doğrulama Özeti (Test Matrix)

| Test Paketi | Test İsmi | Tip | Sonuç |
|---|---|---|---|
| `localforge-deployment` | `test_approval_token_gate_rejection` | Birim/Güvenlik | **PASS** |
| `localforge-deployment` | `test_non_dry_run_fails_safely_with_deployment_not_implemented` | Güvenlik | **PASS** |
| `localforge-deployment` | `test_dry_run_simulation_does_not_mutate_ledger` | Güvenlik | **PASS** |
| `localforge-mcp` | `test_mcp_execute_non_dry_run_is_rejected` | Güvenlik | **PASS** |
| `localforge-platform` | `test_pid_reuse_protection_rejects_mismatch` | Çekirdek Güvenlik | **PASS** |
| `localforge-storage` | `test_path_traversal_detection` | Güvenlik | **PASS** |
| `localforge-runtime` | `test_pool_config_generation` | Birim | **PASS** |
| `localforge-runtime` | `test_invalid_php_version_fails_safely` | Hata Yönetimi | **PASS** |
| `localforge-runtime` | `test_nginx_config_generation` | Birim | **PASS** |
| `localforge-runtime` | `test_r1_1_01_fpm_pool_config` | Entegrasyon | **PASS** |
| `localforge-runtime` | `test_r1_1_02_nginx_vhost_validation` | Entegrasyon | **PASS** |
| `localforge-runtime` | `test_r1_1_03_real_php_fpm_and_nginx_http_execution` | E2E Entegrasyon | **PASS** |
| `localforge-runtime` | `test_r1_1_06_multi_project_isolation` | İzolasyon | **PASS** |
| **Toplam Workspace Testleri** | `cargo test --workspace` (25 test) | Otomatik | **PASS (25/25)** |
| **Statik Kod Analizi** | `cargo clippy --workspace --all-targets -- -D warnings` | Linter | **PASS (0 uyarı)** |
| **Biçimlendirme** | `cargo fmt --all -- --check` | Biçimlendirme | **PASS** |
| **Frontend Derleme** | `tsc && vite build` (apps/desktop) | Derleme | **PASS (6.91s)** |

---

## 13. Güvenlik Değerlendirmesi ve R0 Uyumluluğu (Security Compliance)

* **R0 Fail-Closed Dağıtım Koruması:** `crates/deployment/src/sftp.rs` altındaki `DeploymentNotImplemented` hatası ve onay token kapısı eksiksiz muhafaza edilmiştir.
* **Yetkisiz Çalıştırma (No Root):** Nginx ve PHP-FPM kesinlikle `root` olmadan, unprivileged kullanıcı düzeyinde çalıştırılmıştır.
* **PID Wrap-around Savunması:** Süreç sinyallemeden önce Darwin `proc_pidinfo` zaman damgası ve dosya yolu doğrulaması zorunlu tutulmuştur.
* **Yol Aşımı ve Kaçırma Koruması:** `dunce::canonicalize` ve `VolumeBarrier` ile harici disk sınırları dışına kaçışlar engellenmiştir.

---

## 14. Kabul ve Sonraki Aşama Sınırı (Next Phase Boundary)

* Faz R1.1 kapsamında hedeflenen tüm gereksinimler **sıfır mock, sıfır simülasyon ve tam gerçek zamanlı doğrulamayla** tamamlanmıştır.
* **Sıkı Faz Disiplini Kuralı:**
  * **Faz R1.2 (MariaDB & Database Resilience):** Henüz başlanmamıştır.
  * **Faz R1.3 (Spatial Studio UI Yeniden Tasarımı):** Henüz başlanmamıştır.
  * Bu aşamaların uygulamasına geçilmeden önce Ürün Sahibinin (Product Owner) bu kabul raporunu onaylaması gerekmektedir.

---

**PHASE R1.1 — AWAITING PRODUCT OWNER ACCEPTANCE.**
