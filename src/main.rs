use std::fmt::Display;
use std::future::Future;
use std::time::Duration;

use actix_web::{App, HttpServer, Responder};
use actix_web::web::Data;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

use migration::{ConnectionTrait, MigratorTrait};
use migration::sea_orm::{ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, IntoActiveModel, TransactionTrait};

use crate::span::DomainRootSpanBuilder;

pub mod span;
pub mod store;
pub mod router;
pub mod common;

// #[tokio::main]
#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_target(true);

    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .event_format(format);

    sub.init();

    let db_url = "postgres://postgres:postgres@localhost:5432/tokio-test";
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(2)
        .sqlx_logging(false)
        // .sqlx_logging_level(log::LevelFilter::Debug)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8));
    let conn: DatabaseConnection = Database::connect(opt)
        .await
        .expect("Cannot connect to database");
    // Migrator::up(&conn, None).await?;

    // let (tx, rx) = tokio::sync::mpsc::channel::<()>(10);
    //
    // let mut x = 10;
    // let mut b = || x+=1;
    // b();
    // let mut handles = vec![];
    // let arc_conn = Arc::new(Mutex::new(conn));

    // let count = Arc::new(Mutex::new(1));
    // let res = async_retry(|| {
    //     let num = count.clone();
    //     async move {
    //         let mut count = num.lock().await;
    //         if *count >= 2 {
    //             info!("success");
    //             return Ok(1);
    //         }
    //         *count += 1;
    //         Err(anyhow!("hi")) as Result<i32, anyhow::Error>
    //     }
    // }, 3, Duration::from_secs(1)).await?;
    // for _ in 0..10 {
    //     let conn = arc_conn.clone();
    //     let handle = tokio::spawn(async move {
    //         let c = conn.lock().unwrap().clone();
    //         let tx = c.begin().await.unwrap();
    //         let res = async_retry(|| async {
    //             // Ok(1) as Result<i32, DbErr>
    //             incrment(&tx, 1).await
    //         }, 3, Duration::from_secs(1)).await;
    //         tx.commit().await.unwrap();
    //     });
    //
    //     handles.push(handle);
    // }

    // for handle in handles {
    //     handle.await.unwrap();
    // }

    // let tx = conn.begin().await.unwrap();
    // update_test(&tx).await?;
    // decrment(&tx, 1).await?;

    // tx.commit().await?;
    // debug!("success");

    let arc_conn = Data::new(conn);


    let server = HttpServer::new(move || {
        let app = App::new();
        let tracing = TracingLogger::<DomainRootSpanBuilder>::new();
        app.wrap(tracing).app_data(arc_conn.clone())
            .service(router::index)
    });

    server
        .workers(1)
        .bind("0.0.0.0:8080")?
        .run().await?;

    Ok(())
}