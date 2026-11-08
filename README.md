# Lab 5 — Cuerpos celestes: fase de shaders obligatorios

## Movimiento de accesorios

`src/motion.rs` aplica transformaciones rígidas independientes a los accesorios:
corona de Solaria (giro frontal), luna de Lunaria (órbita alrededor del eje Y),
anillos de Aurelia y Vórtex (precesión lenta), fragmentos de Labyrinthia
(flotación con tres fases distintas) y núcleo de Vórtex (giro propio).
Ocularis conserva su montaje para mantener iris y arcos alineados.

El visor inicia con la vista fija y los accesorios animados. `G` activa/desactiva
el giro general; `A` pausa/reanuda únicamente los accesorios. `ESPACIO` pausa
ambos movimientos y conserva esas preferencias al reanudar. `R` restaura la vista
frontal y detiene el giro general. Las flechas permiten inspeccionar el conjunto
en cualquier momento. Todas las velocidades,
incluidas las flechas, usan tiempo transcurrido; el reloj se conserva al cambiar
de cuerpo. Las normales reciben la misma rotación que su pieza y los patrones
mantienen sus coordenadas locales originales. El encuadre incluye una esfera
conservadora para la trayectoria completa y el desplazamiento de los fragmentos.

Capturar un instante de la animación sin abrir ventana:

```powershell
cargo run --release -- --capture --time=8
```

El tiempo predeterminado es cero, que conserva el montaje original. Las capturas
usan vista frontal fija; solo los accesorios avanzan al instante solicitado.
Cada ejecución reemplaza las capturas del mismo nombre en `assets/previews/`.

## Comparación de técnicas de la presentación de shaders

`src/techniques.rs` adapta los conceptos de la presentación a este rasterizador
en CPU. Los modelos de iluminación utilizan la misma base lavanda y la misma
luz para que la diferencia visible provenga del cálculo y la interpolación.

| Tecla | Modo | Cálculo |
|---|---|---|
| `F1` | Cristal | Shaders artísticos de los seis cuerpos y sus accesorios |
| `F2` | Flat | Normal geométrica y luz una vez por cara; color constante en el triángulo |
| `F3` | Gouraud | Luz y especular por vértice; interpolación del color con perspectiva |
| `F4` | Phong | Normal interpolada y normalizada; difuso y especular por fragmento con vector reflejado |
| `F5` | Toon | Tres bandas de luz y borde oscuro según normal/cámara |
| `F6` | Terreno fBm | Cuatro octavas de value noise determinista, océano, costa, terreno y nieve con smoothstep |
| `F7` | Disolución | Descarte según ruido; borde dorado; los fragmentos descartados no escriben Z |
| `-` / `=` | Umbral de disolución | Mantener pulsada la tecla para disminuir/aumentar; valor en el título |
| `P` | Postprocesado | Normal → grises → inversión → bordes Sobel → normal |

`F2`–`F7` funcionan con los seis modelos. Seleccionar un modo activa el sombreado
y sale de wireframe. Se conservan `1`–`6`, `S`, `T`, espacio y flechas.
El postprocesado recorre el framebuffer CPU después de dibujar la escena; Sobel
lee una copia del frame para evitar modificar los vecinos mientras los procesa.
`mix`, `smoothstep`, ruido y fBm calculan patrones locales, sin imágenes ni MTL.
La disolución es un recorte geométrico aparente, no transparencia real.

Para generar 42 capturas reales (siete modos por cada uno de los seis cuerpos):

```powershell
cargo run --release -- --capture-techniques
```

Se guardan en `assets/previews/`, con sufijos `_flat`, `_gouraud`, `_phong`,
`_toon`, `_terrain` y `_dissolve`. El modo cristal conserva el nombre sin sufijo.
Las capturas fijan el umbral de disolución en 0.5, sin postprocesado.

