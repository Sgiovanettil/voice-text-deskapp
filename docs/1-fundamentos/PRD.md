# PRD — VoiceText (nombre provisional)

> Aplicación de escritorio multiplataforma (Windows y Linux) de dictado por voz, residente en el sistema, activada por hotkey global, con arquitectura por capacidades y orientada a eventos, preparada para crecer hacia una plataforma de interacción con IA desde cualquier parte del sistema operativo.

- **Versión del documento:** 0.3.0
- **Fecha:** 2026-07-02
- **Estado:** En revisión
- **Documento de origen:** [`docs/1-fundamentos/brief-original.md`](brief-original.md)

---

## 1. Visión del producto

Una plataforma de escritorio que permite interactuar con distintas IA desde cualquier parte del sistema operativo, usando la voz como interfaz principal. No es un cliente de ChatGPT, ni un clon de Bridge Voice/Wispr Flow, ni un asistente tipo Jarvis: es una **base técnica excelente** organizada por capacidades (STT, LLM, TTS, Vision, Embeddings, Realtime), donde la primera capacidad entregada es un sistema de dictado de primera clase orientado a productividad en desarrollo de software y trabajo con agentes de IA.

**Diferenciación frente a competidores:**

| Producto | Qué es | Qué nos diferencia |
|---|---|---|
| Wispr Flow / Superwhisper | Dictado pulido, cerrado, mono-capacidad | Nosotros: open-stack, multi-proveedor, arquitectura por capacidades extensible |
| Bridge Voice | Voz → agentes | Nosotros: plataforma general, sin acoplarse a un flujo de agentes concreto |
| ChatGPT/Claude Desktop | Cliente de un proveedor | Nosotros: agnósticos de proveedor por diseño (abstracción `SpeechProvider` y equivalentes futuros) |

En el MVP la funcionalidad se parece a Wispr Flow/Superwhisper; la identidad propia está en la **arquitectura** (capacidades + proveedores intercambiables + eventos + Linux como ciudadano de primera clase) y se materializa en versiones posteriores.

---

## 2. Principios del proyecto

Estos principios gobiernan toda decisión de producto y de arquitectura. Ante un conflicto, se resuelve a favor del principio de número menor.

1. **Privacidad primero.** La privacidad del usuario es un principio fundamental, no una característica. El usuario mantiene siempre el control sobre sus datos.
2. **Seguridad desde el diseño.** La seguridad se considera en cada decisión desde el inicio, nunca como mejora posterior (ver §15).
3. **No interrumpir jamás.** La aplicación nunca debe interrumpir el flujo de trabajo del usuario: no roba el foco, no muestra diálogos inesperados, no bloquea el sistema.
4. **Sensación de inmediatez.** La experiencia debe sentirse instantánea: feedback visual inmediato ante cada acción y latencias mínimas en todo el ciclo.
5. **Ningún proveedor es indispensable.** Toda integración externa se realiza mediante abstracciones; sustituir un proveedor nunca debe requerir cambios en el núcleo.
6. **Capacidades independientes.** Cada capacidad (STT, LLM, TTS, …) evoluciona de forma independiente, con su propio ciclo de madurez.
7. **Núcleo cerrado a modificación, abierto a extensión.** Cada nueva capacidad debe poder incorporarse sin modificar el núcleo del sistema (comunicación por eventos y contratos, ver §8.2).
8. **Mantenibilidad sobre velocidad.** La arquitectura privilegia la mantenibilidad y claridad sobre la rapidez de implementación.
9. **Degradación elegante.** *(adicional)* Cuando una condición del entorno no se cumple (Wayland, sin keyring, sin red), la aplicación degrada a un modo reducido con mensaje claro; nunca falla en silencio ni crashea. Justificación: es la contraparte operacional del principio 3 en un producto multiplataforma con entornos heterogéneos.
10. **Observabilidad sin exposición.** *(adicional)* Todo comportamiento relevante es diagnosticable mediante logs locales, sin registrar jamás datos sensibles ni enviar información fuera del equipo. Justificación: habilita soporte y debugging sin contradecir los principios 1 y 2.

---

## 3. Objetivos del MVP

1. Dictado por voz funcional y confiable: hotkey → hablar → texto en la aplicación activa o en el portapapeles.
2. Arquitectura por capacidades operativa: el core no conoce proveedores concretos (interfaz `SpeechProvider`).
3. Comunicación interna basada en eventos de dominio, preparando la extensión futura sin acoplamiento.
4. Base de ingeniería completa desde el primer commit: Git Flow, CI, releases, calidad, documentación.
5. Latencia y UX suficientemente buenas como para usarlo a diario en el trabajo real del autor.

