# 🚀 Mobile Productivity Backend (Rust + Axum)

Backend berkinerja tinggi (High-Performance) yang dibangun menggunakan **Rust** dan **Axum** untuk mendukung ekosistem aplikasi Mobile Productivity. Sistem ini dirancang dengan arsitektur *Offline-First Sync* (seperti WhatsApp) yang menangani sinkronisasi *Real-Time Chat*, pengiriman *file/media*, dan berbagai fitur produktivitas dengan latensi yang sangat rendah.

## ✨ Fitur Utama
* 💬 **Real-Time WebSockets Chat**: Komunikasi dua arah instan yang ditenagai oleh Tokio dan WebSockets untuk pengalaman obrolan mulus.
* 🗃️ **Offline-First Sync Ready**: Menarik 24 jam riwayat obrolan secara otomatis (*lazy load*) untuk meminimalkan beban server dan mendukung *cache* SQLite lokal (HP/Desktop).
* 📁 **Native Media Storage (BLOB)**: Mengunggah dan mengirim data gambar (`.jpg`/`.png`) atau video (`.mp4`) yang secara instan dikompresi dan dikirimkan secara sekuensial.
* 🤝 **Social Friend System**: Sistem *add friend* menggunakan *Unique Social Code* (Kombinasi ID unik dan Avatar kustom).
* 📝 **Manajemen Produktivitas Terpadu**: CRUD berkecepatan tinggi untuk Todo List, Habit Tracker, Notes, dan Focus Sessions.
* 💰 **Financial Tracker**: Sistem pencatatan transaksi yang tervalidasi dan aman.

## ⚡ Keunggulan (Why Rust?)
1. **Performa Industri (Blazing Fast)**: Ditulis menggunakan `Axum` dan `Tokio` yang mampu menangani puluhan ribu koneksi WebSocket *concurrent* dengan penggunaan RAM yang sangat minim.
2. **Keamanan Memori (Memory Safety)**: Terhindar dari berbagai celah *bug* tradisional seperti *Null Pointer Dereference* berkat sistem *Ownership* Rust.
3. **Database Asynchronous Cepat**: Memanfaatkan `SQLx` dengan PostgreSQL (Supabase) yang memvalidasi kueri SQL pada saat kompilasi (Compile-Time Checked SQL), menjamin tidak ada *syntax error* pada level *runtime*.
4. **Production-Ready & Render-Optimized**: Diproteksi dengan *Dynamic Port Binding* dan *Link Time Optimization (LTO)*, membuatnya sangat hemat memori pada *Environment Free-Tier Cloud Hosting* (seperti Render.com).

## 🛠️ Persyaratan Instalasi
* **Rust** (edisi 2021 atau terbaru): [https://rustup.rs/](https://rustup.rs/)
* **PostgreSQL** Database (Bisa menggunakan Supabase).

## 🚀 Cara Menjalankan Secara Lokal (Local Development)

1. **Kloning Repositori**
   ```bash
   git clone https://github.com/rusdifn08/backend_rust.git
   cd backend_rust
   ```

2. **Konfigurasi Variabel Lingkungan**
   Buat file bernama `.env` di akar folder (satu level dengan `Cargo.toml`), lalu isi dengan kredensial PostgreSQL Anda:
   ```env
   DATABASE_URL=postgres://[user]:[password]@[host]:[port]/[database]
   ```

3. **Migrasi Database**
   Pastikan Anda telah memasang CLI *sqlx* (`cargo install sqlx-cli`). Lalu jalankan migrasi untuk membangun tabel-tabel:
   ```bash
   sqlx migrate run
   ```

4. **Jalankan Server**
   ```bash
   cargo run
   ```
   Server secara otomatis akan mendengarkan di `http://0.0.0.0:5050` (Atau *port* dinamis sesuai variabel `PORT`).

## ☁️ Cara Deploy ke Render.com
1. Sambungkan repositori ini ke *Dashboard Web Service* Render.
2. Atur **Environment** sebagai `Rust`.
3. Atur **Build Command**: `cargo build --release`.
4. Atur **Start Command**: `./target/release/backend`.
5. Masukkan `DATABASE_URL` Supabase Anda ke dalam *tab* **Environment Variables** di Render.
6. Klik Deploy!
