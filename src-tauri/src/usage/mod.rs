//! Ledger local de gastos estimados por transcripción (ADR-0015). El gasto
//! se **estima** localmente — `(segundos_audio / 60) × tarifa_por_minuto` —
//! nunca se consulta una API de facturación (privacidad primero). Acumula por
//! clave `(proveedor, modelo, mes local YYYY-MM)` en `usage.json`, archivo
//! propio separado de settings: telemetría de escritura frecuente con ciclo
//! de vida distinto (reset manual) que no debe arriesgar la config.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Settings;
use crate::persistence::PersistenceError;

/// Versión del esquema de `usage.json` (independiente de settings).
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
const USAGE_FILE: &str = "usage.json";

/// Tarifas por defecto en USD por minuto de audio, por `(proveedor, modelo)`
/// (tabla del ADR-0015; los `gpt-4o-*-transcribe` se cobran por tokens de
/// audio, así que el por-minuto es una aproximación). El usuario puede
/// sobreescribirlas en `settings.pricing.rates` — los precios cambian.
const DEFAULT_RATES: &[(&str, &str, f64)] = &[
    ("openai", "gpt-4o-mini-transcribe", 0.003),
    ("openai", "gpt-4o-transcribe", 0.006),
    ("openai", "whisper-1", 0.006),
    ("groq", "whisper-large-v3-turbo", 0.00067),
    ("groq", "whisper-large-v3", 0.00185),
    ("groq", "distil-whisper-large-v3-en", 0.00033),
];

/// Acumulado de una clave `(proveedor, modelo, mes)` del ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageEntry {
    pub provider: String,
    pub model: String,
    /// Mes local `YYYY-MM` en que se dictó.
    pub month: String,
    pub transcriptions: u32,
    pub audio_seconds: f64,
    /// Gasto estimado acumulado, calculado con la tarifa vigente al momento
    /// de cada dictado (editar una tarifa no reescribe el histórico).
    pub estimated_cost_usd: f64,
}

/// Contenido completo de `usage.json`; también es el DTO de `get_usage`
/// (snake_case en el wire, como `Settings`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageLedger {
    pub schema_version: u32,
    pub entries: Vec<UsageEntry>,
}

impl Default for UsageLedger {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }
}

fn usage_path(config_dir: &Path) -> PathBuf {
    config_dir.join(USAGE_FILE)
}

/// Mes local actual en formato `YYYY-MM` (los proveedores facturan por ciclos
/// mensuales; el desglose usa la misma unidad).
pub fn current_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

/// Carga el ledger. Archivo ausente o corrupto = ledger vacío (misma
/// filosofía que settings: la app nunca deja de arrancar por telemetría).
pub fn load(config_dir: &Path) -> UsageLedger {
    let path = usage_path(config_dir);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return UsageLedger::default(),
        Err(e) => {
            tracing::warn!(error = %e, ?path, "no se pudo leer usage.json — ledger vacío");
            return UsageLedger::default();
        }
    };
    match serde_json::from_str::<UsageLedger>(&raw) {
        Ok(ledger) if ledger.schema_version <= CURRENT_SCHEMA_VERSION => ledger,
        Ok(ledger) => {
            tracing::warn!(
                found = ledger.schema_version,
                "usage.json de una versión más nueva — ledger vacío"
            );
            UsageLedger::default()
        }
        Err(e) => {
            tracing::warn!(error = %e, ?path, "usage.json inválido — ledger vacío");
            UsageLedger::default()
        }
    }
}

