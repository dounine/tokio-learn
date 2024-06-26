use std::fmt::Display;
use std::future::Future;
use std::time::Duration;

use actix_web::{App, HttpServer, Responder};
use actix_web::cookie::time;
use actix_web::cookie::time::UtcOffset;
use actix_web::web::Data;
use time::OffsetDateTime;
use tracing::info;
use tracing_actix_web::TracingLogger;
// use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::OffsetTime;

use migration::{ConnectionTrait, Migrator, MigratorTrait};
use migration::sea_orm::{ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, IntoActiveModel, TransactionTrait};
use crate::file_appender::{AppenderBuilder, Rotation};

use crate::span::DomainRootSpanBuilder;

pub mod span;
pub mod store;
pub mod router;
pub mod file_appender;
pub mod common;

// #[tokio::main]
// async fn main() -> Result<(), anyhow::Error> {
//     let mut headers = Vec::new();
//     //10个并发请求
//     for i in 0..5 {
//         let handle = tokio::spawn(async move {
//             let client = reqwest::Client::builder()
//                 .build().unwrap();
//             let form = reqwest::multipart::Form::new()
//                 .text("app_name", "hello")
//                 .text("app_version", "1.0.0")
//                 .text("app_bundle_id", "a.b.c")
//                 .text("remove_all_plugin", "false")
//                 .text("remove_watch_plugin", "false")
//                 .text("remove_device_limit", "false")
//                 .text("remove_app_jump", "false")
//                 .text("enable_file_share", "false")
//                 .text("zip_level", "Middle")
//                 .part("p12_file", reqwest::multipart::Part::bytes(std::fs::read("/Users/lake/Library/Mobile Documents/com~apple~CloudDocs/证书/lake_13_pm.p12").unwrap()).file_name("lake_13_pm.p12"))
//                 .part("mp_file", reqwest::multipart::Part::bytes(std::fs::read("/Users/lake/Library/Mobile Documents/com~apple~CloudDocs/证书/lake_13_pm.mobileprovision").unwrap()).file_name("lake_13_pm.mobileprovision"))
//                 .text("p12_password", "1");
//
//             let request = client.request(reqwest::Method::POST, format!("http://192.168.0.85:3001/api/ipa/sign/ZxJrE/{}", i))
//                 .multipart(form);
//
//             let response = request.send().await.unwrap();
//             let body = response.text().await.unwrap();
//
//             println!("{}", body);
//         });
//         headers.push(handle);
//     }
//     for handle in headers {
//         handle.await?;
//     }
//     Ok(())
// }


#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let time_offset =
        UtcOffset::current_local_offset().unwrap_or_else(|_| UtcOffset::from_hms(8, 0, 0).unwrap());
    let local_time = OffsetTime::new(
        time_offset,
        time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
        ),
    );

    let format = tracing_subscriber::fmt::format()
        .with_timer(local_time.clone())
        .with_level(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_target(true);

    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .event_format(format);

    // let file_appender = tracing_appender::rolling::Builder::new()
    //     .filename_prefix("")
    //     .filename_suffix("log")
    //     .rotation(Rotation::DAILY)
    //     .build("logs").unwrap();
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let builder = AppenderBuilder::default()
        .rotation(Rotation::Daily)
        .prefix(None)
        .suffix(Some("log"))
        .clone();
    let tracing_file_appender = file_appender::TracingFileAppender::from_builder(
        builder,
        "logs",
    )?;
    let (my_non_blocking, _guard) = tracing_appender::non_blocking(tracing_file_appender);

    sub
        // .with_writer(non_blocking) //正式环境使用
        .with_writer(my_non_blocking)
        .with_timer(local_time)
        .with_ansi(false)
        .init();

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
    info!("hello");
    Migrator::up(&conn, None).await?;

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
        // .workers(1)
        .bind("0.0.0.0:4000")?
        .run().await?;

    Ok(())
}
