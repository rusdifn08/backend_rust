# Docker & Local Development Guide

Panduan ini berlaku dari folder `backend`.

## 1. Setup Awal

```bash
cd backend
cp .env.example .env
```

Di Windows PowerShell, gunakan:

```powershell
cd backend
Copy-Item .env.example .env
```

Pastikan nilai berikut ada di `.env`:

```env
DB_USER=postgres
DB_PASSWORD=yourpassword
DB_NAME=mobile_productivity
PORT=8080
RUST_LOG=info
JWT_SECRET=change-this-to-a-strong-secret
```

## 2. Build dan Jalankan Local Development

```bash
docker compose build
docker compose up -d
```

Compose akan menjalankan:

- `db`: PostgreSQL 16 dengan persistent volume `pgdata`.
- `api`: Rust backend development image dengan bind mount source code dan `cargo watch`.
- `sqlx database setup`: membuat schema dari folder `migrations` sebelum backend dikompilasi.

## 3. Perintah CLI Harian

```bash
docker compose ps
docker compose logs -f api
docker compose logs -f db
docker compose restart api
docker compose down
docker compose down -v
```

Keterangan:

- `docker compose ps`: melihat status container.
- `docker compose logs -f api`: melihat log backend secara realtime.
- `docker compose logs -f db`: melihat log PostgreSQL.
- `docker compose restart api`: restart backend saja.
- `docker compose down`: matikan container, data database tetap aman.
- `docker compose down -v`: matikan container dan hapus volume database.

## 4. Health Check

```bash
curl http://localhost:8080/api/system/health
```

Jika memakai PowerShell:

```powershell
Invoke-RestMethod http://localhost:8080/api/system/health
```

## 5. Koneksi Database dari Komputer Lokal

- Host: `localhost`
- Port: `5432`
- User: sesuai `DB_USER`
- Password: sesuai `DB_PASSWORD`
- Database: sesuai `DB_NAME`

## 6. Koneksi Mobile ke Backend Docker

Gunakan base URL berikut di aplikasi mobile:

| Target mobile | Base URL |
| --- | --- |
| Android Emulator | `http://10.0.2.2:8080/api` |
| iOS Simulator | `http://localhost:8080/api` |
| Physical Android/iPhone di WiFi yang sama | `http://IP_KOMPUTER_ANDA:8080/api` |

Untuk mencari IP komputer di Windows:

```powershell
ipconfig
```

Ambil `IPv4 Address` dari adapter WiFi/LAN aktif, misalnya `192.168.1.20`, lalu pakai:

```text
http://192.168.1.20:8080/api
```

Catatan:

- Pastikan HP dan komputer berada di jaringan WiFi yang sama.
- Pastikan firewall Windows mengizinkan akses masuk ke port `8080`.
- Jangan pakai `localhost` dari physical device, karena itu menunjuk ke device itu sendiri, bukan komputer.

## 7. Build Image Production

Project ini memakai `sqlx::query!`, jadi build production perlu schema database saat kompilasi atau cache `.sqlx`.

Cara praktis untuk build production dari database lokal Docker di PowerShell:

```powershell
docker compose up -d db
docker compose run --rm api sqlx database setup
docker build --target production ^
  --build-arg SQLX_OFFLINE=false ^
  --build-arg DATABASE_URL=postgresql://postgres:yourpassword@host.docker.internal:5432/mobile_productivity ^
  -t mobile-productivity-backend:prod .
```

Jika memakai bash, gunakan `\` sebagai line continuation:

```bash
docker build --target production \
  --build-arg SQLX_OFFLINE=false \
  --build-arg DATABASE_URL=postgresql://postgres:yourpassword@host.docker.internal:5432/mobile_productivity \
  -t mobile-productivity-backend:prod .
```

Untuk CI/CD yang lebih aman, buat cache SQLx terlebih dahulu lalu build offline:

```bash
cargo sqlx prepare -- --bin backend
docker build --target production --build-arg SQLX_OFFLINE=true -t mobile-productivity-backend:prod .
```

## 8. Jalankan Image Production Secara Manual

```bash
docker run --rm -p 8080:8080 \
  --env DATABASE_URL=postgresql://postgres:yourpassword@host.docker.internal:5432/mobile_productivity \
  --env JWT_SECRET=change-this-to-a-strong-secret \
  --env PORT=8080 \
  mobile-productivity-backend:prod
```
