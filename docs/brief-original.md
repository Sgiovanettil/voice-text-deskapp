# Rol

Quiero que actúes como un Arquitecto de Software Senior, Product Manager y Tech Lead.

NO quiero que escribas código.

Quiero diseñar correctamente este proyecto antes de comenzar a implementarlo.

Sé crítico.

Si detectas malas decisiones técnicas, problemas de escalabilidad o tecnologías más adecuadas, quiero que las propongas y justifiques.

Prefiero invertir tiempo en el diseño que rehacer la arquitectura más adelante.

---

# Objetivo del proyecto

Quiero crear una aplicación de escritorio multiplataforma (Windows y Linux) residente en el sistema operativo.

La aplicación será una herramienta para potenciar mi flujo de trabajo de desarrollo de software, Vibe Coding y trabajo con agentes de IA.

Su principal forma de interacción será mediante voz.

NO quiero crear un cliente para ChatGPT.

NO quiero crear un clon de Bridge Voice.

NO quiero crear un asistente tipo Jarvis.

Quiero crear una plataforma que permita interactuar con distintas IA desde cualquier parte del sistema operativo.

Si en algunos años esta plataforma evoluciona naturalmente hacia un asistente personal mucho más completo, excelente, pero ese NO es el objetivo inicial.

Quiero construir primero una excelente base técnica.

---

# Filosofía

La aplicación debe ser:

- Modular
- Escalable
- Desacoplada
- Fácil de mantener
- Fácil de extender

No debe depender de un proveedor específico.

Toda integración debe realizarse mediante abstracciones.

La arquitectura debe permitir incorporar nuevas capacidades sin modificar el núcleo de la aplicación.

---

# Usuario objetivo

Inicialmente el usuario seré yo.

La aplicación estará orientada principalmente a productividad para desarrollo de software.

Quiero acelerar mi trabajo diario utilizando voz.

Ejemplos:

- Dictar prompts
- Dictar documentación
- Dictar código
- Enviar texto a agentes
- Copiar al portapapeles
- Escribir donde esté el cursor

En el futuro podrá realizar muchas más tareas.

---

# MVP

El MVP debe resolver únicamente este flujo:

1. La aplicación está ejecutándose en segundo plano.

2. Presiono un Hotkey global.

3. Aparece un pequeño Overlay.

4. Hablo.

5. La aplicación captura el audio.

6. Envía el audio al proveedor Speech To Text configurado.

7. Obtiene el texto.

8. El texto puede:

- escribirse donde esté el cursor
- copiarse al portapapeles

Nada más.

NO quiero conversaciones.

NO quiero chat.

NO quiero asistentes.

NO quiero automatizaciones.

NO quiero agentes.

NO quiero Tool Calling.

NO quiero memoria.

Solo un excelente sistema de dictado preparado para crecer.

---

# Capacidades futuras

La arquitectura debe organizarse por capacidades.

NO por proveedores.

Las capacidades serán:

## Speech To Text (STT)

Convierte voz en texto.

Ejemplos:

- OpenAI
- Deepgram
- Azure Speech
- Google Speech
- Whisper

---

## Large Language Model (LLM)

Procesamiento de lenguaje.

Ejemplos:

- GPT
- Claude
- Gemini
- DeepSeek
- Llama
- Mistral

---

## Text To Speech (TTS)

Texto → Voz.

Ejemplos:

- ElevenLabs
- OpenAI
- Piper

---

## Vision

Comprensión de imágenes.

Permitirá analizar:

- capturas de pantalla
- documentos
- diagramas
- fotografías

---

## Embeddings

Permitirá implementar posteriormente:

- Memoria
- RAG
- Búsqueda semántica
- Búsqueda sobre documentación
- Búsqueda sobre código

---

## Realtime

Conversaciones de baja latencia.

Ejemplos:

- OpenAI Realtime
- Gemini Live

---

Estas capacidades NO forman parte del MVP.

Solo quiero que la arquitectura quede preparada para incorporarlas más adelante.

---

# Configuración

La aplicación debe tener una interfaz gráfica moderna.

Debe incluir como mínimo:

## General

- Idioma
- Tema
- Inicio automático
- Hotkey
- Minimizar al iniciar

## Speech To Text

- Proveedor
- Modelo
- API Key
- Idioma

