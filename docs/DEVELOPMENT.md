# Guía de desarrollo — VoiceText

## Prerrequisitos por SO

### Windows

- [Rust](https://rustup.rs/) (toolchain estable, MSVC).
- [Node.js](https://nodejs.org/) ≥ 22 y npm ≥ 11 (`npm install -g npm@11` si tu Node trae una versión anterior).
- [Microsoft C++ Build Tools](https://tauri.app/start/prerequisites/) (requeridos por Tauri).
- WebView2 (viene preinstalado en Windows 10/11 actualizados).

### Linux (Debian/Ubuntu)

```bash
sudo apt-get update && sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
  build-essential curl wget file libssl-dev libgtk-3-dev patchelf \
  libasound2-dev
```

`libasound2-dev` (ALSA) es requerido por `cpal` (captura de audio) incluso solo para compilar — sin él, `cargo build`/`check` falla en la etapa de build de `alsa-sys`.

Rust vía [rustup](https://rustup.rs/), Node ≥ 22 y npm ≥ 11 igual que en Windows.

### Linux (Wayland)

Los spikes R1 (hotkey) y parte de R3 (inserción de texto) dependen de `xdg-desktop-portal` con soporte para `GlobalShortcuts` y `RemoteDesktop`. Verificar versión instalada: `xdg-desktop-portal --version` (ver notas en `docs/spikes/r1-hotkey-portal.md` tras ejecutar el spike).

## Comandos

Ver también [`AGENTS.md`](../AGENTS.md) para el resumen rápido. Detalle:

```bash
# Frontend
npm install
npm run dev              # vite dev server (solo frontend, sin ventana Tauri)
npm run build             # tsc + vite build
npm run lint / lint:fix
npm run format / format:check
npm test / npm run test:watch

# Rust (desde la raíz, apuntando al manifest)
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace
cargo build --manifest-path src-tauri/Cargo.toml --workspace --all-targets

# App completa (requiere entorno gráfico)
npm run tauri dev
npm run tauri build
```

## Spikes de riesgo (M0)

Los 3 binarios compilan en CI pero **no se ejecutan ahí** (no hay GUI/D-Bus en los runners). Deben correrse a mano en hardware real y documentar el resultado en `docs/spikes/`.

### Opción A — descargar el binario ya compilado (recomendado, sin instalar nada)

Cada corrida de CI en `develop` publica los 3 spikes como artefacto descargable, ya compilados para Windows y Linux — no hace falta tener Rust ni Node instalados para probarlos:

1. Ir a la pestaña [Actions](https://github.com/Sgiovanettil/voice-text-deskapp/actions/workflows/ci.yml) del repo, abrir la corrida más reciente del workflow **CI** sobre `develop` (con ✅ verde).
2. En la sección **Artifacts** al final de la página, descargar `spike-binaries-windows` o `spike-binaries-linux` según el equipo donde se va a probar.
3. Descomprimir el `.zip` y ejecutar directamente:
   - Windows: `spike-overlay.exe`, `spike-delivery.exe "texto de prueba"`, `spike-hotkey-portal.exe`.
   - Linux: dar permiso de ejecución (`chmod +x spike-*`) y correr `./spike-overlay`, `./spike-delivery "texto de prueba"`, `./spike-hotkey-portal`.

Los artefactos expiran a los 90 días (retención por defecto de GitHub Actions); si ya no están disponibles, basta con volver a correr el workflow CI (push vacío o re-run) para regenerarlos.

### Opción B — compilar localmente

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin spike-overlay
cargo run --manifest-path src-tauri/Cargo.toml --bin spike-delivery -- "texto de prueba"
cargo run --manifest-path src-tauri/Cargo.toml --bin spike-hotkey-portal   # solo Linux/Wayland
```

Ver protocolo detallado en cada plantilla:

- [`docs/spikes/r1-hotkey-portal.md`](spikes/r1-hotkey-portal.md)
- [`docs/spikes/r2-overlay-sin-foco.md`](spikes/r2-overlay-sin-foco.md)
- [`docs/spikes/r3-insercion-texto.md`](spikes/r3-insercion-texto.md)

Los resultados de R3 resuelven [ADR-0005](adr/0005-mecanismo-insercion-texto.md) (pasa de "Propuesta" a "Aceptada"); los de R1/R2 confirman o ajustan [ADR-0004](adr/0004-estrategia-linux-x11-wayland.md).

## Smoke test manual (checklist por release)

A partir de M1, antes de cada release verificar manualmente:

- [ ] Dictado funciona en un editor de código (VS Code).
- [ ] Dictado funciona en una terminal.
- [ ] Dictado funciona en un campo de navegador.
- [ ] Error de API key inválida se muestra con mensaje claro en el overlay.
- [ ] Sin conexión a internet: error claro, la app no crashea.
- [ ] Consumo en reposo dentro de lo esperado (RNF-02: < 1% CPU, < 150 MB RAM).
- [ ] Verificar en Windows, Linux X11 y Linux Wayland (GNOME y KDE).

## Generación de claves de firma del updater (ADR-0010) — diferido a M4

No ejecutar todavía: `release.yml` no dispara hasta el primer tag `v*.*.*`, así que generar y custodiar la clave privada ahora no aporta y sí agrega responsabilidad de custodia temprana. Cuando corresponda (M4):

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/voicetext.key
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo Sgiovanettil/voice-text-deskapp < ~/.tauri/voicetext.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo Sgiovanettil/voice-text-deskapp
```

## Flujo de trabajo (Git Flow)

Ver [PRD §11](PRD.md#11-ingeniería-y-proceso) para el detalle completo. Resumen operativo:

```bash
git checkout develop && git pull
git checkout -b feature/mi-cambio develop
# ... commits Conventional Commits ...
git push -u origin feature/mi-cambio
gh pr create --base develop --title "feat(x): ..." --body "..."
# self-review contra la checklist del PRD §11.2, CI en verde
gh pr merge --squash --delete-branch
```

`release/*` y `hotfix/*` mergean con merge commit a `main` (tag) y de vuelta a `develop` — ver PRD §11.1.