Esta implementación cubre sombreado, funciones de forma, ruido, descarte y
postprocesado de la presentación. Wrapping, filtrado y mipmaps quedan fuera
porque el laboratorio usa patrones procedurales sin texturas. No se implementó
desplazamiento de vértices en esta fase.

Renderer Rust en CPU con minifb. Los modelos actuales están en `assets/*.obj`;
los `.blend` editables están junto a ellos. El visor ignora los archivos MTL:
los colores, patrones y reflejos se calculan en `src/shaders.rs`, sin texturas.

Ejecutar desde la raíz con `cargo run --release`.

| Tecla | Acción actual |
|---|---|
| `1`, `2`, `3` | Solaria, Lunaria y Aurelia con shaders |
| `4`, `5`, `6` | Ocularis, Labyrinthia y Vortex con shaders |
| `S` | Activar/desactivar shaders para comparar con el sólido blanco |
| `T` | Alternar sólido y wireframe |
| `ESPACIO` | Pausa/reanuda las animaciones |
| `G` | Activa/desactiva el giro general (inicia apagado) |
| `A` | Activa/desactiva la animación independiente de accesorios |
| `R` | Vuelve a la vista frontal y detiene el giro general |
| Flechas | Rotar la vista en cualquier momento |
| `ESC` | Salir |

Capas de color de esta fase:

| Pieza | Capas procedurales |
|---|---|
| Solaria | Base durazno, regiones coral con ruido de baja frecuencia, células doradas con ruido de mayor frecuencia, filamentos crema sobre los límites de las células y acabado nacarado. Color emisivo independiente del difuso. |
| Corona | Oro, mezcla menta, franjas claras y brillo nacarado. |
| Lunaria | Base lavanda, minerales violetas, depresiones y bordes circulares de cráteres simulados por color, vetas menta y destellos dorados. |
| Luna | Base violeta gris, variación mineral, cuencas oscuras, bordes crema y acabado pulido propio. |
| Aurelia | Base lavanda, bandas rosa por latitud, perturbación por ruido, bandas azules/crema, tormenta ovalada y nubes claras. |
| Anillos | Franjas radiales oro/menta/crema, huecos por descarte de fragmentos y brillo. Comparten Z-buffer con la esfera. |
| Ocularis | Base lavanda/rosa, variaciones suaves crema, vetas doradas y acabado especular/Fresnel. Iris separado con base menta, filamentos radiales violetas, vetas oro, borde oscuro y pupila central. Arcos con oro, rosa, crema y reflejos. |
| Labyrinthia | Base rosa/lavanda, caminos ondulados menta, contornos crema, líneas doradas y acabado nacarado. Tres fragmentos separados con oro, rosa, variación crema y brillo. La abertura pertenece a la geometría. |
| Vórtex | Cascarones con base lavanda, remolinos rosa, regiones crema, filamentos oro y matices menta. Núcleo emisivo con base rosa, ondas crema y crestas doradas. Anillo con franjas y huecos procedurales. |

El acabado de vidrio se simula con especular blanco, tinte según la orientación
respecto a la cámara y realce Fresnel del borde. Las superficies son opacas:
no hay refracción ni transparencia física. Los patrones usan posiciones locales;
las normales OBJ rotan con la geometría y se interpolan con corrección de
perspectiva, incluso al recortar contra el plano cercano. El ruido es determinista.

Capturas reales del framebuffer CPU, a 800 × 600, sin abrir ventana:

```powershell
cargo run --release -- --capture
```

Resultados en `assets/previews/`: `solaria.bmp`, `lunaria.bmp`, `aurelia.bmp`,
`ocularis.bmp`, `labyrinthia.bmp` y `vortex.bmp`.
La cámara queda frontal y sin animación para facilitar comparaciones. Son
capturas del rasterizador del programa, no renders de Blender.

Verificación: `cargo fmt --check`, `cargo check`, `cargo test`. Las pruebas
incluyen los seis OBJ, normales, independencia del orden de dibujo y variación
de color en los seis shaders. La aceptación de las siluetas de fantasía como
planetas adicionales depende del docente.

