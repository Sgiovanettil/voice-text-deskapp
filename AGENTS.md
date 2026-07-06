# AGENTS.md

Guía operativa para trabajar en este repo. Arquitectura, decisiones y alcance: ver
`docs/1-fundamentos/PRD.md`, `docs/2-arquitectura/ARCHITECTURE.md`, `docs/2-arquitectura/DECISIONS/`. Roadmap: `docs/1-fundamentos/ROADMAP.md`. Setup y
checklist de smoke test: `docs/3-desarrollo/SETUP_DEV.md`.

## Comandos

Frontend (raíz):

```bash
npm install
npm run dev            # vite dev server
npm run build           # tsc + vite build
npm run lint / lint:fix
npm run format:check / format
npm test                 # vitest run
```

Rust (`src-tauri/`):

```bash
cargo check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

App completa (requiere entorno gráfico — no headless):

```bash
npm run tauri dev
npm run tauri build
```

## Convenciones

- Git Flow (`main`/`develop`/`feature/*`/`release/*`/`hotfix/*`), PR obligatorio siempre,
  self-review con la checklist del PRD §11.2, squash merge a `develop`, merge commit a `main`.
- Conventional Commits (commitlint + husky); el título del PR ES el commit de squash → alimenta
  el changelog.
- Todo el negocio en Rust (ADR-002); el frontend solo presenta. Ningún `#[cfg(target_os)]` fuera
  de módulos de plataforma (`delivery/`, `hotkeys/`).
- Toda cadena visible en la UI pasa por i18n (`react-i18next`); ESLint bloquea literales JSX
  sueltos.
- El catálogo de eventos de dominio (`docs/2-arquitectura/ARCHITECTURE.md` §2, espejado en
  `src-tauri/src/core/events.rs` y `src/shared/events.ts`) ES el contrato IPC: se pueden agregar
  campos/eventos, nunca renombrar ni cambiar semántica.

## Tras clonar

```bash
graphify hook install   # una vez por máquina — mantiene graphify-out/ actualizado en cada commit
```
