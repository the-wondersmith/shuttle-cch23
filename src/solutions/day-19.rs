//! ### CCH 2023 Day 19 Solutions
//!

// Standard Library Imports
use core::fmt::Debug;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

// Third-Party Imports
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRef, Json, Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};

// Crate-Level Imports
use crate::state::ShuttleAppState;

// <editor-fold desc="// SocketPongSession ...">

/// A socket-bound ping pong game
#[derive(Debug, Default)]
pub struct SocketPongSession(bool);

impl SocketPongSession {
    fn new() -> Self {
        Self::default()
    }

    #[tracing::instrument(skip_all, fields(socket))]
    async fn play(mut self, mut socket: WebSocket) {
        tracing::Span::current().record("socket", format!("{:p}", &socket));

        while let Some(Ok(message)) = socket.recv().await {
            match (self.0, message.to_text()) {
                (false, Ok("serve")) => {
                    tracing::info!(r#""serve" received"#);
                    self.0 = true;
                }
                (true, Ok("ping")) => {
                    if let Err(error) = socket.send("pong".into()).await {
                        tracing::error!("{error:?}");
                        break;
                    }
                }
                (false, Ok("ping")) => {
                    tracing::warn!(r#"game not yet started, ignoring "ping" message"#);
                }
                (_, Ok(text)) => {
                    tracing::warn!(
                        "ignoring {} message: {text:?}",
                        if text.is_empty() {
                            "empty"
                        } else {
                            "unrecognized"
                        }
                    );
                }
                (_, Err(_)) => {
                    tracing::warn!("ignoring undecodable message: {:?}", &message,);
                }
            }
        }
    }
}

// </editor-fold desc="// SocketPongSession ...">

// <editor-fold desc="// ChatMessage ...">

/// A message from a specific user
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    /// the connected user's name
    #[serde(default)]
    user: String,
    /// the user's inner most thoughts
    message: String,
}

// </editor-fold desc="// ChatMessage ...">

// <editor-fold desc="// WsComPair ...">

#[derive(Clone, Debug)]
struct WsComPair {
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    receiver: Arc<Mutex<SplitStream<WebSocket>>>,
}

impl WsComPair {
    fn new(socket: WebSocket) -> Self {
        let (sender, receiver) = socket.split();

        Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

// </editor-fold desc="// WsComPair ...">

// <editor-fold desc="// ChatRoomConnection ...">

/// A message from a specific user
#[derive(Clone, Debug)]
pub struct ChatRoomConnection {
    /// ...
    socket: WsComPair,
    /// ...
    incoming: Arc<Mutex<broadcast::Receiver<ChatMessage>>>,
}

impl ChatRoomConnection {
    #[allow(unused_variables)]
    #[tracing::instrument(skip(socket, broadcaster))]
    fn new<Username: Debug + AsRef<str>>(
        room: u64,
        user: Username,
        socket: WebSocket,
        broadcaster: Arc<broadcast::Sender<ChatMessage>>,
    ) -> Self {
        let incoming = broadcaster.subscribe();

        tracing::debug!("new connection");

        Self {
            socket: WsComPair::new(socket),
            incoming: Arc::new(Mutex::new(incoming)),
        }
    }
}

// </editor-fold desc="// ChatRoomConnection ...">

// <editor-fold desc="// ChatRoomState ...">

#[derive(Clone, Debug, FromRef)]
pub struct ChatRoomState {
    // running total of "seen" messages
    views: Arc<AtomicU64>,
    // Channel-per-room map for all connected clients
    rooms: Arc<Mutex<BTreeMap<u64, Arc<broadcast::Sender<ChatMessage>>>>>,
}

impl Default for ChatRoomState {
    fn default() -> Self {
        let rooms = BTreeMap::<u64, Arc<broadcast::Sender<ChatMessage>>>::new();

        Self {
            rooms: Arc::new(Mutex::new(rooms)),
            views: Arc::new(AtomicU64::new(0u64)),
        }
    }
}

impl ChatRoomState {
    async fn room_channel(&self, room: u64) -> Arc<broadcast::Sender<ChatMessage>> {
        self.rooms
            .lock()
            .await
            .entry(room)
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel::<ChatMessage>(100);
                Arc::new(sender)
            })
            .clone()
    }