**Criterios de aceptación del MVP (definition of done):**

- En Windows 10/11, Linux X11 y Linux Wayland (GNOME/KDE): presionar el hotkey, dictar 30 segundos, y el texto aparece en la aplicación activa con < 3 s de espera tras soltar la tecla (para dictados cortos, < 2 s). En entornos Wayland sin portals, la degradación es explícita según ADR-004.
- La app corre en segundo plano con ícono de bandeja, consumo en reposo < 1% CPU y < 150 MB RAM.
- La API key se guarda en el almacén seguro del SO y nunca en texto plano.
- Cambiar el proveedor STT (cuando exista un segundo) no requiere tocar el core.
- Todas las cadenas de la UI provienen del sistema i18n (español como idioma por defecto).

---

## 4. Alcance

### 4.1 Incluido en el MVP

- Aplicación residente con ícono en bandeja del sistema (mostrar settings, salir).
- **Hotkey global configurable** en modo **Push-to-Talk** (mantener presionado mientras se habla). **Decisión explícita del MVP** (ADR-008). El flujo oficial es:

  ```
  Hotkey → Overlay → Grabación → Transcripción → Entrega del texto → Cerrar Overlay
  ```

- **Overlay** minimalista sin foco, con estados: Escuchando / Procesando / Completado / Error.
- Captura de audio del micrófono por defecto (16 kHz mono PCM).
- Transcripción vía proveedor **OpenAI** (batch), detrás de la interfaz `SpeechProvider` (decisión confirmada, ADR-003).
- Salida del texto, configurable: (a) **inserción en la aplicación activa** — el mecanismo técnico concreto (APIs nativas, clipboard + pegado, inyección de teclado u otras alternativas multiplataforma) **se evaluará y decidirá en la fase de Arquitectura** (ADR-005, pendiente); (b) **solo copiar al portapapeles** (modo garantizado en toda plataforma).
- Ventana de configuración: General (idioma de la UI, hotkey, inicio automático, minimizar al iniciar, modo de salida) y Speech to Text (proveedor, modelo, API key, idioma de dictado).
- **Internacionalización (i18n) desde el inicio**: todas las cadenas de UI externalizadas y catálogo de claves de error; MVP se entrega con español por defecto (inglés cuando exista traducción).
- Persistencia de preferencias (archivo) y de API keys (keyring del SO).
- Logging estructurado con rotación; manejo de errores consistente con feedback visible en el overlay.
- Empaquetado: instalador Windows (NSIS/MSI según recomendación Tauri), AppImage y .deb en Linux, publicados como assets de Release en GitHub vía CI.
- **Preparación para auto-update**: el MVP no incluye el updater funcional, pero la arquitectura, el empaquetado y el firmado se diseñan compatibles con el mecanismo oficial de actualización de Tauri, de modo que activarlo después no requiera cambios estructurales (ADR-010).

### 4.2 Excluido del MVP (explícito)

- Conversaciones, chat, asistentes, automatizaciones, agentes, tool calling, memoria.
- Capacidades LLM, TTS, Vision, Embeddings, Realtime (solo queda preparada la estructura).
- STT en streaming y STT local (Whisper local).
- **Auto-updater funcional** (la app no descarga ni instala actualizaciones en MVP; sí queda arquitectónicamente preparada, ver §4.1 y ADR-010).
- Event Bus como componente formal (la comunicación por eventos existe desde el MVP; el bus desacoplado con suscriptores dinámicos llega post-MVP, ver §8.2).
- Selector de tema claro/oscuro (se usa el tema del sistema).
- macOS.
- VAD / detección automática de silencio; modo toggle.

---

## 5. Casos de uso

- **CU-01 Dictar prompt a un agente:** el usuario tiene el cursor en un terminal con Claude Code; presiona el hotkey, dicta el prompt, suelta; el texto aparece en el terminal.
- **CU-02 Dictar documentación/código:** ídem sobre un editor (VS Code, etc.).
- **CU-03 Copiar al portapapeles:** el usuario configuró modo "clipboard"; dicta y luego pega manualmente donde quiera.
- **CU-04 Configurar la app:** abre settings desde la bandeja, cambia hotkey, API key, idioma de la UI, idioma de dictado, modo de salida.
- **CU-05 Error de proveedor:** la API falla (key inválida, sin red); el overlay muestra el error de forma clara y el audio no se pierde silenciosamente (opción de reintentar en versiones futuras; en MVP, mensaje claro).

