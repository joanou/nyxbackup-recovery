// Copyright (c) 2026 AltDrive, LLC
// SPDX-License-Identifier: Apache-2.0
// Nyx Backup Recovery - https://nyxbackup.com

//! Tauri-side adapter for the shared Dropbox OAuth flow in
//! [`bkp_oauth::dropbox`].  Wires the Tauri `cancel-dropbox-oauth` event
//! to the shared `CancellationToken` and re-exports the result.

use bkp_oauth::dropbox::{DropboxCreds, DropboxOAuthResult, run_oauth_flow as shared_run};
use serde::Serialize;
use tauri::{AppHandle, Listener};
use tokio_util::sync::CancellationToken;

// Compiled in from DROPBOX_APP_KEY / DROPBOX_APP_SECRET.
// Set these in .env at the workspace root before running build_windows.sh.
const BUNDLED_APP_KEY: &str = env!("DROPBOX_APP_KEY");
const BUNDLED_APP_SECRET: &str = env!("DROPBOX_APP_SECRET");

#[derive(Serialize)]
pub struct DropboxOAuthFrontend {
    pub refresh_token: String,
    pub email: String,
}

impl From<DropboxOAuthResult> for DropboxOAuthFrontend {
    fn from(r: DropboxOAuthResult) -> Self {
        Self {
            refresh_token: r.refresh_token,
            email: r.email,
        }
    }
}

fn creds() -> DropboxCreds<'static> {
    DropboxCreds {
        app_key: BUNDLED_APP_KEY,
        app_secret: BUNDLED_APP_SECRET,
    }
}

pub async fn run_oauth_flow(app: AppHandle) -> anyhow::Result<DropboxOAuthFrontend> {
    let cancel = CancellationToken::new();
    let cancel_for_event = cancel.clone();
    let _unlisten = app.listen("cancel-dropbox-oauth", move |_| {
        cancel_for_event.cancel();
    });

    let result = shared_run(creds(), cancel).await?;
    Ok(result.into())
}

/// Manual (no-local-browser) relay, step 1: `(auth_url, redirect_uri)`.
pub fn manual_auth_url() -> anyhow::Result<(String, String)> {
    bkp_oauth::dropbox::manual_auth_url(&creds())
}

/// Manual relay, step 2: exchange the pasted `code` for a refresh token.
pub async fn exchange_code(
    code: String,
    redirect_uri: String,
) -> anyhow::Result<DropboxOAuthFrontend> {
    Ok(
        bkp_oauth::dropbox::exchange_code(&creds(), &code, &redirect_uri)
            .await?
            .into(),
    )
}
