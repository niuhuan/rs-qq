use std::sync::Arc;
use std::time::UNIX_EPOCH;

use cached::Cached;

use ricq_core::{jce, pb};

use crate::client::event::KickedOfflineEvent;
use crate::client::{Client, NetworkStatus};
use crate::handler::QEvent;

impl Client {
    pub(crate) async fn process_push_notify(self: &Arc<Self>, notify: jce::RequestPushNotify) {
        match notify.msg_type {
            35 | 36 | 37 | 45 | 46 | 84 | 85 | 86 | 87 => {
                // pull group system msg(group request), then process
                match self.get_all_group_system_messages().await {
                    Ok(msgs) => {
                        self.process_group_system_messages(msgs).await;
                    }
                    Err(err) => {
                        tracing::warn!("failed to get group system message {}", err);
                    }
                }
            }
            187 | 188 | 189 | 190 | 191 => {
                // pull friend system msg(friend request), then process
                match self.get_friend_system_messages().await {
                    Ok(msgs) => {
                        self.process_friend_system_messages(msgs).await;
                    }
                    Err(err) => {
                        tracing::warn!("failed to get friend system message {}", err);
                    }
                }
            }
            _ => {
                // TODO tracing.warn!()
            }
        }
        // pull friend msg and other, then process
        let all_message = self.sync_all_message().await;
        match all_message {
            Ok(msgs) => {
                self.process_message_sync(msgs).await;
            }
            Err(err) => {
                tracing::warn!("failed to sync message {}", err);
            }
        }
    }

    pub(crate) async fn process_push_force_offline(
        self: &Arc<Self>,
        offline: jce::RequestPushForceOffline,
    ) {
        self.stop(NetworkStatus::KickedOffline);
        self.handler
            .handle(QEvent::KickedOffline(KickedOfflineEvent {
                client: self.clone(),
                inner: offline,
            }))
            .await;
    }

    pub(crate) async fn process_message_sync(self: &Arc<Self>, msgs: Vec<pb::msg::Message>) {
        for msg in msgs {
            let head = msg.head.clone().unwrap();
            if self.msg_exists(&head).await {
                continue;
            }
            match msg.head.as_ref().unwrap().msg_type() {
                9 | 10 | 31 | 79 | 97 | 120 | 132 | 133 | 166 | 167 => {
                    if let Err(err) = self.process_friend_message(msg).await {
                        tracing::error!("failed to process friend message {err}");
                    }
                }
                33 => {
                    if let Err(err) = self.process_join_group(msg).await {
                        tracing::error!("failed to process join group {err}");
                    }
                }
                140 | 141 => {
                    if let Err(err) = self.process_temp_message(msg).await {
                        tracing::error!("failed to process temp message {err}");
                    }
                }
                208 => {
                    // friend ptt_store
                }
                _ => tracing::warn!("unhandled sync message type"),
            }
        }
    }

    async fn msg_exists(&self, head: &pb::msg::MessageHead) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs() as i32;
        let msg_time = head.msg_time.unwrap_or_default();
        if now - msg_time > 60 || self.start_time > msg_time {
            return true;
        }
        let mut c2c_cache = self.c2c_cache.write().await;
        let key = (
            head.from_uin(),
            head.to_uin(),
            head.msg_seq(),
            head.msg_uid(),
        );
        if c2c_cache.cache_get(&key).is_some() {
            return true;
        }
        c2c_cache.cache_set(key, ());
        if c2c_cache.cache_misses().unwrap_or_default() > 100 {
            c2c_cache.flush();
            c2c_cache.cache_reset_metrics();
        }
        false
    }
}
