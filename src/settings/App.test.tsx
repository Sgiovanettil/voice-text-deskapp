import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import "../i18n";
import App from "./App";

describe("Settings App", () => {
  it("renders the title from the i18n catalog (es, idioma por defecto)", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "VoiceText" })).toBeInTheDocument();
  });
});
