//! Auto-update (ADR-0010). Activa el updater oficial de Tauri: consulta el
//! `latest.json` publicado en el Release de GitHub, compara la versión y —si el
//! usuario acepta— descarga e instala el bundle firmado.
//!
//! Todo el negocio vive acá (ADR-002): el frontend solo dispara los comandos y
//! renderiza los eventos de dominio (`UpdateAvailable`, `UpdateDownloadProgress`,
//! `UpdateFailed`) que este módulo emite al canal `domain-event`.
//!
//! En dev el updater no está disponible (no hay bundle firmado): `check` falla
//! de forma controlada y el auto-chequeo del arranque lo loguea a `debug`.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

use crate::core::events::DomainEvent;
use crate::ipc::commands::IpcError;

/// Emitir progreso a lo sumo cada ~256 KiB para no inundar el canal IPC.
const PROGRESS_THROTTLE_BYTES: u64 = 256 * 1024;

/// Metadatos de una actualización disponible, para que la UI muestre el aviso.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub pub_date: Option<String>,
}

fn network_err(e: &tauri_plugin_updater::Error) -> IpcError {
    IpcError::new(e.to_string(), "err.update.network")
}

/// Consulta el endpoint y devuelve la actualización disponible, si la hay.
/// La usan tanto el comando manual como el auto-chequeo del arranque.
pub async fn check(app: &AppHandle) -> Result<Option<UpdateInfo>, IpcError> {
    let update = app
        .updater()
        .map_err(|e| network_err(&e))?
        .check()
        .await
        .map_err(|e| network_err(&e))?;

    Ok(update.map(|u| UpdateInfo {
        version: u.version.clone(),
        notes: u.body.clone().unwrap_or_default(),
        pub_date: u.date.map(|d| d.to_string()),
    }))
}

/// Comando manual "Buscar actualizaciones": devuelve la info al frontend para
/// que muestre el banner (o el mensaje de "estás al día" si es `None`).
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, IpcError> {
    check(&app).await
}

/// Descarga e instala la actualización disponible, emitiendo progreso, y
/// reinicia la app para aplicarla. Sin actualización, es un no-op.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), IpcError> {
    let update = app
        .updater()
        .map_err(|e| network_err(&e))?
        .check()
        .await
        .map_err(|e| network_err(&e))?;

    let Some(update) = update else {
        return Ok(());
    };

    let progress_app = app.clone();
    let mut downloaded: u64 = 0;
    let mut last_emitted: u64 = 0;

    let result = update
        .download_and_install(
            move |chunk, content_length| {
                downloaded += chunk as u64;
                // Emitir el primer chunk (arranca la barra) y luego con throttle.
                if last_emitted == 0 || downloaded - last_emitted >= PROGRESS_THROTTLE_BYTES {
                    last_emitted = downloaded;
                    let _ = progress_app.emit(
                        "domain-event",
                        &DomainEvent::UpdateDownloadProgress {
                            downloaded,
                            content_length,
                        },
                    );
                }
            },
            || {},
        )
        .await;

    if let Err(e) = result {
        let _ = app.emit(
            "domain-event",
            &DomainEvent::UpdateFailed {
                error_key: "err.update.install".into(),
                detail: e.to_string(),
            },
        );
        return Err(IpcError::new(e.to_string(), "err.update.install"));
    }

    // Reinicia para aplicar. En Windows el instalador ya relanza la app; esto
    // cubre AppImage y cierra la instancia vieja de forma limpia. `restart`
    // no retorna.
    tracing::info!("actualización instalada; reiniciando");
    app.restart();
}
