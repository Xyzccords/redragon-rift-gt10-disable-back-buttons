# Redragon Rift GT10: desactivar los botones traseros

Programa para Windows que anula los 4 botones de atrás (paddles o botones de macro) del mando Redragon Rift GT10. Esos botones repiten X, Y, A y B, y los juegos suelen detectarlos antes que los de adelante, por ejemplo al asignar controles.

Windows muestra este mando como "G710 USB gaming controller" (ShanWan, USB `2563:0575`). Por eso también se busca como Redragon GT710 o G710.

[English summary](README.en.md)

## Qué hace

Windows no permite desactivar botones sueltos de un mando, y este modelo no trae programa del fabricante para hacerlo. El manual tampoco lo menciona. El programa se pone entre el mando y los juegos:

1. Lee el mando directamente.
2. Descarta las señales de los traseros.
3. Entrega el resto a un mando virtual de Xbox 360 (con ViGEmBus).
4. Oculta el mando original a los juegos (con HidHide), para que no vean dos mandos.

Los juegos ven un solo mando de Xbox 360 con los botones de adelante y sin los traseros. También sirve en juegos que solo aceptan mandos de Xbox.

## Instalar

Descarga `FiltroMando_Setup_0.1.0.exe` de la sección [Releases](../../releases). También puedes generarlo con los pasos de [Compilar](#compilar).

1. Ejecuta `FiltroMando_Setup_0.1.0.exe`. Windows puede avisar "Windows protegió su PC" porque el instalador no está firmado. Elige "Más información" y "Ejecutar de todas formas".
2. El instalador agrega ViGEmBus y HidHide si no los tienes. Pide permisos de administrador.
3. Si instaló HidHide, reinicia el PC cuando te lo pida al final. Sin el reinicio, el mando original sigue visible para los juegos y los traseros siguen llegando.
4. Abre "Filtro de mando" desde el menú Inicio. Mientras esté abierto, los traseros no llegan a los juegos.

Con la casilla "Iniciar con Windows", el programa arranca minimizado al iniciar sesión.

## Usar

La ventana muestra si el mando está conectado y si el original está oculto. El botón **Pausar** devuelve el mando original tal como viene de fábrica, con los traseros activos, y **Reanudar** vuelve a filtrar. Al cerrar la ventana se cierra el filtro y el mando original vuelve a verse.

Si el mando se desenchufa y se vuelve a enchufar, el programa lo retoma solo.

Opciones de línea de comandos:

| Opción | Qué hace |
|---|---|
| `--minimizado` | Abre la ventana minimizada (así arranca con Windows). |
| `--sin-ventana [segundos]` | Corre en la consola, sin ventana. Con un número, se detiene solo. |
| `--ver [segundos]` | Solo muestra en la consola lo que pulsas, sin crear el mando virtual ni ocultar el original. Sirve para comprobar el mando. |
| `--liberar` | Vuelve a mostrar el mando original si el programa se cerró a la fuerza y lo dejó oculto. |

El instalador crea un acceso "Mostrar el mando original (si quedó oculto)" que ejecuta `--liberar`.

## Compilar

Requisitos: Windows de 64 bits (probado en Windows 11), [Rust](https://rustup.rs) (toolchain MSVC) e [Inno Setup 6](https://jrsoftware.org/isinfo.php).

```powershell
.\scripts\descargar-dependencias.ps1
cargo build --release
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\filtro_mando.iss
```

Si PowerShell bloquea el script, ejecútalo con `powershell -ExecutionPolicy Bypass -File .\scripts\descargar-dependencias.ps1`.

El primer comando baja los instaladores de ViGEmBus y HidHide a `dependencias\` y comprueba su tamaño y su SHA-256. El último escribe el instalador en `dist\`. Si Inno Setup está en otra carpeta, usa la ruta de tu `ISCC.exe`.

Para probar sin instalar, `target\release\filtro_mando.exe` funciona igual, siempre que ViGEmBus esté instalado. Sin HidHide el programa crea el mando virtual igual, pero avisa que los juegos verán también el mando original.

## Límites conocidos

- Probado con un solo mando, en dos PCs con Windows 11, con ViGEmBus 1.22.0 y HidHide 1.5.230. En el segundo PC, que no tenía ninguno de los dos controladores, el instalador los instaló y el filtro funcionó.
- No se probó en Windows 10 ni con otros mandos.
- El instalador no está firmado.
- ViGEmBus está archivado por su autor y ya no recibe cambios. La versión 1.22.0 funcionó en la prueba.
- Si pulsas un trasero y el botón de adelante que copia al mismo tiempo, ambos se descartan. Esto sale del código y no se probó.
- El botón cuadrado y el botoncito a la derecha del botón central del mando no envían nada al PC, así que no se pueden usar para nada.
- No hay ícono.

## Cómo funciona y otros mandos

[docs/como-funciona.md](docs/como-funciona.md) explica qué envía el mando, cómo se distinguen los traseros y cómo se hicieron las mediciones. La carpeta `herramientas/` tiene los scripts que se usaron. Si tienes otro mando con el mismo chip, las posiciones están en `src/informe.rs`; no se probó con ninguno.

## Licencia

El código es MIT (ver [LICENSE](LICENSE)). El instalador incluye HidHide (MIT) y ViGEmBus (BSD-3-Clause), de Nefarius Software Solutions; sus licencias están en `licencias-terceros/`.
