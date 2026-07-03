import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

import type { ApiKeyStatus, IpcError, Settings } from "../shared/settings";
import "./App.css";

type Feedback = { kind: "ok" | "error"; text: string } | null;

function App() {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [keyStatus, setKeyStatus] = useState<ApiKeyStatus | null>(null);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [hotkeyInput, setHotkeyInput] = useState("");
  const [feedback, setFeedback] = useState<Feedback>(null);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => {
        setSettings(s);
        setHotkeyInput(s.general.hotkey);
      })
      .catch(() => {});
    invoke<ApiKeyStatus>("get_api_key_status")
      .then(setKeyStatus)
      .catch(() => {});
  }, []);

  const showError = (e: unknown) => {
    const err = e as IpcError;
    setFeedback({
      kind: "error",
      text: err?.errorKey ? t(err.errorKey) : t("err.config.invalid"),
    });
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

  const saveHotkey = () => {
    if (!settings) return;
    const updated: Settings = {
      ...settings,
      general: { ...settings.general, hotkey: hotkeyInput },
    };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        setFeedback({ kind: "ok", text: t("settings.hotkey.saved") });
      })
      .catch(showError);
  };

  return (
    <main className="container">
      <h1>{t("settings.title")}</h1>
      <p className="subtitle">{t("settings.subtitle")}</p>

      <section>
        <h2>{t("settings.apiKey.label")}</h2>
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

      <section>
        <h2>{t("settings.hotkey.label")}</h2>
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

      {feedback && (
        <p role="status" className={feedback.kind === "ok" ? "feedback-ok" : "feedback-error"}>
          {feedback.text}
        </p>
      )}
    </main>
  );
}

export default App;
