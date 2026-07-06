import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { DomainEvent, UpdateInfo } from "../shared/events";
import type {
  ApiKeyStatus,
  AudioSettings,
  GeneralSettings,
  IpcError,
  Settings,
  SttSettings,
  VadSettings,
} from "../shared/settings";
import "./App.css";

type Feedback = { kind: "ok" | "error"; text: string } | null;
type CycleStatus = { kind: "idle" | "busy" | "ok" | "error"; text: string };
type SectionId = "general" | "recognition" | "shortcuts" | "about";
type UpdatePhase = "idle" | "checking" | "available" | "downloading" | "error";

// Opciones fijas expuestas en la UI. Los ids de proveedor/modelo son
// identificadores (no se traducen); los idiomas se etiquetan vía i18n.
const PROVIDERS = ["openai", "groq"] as const;
// Nombres de marca de cada proveedor (no se traducen).
const PROVIDER_NAMES: Record<string, string> = { openai: "OpenAI", groq: "Groq" };
// Modelos por proveedor; el primero es el default al elegir el proveedor
// (espejo de providers::default_model en el backend).
const MODELS_BY_PROVIDER: Record<string, string[]> = {
  openai: ["gpt-4o-mini-transcribe", "gpt-4o-transcribe", "whisper-1"],
  groq: ["whisper-large-v3-turbo", "whisper-large-v3"],
};
const STT_LANGUAGES = ["auto", "es", "en"];
// Pausas de silencio (ms) que cierran el dictado en modo toggle; espejo del
// default en config::default_silence_hangover_ms (2000).
const SILENCE_PAUSES_MS = [1200, 2000, 3000] as const;
const UI_LANGUAGES = ["es", "en"];
const SECTIONS: SectionId[] = ["general", "recognition", "shortcuts", "about"];

