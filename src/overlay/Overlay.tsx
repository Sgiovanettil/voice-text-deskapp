import { useTranslation } from "react-i18next";

// Placeholder — la ventana del overlay se crea dinámicamente en runtime
// (ARCHITECTURE §8.5), solo durante el ciclo de dictado. Se conecta a los
// eventos de dominio en M2 (ver docs/ARCHITECTURE.md §4.9).
function Overlay() {
  const { t } = useTranslation();
  return <div>{t("overlay.placeholder")}</div>;
}

export default Overlay;
