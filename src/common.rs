use std::fmt::Display;
use std::future::Future;
use std::time::Duration;
use tracing::{error, info};

pub async fn async_retry<F, Fut, FutT, FutE: Display>(f: F, max_retry: usize, delay: Duration) -> Result<FutT, FutE>
    where F: Fn() -> Fut,
          Fut: Future<Output=Result<FutT, FutE>>
{
    let mut retry = 0;
    loop {
        match f().await {
            ok @ Ok(_) => {
                if retry > 0 {
                    info!("执行成功,重试{retry}次");
                }
                return ok;
            }
            Err(e) if retry < max_retry => {
                error!("错误:{e},重试第{}次",retry+1);
                tokio::time::sleep(delay).await;
                retry += 1;
                continue;
            }
            e @ Err(_) => return e,
        }
    }
}