Las demás capacidades aparecerán en versiones futuras.

---

# Overlay

Quiero un Overlay minimalista.

No debe parecer una ventana tradicional.

Debe ser moderno, elegante y liviano.

Estados mínimos:

- Escuchando
- Procesando
- Completado

Debe aparecer únicamente cuando sea necesario.

---

# Arquitectura

Quiero una arquitectura limpia.

Separar claramente:

- UI
- Overlay
- Core
- Providers
- Speech
- Audio
- Configuration
- Persistence
- Platform
- Hotkeys

Cada componente debe tener una única responsabilidad.

No mezclar lógica de negocio con interfaz.

---

# Ingeniería

Quiero aplicar buenas prácticas desde el primer commit.

No quiero ordenar el proyecto después.

## Git

Definir un flujo basado en Git Flow moderno.

Considerar:

- main
- develop
- feature/*
- release/*
- hotfix/*

Explicar cuándo utilizar cada rama.

---

## Pull Requests

Todo cambio debe realizarse mediante Pull Request.

Definir buenas prácticas para:

- nombres
- descripción
- revisión
- estrategia de merge

---

## Conventional Commits

Quiero utilizar Conventional Commits desde el inicio.

---

## Semantic Versioning

Utilizar SemVer.

---

## GitHub Actions

Diseñar una estrategia CI/CD.

Debe incluir:

- Build
- Lint
- Tests
- Formato
- Releases automáticos

---

## Releases

Cada Release debe generar automáticamente:

Windows

- Instalador recomendado por Tauri

Linux

- AppImage
- .deb

Publicar automáticamente los instaladores como Assets del Release de GitHub.

---

## Actualizaciones automáticas

La aplicación debe ser capaz de:

- detectar nuevas versiones
- notificar al usuario
- descargar actualizaciones
- instalar actualizaciones desde la propia aplicación

Quiero aprovechar el sistema de actualización recomendado por Tauri si es la mejor alternativa.

---

## Configuración segura

Analizar cómo almacenar:

- preferencias
- API Keys
- configuración

Las API Keys deben almacenarse utilizando el mecanismo seguro recomendado para cada sistema operativo.

---

## Logging

Diseñar una estrategia de logging.

---

## Manejo de errores

Diseñar una estrategia consistente.

---

## Testing

Proponer una estrategia de pruebas.

---

## Documentación

Mantener desde el inicio:

- README
- Arquitectura
- ADR (Architecture Decision Records)
- Roadmap
- Changelog
- Guía de desarrollo
- Guía de contribución

---

## Calidad

Proponer herramientas como:

- Rustfmt
- Clippy
- ESLint
- Prettier
- Commitlint
- Husky

---

# Tecnología

Actualmente considero utilizar:

Desktop

- Tauri v2

Frontend

- React
- TypeScript

Backend

- Rust

Analiza si esta elección es adecuada.

Propón alternativas únicamente si aportan ventajas importantes.

---

# Diferenciación

Antes de diseñar la arquitectura quiero que analices cómo se diferencia este proyecto de:

- Bridge Voice
- Wispr Flow
- Superwhisper
- ChatGPT Desktop
- Claude Desktop

Quiero construir un producto con identidad propia.

No un clon.

---

# Entregables

NO escribas código.

Quiero desarrollar primero toda la planificación.

Necesito que entregues:

1. Visión del producto.

2. Objetivos del MVP.

3. Alcance.

4. Funcionalidades incluidas.

5. Funcionalidades excluidas.

6. Casos de uso.

7. Requisitos funcionales.

8. Requisitos no funcionales.

9. Arquitectura general.

10. Componentes.

11. Responsabilidad de cada componente.

12. Organización de carpetas.

13. Flujo completo desde que el usuario presiona el Hotkey hasta que el texto aparece donde está el cursor.

14. Riesgos técnicos.

15. Riesgos de escalabilidad.

16. Decisiones importantes de arquitectura.

17. ADRs que deberían existir desde el inicio.

18. Roadmap del MVP.

19. Roadmap de versiones futuras.

20. Épicas.

21. Historias de usuario.

22. Backlog priorizado.

23. Dependencias entre tareas.

24. Aspectos que deberíamos decidir antes de escribir una sola línea de código.

No avances a la implementación hasta que toda la planificación esté revisada y aprobada.
