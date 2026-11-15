# Beta 4 — aceptación del instalador

Esta hoja corresponde al artefacto de Actions para `v0.1.0-beta.4`. Un preview,
un build local, la CI fuente o el smoke automatizado no sustituyen la prueba del
`.exe` exacto en un Windows limpio ni la aprobación de publicación.

La reparación local del 2026-10-12 completó los 13 gates automatizados del MVP.
Su registro está en [BETA4_REPAIR_EVIDENCE.md](./BETA4_REPAIR_EVIDENCE.md).
El build local y su prueba de arranque se registran por separado del artifact
de Actions descrito en esta hoja; no cambian los resultados manuales pendientes.

El gate posterior [P0-01](./BETA4_P0_01_EVIDENCE.md) conserva la recuperación
SQLite y las referencias de almacenamiento, y requiere que el candidato exacto
se vuelva a validar. Sus pruebas locales no sustituyen la prueba del artifact
de Actions, la actualización en Windows limpio ni la aceptación humana de esta
hoja.

La reparación posterior de [Perfiles](./BETA4_PROFILE_REPAIR_EVIDENCE.md)
requiere reconstruir el candidato. El hash del build previo a esa reparación
no acredita los cambios de Perfiles ni debe utilizarse para su aceptación.

## Identificación del artefacto

Registrar en la evidencia privada de aceptación:

| Dato | Valor |
| --- | --- |
| Archivo `.exe` y tamaño | `Pulsaria_0.1.0-beta.4_x64-setup.exe`; artifact de Actions `pulsaria-p1-04-acceptance-43736026192-1`, `586,930,409` bytes según `size_in_bytes` (ID `14172355088`; el tamaño individual del `.exe` no aparece por separado en los logs) |
| SHA-256 del `.exe` | `812254512BB9B51926B58CAE32B3837819DFB0C3FFC1353AC8D2C2C88F1CC2D6` |
| Tag / source commit del artefacto | Sin tag / `52b598659b0655e54c30b40196d9194a3052e4be` |
| CI canonical / runtime y artifact | [canonical 43734555115 PASS](https://github.com/danielunibe/Pulsaria/actions/runs/43734555115); [runtime PR 43734555116 PASS](https://github.com/danielunibe/Pulsaria/actions/runs/43734555116); dispatch exacto [43736026192 PASS](https://github.com/danielunibe/Pulsaria/actions/runs/43736026192), incluido smoke instalado y retención |
| Digest del ZIP / expiración | `sha256:81bc335aed981782b197f742155bf8dce8b954a1f2c027ea87431c1716f02f23` / `2026-12-15T17:10:17Z` |
| Windows x64 / WebView2 | Smoke NSIS instalado en runner aislado PASS con APPDATA/WebView2 aislados, API health, reinicio y cleanup; aceptación final en Windows limpio pendiente |
| Fecha / operador | 2026-11-15 / runner de GitHub Actions; aceptación por operador humano pendiente |

El artifact retenido del run [43736026192](https://github.com/danielunibe/Pulsaria/actions/runs/43736026192)
corresponde al commit exacto registrado arriba. La aceptación humana queda
pendiente y debe aplicarse únicamente a ese hash. Cualquier reconstrucción o
cambio de archivo invalida la aceptación. El artifact anterior de
`5b0b12b8adc05745f1b606efeb1f88eade6ba97a` quedó superseded; no debe usarse
para aceptar el candidato actual.

El candidato previo de SHA `2fad9a2fa28211f8f0b326b816f761fcb782edc3`, validado
en el run 43710866209, quedó superseded por el candidato actual. El más antiguo
`3295202cff999ae666818d94ac49280a682d8d66`, validado en el run 43701997112,
también es histórico y no debe usarse para aceptar el candidato actual.

## Entorno de prueba y privacidad

- Usar una VM, Windows Sandbox o equipo de prueba x64 sin Node.js, Rust, Python,
  FFmpeg ni FFprobe globales. Registrar la versión de WebView2 y cualquier
  intervención necesaria.
- Para el caso de actualización, instalar primero el Eval.3 público y crear una
  biblioteca sintética. Nunca instalar o migrar sobre la biblioteca personal.
- Usar un perfil de Windows y `PULSAR_DATA_DIR` aislados, con puertos 8080/9001
  libres. No detener procesos ajenos.
- El enlace de TikTok debe ser autorizado y contener voz clara en español. Usarlo
  solo en la ejecución privada; no guardar el enlace ni datos personales en el
  repositorio o en la evidencia pública.
- El perfil personal de AppData está fuera de esta aceptación.

## Recorrido funcional del mismo instalador

| Caso | Criterio de aceptación | Resultado / evidencia |
| --- | --- | --- |
| Instalación nueva | Instala sin herramientas de desarrollo y abre Pulsaria desde el acceso creado | Pendiente |
| Onboarding | Aceptación legal y de derechos explícita; estados y acciones completas, sin desbordes | Pendiente |
| Ventana y teclado | 1280×800 y 860×640; Tab/Shift+Tab/Enter/Escape, foco visible y navegación operable | Pendiente |
| Biblioteca vacía y DEMO | Carga vacía y contenido de demostración identificables, sin mezclar datos reales | Pendiente |
| TikTok desde la app | Enlace autorizado importado desde UI/IPC; job termina y la biblioteca contiene texto de transcripción reconocible en español | Pendiente |
| Cola y errores | Progreso, actividad, URL inválida, deduplicación, error y reintento se comportan correctamente | Pendiente |
| Búsqueda | Coincidencia literal; búsqueda conceptual después de preparar el índice, con resultado que abre la ficha o momento | Pendiente |
| Biblioteca y fuentes | Persisten ficha, playlist y configuración de fuente después de cerrar y volver a abrir | Pendiente |
| Perfiles | Agregar no descarga; consulta carga todo lo disponible o el límite elegido, abre perfil/canal, reproduce online y procesa gradualmente con un video a la vez; conserva pausa y estados reales | Pendiente |
| Cinema y exportación | Reproducción, transcripción sincronizada cuando hay marcas de tiempo y exportaciones MP4/MP3/TXT disponibles | Pendiente |
| Ajustes | Cambios guardados, cancelación y persistencia tras reinicio | Pendiente |
| Actualización Eval.3 | La actualización conserva biblioteca sintética, configuración y archivos; SQLite pasa integridad | Pendiente |
| Desinstalación | Retira la aplicación y conserva los datos de usuario conforme al contrato | Pendiente |

## Evidencia de publicación

- Adjuntar al registro privado hash, captura de la app instalada y diagnóstico
  sanitizado; no incluir URLs de prueba, bases de datos, transcripciones privadas
  ni credenciales.
- Confirmar nombre, ciudad y correo públicos en `legal/release-manifest.json`.
  No publicar domicilio postal. El contacto electrónico/ciudad debe contar con
  aprobación legal registrada fuera del repositorio; el gate permanece bloqueado
  mientras ese dictamen no exista.
- Verificar el SBOM SPDX, hashes, procedencia y materiales de terceros del mismo
  artifact; completar las revisiones de titularidad/licencias.
- Mantener `DIRECT_DOWNLOAD_RELEASE_READY=false` hasta que pase la aceptación
  del hash exacto, la revisión legal y la aprobación requerida del Environment
  `direct-download`. Publicar el mismo artifact, sin recompilar, y releer los
  activos y hashes desde GitHub.
