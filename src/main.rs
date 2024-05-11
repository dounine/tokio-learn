use std::task::Poll;
use tokio::io::{AsyncRead, AsyncReadExt};

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
        // self.inner.poll_read(cx, buf)

        Poll::Pending
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
    let n = crc_read.read(&mut buf).await.unwrap();
}
