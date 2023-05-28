use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time;
use tokio_pty::AsyncPty;
use tracing::instrument;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[derive(Deserialize, Serialize, Debug)]
struct WinSize {
    cols: u16,
    rows: u16,
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    //    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set subscriber");

    let static_assets = warp::fs::dir("./static");
    //    let terminal = warp::post().and(warp::path("terminals")).map(|| "110");
    let resize = warp::post()
        .and(warp::path!("terminals"))
        .and(warp::query::<WinSize>())
        .map(|size| {
            info!("resize windows {:?}", size);
            "ok"
        });
    let ws =
        warp::path!("terminals" / String)
            .and(warp::ws())
            .map(|_pid: String, ws: warp::ws::Ws| {
                ws.on_upgrade(move |socket| client_connected(socket))
            });
    let route = static_assets.or(resize).or(ws);

    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}

#[instrument]
async fn client_connected(ws: WebSocket) {
    info!("client connected");
    //    let mut buf = BytesMut::with_capacity(4096);
    let mut buf = vec![0u8; 4096];
    let (mut ws_tx, mut ws_rx) = ws.split();
    let pty = AsyncPty::open().unwrap();
    pty.resize(120, 32);
    let (mut pt_rx, mut pt_tx) = tokio::io::split(pty);
    let mut ws_keep_alive = time::interval(time::Duration::from_secs(15));

    loop {
        tokio::select! {
            Ok(size) = pt_rx.read(&mut buf) => {
                ws_tx.send(Message::binary(&buf[0..size])).await;
            }
            _ = ws_keep_alive.tick() => {
                ws_tx.send(Message::binary([])).await;
            }
            Some(result) = ws_rx.next() => {
                let msg = result.unwrap();
                pt_tx.write(msg.as_bytes()).await;
            }
        }
    }
}