    #[allow(unused_parens)]
    #[tracing::instrument(skip(state, socket))]
    async fn connect_and_chat(state: Arc<Self>, socket: WebSocket, room: u64, user: String) {
        let broadcaster = state.room_channel(room).await;
        let chat = ChatRoomConnection::new(room, &user, socket, broadcaster.clone());

        // Spawn the first task that will receive broadcast messages
        // and send chat messages over the websocket to our client.
        let mut send_task = tokio::spawn(async move {
            while let Ok(message) = chat.incoming.lock().await.recv().await {
                if message.message.is_empty() {
                    tracing::warn!("declining to propagate empty message");
                } else if 128 < message.message.len() {
                    tracing::warn!(
                        r#"declining to propagate {} character message: "{} ...""#,
                        message.message.len(),
                        &message.message[0..=15]
                    );
                } else {
                    let message = match serde_json::to_string(&message) {
                        Ok(encoded) => encoded,
                        Err(error) => {
                            tracing::error!("error serializing message: {error:?}");
                            break;
                        }
                    };

                    if let Err(error) = chat
                        .socket
                        .sender
                        .lock()
                        .await
                        .send(Message::Text(message))
                        .await
                    {
                        tracing::error!("error propagating message to user: {error:?}");
                        break;
                    }

                    state.views.fetch_add(1u64, Ordering::SeqCst);
                }
            }
        });

        // Spawn a task that takes messages from the websocket, ensures they're
        // properly formatted, and broadcasts them to everyone in the chat room.
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(received))) =
                chat.socket.receiver.lock().await.next().await
            {
                match serde_json::from_str::<ChatMessage>(&received) {
                    Err(error) => {
                        tracing::error!("error deserializing message: {error:?}");
                        break;
                    }
                    Ok(mut message) => {
                        message.user = user.clone();

                        if let Err(error) = broadcaster.send(message) {
                            tracing::error!("error propagating message to room: {error:?}");
                            break;
                        }
                    }
                }
            }
        });

        // If any one of the tasks run to completion, we abort the other.
        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        }

        tracing::debug!("disconnection");
    }
}

// </editor-fold desc="// ChatRoomState ...">

/// Complete [Day 19: Task](https://console.shuttle.rs/cch/challenge/19#:~:text=⭐)
#[tracing::instrument(skip_all)]
pub async fn play_socket_ping_pong(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| SocketPongSession::new().play(socket))
}

/// Endpoint 1/3 for [Day 19: Bonus](https://console.shuttle.rs/cch/challenge/19#:~:text=🎁)
#[tracing::instrument(ret, skip_all, fields(zeroed_from))]
pub async fn reset_chat_count(State(state): State<ShuttleAppState>) -> StatusCode {
    tracing::Span::current().record(
        "zeroed_from",
        state.chat.views.as_ref().swap(0u64, Ordering::SeqCst),
    );

    StatusCode::OK
}

/// Endpoint 2/3 for [Day 19: Bonus](https://console.shuttle.rs/cch/challenge/19#:~:text=🎁)
#[tracing::instrument(ret, skip_all)]
pub async fn get_current_chat_count(State(state): State<ShuttleAppState>) -> Json<u64> {
    Json(state.chat.views.load(Ordering::Relaxed))
}

/// Endpoint 3/3 for [Day 19: Bonus](https://console.shuttle.rs/cch/challenge/19#:~:text=🎁)
#[tracing::instrument(skip_all)]
pub async fn connect_to_chat_room(
    Path((room, user)): Path<(u64, String)>,
    State(state): State<ShuttleAppState>,
    socket: WebSocketUpgrade,
) -> impl IntoResponse {
    socket.on_upgrade(move |socket| ChatRoomState::connect_and_chat(state.chat, socket, room, user))
}

#[cfg(test)]
mod tests {
    //! ## I/O-free Unit Tests

    #![allow(unused_imports, clippy::unit_arg)]

    // Standard Library Imports
    use core::{cmp::PartialEq, fmt::Debug, ops::BitOr, str::FromStr};
    use std::collections::HashMap;

    // Third-Party Imports
    use axum::{
        body::{Body, BoxBody, HttpBody},
        http::{
            header as headers,
            request::{Builder, Parts},
            Method, Request, Response, StatusCode,
        },
        routing::Router,
    };
    use once_cell::sync::Lazy;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    use rstest::{fixture, rstest};
    use serde_json::{error::Error as SerdeJsonError, Value};
    use shuttle_shared_db::Postgres as ShuttleDB;
    use tower::{MakeService, ServiceExt};

    // Crate-Level Imports
    use crate::utils::{service, TestService};
}
