# Documentación — VoiceText

Índice navegable de la documentación del proyecto, organizada según la jerarquía canónica
(`1-fundamentos/`, `2-arquitectura/`, `3-desarrollo/`) y su estado.

## 1 · Fundamentos

| Documento | Estado |
| --- | --- |
| [1-fundamentos/PRD.md](1-fundamentos/PRD.md) — visión, alcance, requisitos, roadmap | ✅ al día |
| [1-fundamentos/ROADMAP.md](1-fundamentos/ROADMAP.md) — avance por hito | ✅ al día |
| [1-fundamentos/DEFINICIONES.md](1-fundamentos/DEFINICIONES.md) — glosario de términos de dominio | ✅ al día |
| [1-fundamentos/KNOWN_ISSUES.md](1-fundamentos/KNOWN_ISSUES.md) — limitaciones y pendientes conocidos | ✅ al día |
| [1-fundamentos/brief-original.md](1-fundamentos/brief-original.md) — brief histórico de origen | ✅ referencia |

## 2 · Arquitectura

| Documento | Estado |
| --- | --- |
| [2-arquitectura/ARCHITECTURE.md](2-arquitectura/ARCHITECTURE.md) — diseño técnico, contrato IPC | ✅ al día |
| [2-arquitectura/DECISIONS/](2-arquitectura/DECISIONS/) — 14 decisiones de arquitectura (ADRs) | ✅ al día |
| [2-arquitectura/spikes/](2-arquitectura/spikes/) — R1/R2/R3 (validados en Windows; Linux pendiente) | ⚠️ parcial |

## 3 · Desarrollo

| Documento | Estado |
| --- | --- |
| [3-desarrollo/SETUP_DEV.md](3-desarrollo/SETUP_DEV.md) — setup y checklist de smoke test | ✅ al día |
| [3-desarrollo/DEPLOY_PREPROD.md](3-desarrollo/DEPLOY_PREPROD.md) — proceso de release y firma | ✅ al día |
| [3-desarrollo/PLAN_MODELOS_DINAMICOS.md](3-desarrollo/PLAN_MODELOS_DINAMICOS.md) — plan de implementación del listado dinámico de modelos por proveedor | 🔜 pendiente de implementar |

> **Changelog:** se genera con git-cliff (`cliff.toml`) al taggear y se publica como cuerpo de
> cada [GitHub Release](../../releases); no se versiona como `CHANGELOG.md`.

_Última auditoría: 2026-07-06 (docreview)._
