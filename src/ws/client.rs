use crate::auth::KalshiAuth;
use crate::env::{KalshiEnvironment, WS_PATH};
use crate::error::KalshiError;
use crate::ws::types::{
    WsEnvelope, WsListSubscriptionsCmd, WsMessage, WsRawEvent, WsSubscribeCmd,
    WsSubscriptionParams, WsUnsubscribeCmd, WsUnsubscribeParams, WsUpdateSubscriptionCmd,
    WsUpdateSubscriptionParams, validate_subscription,
};

use futures::{SinkExt, StreamExt};

use bytes::Bytes;
use rand::random;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderValue, Request};

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Configuration for automatic WebSocket reconnection in [`KalshiWsClient`].
///
/// Uses exponential backoff with jitter. The delay for attempt *n* is
/// `min(base_delay * 2^(n-1), max_delay)` ± `jitter`.
///
/// # Default
///
/// | Field | Value |
/// |-------|-------|
/// | `max_retries` | `None` (unlimited) |
/// | `base_delay` | 250 ms |
/// | `max_delay` | 30 s |
/// | `jitter` | 0.2 |
/// | `resubscribe` | `true` |
#[derive(Debug, Clone)]
pub struct WsReconnectConfig {
    /// Maximum reconnection attempts. `None` means unlimited.
    pub max_retries: Option<u32>,
    /// Initial backoff delay (doubles each attempt).
    pub base_delay: Duration,
    /// Upper bound on backoff delay.
    pub max_delay: Duration,
    /// Random jitter factor applied to each delay (0.0–1.0).
    pub jitter: f64,
    /// Whether to resubscribe to active channels after reconnecting.
    pub resubscribe: bool,
}

impl Default for WsReconnectConfig {
    fn default() -> Self {
        Self {
            max_retries: None,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(30),
            jitter: 0.2,
            resubscribe: true,
        }
    }
}

impl WsReconnectConfig {
    fn backoff_delay(&self, attempt: u32) -> Duration {
        let exp = 2f64.powi(attempt.saturating_sub(1) as i32);
        let mut delay = self.base_delay.mul_f64(exp);
        if delay > self.max_delay {
            delay = self.max_delay;
        }
        let jitter = self.jitter.clamp(0.0, 1.0);
        if jitter > 0.0 {
            let factor = 1.0 - jitter + random::<f64>() * (2.0 * jitter);
            delay = delay.mul_f64(factor);
        }
        delay
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WsReaderMode {
    Owned,
    Raw,
}

#[derive(Debug, Clone)]
pub struct WsReaderConfig {
    pub buffer_size: usize,
    pub mode: WsReaderMode,
}

impl Default for WsReaderConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            mode: WsReaderMode::Owned,
        }
    }
}

/// Events emitted by [`KalshiWsClient::next_event`].
///
/// The high-level client wraps every raw WebSocket message as well as
/// connection lifecycle transitions into this enum.
#[derive(Debug)]
pub enum WsEvent {
    /// A parsed WebSocket message (data, ack, error, etc.).
    Message(WsMessage),
    Raw(WsRawEvent),
    /// Connection was lost and successfully re-established.
    ///
    /// `attempt` is the 1-based retry count that succeeded.
    /// If [`WsReconnectConfig::resubscribe`] is `true`, all previously
    /// active channels have already been resubscribed.
    Reconnected {
        attempt: u32,
    },
    /// Connection was lost and could not be restored within
    /// [`WsReconnectConfig::max_retries`].
    Disconnected {
        error: KalshiError,
    },
}

#[derive(Debug, Clone)]
pub struct WsEventReceiver {
    inner: Arc<Mutex<mpsc::Receiver<WsEvent>>>,
}

impl WsEventReceiver {
    fn new(rx: mpsc::Receiver<WsEvent>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn next(&self) -> Option<WsEvent> {
        let mut rx = self.inner.lock().await;
        rx.recv().await
    }
}

#[derive(Default)]
struct SubscriptionTracker {
    pending: HashMap<u64, WsSubscriptionParams>,
    active: HashMap<u64, WsSubscriptionParams>,
}

impl SubscriptionTracker {
    fn record_subscribe_cmd(&mut self, id: u64, params: WsSubscriptionParams) {
        self.pending.insert(id, params);
    }