---

## 6. Requisitos funcionales

| ID | Requisito |
|---|---|
| RF-01 | La app inicia con el sistema (opcional) y reside en la bandeja. |
| RF-02 | Hotkey global configurable; Push-to-Talk: grabación mientras está presionado. |
| RF-03 | Overlay visible solo durante el ciclo de dictado, sin robar el foco de la ventana activa. |
| RF-04 | Captura de audio del micrófono por defecto y conversión a 16 kHz mono. |
| RF-05 | Envío del audio al `SpeechProvider` configurado y obtención del texto. |
| RF-06 | Salida configurable: insertar en la aplicación activa (mecanismo según ADR-005) o solo copiar al portapapeles. |
| RF-07 | Settings persistentes; API keys en el almacén seguro del SO (Credential Manager / Secret Service). |
| RF-08 | Estados y errores comunicados en el overlay (Escuchando, Procesando, Completado, Error). |
| RF-09 | Logs locales con rotación y nivel configurable. |
| RF-10 | Toda cadena visible al usuario proviene del sistema i18n; idioma de UI configurable (default: español). |

## 7. Requisitos no funcionales

| ID | Requisito | Métrica |
|---|---|---|
| RNF-01 | Latencia | Dictado ≤10 s: texto en < 2 s tras soltar la tecla (excluye latencia de red anómala). |
| RNF-02 | Consumo en reposo | < 1% CPU, < 150 MB RAM. |
| RNF-03 | Arranque | App lista (hotkey activo) en < 3 s. |
| RNF-04 | Tamaño instalador | < 25 MB (ventaja de Tauri sobre Electron). |
| RNF-05 | Seguridad | Según principios de §15. |
| RNF-06 | Privacidad | El audio solo se envía al proveedor configurado y solo por acción explícita del usuario; documentar retención del proveedor. Indicador visual siempre que se graba. |
| RNF-07 | Confiabilidad | Un fallo del proveedor nunca crashea la app; degradación con mensaje (principio 9). |
| RNF-08 | Extensibilidad | Agregar un proveedor STT = implementar el trait + registrarlo; agregar una capacidad = nuevos módulos suscritos a eventos; cero cambios en core (salvo catálogo). |
| RNF-09 | Actualizabilidad | Bundles y firmas compatibles con el updater oficial de Tauri desde la primera release (aunque el updater esté inactivo). |

---

## 8. Arquitectura general

**Stack (validado):** Tauri v2 (shell + IPC + bundling), frontend React + TypeScript (settings y overlay como WebViews), backend Rust (toda la lógica de negocio). La elección es adecuada: binarios livianos, Rust ideal para audio/hotkeys/plataforma, ecosistema de plugins Tauri v2 (global-shortcut, autostart, tray, clipboard, updater). Alternativas descartadas: Electron (peso, RAM), .NET MAUI (Linux débil), apps nativas separadas (doble mantenimiento).

**Principio rector:** el frontend es "tonto" (presentación y formularios); el core en Rust orquesta todo. La comunicación es vía comandos/eventos de Tauri.

### 8.1 Organización por capacidades

Organización por capacidades, no por proveedores. El core define traits por capacidad; los proveedores los implementan en módulos separados:

```rust
// contrato conceptual — no es código de implementación
trait SpeechProvider {
    fn id(&self) -> &str;
    async fn transcribe(&self, audio: AudioData, opts: TranscribeOptions) -> Result<Transcript, SpeechError>;
    fn capabilities(&self) -> ProviderCapabilities; // batch, streaming (futuro), idiomas, formatos
}
```

El core solo conoce `SpeechProvider`; un `ProviderRegistry` resuelve la implementación según configuración. Los futuros `LlmProvider`, `TtsProvider`, etc. seguirán el mismo patrón. Aunque el MVP es batch, el trait declara `capabilities()` para incorporar streaming después sin romper el contrato.

### 8.2 Arquitectura orientada a eventos

La comunicación entre componentes se basa en **eventos de dominio tipados** desde el MVP. El ciclo de dictado no es una secuencia de llamadas directas entre módulos, sino una máquina de estados en el core que reacciona a eventos y los emite:

```
HotkeyPressed → OverlayOpened → RecordingStarted → RecordingStopped
→ TranscriptionStarted → TranscriptionCompleted
→ TextDeliveryStarted → TextDeliveryCompleted → OverlayClosed
```

(más sus variantes de error: `TranscriptionFailed`, `TextDeliveryFailed`, …)

