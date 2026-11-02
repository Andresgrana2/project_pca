//! Pruebas del cálculo de cotizaciones y del contrato HTTP de FastExpress.
//! Los casos unitarios comprueban BR-01 a BR-04; la integración recorre las rutas
//! con solicitudes reales a un servidor local creado únicamente para esta prueba.
use super::packing_service::calculate;
use crate::dto::discount_type::DiscountType;
use crate::dto::order_dto::OrderInput;
use crate::dto::order_type_dto::OrderType;
use crate::dto::packing_dto::PackingError;
use crate::error::AppError;
use crate::services::fee_service::FeeService;
use crate::services::order_service::OrderService;
use axum::body::to_bytes;
use axum::response::IntoResponse;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// f32 puede introducir pequeñas diferencias de representación: se usa una
// tolerancia de 0.0001 en vez de exigir igualdad exacta entre importes calculados.
fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001,
        "expected {expected}, received {actual}");
}

// Genera una prueba por peso para que cada frontera tenga su propio resultado.
// El importe esperado es solo el cargo por peso, sin tarifa base ni ajustes.
macro_rules! valid_weight {
    ($name:ident, $weight:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_close(calculate($weight).expect("valid weight"), $expected);
        }
    };
}
// BR-02: mínimo admitido y su vecino superior, ambos a USD 2 por kg.
valid_weight!(weight_minimum_001, 0.01, 0.02);
valid_weight!(weight_above_minimum_002, 0.02, 0.04);
// Representante interior del primer rango y fronteras del cambio de tarifa.
valid_weight!(weight_representative_3, 3.0, 6.0);
valid_weight!(weight_below_tariff_limit_499, 4.99, 9.98);
valid_weight!(weight_tariff_limit_500, 5.00, 10.00);
valid_weight!(weight_next_tariff_501, 5.01, 20.04);
// Representante del segundo rango y valores próximos al máximo de 20 kg.
valid_weight!(weight_representative_10, 10.0, 40.0);
valid_weight!(weight_below_maximum_1999, 19.99, 79.96);
valid_weight!(weight_maximum_2000, 20.00, 80.00);

// Verifica la clase inválida inferior y los valores no finitos directamente
// sobre la función; NaN e infinitos no se representan como números JSON válidos.
#[test]
fn rejects_non_finite_and_below_minimum_weights() {
    for weight in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0, 0.009] {
        assert!(matches!(calculate(weight), Err(PackingError::InvalidWeight)),
            "unexpected result for {weight}");
    }
}

// BR-02: tanto el vecino de 20 kg como valores muy superiores deben rechazarse.
#[test]
fn rejects_weights_above_maximum() {
    for weight in [20.01, 25.0, f32::MAX] {
        assert!(matches!(calculate(weight), Err(PackingError::InvalidPacking)));
    }
}

// BR-01: el servicio debe proporcionar siempre la tarifa base de USD 5.
#[tokio::test]
async fn base_fee_is_five() {
    assert_close(FeeService::new().get_fee().await.base, 5.0);
}

// Genera las seis reglas de la tabla de decisión de cliente y envío.
// Con 2.5 kg el subtotal es USD 10: permite comprobar fácilmente los factores.
macro_rules! quotation {
    ($name:ident, $order:ident, $discount:ident, $expected:expr) => {
        #[tokio::test]
        async fn $name() {
            // Arrange: prepara una orden válida y las opciones de esta regla.
            let input = OrderInput {
                weight: 2.5,
                order_type: OrderType::$order,
                discount: DiscountType::$discount,
            };
            // Act: ejecuta el cálculo completo con la tarifa base real.
            let output = OrderService::new().create_order(input, &FeeService::new())
                .await.unwrap_or_else(|_| panic!("quotation should succeed"));
            // Assert: comprueba el total y los valores que se serializan en JSON.
            assert_close(output.total_value, $expected);
            let serialized = serde_json::to_value(output).unwrap();
            assert_eq!(serialized["order_type"], stringify!($order));
            assert_eq!(serialized["discount"], stringify!($discount));
        }
    };
}
// BR-03 y BR-04: subtotal de USD 10, recargo de 1.00/1.25 y descuento de
// 1.00/0.90/0.80. Cada combinación se registra como una prueba independiente.
quotation!(new_standard_is_1000, STANDARD, NEW, 10.0);
quotation!(new_express_is_1250, EXPRESS, NEW, 12.5);
quotation!(frequent_standard_is_900, STANDARD, FREQUENT, 9.0);
quotation!(frequent_express_is_1125, EXPRESS, FREQUENT, 11.25);
quotation!(vip_standard_is_800, STANDARD, VIP, 8.0);
quotation!(vip_express_is_1000, EXPRESS, VIP, 10.0);

