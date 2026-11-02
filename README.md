# FastExpress

Módulo de cotización de envíos desarrollado para la asignatura Calidad de Software. Calcula el valor de un envío según el peso, el tipo de cliente y el tipo de envío, mediante una API REST y un formulario web.

## Tecnologías

- Backend: Rust, Axum y Tokio.
- Datos JSON: Serde y serde_json.
- Plantillas HTML: Askama.
- Frontend: JavaScript, Alpine.js y Bootstrap.
- Pruebas: `cargo test` para Rust y `node:test` para JavaScript.
- Cobertura: cargo-llvm-cov para Rust y V8 para JavaScript.

## Requisitos

- Rust y Cargo con soporte para la edición 2024.
- Node.js 24 o posterior para las pruebas y el script de cobertura del frontend.
- PowerShell para ejecutar `scripts/coverage.ps1`.
- Para medir cobertura de Rust: cargo-llvm-cov y el componente `llvm-tools-preview` del toolchain elegido.

El script de cobertura utiliza por defecto `stable-x86_64-pc-windows-msvc`. En Windows se necesita el entorno de compilación de C++ y Windows SDK compatible con ese destino. La interfaz carga Bootstrap y Alpine.js desde CDN, por lo que necesita acceso a Internet para obtener esas bibliotecas.

## Iniciar la aplicación

Ejecutar los comandos desde la raíz del repositorio, donde se encuentra `Cargo.toml`:

```powershell
cargo run --locked
```

