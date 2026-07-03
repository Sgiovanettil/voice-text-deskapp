import { useTranslation } from "react-i18next";

import "./App.css";

function App() {
  const { t } = useTranslation();

  return (
    <main className="container">
      <h1>{t("settings.title")}</h1>
      <p>{t("settings.subtitle")}</p>
    </main>
  );
}

export default App;
