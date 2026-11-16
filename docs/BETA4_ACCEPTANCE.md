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
| Archivo `.exe` y tamaño | `Pulsaria_0.1.0-beta.4_x64-setup.exe`; artifact de Actions `pulsaria-p1-04-acceptance-43751089553-1`, `586,813,528` bytes según `size_in_bytes` (ID `14177740443`; tamaño total del artifact, no del `.exe`) |
| SHA-256 del `.exe` | `D16F658EB6AD5B7E5BD93ED6008355826FC8E589BFFBA3B9725807C0C75E84D0` (registrado para este artifact exacto en las evidencias de P1-04 y QA-05) |
| Tag / source commit del artefacto | Sin tag / `e38731eef28551c55c86aae84560d87c27952be9` |
| CI canonical / runtime y artifact | [canonical 43747260931 PASS](https://github.com/danielunibe/Pulsaria/actions/runs/43747260931); [runtime PR 43747260920 PASS](https://github.com/danielunibe/Pulsaria/actions/runs/43747260920); dispatch exacto [43751089553 PASS](https://github.com/danielunibe/Pulsaria/actions/runs/43751089553), incluido smoke NSIS instalado aislado |
| Digest del ZIP / expiración | `sha256:778185be93e1173aed9c54f0cea1a000246a9a3cba5c40c1b842321ae7df3739`; artifact retenido hasta `2026-12-15T19:36:27Z` |
| Windows x64 / WebView2 | Smoke NSIS instalado en runner aislado PASS; aceptación final en Windows limpio, inspección visual/teclado y prueba live pendiente |
| Fecha / operador | 2026-11-15 / runner de GitHub Actions; aceptación por operador humano pendiente |

El artifact retenido del run [43751089553](https://github.com/danielunibe/Pulsaria/actions/runs/43751089553)
corresponde al código del commit exacto registrado arriba. La rama contiene
commits documentales posteriores que actualizan esta hoja y no modifican el
binario ni requieren reconstruirlo. La aceptación humana queda
pendiente y debe aplicarse únicamente a ese artifact/digest. Cualquier
reconstrucción o cambio de archivo invalida la aceptación. El artifact anterior de
`5b0b12b8adc05745f1b606efeb1f88eade6ba97a` quedó superseded; no debe usarse
para aceptar el candidato actual.

El candidato previo de SHA `2fad9a2fa28211f8f0b326b816f761fcb782edc3`, validado
en el run 43710866209, quedó superseded por el candidato actual. El más antiguo
`3295202cff999ae666818d94ac49280a682d8d66`, validado en el run 43701997112,
también es histórico y no debe usarse para aceptar el candidato actual.

## Verificación local del artifact y disponibilidad del entorno — 2026-11-16

Se descargó el ZIP original del run [43751089553](https://github.com/danielunibe/Pulsaria/actions/runs/43751089553)
sin reconstruirlo. La metadata del artifact (ID `14177740443`, nombre
`pulsaria-p1-04-acceptance-43751089553-1`, commit
`e38731eef28551c55c86aae84560d87c27952be9`, tamaño `586,813,528` bytes) y el
digest publicado y calculado localmente coinciden:
`sha256:778185be93e1173aed9c54f0cea1a000246a9a3cba5c40c1b842321ae7df3739`.
El manifiesto incluido identifica el mismo run, intento 1 y versión
`0.1.0-beta.4`; el SHA-256 calculado del instalador coincide con el manifiesto:
`D16F658EB6AD5B7E5BD93ED6008355826FC8E589BFFBA3B9725807C0C75E84D0`.

El instalador y el ejecutable embebido reportan `NotSigned` en Authenticode.
El stub Unicode NSIS es PE32 (`0x014c`) y el payload `pulsaria.exe` es PE x64
(`0x8664`). Esto identifica el payload x64; no acredita instalación en Windows.

El único host disponible fue Windows 11 Pro x64, build `26300`, un equipo de
desarrollo, no limpio. Windows Sandbox no está instalado y no hubo una VM
Windows accesible en la sesión. No se instaló ni inició el artifact. Por ello
siguen pendientes la matriz de aceptación instalada y la confirmación explícita
requerida inmediatamente antes de instalar o ejecutar NSIS en un entorno
aislado. La aceptación humana debe aplicarse únicamente al hash exacto de la
tabla anterior.

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