// Comprueba que create_order conserva la causa del rechazo al convertir
// PackingError en AppError, en lugar de producir una cotización válida.
#[tokio::test]
async fn quotation_preserves_weight_validation_errors() {
    for (weight, below_minimum) in [(0.0, true), (f32::NAN, true), (25.0, false)] {
        let result = OrderService::new().create_order(OrderInput {
            weight, order_type: OrderType::STANDARD, discount: DiscountType::NEW,
        }, &FeeService::new()).await;
        match result {
            Err(AppError::Packing(PackingError::InvalidWeight)) if below_minimum => {},
            Err(AppError::Packing(PackingError::InvalidPacking)) if !below_minimum => {},
            _ => panic!("wrong validation error for {weight}"),
        }
    }
}

// Verifica la conversión de AppError::NotFound a HTTP 404 y su mensaje JSON.
// La página HTML de ruta inexistente se comprueba en la integración posterior.
#[tokio::test]
async fn not_found_error_has_404_json_contract() {
    let response = AppError::NotFound.into_response();
    assert_eq!(response.status(), 404);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["error"], "Recurso no encontrado");
}

// Un error interno debe devolver HTTP 500 y un mensaje público genérico,
// sin filtrar el diagnóstico interno utilizado para construir el error.
#[tokio::test]
async fn internal_error_has_500_without_exposing_details() {
    let response = AppError::Internal("test-only diagnostic".into()).into_response();
    assert_eq!(response.status(), 500);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["error"], "Error interno del servidor");
    assert!(!String::from_utf8_lossy(&body).contains("test-only diagnostic"));
}

// Envía una solicitud HTTP/1.1 por TCP al servidor de prueba y separa estado,
// cabeceras y cuerpo. Connection: close permite leer hasta el cierre del socket;
// el límite de cinco segundos evita esperar indefinidamente por la respuesta.
async fn request(addr: SocketAddr, method: &str, path: &str,
                 body: &str, content_type: Option<&str>) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let mime = content_type.map(|t| format!("Content-Type: {t}\r\n")).unwrap_or_default();
    let message = format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{mime}Content-Length: {}\r\n\r\n{body}", body.len());
    stream.write_all(message.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    tokio::time::timeout(std::time::Duration::from_secs(5), stream.read_to_end(&mut response))
        .await.expect("HTTP response timeout").unwrap();
    let text = String::from_utf8(response).unwrap();
    let (headers, body) = text.split_once("\r\n\r\n").expect("HTTP response");
    let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, headers.to_string(), body.to_string())
}

