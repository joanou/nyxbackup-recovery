// Copyright (c) 2026 AltDrive, LLC
// SPDX-License-Identifier: Apache-2.0
// Nyx Backup Recovery - https://nyxbackup.com

//! Tauri-side adapter for the shared Google Drive OAuth flow in
//! [`bkp_oauth::google`].  Wires the Tauri `cancel-google-oauth` event
//! to the shared `CancellationToken` and re-exports the result.

use bkp_oauth::google::{GoogleCreds, GoogleDriveOAuthResult, run_oauth_flow as shared_run};
use serde::Serialize;
use tauri::{AppHandle, Listener};
use tokio_util::sync::CancellationToken;

// Compiled in from GOOGLE_OAUTH_CLIENT_ID / GOOGLE_OAUTH_CLIENT_SECRET.
// Set these in .env at the workspace root before running build_windows.sh.
const BUNDLED_CLIENT_ID: &str = env!("GOOGLE_OAUTH_CLIENT_ID");
const BUNDLED_CLIENT_SECRET: &str = env!("GOOGLE_OAUTH_CLIENT_SECRET");

/// Wrapper that re-exposes the shared result as `Serialize` so the Tauri
/// command can pass it straight to the frontend as JSON.
#[derive(Serialize)]
pub struct GoogleOAuthFrontend {
    pub folder_id: String,
    pub refresh_token: String,
    pub email: String,
}

impl From<GoogleDriveOAuthResult> for GoogleOAuthFrontend {
    fn from(r: GoogleDriveOAuthResult) -> Self {
        Self {
            folder_id: r.folder_id,
            refresh_token: r.refresh_token,
            email: r.email,
        }
    }
}

fn creds() -> GoogleCreds<'static> {
    GoogleCreds {
        client_id: BUNDLED_CLIENT_ID,
        client_secret: BUNDLED_CLIENT_SECRET,
    }
}

pub async fn run_oauth_flow(
    folder_url: String,
    app: AppHandle,
) -> anyhow::Result<GoogleOAuthFrontend> {
    let cancel = CancellationToken::new();
    let cancel_for_event = cancel.clone();
    let _unlisten = app.listen("cancel-google-oauth", move |_| {
        cancel_for_event.cancel();
    });

    let result = shared_run(&folder_url, creds(), cancel).await?;
    Ok(result.into())
}

/// Manual (no-local-browser) relay, step 1: `(auth_url, redirect_uri)`.
pub fn manual_auth_url() -> anyhow::Result<(String, String)> {
    bkp_oauth::google::manual_auth_url(&creds())
}

/// Manual relay, step 2: exchange the pasted `code` for a refresh token.
/// `folder_url` is needed to resolve the Drive folder ID.
pub async fn exchange_code(
    folder_url: String,
    code: String,
    redirect_uri: String,
) -> anyhow::Result<GoogleOAuthFrontend> {
    Ok(
        bkp_oauth::google::exchange_code(&creds(), &folder_url, &code, &redirect_uri)
            .await?
            .into(),
    )
}