Abrir [el formulario de cotización](http://127.0.0.1:3000/orders/new). La dirección raíz redirige a ese formulario. El servidor escucha en `127.0.0.1:3000`; se puede detener con `Ctrl+C`.

La implementación actual calcula cotizaciones en memoria y no necesita configurar una base de datos.

## Reglas del cálculo

| Regla | Comportamiento |
| --- | --- |
| BR-01 | Tarifa base de USD 5,00. |
| BR-02 | De 0,01 a 5,00 kg, el cargo es peso × USD 2,00. Para pesos mayores de 5,00 y hasta 20,00 kg, es peso × USD 4,00. Se rechazan pesos no finitos, menores de 0,01 o mayores de 20 kg. |
| BR-03 | Cliente `NEW`: sin descuento; `FREQUENT`: 10 %; `VIP`: 20 %. |
| BR-04 | Envío `STANDARD`: sin recargo; `EXPRESS`: recargo del 25 %. |

```text
subtotal = 5,00 + cargo por peso
total = subtotal × factor de envío × factor de descuento
```

Primero se aplica el factor de envío y después el descuento sobre ese importe. Con 2,50 kg, cliente `FREQUENT` y envío `EXPRESS`, el total es `(5 + 2,50 × 2) × 1,25 × 0,90 = USD 11,25`.

El código admite pesos con más de dos decimales; por ejemplo, 5,005 kg entra en la tarifa de USD 4,00/kg. Los cálculos usan `f32` y la interfaz muestra dos decimales; la API no aplica una política explícita de redondeo monetario. Estas decisiones deben acordarse si se requiere restringir el peso a dos decimales o garantizar importes exactos en centavos.

## API

Los nombres y valores del JSON se envían en inglés, tal como los define el código.

| Método | Ruta | Resultado |
| --- | --- | --- |
| `GET` | `/api/fee` | Tarifa base; responde `200`. |
| `POST` | `/api/order/new` | Calcula la cotización; responde `201`. |
| `GET` | `/orders/new` | Formulario HTML. |
| `GET` | `/` | Redirección `303` al formulario. |

Ejemplo desde PowerShell, con la aplicación iniciada en otra terminal:

```powershell
$cotizacion = @{
    weight = 2.5
    order_type = 'EXPRESS'
    discount = 'FREQUENT'
} | ConvertTo-Json

Invoke-RestMethod -Method Post `
    -Uri 'http://127.0.0.1:3000/api/order/new' `
    -ContentType 'application/json' `
    -Body $cotizacion
```

La respuesta JSON contiene `order_type`, `discount` y `total_value`; en este ejemplo el total es 11,25.

Los pesos fuera de rango y el JSON mal formado producen `400`. Los campos requeridos ausentes, tipos incorrectos y opciones desconocidas producen `422`. La ausencia del tipo de contenido JSON produce `415` y un método no admitido para una ruta existente produce `405`. Los errores de peso utilizan el campo JSON `error`; los errores de extracción de JSON de Axum pueden tener otro formato de respuesta.

## Estructura

```text
src/
  main.rs                    Arranque del servidor y composición de rutas
  dto/                       Entradas, salidas y tipos de cliente y envío
  handlers/                  Controladores REST y HTML
  routes/                    Definición de rutas
  services/
    fee_service.rs           Tarifa base
    packing_service.rs       Validación y cargo por peso
    order_service.rs         Recargo, descuento y total
    coverage_tests.rs        Pruebas del backend e integración HTTP
statics/js/order.js           Comportamiento del formulario
templates/                   Plantillas HTML
tests/frontend.test.cjs       Pruebas unitarias del frontend
scripts/coverage.ps1          Ejecución de pruebas y generación de cobertura
```

## Suite de pruebas

### Backend

```powershell
cargo test --locked
```

Las pruebas están en `src/services/coverage_tests.rs`, incluido mediante `#[cfg(test)]` en `src/services/mod.rs`. La suite contiene 22 pruebas: nueve casos de pesos válidos, rechazo de pesos inválidos, tarifa base, seis combinaciones de cotización, propagación de errores, respuestas 404/500 y una prueba de integración con 28 escenarios HTTP.

Las pruebas unitarias comparan los resultados `f32` con una tolerancia de `0.0001`. La integración inicia un servidor local en un puerto disponible, comprueba rutas, códigos HTTP y cuerpos de respuesta, y lo detiene al terminar. No hace falta iniciar la aplicación manualmente para ejecutar la suite.

Para ver el detalle de los escenarios HTTP:

```powershell
cargo test --locked http_integration_validates_routes_quotes_errors_and_html -- --nocapture
```

### Frontend

```powershell
node --test --test-isolation=none tests/frontend.test.cjs
```

Las 15 pruebas utilizan módulos incluidos en Node.js; no requieren `npm install`. Cargan `order.js` en un contexto aislado y simulan `fetch`, el enfoque de campos y `$nextTick` de Alpine.js.

Comprueban límites del peso, opciones de cliente y envío, limpieza de mensajes, bloqueo de envíos simultáneos, petición JSON, respuesta exitosa, errores de API y conexión, respuestas inválidas y formato del total. Son pruebas de la lógica JavaScript; la presentación visual y la integración con un navegador deben verificarse por separado.

## Generar cobertura

Con los requisitos de cobertura instalados, ejecutar desde PowerShell:

```powershell
./scripts/coverage.ps1
```

El script ejecuta las pruebas de Rust y JavaScript, genera los reportes en `target/quality-report` y exige al menos 85 % de cobertura de **líneas** del backend. Termina con error si falla una suite o no se alcanza ese umbral.

Los parámetros permiten elegir otro directorio, toolchain o ejecutable de Node:

```powershell
./scripts/coverage.ps1 -OutputDirectory 'target/mi-reporte' `
    -RustToolchain 'stable-x86_64-pc-windows-msvc' -NodeCommand 'node'
```

| Archivo generado | Contenido |
| --- | --- |
| `rust/html/index.html` | Reporte navegable de Rust. |
| `rust/coverage.json` | Datos de cobertura de Rust. |
| `rust-summary.txt` | Resumen por archivo. |
| `rust-tests.log` | Resultado de las pruebas de Rust y escenarios HTTP. |
| `frontend.lcov` | Cobertura del archivo `statics/js/order.js`. |
| `frontend-tests.log` | Resultado de las pruebas y cobertura de JavaScript. |
| `rust-tool.txt`, `node-tool.txt` | Versiones utilizadas en la medición. |

La medición guardada del 28 de octubre de 2026 reportó 22 pruebas de Rust y 15 de JavaScript aprobadas, 85,82 % de líneas del backend (121/141) y 100 % de líneas del formulario (77/77). Esos valores corresponden a esa ejecución y deben actualizarse al generar una nueva evidencia.

La cobertura de ramas de Rust no se midió en ese reporte. La guía académica exige al menos 85 % tanto de líneas como de ramas; cumplir el umbral de líneas por sí solo deja pendiente la evidencia de ramas del backend. La cobertura de regiones tampoco sustituye esa métrica.

El directorio `target/` está excluido de Git. Para entregar evidencias, adjuntar los reportes o publicarlos como artefactos de la entrega. El script genera registros y reportes; el PDF y el ZIP guardados de la medición anterior son documentos adicionales y no se regeneran con este comando.

## Documentación de calidad

El informe final debe relacionar las reglas de negocio con la matriz de casos de caja negra, los valores límite, la tabla de decisión, los caminos independientes del CFG y las evidencias de ejecución y cobertura. La guía menciona pruebas del campo «Monto» sin definirlo como entrada: se debe aclarar con el docente su interpretación. También deben quedar registrados los roles de los integrantes y el commit evaluado.
