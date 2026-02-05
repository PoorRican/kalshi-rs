use crate::auth::KalshiAuth;
use crate::env::{KalshiEnvironment, WS_PATH};
use crate::error::KalshiError;
use crate::ws::types::{
    validate_subscription, WsEnvelope, WsListSubscriptionsCmd, WsMessage, WsSubscribeCmd,
    WsSubscriptionParams, WsUnsubscribeCmd, WsUnsubscribeParams, WsUpdateSubscriptionCmd,
    WsUpdateSubscriptionParams,
};

use futures::{SinkExt, StreamExt};

use rand::random;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::http::{HeaderValue, Request};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
>;

#[derive(Debug, Clone)]
pub struct WsReconnectConfig {
    pub max_retries: Option<u32>,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: f64,
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

#[derive(Debug)]
pub enum WsEvent {
    Message(WsMessage),
    Reconnected { attempt: u32 },
    Disconnected { error: KalshiError },
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
            WsMessage::Subscribed { id: Some(id), sid: Some(sid) } => {
                if let Some(params) = self.pending.remove(id) {
                    self.active.insert(*sid, params);
                }
            }
            WsMessage::Unsubscribed { sid: Some(sid), .. } => {
                self.active.remove(sid);
            }
            _ => {}
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

pub struct KalshiWsLowLevelClient {
    write: futures::stream::SplitSink<WsStream, Message>,
    read: futures::stream::SplitStream<WsStream>,
    next_id: u64,
    authenticated: bool,
}

impl KalshiWsLowLevelClient {
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
    pub async fn connect_authenticated(env: KalshiEnvironment, auth: KalshiAuth) -> Result<Self, KalshiError> {
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
            HeaderValue::from_str(&headers.signature).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&headers.timestamp_ms).map_err(|e| KalshiError::Header(e.to_string()))?,
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

