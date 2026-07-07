DELETE FROM app_versions;

INSERT INTO app_versions (
    version_code,
    version_name,
    download_url,
    release_notes,
    is_mandatory
)
VALUES (
    21,
    '2.1.0',
    'https://rust-labs.onrender.com/api/assets/app-release.apk',
    'Update 2.1: perbaikan koneksi chat realtime WebSocket, konfigurasi API production, dan peningkatan stabilitas OTA.',
    FALSE
);