Reglas:

- Los eventos son el **contrato público** entre componentes: un módulo emite eventos y reacciona a eventos; no invoca internamente a otros módulos de negocio.
- En el MVP **no se implementa un Event Bus** como componente formal: basta un despachador simple dentro del core (los eventos ya cruzan el IPC de Tauri hacia overlay/UI). El diseño de los eventos, sin embargo, se hace como si el bus existiera: tipados, serializables, con nombre y payload estables.
- Este enfoque es lo que permitirá incorporar después **LLM, TTS, Vision, Embeddings, plugins, automatizaciones, agentes y telemetría opt-in** como suscriptores de eventos existentes (p. ej., un post-procesador LLM que escuche `TranscriptionCompleted`), sin modificar el núcleo (principio 7).

Documentado como ADR-009.

### 8.3 Componentes y responsabilidades

| Componente | Responsabilidad única |
|---|---|
| **Core (orquestador)** | Máquina de estados del ciclo de dictado (Idle → Recording → Transcribing → Delivering → Idle/Error) dirigida por eventos de dominio (§8.2). No conoce proveedores ni SO. |
| **Hotkeys** | Registro del atajo global (plugin Tauri global-shortcut); emite `HotkeyPressed`/`HotkeyReleased`. |
| **Audio** | Captura del micrófono (cpal), resampling a 16 kHz mono, buffer en memoria; emite `RecordingStarted`/`RecordingStopped`. |
| **Speech** | Definición de `SpeechProvider`, tipos (`AudioData`, `Transcript`, `SpeechError`) y `ProviderRegistry`. |
| **Providers** | Implementaciones concretas (MVP: `openai`). Un módulo por proveedor, aislado. |
| **Delivery/Platform** | Inserción de texto en la app activa (mecanismo según ADR-005) y modo clipboard; abstracción con backends por plataforma: Windows, Linux X11 y Linux Wayland como targets de primera clase (ADR-004). |
| **Configuration** | Modelo de settings, validación, defaults, migraciones de esquema. |
| **Persistence** | Settings en archivo (JSON/TOML en dir de config del SO); secretos en keyring del SO. |
| **i18n** | Catálogo de cadenas de UI y claves de mensajes de error; resolución de idioma (default español). |
| **Overlay (UI)** | WebView sin foco, siempre-encima, que refleja el estado del core suscribiéndose a los eventos de dominio. |
| **Settings UI** | WebView React para configuración; solo lee/escribe vía comandos Tauri. |
| **Tray** | Ícono de bandeja, menú (Settings, Quit). |
| **Updater (futuro)** | Integración con el updater oficial de Tauri; en MVP solo se garantiza compatibilidad de bundles/firma (ADR-010). |
| **Logging/Errores** | tracing + rotación de archivos; taxonomía de errores (`AudioError`, `SpeechError`, `DeliveryError`, `ConfigError`) mapeada a claves i18n de mensajes de usuario. |

### 8.4 Organización de carpetas (propuesta)

```
voice-text-deskapp/
├── docs/                    # PRD, ARCHITECTURE.md, adr/, ROADMAP.md, DEVELOPMENT.md
├── src/                     # Frontend React+TS
│   ├── overlay/             # UI del overlay
│   ├── settings/            # UI de configuración
│   ├── i18n/                # catálogos de cadenas (es, en, …)
│   └── shared/              # tipos IPC (eventos de dominio), hooks, estilos
├── src-tauri/
│   ├── src/
│   │   ├── core/            # máquina de estados, eventos de dominio, orquestación
│   │   ├── audio/
│   │   ├── speech/          # trait + registry + tipos
│   │   ├── providers/openai/
│   │   ├── delivery/        # inserción de texto, por plataforma
│   │   ├── config/
│   │   ├── persistence/
│   │   ├── hotkeys/
│   │   └── ipc/             # comandos y puente de eventos Tauri
│   └── tauri.conf.json
└── .github/workflows/       # ci.yml, release.yml
```

(Si `providers/` crece, se migra a workspace de crates: `core`, `providers-*`; en MVP un solo crate con módulos es suficiente — supuesto en §17.)

### 8.5 Flujo completo: hotkey → texto en la aplicación activa

Flujo oficial del MVP (Push-to-Talk), expresado en eventos:

