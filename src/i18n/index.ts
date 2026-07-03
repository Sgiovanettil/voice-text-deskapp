// Sistema de i18n (react-i18next), español por defecto (PRD §17, ratificado
// por el autor). Las claves de error del backend (`err.*`, ver
// docs/ARCHITECTURE.md §4.10) viven en el mismo catálogo que las cadenas de
// UI: el backend envía la clave, el frontend la traduce.

import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import en from "./en/translation.json";
import es from "./es/translation.json";

void i18n.use(initReactI18next).init({
  resources: {
    es: { translation: es },
    en: { translation: en },
  },
  lng: "es",
  fallbackLng: "es",
  interpolation: { escapeValue: false },
});

export default i18n;
