# Cómo funciona

Mediciones hechas en noviembre de 2026 con un Redragon Rift GT10 (USB `2563:0575`, "G710 USB gaming controller") en Windows 11. Todo lo que sigue sale de esas lecturas, no de documentación del fabricante.

## Qué ve Windows

Windows ve 13 botones, una cruceta y los dos palos. Cada trasero llega como uno de los botones 1 a 4, el mismo que su gemelo de adelante. En el panel de controladores de Windows y en el navegador (Gamepad API) un trasero y su botón de adelante son idénticos, por eso ningún programa que lea el mando por la vía normal puede separarlos.

## Qué hay en el informe crudo

El mando manda informes HID de 28 bytes: un byte de identificador (0) y 27 de datos. `hidapi` entrega solo los 27 de datos, y las posiciones de abajo cuentan desde ahí. Los scripts de `herramientas/` muestran el informe completo, así que a sus posiciones hay que sumarles 1.

| Posición | Contenido |
|---|---|
| 0 | Botones 1 a 8, un bit cada uno |
| 1 | Botones 9 a 16, un bit cada uno |
| 2, bits bajos (`0x0F`) | Cruceta: 0 arriba, 1 arriba derecha, 2 derecha, y así en sentido horario hasta 7; 8 es reposo |
| 2, bits altos (`0xF0`) | Traseros (ver más abajo) |
| 3 a 6 | Palo izquierdo X, Y y palo derecho X, Y, centrados en `0x80` (ese orden se dedujo del orden de los ejes en Windows; no se verificó eje por eje) |

La diferencia entre un trasero y un botón de adelante está en los bits altos de la posición 2. Cada trasero activa el bit del botón que copia y además un bit propio:

| Trasero | Bit propio | Botón que copia |
|---|---|---|
| 1 | `0x80` | 1 (Y, el de arriba) |
| 2 | `0x20` | 2 (B, el de la derecha) |
| 3 | `0x10` | 3 (A, el de abajo) |
| 4 | `0x40` | 4 (X, el de la izquierda) |

Los botones de adelante dejan esos bits en 0. Windows ignora los bits altos de esa posición, y por eso desde fuera no se nota la diferencia.

En las lecturas, un botón de adelante también subía un byte de presión analógica más adelante en el informe, y un trasero no. El programa no usa ese byte.

## Mapa de botones

Numeración de Windows, desde 1.

| Botón | Control |
|---|---|
| 1 | Y, el de arriba |
| 2 | B, el de la derecha |
| 3 | A, el de abajo |
| 4 | X, el de la izquierda |
| 5, 6 | L1, R1 |
| 7, 8 | L2, R2 (el programa los lee como botones) |
| 9, 10 | Select, Start (los dos botones pequeños de arriba, a cada lado del logo) |
| 11, 12 | Clic del palo izquierdo, clic del palo derecho |
| 13 | Botón grande del centro |

Los 4 botones de la derecha se midieron por posición con una foto del mando, no por las letras impresas. El botón cuadrado y el botoncito a la derecha del botón central no envían nada.

## El filtro

`src/informe.rs` lee el informe y, si algún bit de trasero está activo, borra el botón de adelante que ese trasero arrastra. El resultado se convierte a un mando de Xbox 360: los botones 1 a 4 pasan a Y, B, A y X, L2 y R2 se mandan como gatillos al máximo, y la cruceta y los palos se convierten a los rangos de XInput.

`src/puente.rs` hace el ciclo: busca el mando, crea el mando virtual con [ViGEmBus](https://github.com/nefarius/ViGEmBus), lo oculta con [HidHide](https://github.com/nefarius/HidHide), lee y envía. Si el mando se desenchufa, suelta todo y espera a que vuelva.

## Ocultar el mando original

Sin ocultarlo, los juegos que leen todos los mandos verían el original, con los traseros, además del virtual. `src/ocultar.rs` usa `HidHideCLI.exe` para:

1. Registrar el programa como una de las aplicaciones que pueden ver dispositivos ocultos.
2. Ocultar el mando original por su ruta de dispositivo.
3. Activar el ocultado.

Al terminar, devuelve todo a como estaba. Si el ocultado ya estaba activado antes por otra razón, lo deja así.

HidHide funciona con un controlador de filtro. Recién instalado, el ocultado se guarda pero no afectó al mando que ya estaba conectado hasta reiniciar el PC (al reiniciar el dispositivo, Windows indicó que hacía falta reiniciar el sistema). Lo que se observó: sin reiniciar, un programa no registrado seguía pudiendo abrir el mando oculto; tras reiniciar, ya no podía, y el programa registrado sí.

## Por qué no es un controlador propio

Un controlador de filtro propio necesita firma de Microsoft para cargarse en Windows sin desactivar la verificación de firmas. Reutilizar ViGEmBus y HidHide, que ya están firmados, evita eso.

## Cómo se midió

Los scripts de `herramientas/`:

- `leer_botones.py`: muestra qué botón se pulsa, con la interfaz de Windows (winmm). Se usó para ver que los traseros dan los mismos números que los de adelante.
- `leer_hid_crudo.py`: imprime el informe HID completo cada vez que cambia. Se usó para encontrar los bits de los traseros: se pulsó un solo trasero y se comparó con su botón de adelante. Un informe cambia de forma continua en algunos bytes finales aunque no se toque nada, así que conviene mirar solo los cambios en las primeras posiciones.
- `probador-en-navegador.html`: muestra en vivo los botones y ejes que ve el navegador. Necesita abrirse con el mando conectado y pulsar un botón primero.

Estas pruebas leen el mando sin modificarlo.