// Prueba de integración: reúne 28 escenarios en un único test de Tokio.
// Comprueba la colaboración entre rutas, extracción JSON, servicios y plantillas.
#[tokio::test]
async fn http_integration_validates_routes_quotes_errors_and_html() {
    // Arrange: compone las rutas con los servicios reales de la aplicación.
    let state = crate::states::AppState {
        fee_service: FeeService::new(), order_service: OrderService::new(),
    };
    let app = axum::Router::new()
        .merge(crate::routes::html_route::html_routes())
        .nest("/api", crate::routes::fee_route::fee_routes())
        .nest("/api", crate::routes::order_route::order_routes())
        .nest_service("/statics", tower_http::services::ServeDir::new("statics"))
        .fallback(crate::handlers::html::error_controller::not_found)
        .with_state(state);
    // El puerto 0 solicita un puerto libre al sistema y evita depender del 3000.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).with_graceful_shutdown(async { let _ = stopped.await; })
            .await.unwrap();
    });
    let mime = Some("application/json");
    let mut scenarios = 0;
    // Escenarios 1-6: tabla de decisión completa; valida HTTP 201, tipo de
    // contenido, total calculado y conservación de las opciones seleccionadas.
    for (order_type, discount, expected) in [
        ("STANDARD", "NEW", 10.0), ("EXPRESS", "NEW", 12.5),
        ("STANDARD", "FREQUENT", 9.0), ("EXPRESS", "FREQUENT", 11.25),
        ("STANDARD", "VIP", 8.0), ("EXPRESS", "VIP", 10.0),
    ] {
        let input = json!({"weight":2.5, "order_type":order_type, "discount":discount}).to_string();
        let (status, headers, body) = request(addr,"POST","/api/order/new", &input,mime).await;
        assert_eq!(status, 201, "{order_type}/{discount}: {body}");
        assert!(headers.to_lowercase().contains("application/json"));
        let output: Value = serde_json::from_str(&body).unwrap();
        assert_close(output["total_value"].as_f64().unwrap() as f32, expected);
        assert_eq!(output["order_type"], order_type);
        assert_eq!(output["discount"], discount);
        scenarios += 1;
        println!("HTTP_PASS quote {order_type}/{discount}: 201, total={expected}");
    }
    // Escenarios 7-11: fronteras de BR-02 por HTTP. Aquí los totales sí incluyen
    // la tarifa base, a diferencia de las pruebas unitarias de calculate.
    for (weight, expected) in [(0.01,5.02),(4.99,14.98),(5.0,15.0),(5.01,25.04),(20.0,85.0)] {
        let input = json!({"weight":weight,"order_type":"STANDARD","discount":"NEW"}).to_string();
        let (status, _, body) = request(addr,"POST","/api/order/new",&input,mime).await;
        assert_eq!(status,201);
        let output: Value = serde_json::from_str(&body).unwrap();
        assert_close(output["total_value"].as_f64().unwrap() as f32, expected);
        scenarios += 1;
        println!("HTTP_PASS boundary weight={weight}: 201, total={expected}");
    }
    // Escenarios 12-16: pesos fuera del rango; comprueba HTTP 400 y el mensaje
    // de negocio específico, incluso para una orden EXPRESS con cliente VIP.
    for (weight, message) in [
        (0.0,"El peso debe ser un número finito mayor o igual a 0.01 kg"),
        (-1.0,"El peso debe ser un número finito mayor o igual a 0.01 kg"),
        (0.009,"El peso debe ser un número finito mayor o igual a 0.01 kg"),
        (20.01,"Excede el peso máximo"),(25.0,"Excede el peso máximo"),
    ] {
        let input = json!({"weight":weight,"order_type":"EXPRESS","discount":"VIP"}).to_string();
        let (status, _, body) = request(addr,"POST","/api/order/new",&input,mime).await;
        assert_eq!(status,400);
        let output: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(output["error"],message);
        scenarios += 1;
        println!("HTTP_PASS invalid weight={weight}: 400");
    }
    // Escenarios 17-22: errores de entrada rechazados por el extractor JSON
    // antes del cálculo: sintaxis 400, datos incompatibles 422 y contenido 415.
    for (name, input, content_type, expected) in [
        ("malformed JSON", "{", mime, 400),
        ("missing weight", r#"{"order_type":"STANDARD","discount":"NEW"}"#, mime,422),
        ("text weight",r#"{"weight":"abc","order_type":"STANDARD","discount":"NEW"}"#,mime,422),
        ("unknown client",r#"{"weight":2.5,"order_type":"STANDARD","discount":"OTHER"}"#,mime,422),
        ("unknown shipping",r#"{"weight":2.5,"order_type":"OTHER","discount":"NEW"}"#,mime,422),
        ("missing JSON content type","{}",None,415),
    ] {
        let (status, _, body) = request(addr,"POST","/api/order/new",input,content_type).await;
        assert_eq!(status,expected,"{name}: {body}");
        scenarios += 1;
        println!("HTTP_PASS {name}: {status}");
    }
    // Escenario 23: consulta pública de la tarifa base y valor JSON esperado.
    let (status,_,body) = request(addr,"GET","/api/fee","",None).await;
    assert_eq!(status,200);
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["base"],5.0);
    println!("HTTP_PASS base fee: 200, base=5"); scenarios += 1;
    // Escenario 24: la raíz debe redirigir al formulario con HTTP 303.
    let (status,headers,_) = request(addr,"GET","/","",None).await;
    assert_eq!(status,303); assert!(headers.contains("/orders/new"));
    println!("HTTP_PASS root redirect: 303"); scenarios += 1;
    // Escenario 25: renderizado HTML y presencia del contenedor de error de peso.
    let (status,_,body) = request(addr,"GET","/orders/new","",None).await;
    assert_eq!(status,200); assert!(body.contains("Nuevo pedido"));
    assert!(body.contains("weight-error"));
    println!("HTTP_PASS order form: 200"); scenarios += 1;
    // Escenario 26: una ruta desconocida utiliza la página HTML de error 404.
    let (status,_,body) = request(addr,"GET","/not-a-route","",None).await;
    assert_eq!(status,404); assert!(body.contains("Página no encontrada"));
    println!("HTTP_PASS missing page: 404"); scenarios += 1;
    // Escenario 27: el servidor entrega el archivo JavaScript del formulario.
    let (status,_,body) = request(addr,"GET","/statics/js/order.js","",None).await;
    assert_eq!(status,200); assert!(body.contains("function orderForm"));
    println!("HTTP_PASS static JavaScript: 200"); scenarios += 1;
    // Escenario 28: la ruta de cotización acepta POST, por lo que GET devuelve 405.
    let (status,_,_) = request(addr,"GET","/api/order/new","",None).await;
    assert_eq!(status,405);
    println!("HTTP_PASS wrong HTTP method: 405"); scenarios += 1;
    // Cierra el servidor y espera su finalización antes de registrar el resumen.
    shutdown.send(()).unwrap(); server.await.unwrap();
    println!("HTTP_SCENARIOS_PASSED={scenarios}");
}
