use actix_web::{get, HttpResponse, Responder};
use actix_web::web::Data;
use tracing::{info, info_span, instrument};
use tracing_actix_web::RootSpan;
use migration::sea_orm::{DatabaseConnection, TransactionTrait};
use crate::store;

#[get("/")]
#[instrument(skip_all)]
pub async fn index(conn: Data<DatabaseConnection>) -> impl Responder {
    let span = info_span!("-----");
    let _a = span.enter();
    let conn = &conn;
    // root_span.field("admin_name");
    let tx = conn.begin().await.unwrap();
    store::incrment(&tx, 1).await.unwrap();
    tx.commit().await.unwrap();
    // root_span.record("admin_name", "lake");
    HttpResponse::Ok().body("hello")
}