1. Usuario presiona y mantiene el hotkey → `HotkeyPressed`.
2. Core pasa a `Recording`; se emiten `OverlayOpened` (overlay en "Escuchando", sin robar el foco: la ventana destino lo conserva) y `RecordingStarted`.
3. Usuario habla; Audio acumula PCM y hace resampling.
4. Usuario suelta el hotkey → `HotkeyReleased` → `RecordingStopped`. Core pasa a `Transcribing` y emite `TranscriptionStarted`; overlay muestra "Procesando".
5. El `SpeechProvider` resuelto por el registry (OpenAI) transcribe: encode → HTTPS → texto → `TranscriptionCompleted`.
6. Core pasa a `Delivering` y emite `TextDeliveryStarted`: según configuración, Delivery inserta el texto en la aplicación activa (mecanismo según ADR-005) o lo copia al portapapeles → `TextDeliveryCompleted`.
7. Overlay muestra "Completado" ~1 s → `OverlayClosed`. Core vuelve a `Idle`.
8. Ante error en cualquier paso: evento de fallo correspondiente, estado `Error` en el overlay con mensaje accionable (clave i18n), log detallado, vuelta a `Idle`.

---

## 9. Riesgos

### 9.1 Riesgos técnicos

| # | Riesgo | Impacto | Mitigación |
|---|---|---|---|
| R1 | **Wayland**: hotkeys globales e inserción de texto restringidos por diseño, con soporte desigual entre compositores | Alto (target de primera clase) | Backends por función (ADR-004): portal GlobalShortcuts para hotkeys, portal RemoteDesktop/ydotool para síntesis de input; degradación funcional **explícita** cuando falten portals (activación externa + modo clipboard). Spike obligatorio en GNOME/KDE antes de M1. |
| R2 | Overlay roba el foco → falla la inserción de texto | Alto | Ventana con flags no-focusable/no-activate; **spike de validación temprana en ambas plataformas antes de construir encima**. |
| R3 | El mecanismo de inserción elegido (ADR-005) no funciona uniformemente en todas las apps (p. ej., terminales con atajos de pegado distintos, apps que bloquean input sintético) | Alto | La evaluación de alternativas en fase de Arquitectura incluye una matriz de compatibilidad por tipo de app (editor, terminal, navegador) y por SO; el modo clipboard es siempre el fallback garantizado. |
| R4 | Keyring ausente en Linux (sin Secret Service) | Medio | Detectar y ofrecer fallback (archivo con advertencia explícita) o exigir keyring. |
| R5 | Captura de audio: dispositivos, formatos, permisos | Medio | cpal con device por defecto; test manual multi-hardware; selección de dispositivo post-MVP. |
| R6 | Latencia de proveedor variable | Medio | Timeout configurado, estado "Procesando" honesto, medir y registrar latencias en logs. |
| R7 | Sobrecosto del proceso Git Flow + PRs con un solo desarrollador | Bajo | Variante liviana (§11): ramas `release/*` cortas, squash merge, self-review con checklist; se revisará si frena la velocidad. |
| R8 | i18n a medias: cadenas hardcodeadas que se cuelan y encarecen la traducción futura | Bajo | Lint/convención desde el primer commit: toda cadena visible pasa por el sistema i18n (RF-10). |

### 9.2 Riesgos de escalabilidad (arquitectura)

- Trait `SpeechProvider` batch-only bloquearía streaming/Realtime → se mitiga con `capabilities()` y tipos de entrada/salida pensados para extenderse (ADR-003).
- Eventos de dominio mal diseñados (payloads inestables, nombres ambiguos) serían un contrato roto para futuros suscriptores (plugins, telemetría) → los eventos se versionan conceptualmente y se documentan como API interna (ADR-009).
- Lógica filtrada al frontend haría difícil agregar superficies (CLI, API local) → regla estricta: negocio solo en Rust.
- Un solo crate podría volverse monolito → límites de módulos claros hoy, workspace de crates cuando entre el segundo proveedor.
- Config sin versionado de esquema rompería upgrades → campo `schema_version` + migraciones desde el día 1.
- Releases sin firma/manifiesto de updater obligarían a re-empaquetar todo al activar auto-update → firmado y estructura de artefactos compatibles con el updater de Tauri desde la primera release (ADR-010, RNF-09).

---

## 10. Decisiones de arquitectura y ADRs

Criterio: es **ADR** lo que fija una dirección arquitectónica costosa de revertir o que condiciona el diseño futuro; es **decisión de PRD** lo que define alcance o proceso y puede cambiarse sin impacto estructural.

### ADRs