    fn handle_message(&mut self, msg: &WsMessage) {
        match msg {
            WsMessage::Subscribed {
                id: Some(id),
                sid: Some(sid),
            } => {
                self.handle_subscribed(Some(*id), Some(*sid));
            }
            WsMessage::Unsubscribed { sid: Some(sid), .. } => {
                self.handle_unsubscribed(Some(*sid));
            }
            _ => {}
        }
    }

    fn handle_subscribed(&mut self, id: Option<u64>, sid: Option<u64>) {
        let (id, sid) = match (id, sid) {
            (Some(id), Some(sid)) => (id, sid),
            _ => return,
        };
        if let Some(params) = self.pending.remove(&id) {
            self.active.insert(sid, params);
        }
    }

    fn handle_unsubscribed(&mut self, sid: Option<u64>) {
        if let Some(sid) = sid {
            self.active.remove(&sid);
        }
    }

    fn drop_active(&mut self, sid: u64) {
        self.active.remove(&sid);
    }

    fn apply_update(&mut self, update: &WsUpdateSubscriptionParams) {
        if let Some(params) = self.active.get_mut(&update.sid) {
            if let Some(value) = update.market_tickers.clone() {
                params.market_tickers = Some(value);
            }
            if let Some(value) = update.market_ids.clone() {
                params.market_ids = Some(value);
            }
            if let Some(value) = update.event_tickers.clone() {
                params.event_tickers = Some(value);
            }
            if let Some(value) = update.send_initial_snapshot {
                params.send_initial_snapshot = Some(value);
            }
            if let Some(value) = update.shard_factor {
                params.shard_factor = Some(value);
            }
            if let Some(value) = update.shard_key.clone() {
                params.shard_key = Some(value);
            }
        }
    }

    fn prepare_resubscribe(&mut self) -> Vec<WsSubscriptionParams> {
        let mut params: Vec<WsSubscriptionParams> = self.active.values().cloned().collect();
        params.extend(self.pending.values().cloned());
        self.active.clear();
        self.pending.clear();
        params
    }
}

/// Low-level WebSocket client with split read/write streams.
///
/// Provides direct access to the Kalshi WebSocket protocol: subscribe,
/// unsubscribe, update, and read raw envelopes or parsed messages.
/// No automatic reconnection or subscription tracking — use
/// [`KalshiWsClient`] if you need those.
///
/// Read and write halves are split at construction time so sending a
/// command never blocks receiving data.
pub struct KalshiWsLowLevelClient {
    write: futures::stream::SplitSink<WsStream, Message>,
    read: futures::stream::SplitStream<WsStream>,
    next_id: u64,
    authenticated: bool,
}

impl KalshiWsLowLevelClient {
    // -----------------------------------------------
    // Connection
    // -----------------------------------------------

