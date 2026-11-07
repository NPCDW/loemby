use std::{collections::HashMap, sync::{Arc, atomic::{AtomicU32, Ordering}}, time::Duration};

use anyhow::{Context, Result};
use serde_json::{Value, json};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf}, sync::{Mutex, RwLock, mpsc, oneshot}};
use interprocess::local_socket::tokio::Stream;

#[cfg(unix)]
pub type Sender = Arc<RwLock<WriteHalf<Stream>>>;
#[cfg(windows)]
pub type Sender = Arc<RwLock<interprocess::os::windows::named_pipe::tokio::SendPipeStream<interprocess::os::windows::named_pipe::pipe_mode::Bytes>>>;
type Pending = Arc<Mutex<HashMap<u32, oneshot::Sender<Result<Value>>>>>;

#[derive(Clone)]
pub struct Client {
    pub sender: Sender,
    pending: Pending,
    next_id: Arc<AtomicU32>,
}

pub async fn connect(path: &str) -> Result<(Client, mpsc::UnboundedReceiver<Value>)> {
    let (reader, writer) = super::player_svc::get_pipe_rw(path).await?;
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let client = Client { sender: Arc::new(RwLock::new(writer)), pending: pending.clone(), next_id: Arc::new(AtomicU32::new(1_000_000)) };
    let (events, receiver) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        let result: Result<()> = async {
            loop {
                let mut line = String::new();
                let read = tokio::select! {
                    read = reader.read_line(&mut line) => read?,
                    _ = events.closed() => break,
                };
                if read == 0 { break; }
                let message: Value = serde_json::from_str(&line)?;
                if let Some(id) = message.get("request_id").and_then(Value::as_u64) {
                    if let Some(reply) = pending.lock().await.remove(&(id as u32)) {
                        let response = if message.get("error").and_then(Value::as_str) == Some("success") {
                            Ok(message.get("data").cloned().unwrap_or(Value::Null))
                        } else {
                            Err(anyhow::anyhow!("mpv command failed: {}", message))
                        };
                        let _ = reply.send(response);
                        continue;
                    }
                }
                if events.send(message).is_err() { break; }
            }
            Ok(())
        }.await;
        let detail = result.err().map(|error| error.to_string());
        for (_, reply) in pending.lock().await.drain() {
            let _ = reply.send(Err(anyhow::anyhow!("mpv IPC 已断开")));
        }
        let _ = events.send(json!({"event":"ipc-disconnected", "detail":detail}));
    });
    Ok((client, receiver))
}

impl Client {
    pub async fn request(&self, command: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (reply, receiver) = oneshot::channel();
        self.pending.lock().await.insert(id, reply);
        let result = tokio::time::timeout(Duration::from_secs(10), async {
            let bytes = encode(command, id)?;
            let mut writer = self.sender.write().await;
            writer.write_all(&bytes).await?;
            writer.flush().await?;
            drop(writer);
            receiver.await.context("mpv 命令响应通道关闭")?
        }).await.context("mpv 命令发送或响应超时").and_then(|result| result);
        self.pending.lock().await.remove(&id);
        result
    }
}

fn encode(command: Value, id: u32) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(&json!({"command":command, "request_id":id}))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn requests_and_events_are_dispatched_independently() {
        let path = std::env::temp_dir().join(format!("loemby-ipc-test-{}.sock", uuid::Uuid::new_v4().simple()));
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            writer.write_all(b"{\"event\":\"file-loaded\"}\n").await.unwrap();
            let response = json!({"request_id":request["request_id"], "error":"success", "data":12.5});
            writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
        });
        let (client, mut events) = connect(path.to_str().unwrap()).await.unwrap();
        assert_eq!(client.request(json!(["get_property", "duration"])).await.unwrap(), json!(12.5));
        assert_eq!(events.recv().await.unwrap()["event"], "file-loaded");
        assert_eq!(events.recv().await.unwrap()["event"], "ipc-disconnected");
        server.await.unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn disconnect_fails_pending_request_without_waiting_for_timeout() {
        let path = std::env::temp_dir().join(format!("loemby-ipc-test-{}.sock", uuid::Uuid::new_v4().simple()));
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
        });
        let (client, _) = connect(path.to_str().unwrap()).await.unwrap();
        let response = tokio::time::timeout(Duration::from_secs(2), client.request(json!(["get_property", "duration"]))).await.unwrap();
        assert!(response.is_err());
        server.await.unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn json_commands_escape_metadata_and_end_in_one_newline() {
        let title = "title \"quoted\"\\path\nsecond line";
        let bytes = encode(json!(["set_property", "force-media-title", title]), 123).unwrap();
        assert_eq!(bytes.iter().filter(|byte| **byte == b'\n').count(), 1);
        assert_eq!(bytes.last(), Some(&b'\n'));
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["command"][2], title);
        assert_eq!(value["request_id"], 123);
    }
}