| ADR | Decisión | Estado |
|---|---|---|
| ADR-001 | Tauri v2 + React/TS + Rust como stack (vs Electron/nativo). | Aceptada |
| ADR-002 | Toda la lógica de negocio en Rust; frontend solo presentación. | Aceptada |
| ADR-003 | Abstracción `SpeechProvider` + `ProviderRegistry`; organización por capacidades. | Aceptada (confirmada por el autor) |
| ADR-004 | Estrategia Linux: X11 **y Wayland como targets de primera clase**, con backends por función (portals) y degradación funcional explícita donde el protocolo lo imponga. | Aceptada (revisada) |
| ADR-005 | Mecanismo de inserción de texto en la aplicación activa (APIs nativas vs clipboard+pegado vs inyección de teclado vs otras alternativas multiplataforma). | **Pendiente — se evalúa y decide en la fase de Arquitectura**, con spike y matriz de compatibilidad (R3) |
| ADR-006 | Secretos en keyring del SO (keyring-rs); settings en archivo de config estándar del SO con `schema_version`. | Aceptada |
| ADR-008 | Push-to-Talk como modo de activación del MVP (toggle/VAD futuros). | Aceptada (decisión explícita del MVP) |
| ADR-009 | Arquitectura orientada a eventos de dominio; Event Bus formal diferido a post-MVP pero eventos diseñados como contrato estable desde el MVP. | Aceptada |
| ADR-010 | Auto-update mediante el mecanismo oficial de Tauri como capacidad futura: bundles, firma y estructura de release compatibles desde la primera versión; activación post-MVP sin cambios de arquitectura. | Aceptada |

### Decisiones de PRD (no ameritan ADR)

- Flujo Git Flow y prácticas de PR (§11): proceso de desarrollo, reversible sin impacto en el código. *(El antiguo ADR-007 trunk-based queda anulado por esta decisión.)*
- i18n desde el inicio con español por defecto: decisión de producto; la elección de librería/formato de catálogos se hará en fase de Arquitectura (menor, no requiere ADR salvo que condicione el IPC).
- Alcance del MVP, principios del proyecto (§2) y principios de seguridad (§15): decisiones de producto documentadas en este PRD.

---

## 11. Ingeniería y proceso

### 11.1 Flujo Git — Git Flow (variante moderna)

Se adopta **Git Flow** para tener un flujo profesional desde el primer día y facilitar la incorporación futura de colaboradores, en una variante liviana (ramas de release cortas, squash merge) que mantiene el proceso ágil para un equipo pequeño.

| Rama | Propósito |
|---|---|
| `main` | Solo código liberado. Cada merge a `main` corresponde a una versión etiquetada (`vX.Y.Z`). Siempre estable e instalable. |
| `develop` | Rama de integración. Reúne las features terminadas para la próxima versión. Debe mantenerse siempre en verde (CI). |
| `feature/*` | Una rama por funcionalidad o fix no urgente, creada desde `develop` y devuelta a `develop` vía PR. Vida corta (días, no semanas). |
| `release/*` | Estabilización de una versión: se crea desde `develop` al congelar alcance; solo admite fixes, docs y ajustes de versión. Al cerrar, se mergea a `main` (tag) y de vuelta a `develop`. |
| `hotfix/*` | Corrección urgente sobre producción: se crea desde `main`, y al cerrar se mergea a `main` (tag de patch) y a `develop`. |

### 11.2 Pull Requests

Todo cambio llega a `develop`/`main` mediante Pull Request, sin excepciones (incluido el trabajo del autor, con self-review asistido por herramientas).

- **Nombres de ramas:** `tipo/descripcion-corta-kebab` usando los tipos de Conventional Commits: `feature/overlay-estados`, `fix/hotkey-linux`, `release/1.2.0`, `hotfix/crash-keyring`.
- **Nombre del PR:** formato Conventional Commit — `feat(overlay): estados escuchando/procesando/completado`. Es el mensaje del squash, por lo que alimenta el changelog y el versionado.
- **Descripción:** plantilla con tres secciones — *Qué* cambia, *Por qué* (issue/motivación), *Cómo probarlo* (pasos de verificación manual y tests). Capturas cuando hay UI.
- **Revisión:** checklist mínima — CI en verde, sin cadenas fuera de i18n, sin secretos ni logs sensibles, ADR nuevo si la decisión lo amerita, docs actualizadas. Con colaboradores: al menos 1 aprobación; mientras tanto, self-review contra la misma checklist.
- **Estrategia de merge:** **squash merge** a `develop` (historial lineal, un commit por PR); **merge commit** de `release/*` y `hotfix/*` a `main` (preserva trazabilidad de la versión).

