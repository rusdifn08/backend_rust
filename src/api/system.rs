use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OtaManifest {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
}

pub async fn get_latest_ota() -> Result<Json<OtaManifest>, (StatusCode, String)> {
    // In a real app, this might come from a database or config file.
    // We are currently on version 1.0.0. We will serve version 1.1.0 for OTA.
    let manifest = OtaManifest {
        version: "1.1.0".to_string(),
        download_url: "https://rust-labs.onrender.com/api/assets/app-release.apk".to_string(),
        release_notes: "Added data isolation per user, fixed empty store, and synced to-dos with the server!".to_string(),
    };

    Ok(Json(manifest))
}
