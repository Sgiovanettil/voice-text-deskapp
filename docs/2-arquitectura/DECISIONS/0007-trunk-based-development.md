# ADR-0007 — Trunk-Based Development

- **Estado:** Rechazada · **Fecha:** 2026-07-02

## Contexto
Se propuso trunk-based development (main + feature/*, releases por tag) por simplicidad para un único desarrollador.

## Decisión
**Rechazada.** El autor decidió adoptar **Git Flow (variante moderna liviana)** — main / develop / feature/* / release/* / hotfix/* — para tener un flujo profesional desde el primer día y facilitar la incorporación futura de colaboradores. Al ser una decisión de proceso (reversible, sin impacto estructural en el código) se documenta en el **PRD §11**, no como ADR propio. Este registro se conserva por trazabilidad.

## Consecuencias
- Sobrecosto de proceso asumido conscientemente (riesgo R7 del PRD, con revisión si frena la velocidad).