### 11.3 Resto del proceso

- **Conventional Commits + SemVer:** commitlint + husky; versión derivada de commits (release-please o equivalente adaptado a Git Flow).
- **CI (GitHub Actions):** en cada PR: format check (rustfmt/prettier), lint (clippy/eslint), tests, build en matriz windows-latest + ubuntu-LTS. En tag sobre `main`: build de bundles (NSIS/MSI, AppImage, .deb) **firmados y con estructura compatible con el updater de Tauri (ADR-010)** y publicación como assets del Release con changelog generado.
- **Calidad:** rustfmt, clippy (deny warnings), ESLint, Prettier, commitlint, husky.
- **Testing:** unit tests en Rust (core/máquina de estados y eventos, config, providers con HTTP mockeado — wiremock), tests de frontend mínimos (vitest) para settings, smoke test manual documentado por release (checklist); e2e automatizado post-MVP.
- **Logging:** crate `tracing` + `tracing-appender` (rotación diaria, retención 7 días), nivel configurable, sin PII ni keys en logs (§15).
- **Errores:** `thiserror` para taxonomía por módulo; en el borde IPC se mapean a códigos + claves i18n de mensajes de usuario; el overlay muestra el mensaje, el log guarda el detalle.
- **Documentación viva:** README, `docs/2-arquitectura/ARCHITECTURE.md`, `docs/2-arquitectura/DECISIONS/`, `docs/1-fundamentos/ROADMAP.md`, CHANGELOG (generado), `docs/3-desarrollo/SETUP_DEV.md`. Guía de contribución cuando se abra a colaboradores.

---

## 12. Roadmap

### MVP (v0.x → v1.0)

| Hito | Contenido |
|---|---|
| M0 — Fundaciones | Repo Git (Git Flow), scaffold Tauri, CI, calidad, i18n base, docs, ADRs 001–010 (ADR-005 se resuelve en la fase de Arquitectura, previa a M0 de implementación). **Incluye spikes de riesgo en Windows, X11 y Wayland GNOME/KDE: overlay sin foco (R2), mecanismo de inserción de texto (R3) y hotkey vía portal (R1).** |
| M1 — Ciclo de dictado core | Hotkey PTT + eventos de dominio + captura de audio + provider OpenAI + texto al clipboard. Primera versión usable. |
| M2 — Inserción y overlay | Inserción de texto en la app activa (según ADR-005 ya decidido); overlay con estados. |
| M3 — Settings y residencia | Tray, settings UI completa (con i18n), keyring, autostart, persistencia. |
| M4 — Endurecimiento y release | Manejo de errores pulido, logging, empaquetado firmado compatible con updater y release automatizado → **v1.0**. |

### Post-MVP (orden tentativo)

1. **v1.x:** activación del auto-updater (Windows + AppImage), toggle + VAD (ADR-0011), selección de micrófono (ARCHITECTURE §4.3), segundo proveedor STT: Groq (ADR-0012, valida ADR-003), diccionario personal/reemplazos (ADR-0013), inglés en la UI, ampliación de la matriz Wayland (compositores wlroots).
2. **v2.x:** streaming STT, Event Bus formal con suscriptores dinámicos, capacidad **LLM** (post-procesado del dictado: limpieza, formato, comandos de voz "en modo prompt" — spec anticipada en ADR-0014).
3. **v3.x+:** TTS, Vision (capturas), Embeddings/RAG, Realtime, plugins; macOS.

---

## 13. Épicas e historias de usuario (MVP)

- **E1 Fundaciones** — HU: como dev, tengo repo con Git Flow y CI que rechaza código mal formateado o sin tests verdes; tengo ADRs que documentan las decisiones clave.
- **E2 Captura por voz** — HU: como usuario, mantengo presionado el hotkey y la app graba mi voz mostrando que escucha; al soltar, deja de grabar.
- **E3 Transcripción** — HU: como usuario, mi voz se transcribe con el proveedor configurado y veo "Procesando" mientras tanto; si falla, veo un error claro en mi idioma.
- **E4 Entrega de texto** — HU: como usuario, el texto aparece en la aplicación donde estoy trabajando sin perder el contenido de mi portapapeles; alternativamente, elijo que solo se copie.
- **E5 Configuración y residencia** — HU: como usuario, configuro hotkey, API key (guardada de forma segura), idioma de la UI, idioma de dictado y modo de salida; la app inicia con el sistema minimizada en la bandeja.
- **E6 Release** — HU: como usuario, descargo un instalador desde GitHub Releases para mi SO y funciona sin pasos manuales; el instalador queda preparado para actualizarse automáticamente en el futuro.