    /// Subscribe to channels.
    pub async fn subscribe(&mut self, params: WsSubscriptionParams) -> Result<u64, KalshiError> {
        let needs_auth = params.channels.iter().any(|c| c.is_private());
        if needs_auth && !self.authenticated {
            return Err(KalshiError::AuthRequired("WebSocket private channel subscription"));
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

    /// Unsubscribe from a subscription id.
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

    /// Update an existing subscription.
    pub async fn update_subscription(&mut self, params: WsUpdateSubscriptionParams) -> Result<u64, KalshiError> {
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

    /// List active subscriptions.
    pub async fn list_subscriptions(&mut self) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsListSubscriptionsCmd { id, cmd: "list_subscriptions" };
        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Read the next JSON envelope from the stream.
    pub async fn next_envelope(&mut self) -> Result<WsEnvelope, KalshiError> {
        while let Some(msg) = self.read.next().await {
            let msg = msg.map_err(|e| KalshiError::Ws(e.to_string()))?;
            match msg {
                Message::Text(s) => return Ok(serde_json::from_str::<WsEnvelope>(&s)?),
                Message::Binary(b) => {
                    return Ok(serde_json::from_slice::<WsEnvelope>(&b)?);
                }
                Message::Ping(payload) => {
                    self.write
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|e| KalshiError::Ws(e.to_string()))?;
                    self.write
                        .flush()
                        .await
                        .map_err(|e| KalshiError::Ws(e.to_string()))?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => return Err(KalshiError::Ws("websocket closed".to_string())),
                _ => {}
            }
        }
        Err(KalshiError::Ws("websocket stream ended".to_string()))
    }

    /// Read the next JSON message and parse into a typed WsMessage.
    pub async fn next_message(&mut self) -> Result<WsMessage, KalshiError> {
        let env = self.next_envelope().await?;
        env.into_message()
    }
}

pub struct KalshiWsClient {
    env: KalshiEnvironment,
    auth: Option<KalshiAuth>,
    client: KalshiWsLowLevelClient,
    config: WsReconnectConfig,
    tracker: SubscriptionTracker,
}

impl KalshiWsClient {
    pub async fn connect(
        env: KalshiEnvironment,
        config: WsReconnectConfig,
    ) -> Result<Self, KalshiError> {
        let client = KalshiWsLowLevelClient::connect(env.clone()).await?;
        Ok(Self {
            env,
            auth: None,
            client,
            config,
            tracker: SubscriptionTracker::default(),
        })
    }

    pub async fn connect_authenticated(
        env: KalshiEnvironment,
        auth: KalshiAuth,
        config: WsReconnectConfig,
    ) -> Result<Self, KalshiError> {
        let client = KalshiWsLowLevelClient::connect_authenticated(env.clone(), auth.clone()).await?;
        Ok(Self {
            env,
            auth: Some(auth),
            client,
            config,
            tracker: SubscriptionTracker::default(),
        })
    }

    pub async fn subscribe(&mut self, params: WsSubscriptionParams) -> Result<u64, KalshiError> {
        let id = self.client.subscribe(params.clone()).await?;
        self.tracker.record_subscribe_cmd(id, params);
        Ok(id)
    }

    pub async fn unsubscribe(&mut self, sid: u64) -> Result<u64, KalshiError> {
        self.tracker.drop_active(sid);
        self.client.unsubscribe(sid).await
    }

    pub async fn update_subscription(
        &mut self,
        params: WsUpdateSubscriptionParams,
    ) -> Result<u64, KalshiError> {
        self.tracker.apply_update(&params);
        self.client.update_subscription(params).await
    }

    pub async fn list_subscriptions(&mut self) -> Result<u64, KalshiError> {
        self.client.list_subscriptions().await
    }

    pub async fn next_event(&mut self) -> Result<WsEvent, KalshiError> {
        match self.client.next_message().await {
            Ok(msg) => {
                self.tracker.handle_message(&msg);
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

            let delay = self.backoff_delay(attempt);
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
                KalshiWsLowLevelClient::connect_authenticated(self.env.clone(), auth.clone()).await?
            }
            None => KalshiWsLowLevelClient::connect(self.env.clone()).await?,
        };
        self.client = new_client;

        if self.config.resubscribe {
            let params = self.tracker.prepare_resubscribe();
            for p in params {
                let id = self.client.subscribe(p.clone()).await?;
                self.tracker.record_subscribe_cmd(id, p);
            }
        }

        Ok(())
    }

    fn backoff_delay(&self, attempt: u32) -> Duration {
        let exp = 2f64.powi(attempt.saturating_sub(1) as i32);
        let mut delay = self.config.base_delay.mul_f64(exp);
        if delay > self.config.max_delay {
            delay = self.config.max_delay;
        }
        let jitter = self.config.jitter.clamp(0.0, 1.0);
        if jitter > 0.0 {
            let factor = 1.0 - jitter + random::<f64>() * (2.0 * jitter);
            delay = delay.mul_f64(factor);
        }
        delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::types::WsChannel;
    use crate::{KalshiAuth, KalshiEnvironment};
    use tokio::time::Duration;
    use tokio_tungstenite::tungstenite::Message;

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
    async fn reconnect_emits_reconnected_event() {
        dotenvy::from_filename(".env.test").ok();
        let auth = KalshiAuth::from_pem_file(
            std::env::var("KALSHI_KEY_ID").expect("KALSHI_KEY_ID required"),
            std::env::var("KALSHI_PRIVATE_KEY_PATH").expect("KALSHI_PRIVATE_KEY_PATH required"),
        )
        .expect("load auth");

        let env = KalshiEnvironment::demo();
        let config = WsReconnectConfig {
            max_retries: Some(1),
            base_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            jitter: 0.0,
            resubscribe: false,
        };
        let mut client =
            KalshiWsClient::connect_authenticated(env, auth, config)
                .await
                .expect("connect");

        // Force close to trigger reconnect on next_event.
        client
            .client
            .write
            .send(Message::Close(None))
            .await
            .expect("close");

        let event = client.next_event().await.expect("next_event");
        match event {
            WsEvent::Reconnected { .. } => {}
            other => panic!("expected reconnected event, got {:?}", other),
        }
    }
}