---

## Documentación histórica de la base del Lab 6

La descripción y los controles siguientes corresponden al visor original de la nave.

Renderizador de una nave OBJ escrito en Rust con minifb. Incluye rotación automática y manual, y dos modos de visualización:

- **Esqueleto**: dibuja las aristas de todos los triángulos, incluidas las posteriores.
- **Pintado**: rellena las superficies con el color difuso del material MTL y aplica iluminación por cara. Un Z-buffer resuelve la visibilidad. Sin material se utiliza el color base `0x87CEEB`.

## Capturas del visor

Capturas reales del área de dibujo de minifb, a 800 × 600 píxeles. Ambas muestran la misma orientación manual: rotación de 0.55 radianes en X y 0.65 radianes en Y. Para fijar esta vista durante la captura se ajustaron temporalmente los valores iniciales; el programa conserva su inicio habitual en modo automático y esqueleto.

### Modo esqueleto

Permite inspeccionar la triangulación y las aristas de la nave.

![Nave completa en modo esqueleto, vista oblicua](assets/Imagen_Esqueleto.png)

### Modo pintado

Muestra las superficies visibles con el material gris del modelo y la iluminación por cara.

![Nave completa en modo pintado, desde el mismo ángulo](assets/Imagen_Pintado.png)

## Controles

| Tecla | Acción |
|-------|--------|
| `ESPACIO` | Cambiar entre rotación automática y manual |
| `T` | Cambiar entre esqueleto y pintado |
| `←` / `→` | Rotar en Y en modo manual |
| `↑` / `↓` | Rotar en X en modo manual |
| `ESC` | Salir |

## Compilar y ejecutar

Ejecuta los comandos desde la raíz del proyecto para resolver las rutas de `assets/`.

```bash
cargo build
cargo run
cargo run --release
```

## Visibilidad y materiales

El relleno comparte un Z-buffer entre todas las caras y lo limpia cada fotograma. Con la cámara en `z = 5`, interpola baricéntricamente `1 / (5 - z)` en pantalla: el valor mayor corresponde a la superficie más cercana. Recorta contra una distancia cercana de `0.1` antes de proyectar y limita la rasterización a la ventana. Omite los triángulos degenerados o no finitos.

El OBJ incluido contiene 730 vértices y 1512 caras, todas triangulares. Hay 756 copias geométricas exactas; el cargador conserva una por material, incluso si la copia usa otros índices u otro orden. El archivo OBJ original se conserva.

`assets/nave_1.mtl` define un solo material gris (`Kd 0.8 0.8 0.8`), por lo que este modelo no contiene colores distintos por pieza. El cargador conserva `usemtl` por cara; la iluminación es independiente de la selección de color y del índice del triángulo.

La inspección geométrica detectó 128 caras de área prácticamente nula en el archivo original, incluidas sus copias, 32 aristas con seis incidencias y 32 aristas de dos incidencias con orientación inconsistente. No se reparó la topología del modelo. Se mantienen ambas orientaciones visibles, sin backface culling.

El cargador requiere OBJ triangulados y devuelve un error ante otros polígonos, en lugar de crear abanicos incorrectos sobre caras cóncavas.

## Verificación

```bash
cargo fmt --check
cargo check
cargo test
```

Las pruebas cubren profundidad en perspectiva, independencia del orden de dibujo, ambos órdenes de vértices, recorte cercano, degenerados, caras fuera de pantalla, buffers inválidos, materiales, índices OBJ negativos y cuatro rotaciones de la nave.

Para comprobar visualmente el resultado, ejecuta el visor, pulsa `T`, cambia a manual con `ESPACIO` y usa las flechas para observar el frente, los lados y la parte posterior.

## Repositorio

[Código fuente en GitHub](https://github.com/MagicBag-tab/lab_6_carga_de_modelo)
