import React from "react";
import ReactDOM from "react-dom/client";

import "./i18n";
import Overlay from "./overlay/Overlay";
import "./overlay/Overlay.css";
import { lockDownWebChrome } from "./shared/webChrome";

lockDownWebChrome();

ReactDOM.createRoot(document.getElementById("overlay-root") as HTMLElement).render(
  <React.StrictMode>
    <Overlay />
  </React.StrictMode>,
);
