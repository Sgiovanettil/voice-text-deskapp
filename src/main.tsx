import React from "react";
import ReactDOM from "react-dom/client";

import "./i18n";
import App from "./settings/App";
import { lockDownWebChrome } from "./shared/webChrome";

lockDownWebChrome();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
