# Releasing — VoiceText

Cómo se publica una versión y cómo está configurada la firma. Contexto de decisión:
`docs/1-fundamentos/PRD.md` §11.4 (CI/CD), RNF-09, ADR-010.

## Las dos firmas (no confundir)

| Firma | Para qué | Estado en v1.0 | Costo |
| --- | --- | --- | --- |
| **Firma de updater** (minisign de Tauri) | Que el updater oficial pueda **verificar** que un bundle/`latest.json` es auténtico antes de instalarlo. Exigida por RNF-09/ADR-010 desde la primera release. | ✅ Activa | Gratis |
| **Authenticode** (code-signing de Windows) | Quitar el aviso "editor desconocido" de SmartScreen al instalar. **No** es requisito del updater. | ⏸️ Diferida (post-v1.0) | Certificado de CA (~US$200-400/año) |

> El **updater en runtime está activo** (ADR-010, v1.x): la app consulta el `latest.json` del
> Release al arrancar y desde el botón "Buscar actualizaciones" (Configuración → Acerca de),
> avisa si hay versión nueva y —si el usuario acepta— descarga el bundle firmado y reinicia.
> El endpoint configurado en `plugins.updater.endpoints` es
> `https://github.com/Sgiovanettil/voice-text-deskapp/releases/latest/download/latest.json`, que
> solo sirve el `latest.json` del **Release publicado** más reciente. Por eso un Release en
> **borrador no es visible** para las apps instaladas: la actualización llega recién al publicarlo
> a mano (control de calidad — ver "Publicar una versión").

## Firma de updater: claves y secrets

El par de claves minisign se generó con `npx tauri signer generate`. La **clave pública** vive en
el repo (`src-tauri/tauri.conf.json` → `plugins.updater.pubkey`); la **clave privada** NO se
versiona: va como *secret* de GitHub Actions y la usa `release.yml` para firmar.

Secrets requeridos en el repo (Settings → Secrets and variables → Actions):

| Secret | Valor |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Contenido completo del archivo de clave privada (`voicetext_updater.key`). |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | La contraseña de la clave (vacía en la generación actual → secret con valor vacío). |

Cargarlos con `gh` (desde una copia local del archivo de clave privada):

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < ruta/al/voicetext_updater.key
printf '' | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

> ⚠️ Si se pierde la clave privada, no se podrán firmar nuevas actualizaciones verificables por
> las versiones ya instaladas: habría que rotar `pubkey` y re-firmar. Guárdala fuera del repo.

## Publicar una versión

`release.yml` dispara con un **tag `vX.Y.Z`** sobre `main` (Git Flow: `release/*` → merge a `main`
→ tag). El workflow:

1. Genera el changelog con git-cliff (`cliff.toml`).
2. Compila los bundles firmados en Windows y Linux (`tauri-action`), con `createUpdaterArtifacts`
   e `includeUpdaterJson` → produce los `.sig` y el `latest.json`.
3. Publica un **Release en borrador** (`releaseDraft: true`) con los bundles + `latest.json` +
   changelog. Se revisa y se publica a mano.

Pasos:

```bash
# En una rama release/X.Y.Z: subir version en package.json y src-tauri/tauri.conf.json,
# merge a main, luego:
git tag vX.Y.Z
git push origin vX.Y.Z
```

Prerrequisito: los secrets de firma cargados (arriba), o el job de release falla al firmar.
