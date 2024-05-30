use std::future::Future;
use std::os::unix::raw::mode_t;
use std::pin::Pin;
use std::task::{Poll, ready};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

enum Event {
    Zip {
        file_name: String,
        responder: tokio::sync::oneshot::Sender<String>,
    },
}

struct Crc {
    amt: u32,
    hasher: crc32fast::Hasher,
}

struct CrcReader<R> {
    inner: R,
    crc: Crc,
}

impl<R: AsyncRead> CrcReader<R> {
    fn new(r: R) -> CrcReader<R> {
        Self {
            inner: r,
            crc: Crc::new(),
        }
    }
}

impl<R: AsyncRead + std::marker::Unpin> AsyncRead for CrcReader<R> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let mut innser_pin = Pin::new(&mut this.inner);
        let res = innser_pin.as_mut().poll_read(cx, buf)?;
        if res.is_pending() {
            return Poll::Pending;
        }
        let bytes = buf.filled();
        this.crc.update(bytes);
        Poll::Ready(Ok(()))
    }
}

impl Crc {
    pub fn new() -> Self {
        Self {
            amt: 0,
            hasher: crc32fast::Hasher::new(),
        }
    }

    pub fn amount(&self) -> u32 {
        self.amt
    }

    pub fn sum(&self) -> u32 {
        self.hasher.clone().finalize()
    }

    pub fn update(&mut self, data: &[u8]) {
        self.amt = self.amt.wrapping_add(data.len() as u32);
        self.hasher.update(data);
    }
}

#[tokio::main]
async fn main() {
    read_data(tokio::fs::File::open("Cargo.toml").await.unwrap()).await;
    if true {
        return;
    }
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(2);
    let my_num_cpus = num_cpus::get();
    for i in 0..2 {
        let tx = tx.clone();
        tokio::spawn(async move {
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel::<String>();
            let event = Event::Zip {
                file_name: format!("file name {}", i),
                responder: resp_tx,
            };
            tx.send(event).await.unwrap();
            if let Some(msg) = resp_rx.await.ok() {
                println!("response: {}", msg);
            }
        });
    }
    drop(tx);
    let mut list = Vec::new();
    while let Some(event) = rx.recv().await {
        match event {
            Event::Zip {
                file_name,
                responder,
            } => {
                println!("file_name: {}", file_name);
                responder.send("done".to_string()).unwrap();
                list.push(file_name);
            }
        };
    }
    println!("{:?}", list);
}

async fn read_data<R: AsyncRead>(source: R) {
    let file = tokio::fs::File::open("Cargo.toml").await.unwrap();
    let mut crc_read = CrcReader::new(file);
    let mut buf = vec![0u8; 1024];
    loop {
        let n = crc_read.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        let content = std::str::from_utf8(&buf[..n]).unwrap();
        println!("content ----> {}", content);
    }
    println!("crc amount: {}", crc_read.crc.sum());


    //定义一个可以返回future的函数，参数是一个异步闭包
    let res = future_in_future(|str1: &str, str2: &mut str| async move{
        hi().await
    }).await;
}

async fn hi() -> String {
    return "".to_string();
}

async fn future_in_future<F, R, T>(closure: F) -> T
    where
        F: FnOnce(&str, &mut str) -> R, // 闭包参数是一个 String，返回一个 Future
        R: Future<Output=T>, // Future 的输出类型是 i32
{
    // let result = closure("Hello".to_string()).await; // 调用传入的闭包并等待其返回的 Future
    // result + 42 // 在这个示例中，我们假设 Future 的输出是 i32 类型，并对其进行一些处理
    let mut str2 = "abc".to_string();
    closure("Hello", &mut str2).await
}

// async fn future_in_future<F, T>(f: F) -> T
//     where
//         F: FnOnce(String) -> T,
// {
//     f("hello".to_string())
// }