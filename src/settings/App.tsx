import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { DomainEvent } from "../shared/events";
import type {
  ApiKeyStatus,
  GeneralSettings,
  IpcError,
  Settings,
  SttSettings,
} from "../shared/settings";
import "./App.css";

type Feedback = { kind: "ok" | "error"; text: string } | null;
type CycleStatus = { kind: "idle" | "busy" | "ok" | "error"; text: string };
type SectionId = "general" | "recognition" | "shortcuts" | "about";

// Opciones fijas expuestas en la UI. Los ids de modelo son identificadores del
// proveedor (no se traducen); los idiomas se etiquetan vía i18n.
const STT_MODELS = ["gpt-4o-mini-transcribe", "gpt-4o-transcribe", "whisper-1"];
const STT_LANGUAGES = ["auto", "es", "en"];
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

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => {
        setSettings(s);
        setHotkeyInput(s.general.hotkey);
        // La UI arranca en el idioma persistido (i18n se inicializa en es).
        void i18n.changeLanguage(s.general.ui_language);
      })
      .catch(() => {});
    invoke<ApiKeyStatus>("get_api_key_status")
      .then(setKeyStatus)
      .catch(() => {});
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

  const saveApiKey = () => {
    invoke<ApiKeyStatus>("set_api_key", { key: apiKeyInput })
      .then((status) => {
        setKeyStatus(status);
        setApiKeyInput("");
        setFeedback({ kind: "ok", text: t("settings.apiKey.saved") });
      })
      .catch(showError);
  };

  const testProvider = () => {
    setTesting(true);
    invoke<boolean>("test_provider")
      .then(() => setFeedback({ kind: "ok", text: t("settings.apiKey.testOk") }))
      .catch(showError)
      .finally(() => setTesting(false));
  };

  const saveHotkey = () => patchGeneral({ hotkey: hotkeyInput }, "settings.hotkey.saved");

  return (
    <div className="shell">
      <nav className="side">
        <div className="avatar" aria-hidden="true">
          <span className="avatar-heart" />
          <span className="avatar-glass" />
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
              <h3>{t("settings.apiKey.label")}</h3>
              <p className="hint">
                {keyStatus?.isSet
                  ? t("settings.apiKey.set", { masked: keyStatus.masked })
                  : t("settings.apiKey.notSet")}
              </p>
              <div className="row">
                <input
                  type="password"
                  value={apiKeyInput}
                  onChange={(e) => setApiKeyInput(e.target.value)}
                  placeholder={t("settings.apiKey.placeholder")}
                  aria-label={t("settings.apiKey.label")}
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
                  {STT_MODELS.map((m) => (
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
          </>
        )}

        {section === "shortcuts" && (
          <>
            <h2 className="panel-title">{t("settings.nav.shortcuts")}</h2>
            <section className="group">
              <h3>{t("settings.hotkey.label")}</h3>
              <p className="hint">{t("settings.hotkey.help")}</p>
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
                {cycleStatus?.text ?? t("status.idle", { hotkey: settings?.general.hotkey ?? "…" })}
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
