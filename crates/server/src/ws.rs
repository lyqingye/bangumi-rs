use crate::{error::ServerError, logger::LogMessage};
use actix_web::{web, HttpRequest, Responder};
use actix_ws::Message;
use anyhow::Result;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::info;

pub async fn ws_handler(
    log_tx: actix_web::web::Data<broadcast::Sender<LogMessage>>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<impl Responder, ServerError> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    info!("WebSocket connected");

    // 启动WebSocket处理任务
    actix_web::rt::spawn(async move {
        let mut rx = log_tx.subscribe();
        loop {
            tokio::select! {
                Some(msg) = msg_stream.next() => {
                    match msg {
                        Ok(Message::Ping(bytes)) => {
                            if session.pong(&bytes).await.is_err() {
                                return;
                            }
                        }
                        Ok(Message::Text(msg)) => println!("Got text: {msg}"),
                        _ => break,
                    }
                }
                Ok(log_msg) = rx.recv() => {
                    if session.text(log_msg.content).await.is_err() {
                        return;
                    }
                }
                else => break,
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}