function App() {
  const { t, i18n } = useTranslation();
  const [section, setSection] = useState<SectionId>("general");
  const [settings, setSettings] = useState<Settings | null>(null);
  const [keyStatus, setKeyStatus] = useState<ApiKeyStatus | null>(null);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [hotkeyInput, setHotkeyInput] = useState("");
  const [feedback, setFeedback] = useState<Feedback>(null);
  const [testing, setTesting] = useState(false);
  const [cycleStatus, setCycleStatus] = useState<CycleStatus | null>(null);
  const [lastTranscript, setLastTranscript] = useState<string | null>(null);
  const [version, setVersion] = useState<string | null>(null);
  const [devices, setDevices] = useState<string[]>([]);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [updatePhase, setUpdatePhase] = useState<UpdatePhase>("idle");
  const [updateProgress, setUpdateProgress] = useState<{
    downloaded: number;
    total: number | null;
  }>({ downloaded: 0, total: null });
  const [updateError, setUpdateError] = useState<string | null>(null);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => {
        setSettings(s);
        setHotkeyInput(s.general.hotkey);
        // La UI arranca en el idioma persistido (i18n se inicializa en es).
        void i18n.changeLanguage(s.general.ui_language);
        // El estado de la key es por proveedor: se pide para el elegido.
        invoke<ApiKeyStatus>("get_api_key_status", { provider: s.stt.provider })
          .then(setKeyStatus)
          .catch(() => setKeyStatus(null));
      })
      .catch(() => {});
    invoke<string[]>("list_input_devices")
      .then(setDevices)
      .catch(() => setDevices([]));
    import("@tauri-apps/api/app")
      .then((m) => m.getVersion())
      .then(setVersion)
      .catch(() => {});
  }, [i18n]);

  useEffect(() => {
    const unlisten = listen<DomainEvent>("domain-event", ({ payload: ev }) => {
      switch (ev.event) {
        case "recordingStarted":
          setCycleStatus({ kind: "busy", text: t("status.recording") });
          break;
        case "recordingStopped":
          setCycleStatus({ kind: "busy", text: t("status.processing") });
          break;
        case "transcriptionStarted":
          setCycleStatus({ kind: "busy", text: t("status.transcribing") });
          break;
        case "transcriptionCompleted":
          setLastTranscript(ev.payload.text);
          break;
        case "textDeliveryCompleted":
          setCycleStatus({
            kind: "ok",
            text: t("status.delivered", { chars: ev.payload.chars }),
          });
          break;
        case "recordingFailed":
        case "transcriptionFailed":
        case "textDeliveryFailed":
          setCycleStatus({ kind: "error", text: t(ev.payload.errorKey) });
          break;
        case "updateAvailable":
          setUpdate(ev.payload);
          setUpdatePhase("available");
          setUpdateError(null);
          break;
        case "updateDownloadProgress":
          setUpdatePhase("downloading");
          setUpdateProgress({
            downloaded: ev.payload.downloaded,
            total: ev.payload.contentLength,
          });
          break;
        case "updateFailed":
          setUpdatePhase("error");
          setUpdateError(t(ev.payload.errorKey));
          break;
        default:
          break;
      }
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [t]);

  const showError = (e: unknown) => {
    const err = e as IpcError;
    setFeedback({
      kind: "error",
      text: err?.errorKey ? t(err.errorKey) : t("err.config.invalid"),
    });
  };

  // Persiste un cambio parcial de `general`/`stt` y refleja el estado en la UI.
  // `after` corre solo tras un guardado exitoso (p. ej. cambiar el idioma vivo).
  const patchGeneral = (patch: Partial<GeneralSettings>, okKey: string, after?: () => void) => {
    if (!settings) return;
    const updated: Settings = { ...settings, general: { ...settings.general, ...patch } };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        after?.();
        setFeedback({ kind: "ok", text: t(okKey) });
      })
      .catch(showError);
  };

  const patchStt = (patch: Partial<SttSettings>, okKey: string) => {
    if (!settings) return;
    const updated: Settings = { ...settings, stt: { ...settings.stt, ...patch } };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        setFeedback({ kind: "ok", text: t(okKey) });
      })
      .catch(showError);
  };

  const patchVad = (patch: Partial<VadSettings>, okKey: string) => {
    if (!settings) return;
    const updated: Settings = { ...settings, vad: { ...settings.vad, ...patch } };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        setFeedback({ kind: "ok", text: t(okKey) });
      })
      .catch(showError);
  };

  const patchAudio = (patch: Partial<AudioSettings>, okKey: string) => {
    if (!settings) return;
    const updated: Settings = { ...settings, audio: { ...settings.audio, ...patch } };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        setFeedback({ kind: "ok", text: t(okKey) });
      })
      .catch(showError);
  };

  // Proveedor STT elegido y su nombre de marca para las cadenas de la UI.
  const provider = settings?.stt.provider ?? "openai";
  const providerName = PROVIDER_NAMES[provider] ?? provider;

  const refreshKeyStatus = (forProvider: string) => {
    invoke<ApiKeyStatus>("get_api_key_status", { provider: forProvider })
      .then(setKeyStatus)
      .catch(() => setKeyStatus(null));
  };

  const saveApiKey = () => {
    invoke<ApiKeyStatus>("set_api_key", { provider, key: apiKeyInput })
      .then((status) => {
        setKeyStatus(status);
        setApiKeyInput("");
        setFeedback({ kind: "ok", text: t("settings.apiKey.saved") });
      })
      .catch(showError);
  };

  // Cambia de proveedor: resetea el modelo al default del nuevo proveedor y
  // refresca el estado de la key (cada proveedor tiene la suya).
  const changeProvider = (next: string) => {
    if (!settings) return;
    const model = MODELS_BY_PROVIDER[next]?.[0] ?? settings.stt.model;
    const updated: Settings = { ...settings, stt: { ...settings.stt, provider: next, model } };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        setApiKeyInput("");
        refreshKeyStatus(next);
        setFeedback({ kind: "ok", text: t("settings.stt.saved") });
      })
      .catch(showError);
  };

  const testProvider = () => {
    setTesting(true);
    invoke<boolean>("test_provider", { provider, model: settings?.stt.model ?? "" })
      .then(() => setFeedback({ kind: "ok", text: t("settings.apiKey.testOk") }))
      .catch(showError)
      .finally(() => setTesting(false));
  };

  const saveHotkey = () => patchGeneral({ hotkey: hotkeyInput }, "settings.hotkey.saved");

  // Chequeo manual: si hay versión nueva muestra el banner; si no, avisa que la
  // app está al día. El chequeo automático del arranque llega por `domain-event`.
  const checkForUpdate = () => {
    setUpdatePhase("checking");
    setUpdateError(null);
    invoke<UpdateInfo | null>("check_for_update")
      .then((info) => {
        if (info) {
          setUpdate(info);
          setUpdatePhase("available");
        } else {
          setUpdatePhase("idle");
          setFeedback({ kind: "ok", text: t("settings.update.upToDate") });
        }
      })
      .catch((e) => {
        setUpdatePhase("idle");
        const err = e as IpcError;
        setFeedback({
          kind: "error",
          text: err?.errorKey ? t(err.errorKey) : t("err.update.network"),
        });
      });
  };

  // Descarga e instala: la app se reinicia al terminar (la promesa no resuelve).
  // El progreso y los fallos llegan por `domain-event`.
  const installUpdate = () => {
    setUpdatePhase("downloading");
    setUpdateProgress({ downloaded: 0, total: null });
    setUpdateError(null);
    invoke("install_update").catch((e) => {
      setUpdatePhase("error");
      const err = e as IpcError;
      setUpdateError(err?.errorKey ? t(err.errorKey) : t("err.update.install"));
    });
  };

  return (
    <div className="shell">
      <nav className="side">
        <div className="brand">
          <div className="avatar" aria-hidden="true">
            <span className="avatar-heart" />
            <span className="avatar-glass" />
          </div>
          <div className="brand-text">
            <span className="brand-name">{t("settings.title")}</span>
            <span className="brand-sub">{t("settings.subtitle")}</span>
          </div>
        </div>
        {SECTIONS.map((id) => (
          <button
            key={id}
            className={section === id ? "nav-item active" : "nav-item"}
            onClick={() => setSection(id)}
          >
            {t(`settings.nav.${id}`)}
          </button>
        ))}
      </nav>

      <main className="panel">
        {update && (
          <div className="update-banner" role="status">
            <strong className="update-title">
              {t("settings.update.available", { version: update.version })}
            </strong>
            {updatePhase === "available" && (
              <>
                {update.notes && <p className="hint update-notes">{update.notes}</p>}
                <button className="update-cta" onClick={installUpdate}>
                  {t("settings.update.install")}
                </button>
              </>
            )}
            {updatePhase === "downloading" && (
              <div className="update-progress">
                <progress
                  value={updateProgress.total ? updateProgress.downloaded : undefined}
                  max={updateProgress.total ?? undefined}
                />
                <span className="hint">
                  {updateProgress.total
                    ? t("settings.update.downloading", {
                        percent: Math.round(
                          (updateProgress.downloaded / updateProgress.total) * 100,
                        ),
                      })
                    : t("settings.update.preparing")}
                </span>
              </div>
            )}
            {updatePhase === "error" && updateError && (
              <p className="feedback-error">{updateError}</p>
            )}
          </div>
        )}

        {section === "general" && (
          <>
            <h2 className="panel-title">{t("settings.nav.general")}</h2>

            <section className="group">
              <h3>{t("settings.language.label")}</h3>
              <p className="hint">{t("settings.language.hint")}</p>
              <label className="field">
                {t("settings.language.label")}
                <select
                  value={settings?.general.ui_language ?? "es"}
                  disabled={!settings}
                  onChange={(e) => {
                    const lang = e.target.value;
                    patchGeneral({ ui_language: lang }, "settings.language.saved", () => {
                      void i18n.changeLanguage(lang);
                    });
                  }}
                >
                  {UI_LANGUAGES.map((lang) => (
                    <option key={lang} value={lang}>
                      {t(`settings.language.options.${lang}`)}
                    </option>
                  ))}
                </select>
              </label>
            </section>

            <section className="group">
              <h3>{t("settings.startup.label")}</h3>
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={settings?.general.autostart ?? false}
                  disabled={!settings}
                  onChange={(e) =>
                    patchGeneral({ autostart: e.target.checked }, "settings.startup.saved")
                  }
                />
                {t("settings.startup.autostart")}
              </label>
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={settings?.general.start_minimized ?? false}
                  disabled={!settings}
                  onChange={(e) =>
                    patchGeneral({ start_minimized: e.target.checked }, "settings.startup.saved")
                  }
                />
                {t("settings.startup.startMinimized")}
              </label>
            </section>

            <section className="group">
              <h3>{t("settings.outputMode.label")}</h3>
              <label className="check-row">
                <input
                  type="radio"
                  name="output-mode"
                  checked={settings?.general.output_mode === "insert"}
                  disabled={!settings}
                  onChange={() =>
                    patchGeneral({ output_mode: "insert" }, "settings.outputMode.saved")
                  }
                />
                {t("settings.outputMode.insert")}
              </label>
              <label className="check-row">
                <input
                  type="radio"
                  name="output-mode"
                  checked={settings?.general.output_mode === "clipboard"}
                  disabled={!settings}
                  onChange={() =>
                    patchGeneral({ output_mode: "clipboard" }, "settings.outputMode.saved")
                  }
                />
                {t("settings.outputMode.clipboard")}
              </label>
            </section>
          </>
        )}

        {section === "recognition" && (
          <>
            <h2 className="panel-title">{t("settings.stt.label")}</h2>

            <section className="group">
              <h3>{t("settings.provider.label")}</h3>
              <label className="field">
                {t("settings.provider.label")}
                <select
                  value={provider}
                  disabled={!settings}
                  onChange={(e) => changeProvider(e.target.value)}
                >
                  {PROVIDERS.map((id) => (
                    <option key={id} value={id}>
                      {PROVIDER_NAMES[id]}
                    </option>
                  ))}
                </select>
              </label>
            </section>

            <section className="group">
              <h3>{t("settings.apiKey.label", { provider: providerName })}</h3>
              <p className="hint">
                {keyStatus?.isSet
                  ? t("settings.apiKey.set", { masked: keyStatus.masked })
                  : t("settings.apiKey.notSet")}
              </p>
              <p className="hint">{t(`settings.apiKey.hint.${provider}`)}</p>
              <div className="row">
                <input
                  type="password"
                  value={apiKeyInput}
                  onChange={(e) => setApiKeyInput(e.target.value)}
                  placeholder={t("settings.apiKey.placeholder")}
                  aria-label={t("settings.apiKey.label", { provider: providerName })}
                />
                <button onClick={saveApiKey} disabled={apiKeyInput.trim() === ""}>
                  {t("settings.apiKey.save")}
                </button>
                <button onClick={testProvider} disabled={testing || !keyStatus?.isSet}>
                  {testing ? t("settings.apiKey.testing") : t("settings.apiKey.test")}
                </button>
              </div>
            </section>

            <section className="group">
              <h3>{t("settings.stt.label")}</h3>
              <label className="field">
                {t("settings.stt.model")}
                <select
                  value={settings?.stt.model ?? ""}
                  disabled={!settings}
                  onChange={(e) => patchStt({ model: e.target.value }, "settings.stt.saved")}
                >
                  {(MODELS_BY_PROVIDER[provider] ?? []).map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </label>
              <label className="field">
                {t("settings.stt.language")}
                <select
                  value={settings?.stt.language ?? "auto"}
                  disabled={!settings}
                  onChange={(e) => patchStt({ language: e.target.value }, "settings.stt.saved")}
                >
                  {STT_LANGUAGES.map((lang) => (
                    <option key={lang} value={lang}>
                      {t(`settings.stt.languages.${lang}`)}
                    </option>
                  ))}
                </select>
              </label>
            </section>

            <section className="group">
              <h3>{t("settings.audio.label")}</h3>
              <label className="field">
                {t("settings.audio.device")}
                <select
                  value={settings?.audio.input_device ?? ""}
                  disabled={!settings}
                  onChange={(e) =>
                    patchAudio(
                      { input_device: e.target.value === "" ? null : e.target.value },
                      "settings.audio.saved",
                    )
                  }
                >
                  <option value="">{t("settings.audio.default")}</option>
                  {devices.map((d) => (
                    <option key={d} value={d}>
                      {d}
                    </option>
                  ))}
                </select>
              </label>
            </section>
          </>
        )}

        {section === "shortcuts" && (
          <>
            <h2 className="panel-title">{t("settings.nav.shortcuts")}</h2>
            <section className="group">
              <h3>{t("settings.activation.label")}</h3>
              <label className="check-row">
                <input
                  type="radio"
                  name="activation-mode"
                  checked={settings?.general.activation_mode !== "toggle"}
                  disabled={!settings}
                  onChange={() =>
                    patchGeneral({ activation_mode: "ptt" }, "settings.activation.saved")
                  }
                />
                {t("settings.activation.ptt")}
              </label>
              <label className="check-row">
                <input
                  type="radio"
                  name="activation-mode"
                  checked={settings?.general.activation_mode === "toggle"}
                  disabled={!settings}
                  onChange={() =>
                    patchGeneral({ activation_mode: "toggle" }, "settings.activation.saved")
                  }
                />
                {t("settings.activation.toggle")}
              </label>
            </section>
            {settings?.general.activation_mode === "toggle" && (
              <section className="group">
                <h3>{t("settings.pause.label")}</h3>
                <p className="hint">{t("settings.pause.hint")}</p>
                <label className="field">
                  {t("settings.pause.label")}
                  <select
                    value={String(settings.vad.silence_hangover_ms)}
                    onChange={(e) =>
                      patchVad(
                        { silence_hangover_ms: Number(e.target.value) },
                        "settings.pause.saved",
                      )
                    }
                  >
                    {SILENCE_PAUSES_MS.map((ms) => (
                      <option key={ms} value={String(ms)}>
                        {t(`settings.pause.options.${ms}`)}
                      </option>
                    ))}
                    {!SILENCE_PAUSES_MS.some((ms) => ms === settings.vad.silence_hangover_ms) && (
                      <option value={String(settings.vad.silence_hangover_ms)}>
                        {t("settings.pause.custom", {
                          seconds: settings.vad.silence_hangover_ms / 1000,
                        })}
                      </option>
                    )}
                  </select>
                </label>
              </section>
            )}
            <section className="group">
              <h3>{t("settings.hotkey.label")}</h3>
              <p className="hint">
                {settings?.general.activation_mode === "toggle"
                  ? t("settings.hotkey.helpToggle")
                  : t("settings.hotkey.help")}
              </p>
              <div className="row">
                <input
                  type="text"
                  value={hotkeyInput}
                  onChange={(e) => setHotkeyInput(e.target.value)}
                  aria-label={t("settings.hotkey.label")}
                />
                <button onClick={saveHotkey} disabled={!settings}>
                  {t("settings.hotkey.save")}
                </button>
              </div>
            </section>
          </>
        )}

        {section === "about" && (
          <>
            <h2 className="panel-title">{t("settings.nav.about")}</h2>

            <section className="group">
              <h3>{t("status.label")}</h3>
              <p role="status" className={`status status-${cycleStatus?.kind ?? "idle"}`}>
                {cycleStatus?.text ??
                  t(
                    settings?.general.activation_mode === "toggle"
                      ? "status.idleToggle"
                      : "status.idle",
                    { hotkey: settings?.general.hotkey ?? "…" },
                  )}
              </p>
              {lastTranscript !== null && (
                <>
                  <p className="hint">{t("status.lastTranscript")}</p>
                  <p className="transcript">{lastTranscript}</p>
                </>
              )}
            </section>

            <section className="group">
              <h3>{t("app.name")}</h3>
              {version !== null && (
                <p className="hint">
                  {t("settings.about.version")} {version}
                </p>
              )}
              <div className="row">
                <button
                  onClick={checkForUpdate}
                  disabled={updatePhase === "checking" || updatePhase === "downloading"}
                >
                  {updatePhase === "checking"
                    ? t("settings.update.checking")
                    : t("settings.update.check")}
                </button>
              </div>
            </section>
          </>
        )}

        {feedback && (
          <p role="status" className={feedback.kind === "ok" ? "feedback-ok" : "feedback-error"}>
            {feedback.text}
          </p>
        )}
      </main>
    </div>
  );
}

export default App;
