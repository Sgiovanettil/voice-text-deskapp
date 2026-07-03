# VoiceText (nombre provisional)

> Aplicación de escritorio multiplataforma (Windows y Linux) de dictado por voz, residente en
> el sistema, activada por hotkey global, con arquitectura por capacidades y orientada a
> eventos, preparada para crecer hacia una plataforma de interacción con IA desde cualquier
> parte del sistema operativo.

**Estado del proyecto:** en desarrollo (M0 — Fundaciones). El MVP todavía no es funcional.

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
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — guía de desarrollo y checklist de smoke test.
- [AGENTS.md](AGENTS.md) — comandos y convenciones operativas.

## Privacidad y proveedor STT

El proveedor de transcripción por defecto es OpenAI (`gpt-4o-mini-transcribe`). El audio solo
se envía al proveedor configurado por acción explícita del usuario y nunca se persiste en
disco salvo flag de debug explícito. Política de retención del proveedor: TODO — documentar
enlace a la política vigente de OpenAI antes de la primera release (PRD §15.8).

## Licencia

TODO — pendiente de decidir.