    /// Connect without auth (public channels only).
    pub async fn connect(env: KalshiEnvironment) -> Result<Self, KalshiError> {
        let (ws_stream, _resp) = connect_async(&env.ws_url)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            next_id: 1,
            authenticated: false,
        })
    }

    /// Connect with auth headers so you can subscribe to private channels.
    ///
    /// **Requires auth.**
    pub async fn connect_authenticated(
        env: KalshiEnvironment,
        auth: KalshiAuth,
    ) -> Result<Self, KalshiError> {
        let mut req: Request<()> = env
            .ws_url
            .into_client_request()
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        // WS signing: timestamp + "GET" + "/trade-api/ws/v2"
        let headers = auth.build_headers("GET", WS_PATH)?;

        req.headers_mut().insert(
            "KALSHI-ACCESS-KEY",
            HeaderValue::from_str(&headers.key).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-SIGNATURE",
            HeaderValue::from_str(&headers.signature)
                .map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&headers.timestamp_ms)
                .map_err(|e| KalshiError::Header(e.to_string()))?,
        );

        let (ws_stream, _resp) = connect_async(req)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            next_id: 1,
            authenticated: true,
        })
    }

    pub async fn send_raw(&mut self, msg: Message) -> Result<(), KalshiError> {
        self.write
            .send(msg)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;
        self.write
            .flush()
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;
        Ok(())
    }

    pub async fn next_frame(&mut self) -> Result<Message, KalshiError> {
        match self.read.next().await {
            Some(Ok(msg)) => Ok(msg),
            Some(Err(e)) => Err(KalshiError::Ws(e.to_string())),
            None => Err(KalshiError::Ws("websocket stream ended".to_string())),
        }
    }

    pub async fn next_json_bytes(&mut self) -> Result<Bytes, KalshiError> {
        loop {
            let msg = self.next_frame().await?;
            match msg {
                Message::Text(s) => return Ok(Bytes::from(s)),
                Message::Binary(b) => return Ok(Bytes::from(b)),
                Message::Ping(payload) => {
                    self.send_raw(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => {
                    return Err(KalshiError::Ws("websocket closed".to_string()));
                }
                _ => {}
            }
        }
    }

    // -----------------------------------------------
    // Commands
    // -----------------------------------------------

    /// Subscribe to one or more channels. Returns the command `id`.
    ///
    /// Private channels (e.g. [`WsChannel::Fill`](crate::WsChannel::Fill))
    /// require an authenticated connection — see [`connect_authenticated`](Self::connect_authenticated).
    pub async fn subscribe(&mut self, params: WsSubscriptionParams) -> Result<u64, KalshiError> {
        let needs_auth = params.channels.iter().any(|c| c.is_private());
        if needs_auth && !self.authenticated {
            return Err(KalshiError::AuthRequired(
                "WebSocket private channel subscription",
            ));
        }

        validate_subscription(&params)?;

        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsSubscribeCmd {
            id,
            cmd: "subscribe",
            params,
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Unsubscribe from a subscription by its `sid`. Returns the command `id`.
    pub async fn unsubscribe(&mut self, sid: u64) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsUnsubscribeCmd {
            id,
            cmd: "unsubscribe",
            params: WsUnsubscribeParams { sid },
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Update an existing subscription (e.g. change market tickers). Returns the command `id`.
    pub async fn update_subscription(
        &mut self,
        params: WsUpdateSubscriptionParams,
    ) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsUpdateSubscriptionCmd {
            id,
            cmd: "update_subscription",
            params,
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Request a list of active subscriptions from the server. Returns the command `id`.
    pub async fn list_subscriptions(&mut self) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsListSubscriptionsCmd {
            id,
            cmd: "list_subscriptions",
        };
        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    // -----------------------------------------------
    // Reading
    // -----------------------------------------------

    /// Read the next JSON envelope from the stream.
    ///
    /// Transparently handles Ping/Pong frames. Returns an error on
    /// Close frames or stream termination.
    pub async fn next_envelope(&mut self) -> Result<WsEnvelope, KalshiError> {
        let bytes = self.next_json_bytes().await?;
        Ok(serde_json::from_slice::<WsEnvelope>(&bytes)?)
    }

    /// Read the next message and parse it into a typed [`WsMessage`].
    ///
    /// Equivalent to calling [`next_envelope`](Self::next_envelope) followed
    /// by [`WsEnvelope::into_message`].
    pub async fn next_message(&mut self) -> Result<WsMessage, KalshiError> {
        let bytes = self.next_json_bytes().await?;
        WsMessage::from_bytes(&bytes)
    }
}

/// High-level WebSocket client with automatic reconnection and resubscription.
///
/// Wraps [`KalshiWsLowLevelClient`] and adds:
/// - Exponential-backoff reconnection (configurable via [`WsReconnectConfig`])
/// - Subscription tracking — active channels are resubscribed after reconnect
/// - A unified event loop via [`next_event`](Self::next_event)
///
/// # Example
///
/// ```no_run
/// use kalshi_fast::{
///     KalshiEnvironment, KalshiWsClient, WsChannel,
///     WsDataMessage, WsEvent, WsMessage, WsReconnectConfig, WsSubscriptionParams,
/// };
///
/// # async fn run() -> Result<(), kalshi_fast::KalshiError> {
/// let mut ws = KalshiWsClient::connect(
///     KalshiEnvironment::demo(),
///     WsReconnectConfig::default(),
/// ).await?;
///
/// ws.subscribe(WsSubscriptionParams {
///     channels: vec![WsChannel::Trade],
///     ..Default::default()
/// }).await?;
///
/// loop {
///     match ws.next_event().await? {
///         WsEvent::Message(WsMessage::Data(WsDataMessage::Trade { msg, .. })) => {
///             println!("trade: {} @ {}", msg.ticker, msg.price.unwrap_or(0));
///         }
///         WsEvent::Disconnected { .. } => break,
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct KalshiWsClient {
    env: KalshiEnvironment,
    auth: Option<KalshiAuth>,
    client: Option<KalshiWsLowLevelClient>,
    config: WsReconnectConfig,
    tracker: Arc<Mutex<SubscriptionTracker>>,
    reader: Option<WsEventReceiver>,
    outgoing: Option<mpsc::Sender<Message>>,
    reader_task: Option<JoinHandle<()>>,
    next_id: u64,
}

impl KalshiWsClient {
    // -----------------------------------------------
    // Connection
    // -----------------------------------------------

    /// Connect without auth (public channels only).
    pub async fn connect(
        env: KalshiEnvironment,
        config: WsReconnectConfig,
    ) -> Result<Self, KalshiError> {
        let client = KalshiWsLowLevelClient::connect(env.clone()).await?;
        Ok(Self {
            env,
            auth: None,
            client: Some(client),
            config,
            tracker: Arc::new(Mutex::new(SubscriptionTracker::default())),
            reader: None,
            outgoing: None,
            reader_task: None,
            next_id: 1,
        })
    }

    /// Connect with auth headers for private channels.
    ///
    /// **Requires auth.**
    pub async fn connect_authenticated(
        env: KalshiEnvironment,
        auth: KalshiAuth,
        config: WsReconnectConfig,
    ) -> Result<Self, KalshiError> {
        let client =
            KalshiWsLowLevelClient::connect_authenticated(env.clone(), auth.clone()).await?;
        Ok(Self {
            env,
            auth: Some(auth),
            client: Some(client),
            config,
            tracker: Arc::new(Mutex::new(SubscriptionTracker::default())),
            reader: None,
            outgoing: None,
            reader_task: None,
            next_id: 1,
        })
    }

    async fn send_command(&mut self, msg: Message) -> Result<(), KalshiError> {
        if let Some(sender) = &self.outgoing {
            sender
                .send(msg)
                .await
                .map_err(|_| KalshiError::Ws("websocket writer closed".to_string()))?;
            return Ok(());
        }
        if let Some(client) = &mut self.client {
            return client.send_raw(msg).await;
        }
        Err(KalshiError::Ws(
            "websocket client not connected".to_string(),
        ))
    }

    // -----------------------------------------------
    // Commands
    // -----------------------------------------------

    /// Subscribe to one or more channels. Returns the command `id`.
    ///
    /// The subscription is tracked internally so it can be resubscribed
    /// automatically after a reconnect.
    pub async fn subscribe(&mut self, params: WsSubscriptionParams) -> Result<u64, KalshiError> {
        let needs_auth = params.channels.iter().any(|c| c.is_private());
        if needs_auth && self.auth.is_none() {
            return Err(KalshiError::AuthRequired(
                "WebSocket private channel subscription",
            ));
        }

        validate_subscription(&params)?;

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        {
            let mut tracker = self.tracker.lock().await;
            tracker.record_subscribe_cmd(id, params.clone());
        }

        let cmd = WsSubscribeCmd {
            id,
            cmd: "subscribe",
            params,
        };

        let text = serde_json::to_string(&cmd)?;
        self.send_command(Message::Text(text)).await?;
        Ok(id)
    }

    /// Unsubscribe from a subscription by its `sid`. Returns the command `id`.
    pub async fn unsubscribe(&mut self, sid: u64) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        {
            let mut tracker = self.tracker.lock().await;
            tracker.drop_active(sid);
        }

        let cmd = WsUnsubscribeCmd {
            id,
            cmd: "unsubscribe",
            params: WsUnsubscribeParams { sid },
        };

        let text = serde_json::to_string(&cmd)?;
        self.send_command(Message::Text(text)).await?;
        Ok(id)
    }

    /// Update an existing subscription (e.g. change market tickers). Returns the command `id`.
    pub async fn update_subscription(
        &mut self,
        params: WsUpdateSubscriptionParams,
    ) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        {
            let mut tracker = self.tracker.lock().await;
            tracker.apply_update(&params);
        }

        let cmd = WsUpdateSubscriptionCmd {
            id,
            cmd: "update_subscription",
            params,
        };
        let text = serde_json::to_string(&cmd)?;
        self.send_command(Message::Text(text)).await?;
        Ok(id)
    }

    /// Request a list of active subscriptions from the server. Returns the command `id`.
    pub async fn list_subscriptions(&mut self) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let cmd = WsListSubscriptionsCmd {
            id,
            cmd: "list_subscriptions",
        };
        let text = serde_json::to_string(&cmd)?;
        self.send_command(Message::Text(text)).await?;
        Ok(id)
    }

    pub async fn start_reader(
        &mut self,
        config: WsReaderConfig,
    ) -> Result<WsEventReceiver, KalshiError> {
        if self.reader.is_some() {
            return Err(KalshiError::InvalidParams(
                "websocket reader already started".to_string(),
            ));
        }
        if config.buffer_size == 0 {
            return Err(KalshiError::InvalidParams(
                "websocket reader buffer_size must be > 0".to_string(),
            ));
        }

        let client = self
            .client
            .take()
            .ok_or_else(|| KalshiError::Ws("websocket client not connected".to_string()))?;

        let (event_tx, event_rx) = mpsc::channel(config.buffer_size);
        let (outgoing_tx, outgoing_rx) = mpsc::channel(config.buffer_size);

        let tracker = self.tracker.clone();
        let env = self.env.clone();
        let auth = self.auth.clone();
        let reconnect_cfg = self.config.clone();
        let mode = config.mode;

        let task = tokio::spawn(async move {
            reader_loop(
                client,
                env,
                auth,
                reconnect_cfg,
                tracker,
                event_tx,
                outgoing_rx,
                mode,
            )
            .await;
        });

        let receiver = WsEventReceiver::new(event_rx);
        self.reader = Some(receiver.clone());
        self.outgoing = Some(outgoing_tx);
        self.reader_task = Some(task);

        Ok(receiver)
    }

    // -----------------------------------------------
    // Event loop
    // -----------------------------------------------

    /// Wait for the next event (message, reconnect, or disconnect).
    ///
    /// This is the primary event-loop driver. On connection loss it
    /// automatically attempts reconnection per [`WsReconnectConfig`],
    /// returning [`WsEvent::Reconnected`] on success or
    /// [`WsEvent::Disconnected`] when retries are exhausted.
    pub async fn next_event(&mut self) -> Result<WsEvent, KalshiError> {
        if let Some(reader) = &self.reader {
            return reader
                .next()
                .await
                .ok_or_else(|| KalshiError::Ws("websocket reader closed".to_string()));
        }

        let client = self
            .client
            .as_mut()
            .ok_or_else(|| KalshiError::Ws("websocket client not connected".to_string()))?;

        match client.next_message().await {
            Ok(msg) => {
                let mut tracker = self.tracker.lock().await;
                tracker.handle_message(&msg);
                Ok(WsEvent::Message(msg))
            }
            Err(err) => self.reconnect_loop(err).await,
        }
    }

    async fn reconnect_loop(&mut self, mut err: KalshiError) -> Result<WsEvent, KalshiError> {
        let mut attempt: u32 = 0;
        loop {
            attempt = attempt.saturating_add(1);
            if let Some(max) = self.config.max_retries {
                if attempt > max {
                    return Ok(WsEvent::Disconnected { error: err });
                }
            }

            let delay = self.config.backoff_delay(attempt);
            if !delay.is_zero() {
                sleep(delay).await;
            }

            match self.reconnect().await {
                Ok(()) => return Ok(WsEvent::Reconnected { attempt }),
                Err(e) => {
                    err = e;
                    continue;
                }
            }
        }
    }

    async fn reconnect(&mut self) -> Result<(), KalshiError> {
        let new_client = match &self.auth {
            Some(auth) => {
                KalshiWsLowLevelClient::connect_authenticated(self.env.clone(), auth.clone())
                    .await?
            }
            None => KalshiWsLowLevelClient::connect(self.env.clone()).await?,
        };
        self.client = Some(new_client);

        if self.config.resubscribe {
            let params = {
                let mut tracker = self.tracker.lock().await;
                tracker.prepare_resubscribe()
            };
            for p in params {
                let id = self.next_id;
                self.next_id = self.next_id.saturating_add(1);

                let cmd = WsSubscribeCmd {
                    id,
                    cmd: "subscribe",
                    params: p.clone(),
                };
                let text = serde_json::to_string(&cmd)?;
                let client = self
                    .client
                    .as_mut()
                    .ok_or_else(|| KalshiError::Ws("websocket client not connected".to_string()))?;
                client.send_raw(Message::Text(text)).await?;
                let mut tracker = self.tracker.lock().await;
                tracker.record_subscribe_cmd(id, p);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsControlMessage {
    #[serde(rename = "subscribed")]
    Subscribed { id: Option<u64>, sid: Option<u64> },
    #[serde(rename = "unsubscribed")]
    Unsubscribed { sid: Option<u64> },
    #[serde(other)]
    Other,
}

async fn reader_loop(
    mut client: KalshiWsLowLevelClient,
    env: KalshiEnvironment,
    auth: Option<KalshiAuth>,
    config: WsReconnectConfig,
    tracker: Arc<Mutex<SubscriptionTracker>>,
    event_tx: mpsc::Sender<WsEvent>,
    mut outgoing_rx: mpsc::Receiver<Message>,
    mode: WsReaderMode,
) {
    let mut outgoing_closed = false;

    loop {
        let result: Result<(), KalshiError> = tokio::select! {
            frame = client.next_frame() => {
                match frame {
                    Ok(msg) => handle_incoming_message(msg, &mut client, &tracker, &event_tx, mode).await,
                    Err(err) => Err(err),
                }
            }
            maybe_out = outgoing_rx.recv(), if !outgoing_closed => {
                match maybe_out {
                    Some(msg) => client.send_raw(msg).await,
                    None => {
                        outgoing_closed = true;
                        Ok(())
                    }
                }
            }
        };

        if let Err(_err) = result {
            match handle_reconnect(&mut client, &env, &auth, &config, &tracker, &event_tx).await {
                Ok(()) => {}
                Err(err) => {
                    let _ = event_tx.send(WsEvent::Disconnected { error: err }).await;
                    return;
                }
            }
        }
    }
}

async fn handle_incoming_message(
    msg: Message,
    client: &mut KalshiWsLowLevelClient,
    tracker: &Arc<Mutex<SubscriptionTracker>>,
    event_tx: &mpsc::Sender<WsEvent>,
    mode: WsReaderMode,
) -> Result<(), KalshiError> {
    match msg {
        Message::Ping(payload) => {
            client.send_raw(Message::Pong(payload)).await?;
            Ok(())
        }
        Message::Pong(_) => Ok(()),
        Message::Close(_) => Err(KalshiError::Ws("websocket closed".to_string())),
        Message::Text(text) => handle_payload(Bytes::from(text), tracker, event_tx, mode).await,
        Message::Binary(data) => handle_payload(Bytes::from(data), tracker, event_tx, mode).await,
        _ => Ok(()),
    }
}

async fn handle_payload(
    bytes: Bytes,
    tracker: &Arc<Mutex<SubscriptionTracker>>,
    event_tx: &mpsc::Sender<WsEvent>,
    mode: WsReaderMode,
) -> Result<(), KalshiError> {
    match mode {
        WsReaderMode::Owned => {
            let msg = WsMessage::from_bytes(&bytes)?;
            {
                let mut tracker = tracker.lock().await;
                tracker.handle_message(&msg);
            }
            event_tx
                .send(WsEvent::Message(msg))
                .await
                .map_err(|_| KalshiError::Ws("websocket reader closed".to_string()))?;
        }
        WsReaderMode::Raw => {
            if let Ok(control) = serde_json::from_slice::<WsControlMessage>(&bytes) {
                let mut tracker = tracker.lock().await;
                match control {
                    WsControlMessage::Subscribed { id, sid } => {
                        tracker.handle_subscribed(id, sid);
                    }
                    WsControlMessage::Unsubscribed { sid } => {
                        tracker.handle_unsubscribed(sid);
                    }
                    WsControlMessage::Other => {}
                }
            }

            event_tx
                .send(WsEvent::Raw(WsRawEvent::new(bytes)))
                .await
                .map_err(|_| KalshiError::Ws("websocket reader closed".to_string()))?;
        }
    }

    Ok(())
}

async fn handle_reconnect(
    client: &mut KalshiWsLowLevelClient,
    env: &KalshiEnvironment,
    auth: &Option<KalshiAuth>,
    config: &WsReconnectConfig,
    tracker: &Arc<Mutex<SubscriptionTracker>>,
    event_tx: &mpsc::Sender<WsEvent>,
) -> Result<(), KalshiError> {
    let mut attempt: u32 = 0;
    let mut last_err = KalshiError::Ws("websocket disconnected".to_string());

    loop {
        attempt = attempt.saturating_add(1);
        if let Some(max) = config.max_retries {
            if attempt > max {
                return Err(last_err);
            }
        }

        let delay = config.backoff_delay(attempt);
        if !delay.is_zero() {
            sleep(delay).await;
        }

        let reconnect = match auth {
            Some(auth) => {
                KalshiWsLowLevelClient::connect_authenticated(env.clone(), auth.clone()).await
            }
            None => KalshiWsLowLevelClient::connect(env.clone()).await,
        };

        match reconnect {
            Ok(new_client) => {
                *client = new_client;
                if config.resubscribe {
                    let params = {
                        let mut tracker = tracker.lock().await;
                        tracker.prepare_resubscribe()
                    };
                    let mut resubscribe_err: Option<KalshiError> = None;
                    for p in params {
                        match client.subscribe(p.clone()).await {
                            Ok(id) => {
                                let mut tracker = tracker.lock().await;
                                tracker.record_subscribe_cmd(id, p);
                            }
                            Err(err) => {
                                resubscribe_err = Some(err);
                                break;
                            }
                        }
                    }
                    if let Some(err) = resubscribe_err {
                        last_err = err;
                        continue;
                    }
                }

                let _ = event_tx.send(WsEvent::Reconnected { attempt }).await;
                return Ok(());
            }
            Err(err) => {
                last_err = err;
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KalshiEnvironment;
    use crate::ws::types::WsChannel;
    use tokio::net::TcpListener;
    use tokio::time::{Duration, timeout};
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;
    use url::Url;

    #[test]
    fn private_channel_check() {
        assert!(WsChannel::Fill.is_private());
        assert!(WsChannel::OrderbookDelta.is_private());
        assert!(WsChannel::MarketPositions.is_private());
        assert!(WsChannel::Communications.is_private());
        assert!(WsChannel::OrderGroupUpdates.is_private());

        assert!(!WsChannel::Ticker.is_private());
        assert!(!WsChannel::TickerV2.is_private());
        assert!(!WsChannel::Trade.is_private());
        assert!(!WsChannel::MarketLifecycleV2.is_private());
        assert!(!WsChannel::Multivariate.is_private());
    }

    #[test]
    fn subscription_tracker_moves_pending_to_active() {
        let mut tracker = SubscriptionTracker::default();
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::Ticker],
            ..Default::default()
        };
        tracker.record_subscribe_cmd(1, params.clone());
        tracker.handle_message(&WsMessage::Subscribed {
            id: Some(1),
            sid: Some(42),
        });

        assert!(tracker.pending.is_empty());
        assert_eq!(tracker.active.len(), 1);
        assert_eq!(tracker.active.get(&42), Some(&params));
    }

    #[test]
    fn subscription_tracker_prepare_resubscribe_clears_state() {
        let mut tracker = SubscriptionTracker::default();
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::Ticker],
            ..Default::default()
        };
        tracker.record_subscribe_cmd(1, params.clone());
        tracker.handle_message(&WsMessage::Subscribed {
            id: Some(1),
            sid: Some(42),
        });

        let params = tracker.prepare_resubscribe();
        assert_eq!(params.len(), 1);
        assert!(tracker.pending.is_empty());
        assert!(tracker.active.is_empty());
    }

    #[test]
    fn subscription_tracker_apply_update_changes_fields() {
        let mut tracker = SubscriptionTracker::default();
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::OrderbookDelta],
            market_tickers: Some(vec!["A".to_string()]),
            ..Default::default()
        };
        tracker.active.insert(10, params);

        let update = WsUpdateSubscriptionParams {
            sid: 10,
            market_tickers: Some(vec!["B".to_string()]),
            market_ids: None,
            event_tickers: None,
            send_initial_snapshot: Some(true),
            shard_factor: None,
            shard_key: None,
        };
        tracker.apply_update(&update);

        let updated = tracker.active.get(&10).unwrap();
        assert_eq!(updated.market_tickers.as_ref().unwrap()[0], "B");
        assert_eq!(updated.send_initial_snapshot, Some(true));
    }

    #[tokio::test]
    async fn reader_backpressure_preserves_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut ws = accept_async(stream).await.expect("accept ws");
            let msg1 = r#"{"type":"ticker","sid":1,"seq":1,"msg":{"market_ticker":"A","market_id":"1","price":1,"yes_bid":1,"yes_ask":2,"price_dollars":"0.01","yes_bid_dollars":"0.01","yes_ask_dollars":"0.02","volume":0,"volume_fp":"0","open_interest":0,"open_interest_fp":"0","dollar_volume":0,"dollar_open_interest":0,"ts":0}}"#;
            let msg2 = r#"{"type":"ticker","sid":2,"seq":2,"msg":{"market_ticker":"B","market_id":"2","price":1,"yes_bid":1,"yes_ask":2,"price_dollars":"0.01","yes_bid_dollars":"0.01","yes_ask_dollars":"0.02","volume":0,"volume_fp":"0","open_interest":0,"open_interest_fp":"0","dollar_volume":0,"dollar_open_interest":0,"ts":0}}"#;
            ws.send(Message::Text(msg1.to_string()))
                .await
                .expect("send 1");
            ws.send(Message::Text(msg2.to_string()))
                .await
                .expect("send 2");
        });

        let env = KalshiEnvironment {
            rest_origin: Url::parse("http://127.0.0.1/").expect("url"),
            ws_url: format!("ws://{}", addr),
        };
        let mut client = KalshiWsClient::connect(env, WsReconnectConfig::default())
            .await
            .expect("connect");

        let receiver = client
            .start_reader(WsReaderConfig {
                buffer_size: 1,
                mode: WsReaderMode::Owned,
            })
            .await
            .expect("start reader");

        let first = timeout(Duration::from_secs(2), receiver.next())
            .await
            .expect("timeout 1")
            .expect("event 1");
        let second = timeout(Duration::from_secs(2), receiver.next())
            .await
            .expect("timeout 2")
            .expect("event 2");

        assert!(matches!(first, WsEvent::Message(_)));
        assert!(matches!(second, WsEvent::Message(_)));

        server.await.expect("server");
    }

    #[tokio::test]
    async fn reader_reconnect_emits_reconnected_event() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept 1");
            let mut ws = accept_async(stream).await.expect("accept ws 1");
            let msg1 = r#"{"type":"ticker","sid":1,"seq":1,"msg":{"market_ticker":"A","market_id":"1","price":1,"yes_bid":1,"yes_ask":2,"price_dollars":"0.01","yes_bid_dollars":"0.01","yes_ask_dollars":"0.02","volume":0,"volume_fp":"0","open_interest":0,"open_interest_fp":"0","dollar_volume":0,"dollar_open_interest":0,"ts":0}}"#;
            ws.send(Message::Text(msg1.to_string()))
                .await
                .expect("send 1");
            ws.close(None).await.expect("close 1");

            let (stream, _) = listener.accept().await.expect("accept 2");
            let mut ws = accept_async(stream).await.expect("accept ws 2");
            let msg2 = r#"{"type":"ticker","sid":2,"seq":2,"msg":{"market_ticker":"B","market_id":"2","price":1,"yes_bid":1,"yes_ask":2,"price_dollars":"0.01","yes_bid_dollars":"0.01","yes_ask_dollars":"0.02","volume":0,"volume_fp":"0","open_interest":0,"open_interest_fp":"0","dollar_volume":0,"dollar_open_interest":0,"ts":0}}"#;
            ws.send(Message::Text(msg2.to_string()))
                .await
                .expect("send 2");
        });

        let env = KalshiEnvironment {
            rest_origin: Url::parse("http://127.0.0.1/").expect("url"),
            ws_url: format!("ws://{}", addr),
        };
        let config = WsReconnectConfig {
            max_retries: Some(3),
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            jitter: 0.0,
            resubscribe: false,
        };
        let mut client = KalshiWsClient::connect(env, config).await.expect("connect");

        let receiver = client
            .start_reader(WsReaderConfig {
                buffer_size: 4,
                mode: WsReaderMode::Owned,
            })
            .await
            .expect("start reader");

        let first = timeout(Duration::from_secs(2), receiver.next())
            .await
            .expect("timeout 1")
            .expect("event 1");
        assert!(matches!(first, WsEvent::Message(_)));

        let reconnect = timeout(Duration::from_secs(2), receiver.next())
            .await
            .expect("timeout reconnect")
            .expect("event reconnect");
        assert!(matches!(reconnect, WsEvent::Reconnected { .. }));

        let second = timeout(Duration::from_secs(2), receiver.next())
            .await
            .expect("timeout 2")
            .expect("event 2");
        assert!(matches!(second, WsEvent::Message(_)));

        server.await.expect("server");
    }
}
