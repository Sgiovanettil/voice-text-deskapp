# ADR-0001 — Stack: Tauri v2 + React/TypeScript + Rust

- **Estado:** Aceptada · **Fecha:** 2026-07-02

## Contexto
App de escritorio residente para Windows y Linux, con overlay liviano, hotkeys globales, captura de audio y consumo bajo en reposo (RNF-02/04). Se evaluaron Electron, .NET MAUI y apps nativas separadas.

## Decisión
Tauri v2 como shell (IPC, bundling, plugins), frontend React + TypeScript para overlay y settings, backend Rust para toda la lógica.

## Justificación
- Binarios < 25 MB y RAM muy inferior a Electron (usa WebView del SO).
- Rust es idóneo para audio (cpal), hotkeys, plataforma y concurrencia segura.
- Ecosistema de plugins v2 cubre exactamente las necesidades: global-shortcut, tray, autostart, clipboard-manager, updater.
- Electron: peso/RAM excesivos para una app residente. MAUI: soporte Linux débil. Nativo doble: dos codebases.

## Consecuencias
- (+) Cumplimiento directo de RNF-02/03/04.
- (−) Dos lenguajes (Rust + TS); el contrato IPC debe mantenerse sincronizado (mitigado: catálogo de eventos como fuente única, ARCHITECTURE §4.8).
- (−) WebView del SO varía entre plataformas (WebView2 / WebKitGTK): probar overlay en ambas.
