# VoiceText (nombre provisional)

> Aplicación de escritorio multiplataforma (Windows y Linux) de dictado por voz, residente en
> el sistema, activada por hotkey global, con arquitectura por capacidades y orientada a
> eventos, preparada para crecer hacia una plataforma de interacción con IA desde cualquier
> parte del sistema operativo.

**Estado del proyecto:** **v1.2.0** publicada. El MVP (hitos M0–M4) está completo y validado
end-to-end en Windows 11; el desarrollo continúa con mejoras post-MVP v1.x (modo toggle + corte
por silencio VAD, segundo proveedor STT Groq, selección de micrófono). Ver
[ROADMAP](docs/1-fundamentos/ROADMAP.md) para el detalle de avance por hito. Validación en Linux (X11/Wayland)
pendiente de retomar los spikes R1/R2/R3.

## Matriz de soporte

| Plataforma | Estado objetivo |
|---|---|
| Windows 10/11 | Primera clase |
| Linux X11 | Primera clase |
| Linux Wayland (GNOME/KDE) | Primera clase, con degradación funcional explícita donde el protocolo lo imponga (ver [ADR-0004](docs/2-arquitectura/DECISIONS/0004-estrategia-linux-x11-wayland.md)) |
| macOS | Fuera de alcance del MVP |

## Documentación

- [PRD](docs/1-fundamentos/PRD.md) — visión, alcance, requisitos y roadmap.
- [ARCHITECTURE.md](docs/2-arquitectura/ARCHITECTURE.md) — arquitectura técnica detallada.
- [ADRs](docs/2-arquitectura/DECISIONS/) — decisiones de arquitectura.
- [docs/1-fundamentos/ROADMAP.md](docs/1-fundamentos/ROADMAP.md) — seguimiento de avance por hito.
- [docs/3-desarrollo/SETUP_DEV.md](docs/3-desarrollo/SETUP_DEV.md) — guía de desarrollo y checklist de smoke test.
- [AGENTS.md](AGENTS.md) — comandos y convenciones operativas.

## Privacidad y proveedor STT

El proveedor de transcripción por defecto es OpenAI (`gpt-4o-mini-transcribe`); también puede
seleccionarse Groq ([ADR-0012](docs/2-arquitectura/DECISIONS/0012-segundo-proveedor-stt-groq.md)). El audio solo
se envía al proveedor configurado por acción explícita del usuario y nunca se persiste en
disco salvo flag de debug explícito. Política de retención del proveedor: TODO — documentar
enlace a la política vigente de OpenAI antes de la primera release (PRD §15.8).

## Actualizaciones automáticas

La app comprueba si hay una versión nueva al arrancar (y desde **Configuración → Acerca de →
Buscar actualizaciones**). Si la hay, avisa y actualiza solo con tu confirmación; nunca instala
en silencio. Los bundles vienen firmados y verificados (updater oficial de Tauri, ver
[ADR-0010](docs/2-arquitectura/DECISIONS/0010-auto-update-preparado.md) y [docs/3-desarrollo/DEPLOY_PREPROD.md](docs/3-desarrollo/DEPLOY_PREPROD.md)).

En **Linux** el auto-update cubre el **AppImage**; el paquete **`.deb`** se actualiza por el
gestor de paquetes de la distribución, no por la app.

## Licencia

TODO — pendiente de decidir.
