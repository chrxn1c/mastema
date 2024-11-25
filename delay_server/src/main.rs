use actix_web::{get, rt::time::sleep, web, App, HttpServer, Responder};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

#[get("/{delay}/{message}")]
async fn delay(path: web::Path<(u64, String)>) -> impl Responder {
    let (delay_ms, message) = path.into_inner();
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("Message received: #{count} - {delay_ms}ms: {message}");
    sleep(Duration::from_millis(delay_ms)).await;
    message
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url = String::from("127.0.0.1");

    println!("Starting HTTP server..");
    HttpServer::new(|| App::new().service(delay))
        .bind((url, 8080))?
        .run()
        .await
}