/// Escritura atómica (tmp + rename), como settings.
pub fn save(config_dir: &Path, ledger: &UsageLedger) -> Result<(), PersistenceError> {
    std::fs::create_dir_all(config_dir)?;
    let path = usage_path(config_dir);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(ledger)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// Registra una transcripción exitosa en el mes local actual. Load-modify-save
/// completo por dictado: la frecuencia es humana (un write por dictado), no
/// amerita caché en memoria.
pub fn record(
    config_dir: &Path,
    provider: &str,
    model: &str,
    audio_seconds: f64,
    rate_per_min: f64,
) -> Result<(), PersistenceError> {
    let month = current_month();
    let mut ledger = load(config_dir);
    let entry = ledger
        .entries
        .iter_mut()
        .find(|e| e.provider == provider && e.model == model && e.month == month);
    match entry {
        Some(e) => {
            e.transcriptions += 1;
            e.audio_seconds += audio_seconds;
            e.estimated_cost_usd += (audio_seconds / 60.0) * rate_per_min;
        }
        None => ledger.entries.push(UsageEntry {
            provider: provider.into(),
            model: model.into(),
            month,
            transcriptions: 1,
            audio_seconds,
            estimated_cost_usd: (audio_seconds / 60.0) * rate_per_min,
        }),
    }
    save(config_dir, &ledger)
}

/// Borra el acumulado (reset manual del ADR-0015): de un proveedor, o todo.
/// Las tarifas no se tocan (viven en settings).
pub fn reset(config_dir: &Path, provider: Option<&str>) -> Result<UsageLedger, PersistenceError> {
    let mut ledger = load(config_dir);
    match provider {
        Some(p) => ledger.entries.retain(|e| e.provider != p),
        None => ledger.entries.clear(),
    }
    save(config_dir, &ledger)?;
    Ok(ledger)
}

/// Tarifa por defecto (USD/min) embarcada en código; `None` para modelos sin
/// precio conocido (contarán segundos con costo 0 hasta que el usuario fije
/// una tarifa).
pub fn default_rate(provider: &str, model: &str) -> Option<f64> {
    DEFAULT_RATES
        .iter()
        .find(|(p, m, _)| *p == provider && *m == model)
        .map(|(_, _, rate)| *rate)
}

/// Clave de override en `settings.pricing.rates`.
fn rate_key(provider: &str, model: &str) -> String {
    format!("{provider}/{model}")
}

/// Tarifa efectiva: override del usuario > default en código > 0.0.
pub fn effective_rate(settings: &Settings, provider: &str, model: &str) -> f64 {
    settings
        .pricing
        .rates
        .get(&rate_key(provider, model))
        .copied()
        .or_else(|| default_rate(provider, model))
        .unwrap_or(0.0)
}

/// Tabla de tarifas efectivas para la UI: defaults en código fusionados con
/// los overrides del usuario (estos ganan), claves `proveedor/modelo`.
pub fn effective_rates(settings: &Settings) -> HashMap<String, f64> {
    let mut rates: HashMap<String, f64> = DEFAULT_RATES
        .iter()
        .map(|(p, m, rate)| (rate_key(p, m), *rate))
        .collect();
    for (key, rate) in &settings.pricing.rates {
        rates.insert(key.clone(), *rate);
    }
    rates
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("voicetext-usage-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn load_sin_archivo_devuelve_ledger_vacio() {
        let dir = temp_config_dir("load-missing");
        assert_eq!(load(&dir), UsageLedger::default());
    }

    #[test]
    fn record_crea_y_acumula_por_clave() {
        let dir = temp_config_dir("record");
        record(&dir, "openai", "whisper-1", 60.0, 0.006).unwrap();
        record(&dir, "openai", "whisper-1", 30.0, 0.006).unwrap();
        record(&dir, "groq", "whisper-large-v3-turbo", 60.0, 0.00067).unwrap();

        let ledger = load(&dir);
        assert_eq!(ledger.entries.len(), 2);
        let openai = ledger
            .entries
            .iter()
            .find(|e| e.provider == "openai")
            .unwrap();
        assert_eq!(openai.transcriptions, 2);
        assert!((openai.audio_seconds - 90.0).abs() < f64::EPSILON);
        assert!((openai.estimated_cost_usd - 0.009).abs() < 1e-9);
        assert_eq!(openai.month, current_month());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_por_proveedor_y_global() {
        let dir = temp_config_dir("reset");
        record(&dir, "openai", "whisper-1", 60.0, 0.006).unwrap();
        record(&dir, "groq", "whisper-large-v3-turbo", 60.0, 0.00067).unwrap();

        let ledger = reset(&dir, Some("openai")).unwrap();
        assert_eq!(ledger.entries.len(), 1);
        assert_eq!(ledger.entries[0].provider, "groq");

        let ledger = reset(&dir, None).unwrap();
        assert!(ledger.entries.is_empty());
        assert!(load(&dir).entries.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn archivo_corrupto_cae_a_ledger_vacio() {
        let dir = temp_config_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(USAGE_FILE), "{esto no es json").unwrap();
        assert_eq!(load(&dir), UsageLedger::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn schema_del_futuro_cae_a_ledger_vacio() {
        let dir = temp_config_dir("future");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(USAGE_FILE),
            r#"{ "schema_version": 99, "entries": [] }"#,
        )
        .unwrap();
        assert_eq!(load(&dir), UsageLedger::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_rate_prioriza_override_luego_default() {
        let mut settings = Settings::default();
        // Sin override: default de código.
        assert!((effective_rate(&settings, "openai", "whisper-1") - 0.006).abs() < 1e-9);
        // Modelo desconocido sin override: 0.
        assert_eq!(effective_rate(&settings, "openai", "modelo-nuevo"), 0.0);
        // Override del usuario gana al default.
        settings
            .pricing
            .rates
            .insert("openai/whisper-1".into(), 0.01);
        assert!((effective_rate(&settings, "openai", "whisper-1") - 0.01).abs() < 1e-9);
    }

    #[test]
    fn effective_rates_fusiona_overrides_sobre_defaults() {
        let mut settings = Settings::default();
        settings
            .pricing
            .rates
            .insert("openai/whisper-1".into(), 0.01);
        settings
            .pricing
            .rates
            .insert("openai/modelo-nuevo".into(), 0.002);
        let rates = effective_rates(&settings);
        assert!((rates["openai/whisper-1"] - 0.01).abs() < 1e-9);
        assert!((rates["openai/gpt-4o-transcribe"] - 0.006).abs() < 1e-9);
        assert!((rates["openai/modelo-nuevo"] - 0.002).abs() < 1e-9);
    }

    #[test]
    fn current_month_tiene_formato_yyyy_mm() {
        let month = current_month();
        assert_eq!(month.len(), 7);
        assert_eq!(&month[4..5], "-");
        assert!(month[0..4].chars().all(|c| c.is_ascii_digit()));
        assert!(month[5..7].chars().all(|c| c.is_ascii_digit()));
    }
}
