# VoiceText (nombre provisional)

> Aplicación de escritorio multiplataforma (Windows y Linux) de dictado por voz, residente en
> el sistema, activada por hotkey global, con arquitectura por capacidades y orientada a
> eventos, preparada para crecer hacia una plataforma de interacción con IA desde cualquier
> parte del sistema operativo.

**Estado del proyecto:** en desarrollo (M0 — Fundaciones completo, ver [ROADMAP](docs/ROADMAP.md)). El MVP todavía no es funcional.

## Matriz de soporte

| Plataforma | Estado objetivo |
|---|---|
| Windows 10/11 | Primera clase |
| Linux X11 | Primera clase |
| Linux Wayland (GNOME/KDE) | Primera clase, con degradación funcional explícita donde el protocolo lo imponga (ver [ADR-0004](docs/adr/0004-estrategia-linux-x11-wayland.md)) |
| macOS | Fuera de alcance del MVP |

## Documentación

- [PRD](docs/PRD.md) — visión, alcance, requisitos y roadmap.
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — arquitectura técnica detallada.
- [ADRs](docs/adr/) — decisiones de arquitectura.
- [docs/ROADMAP.md](docs/ROADMAP.md) — seguimiento de avance por hito.
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — guía de desarrollo y checklist de smoke test.
- [AGENTS.md](AGENTS.md) — comandos y convenciones operativas.

## Privacidad y proveedor STT

El proveedor de transcripción por defecto es OpenAI (`gpt-4o-mini-transcribe`). El audio solo
se envía al proveedor configurado por acción explícita del usuario y nunca se persiste en
disco salvo flag de debug explícito. Política de retención del proveedor: TODO — documentar
enlace a la política vigente de OpenAI antes de la primera release (PRD §15.8).

## Actualizaciones automáticas

La app comprueba si hay una versión nueva al arrancar (y desde **Configuración → Acerca de →
Buscar actualizaciones**). Si la hay, avisa y actualiza solo con tu confirmación; nunca instala
en silencio. Los bundles vienen firmados y verificados (updater oficial de Tauri, ver
[ADR-0010](docs/adr/0010-auto-update-preparado.md) y [docs/RELEASING.md](docs/RELEASING.md)).

En **Linux** el auto-update cubre el **AppImage**; el paquete **`.deb`** se actualiza por el
gestor de paquetes de la distribución, no por la app.

## Licencia

TODO — pendiente de decidir.