## 14. Backlog priorizado y dependencias

Orden de ejecución (cada ítem depende del anterior salvo indicación):

1. **Fase de Arquitectura** (previa al código): resolver ADR-005 con spike y matriz de compatibilidad; diseñar eventos de dominio y contratos; elegir librería i18n. — bloquea todo lo demás.
2. E1 Fundaciones (M0) — bloquea el resto de la implementación.
3. **Spike R2** (overlay sin foco en Win/X11) — bloquea E4 y valida ADR-004.
4. E2 Captura (hotkey + audio + eventos) — depende de E1.
5. E3 Transcripción (trait + provider OpenAI) — depende de E2 para probar end-to-end; el trait puede desarrollarse en paralelo.
6. E4 Entrega (clipboard/inserción según ADR-005) — depende de E3 y de los spikes.
7. E5 Settings/tray/keyring/i18n — parcialmente paralelo a E3–E4 (necesita config mínima antes: API key).
8. E6 Release — depende de todo; el pipeline se construye en E1 y se completa aquí.

---

## 15. Principios de seguridad y privacidad

Principios establecidos desde el diseño (la arquitectura de seguridad detallada se desarrollará en la fase de Arquitectura):

1. **Nunca registrar API keys ni secretos en logs**, trazas, mensajes de error ni telemetría.
2. **Nunca almacenar secretos en texto plano**; se usa el mecanismo seguro recomendado por cada SO (Windows Credential Manager, Secret Service/keyring en Linux).
3. **Toda comunicación con proveedores se realiza mediante HTTPS** (TLS); sin excepciones ni downgrade.
4. **Ningún dato sale del equipo sin una acción explícita del usuario**: el audio se envía al proveedor STT solo como consecuencia directa de un dictado iniciado por el usuario; no existe telemetría en MVP, y si se incorpora será opt-in.
5. **El usuario mantiene siempre el control sobre sus datos**: puede ver y borrar su configuración, sus logs y sus keys; el audio vive solo en memoria y se descarta tras transcribir (salvo flag de debug explícito y documentado).
6. **Indicador de grabación siempre visible**: nunca se captura audio sin overlay en pantalla.
7. **Superficie mínima**: la app no abre puertos, no ejecuta contenido remoto y el WebView solo carga recursos locales (CSP estricta de Tauri).
8. **Documentar la política de retención** del proveedor STT configurado (OpenAI en MVP) en el README.

---

## 16. Decisiones cerradas antes de la primera línea de código

| Decisión | Estado |
|---|---|
| Proveedor STT de referencia: OpenAI, tras interfaz `SpeechProvider` | ✅ Confirmada (ADR-003) |
| Stack Tauri v2 + React/TS + Rust | ✅ Validada (ADR-001) |
| Lógica de negocio 100% en Rust | ✅ (ADR-002) |
| **Push-to-Talk como modo de activación del MVP** | ✅ Decisión explícita (ADR-008) |
| **Git Flow (variante moderna) + PRs obligatorios** | ✅ Confirmada por el autor (§11) |
| **Arquitectura orientada a eventos; Event Bus formal post-MVP** | ✅ (ADR-009) |
| **Auto-update: preparado desde el inicio, activo post-MVP** | ✅ (ADR-010) |
| **i18n desde el inicio; español por defecto** | ✅ Confirmada por el autor |
| Mecanismo de inserción de texto | ⏳ **Diferida a la fase de Arquitectura** (ADR-005) |
| Nombre del producto | ⏳ Pendiente (no bloqueante; placeholder "VoiceText") |

## 17. Supuestos por ratificar

Adoptados por defecto; cambiar cualquiera implica revisar §4, §8 y roadmap:

1. ~~X11 primero~~ **Resuelto (decisión del autor):** X11 y Wayland son targets de primera clase con degradación funcional explícita (ADR-004 revisado).
2. **Un solo crate** Rust con módulos (workspace de crates recién al agregar el segundo proveedor).
3. ~~Modelo OpenAI concreto~~ **Resuelto (ratificado por el autor):** `gpt-4o-mini-transcribe` por defecto; timeout 30 s; 1 reintento automático solo ante error de red/rate-limit (ARCHITECTURE §4.5).
4. ~~Comportamiento ante fallo~~ **Resuelto (ratificado por el autor):** en MVP el dictado se pierde tras el error con mensaje claro; retención en memoria para reintento manual es candidato a v1.x.
