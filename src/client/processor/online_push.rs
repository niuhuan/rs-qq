use std::sync::Arc;

use bytes::Buf;
use cached::Cached;
use futures::{stream, StreamExt};

use rq_engine::command::common::PbToBytes;
use rq_engine::msg::MessageChain;
use rq_engine::pb::msg;
use rq_engine::structs::{FriendMessageRecall, GroupMessage, GroupMute};
use rq_engine::{jce, pb};

use crate::client::event::{FriendMessageRecallEvent, GroupMessageEvent, GroupMuteEvent};
use crate::client::handler::QEvent;
use crate::client::Client;
use crate::engine::command::online_push::GroupMessagePart;
use crate::{RQError, RQResult};

impl Client {
    pub async fn process_group_message_part(
        self: &Arc<Self>,
        group_message_part: GroupMessagePart,
    ) -> Result<(), RQError> {
        // self.mark_group_message_readed(group_message_part.group_code, group_message_part.seq).await;

        // receipt message
        if group_message_part.from_uin == self.uin().await {
            if let Some(tx) = self
                .receipt_waiters
                .lock()
                .await
                .remove(&group_message_part.rand)
            {
                let _ = tx.send(group_message_part.seq);
            }
            return Ok(());
        }

        // merge parts
        let pkg_num = group_message_part.pkg_num;
        let group_msg = if pkg_num > 1 {
            let mut builder = self.group_message_builder.write().await;
            if builder.cache_misses().unwrap_or_default() > 100 {
                builder.flush();
                builder.cache_reset_metrics();
            }
            // muti-part
            let div_seq = group_message_part.div_seq;
            let parts = builder.cache_get_or_set_with(div_seq, Vec::new);
            parts.push(group_message_part);
            if parts.len() < pkg_num as usize {
                // wait for more parts
                None
            } else {
                Some(builder.cache_remove(&div_seq).unwrap_or_default())
            }
        } else {
            // single-part
            Some(vec![group_message_part])
        };

        // handle message
        if let Some(group_msg) = group_msg {
            // message is finish
            self.handler
                .handle(QEvent::GroupMessage(GroupMessageEvent {
                    client: self.clone(),
                    message: self.parse_group_message(group_msg).await?,
                }))
                .await; //todo
        }
        Ok(())
    }

    pub(crate) async fn parse_group_message(
        &self,
        mut parts: Vec<GroupMessagePart>,
    ) -> RQResult<GroupMessage> {
        parts.sort_by(|a, b| a.pkg_index.cmp(&b.pkg_index));
        let group_message = GroupMessage {
            seqs: parts.iter().map(|p| p.seq).collect(),
            rands: parts.iter().map(|p| p.rand).collect(),
            group_code: parts.first().map(|p| p.group_code).unwrap_or_default(),
            from_uin: parts.first().map(|p| p.from_uin).unwrap_or_default(),
            time: parts.first().map(|p| p.time).unwrap_or_default(),
            elements: MessageChain::from(
                parts
                    .into_iter()
                    .map(|p| p.elems)
                    .flatten()
                    .collect::<Vec<msg::Elem>>(),
            ),
        };
        //todo extInfo
        //todo group_card_update
        //todo ptt
        Ok(group_message)
    }

    pub(crate) async fn process_push_req(self: &Arc<Self>, msg_infos: Vec<jce::PushMessageInfo>) {
        for info in msg_infos {
            // let msg_seq = info.msg_seq;
            // let msg_time = info.msg_time;
            // let msg_uid = info.msg_uid;
            match info.msg_type {
                732 => {
                    let mut r = info.v_msg;
                    let group_code = r.get_u32() as i64;
                    let i_type = r.get_u8();
                    r.get_u8();
                    match i_type {
                        0x0c => {
                            let operator = r.get_u32() as i64;
                            if operator == self.uin().await {
                                continue;
                            }
                            r.advance(6);
                            let target = r.get_u32() as i64;
                            let time = r.get_u32();
                            self.handler
                                .handle(QEvent::GroupMute(GroupMuteEvent {
                                    client: self.clone(),
                                    group_mute: GroupMute {
                                        group_code,
                                        operator_uin: operator,
                                        target_uin: target,
                                        time,
                                    },
                                }))
                                .await;
                        }
                        0x10 | 0x11 | 0x14 | 0x15 => {
                            // group notify msg
                            // TODO proto 改成 oneof
                        }
                        _ => {}
                    }
                }
                528 => {
                    let mut v_msg = info.v_msg;
                    let msg: jce::MsgType0x210 = jcers::from_buf(&mut v_msg).unwrap();
                    match msg.sub_msg_type {
                        0x8A => {
                            let s8a = pb::Sub8A::from_bytes(&msg.v_protobuf).unwrap();
                            stream::iter(s8a.msg_info)
                                .map(|m| FriendMessageRecall {
                                    msg_seq: m.msg_seq,
                                    friend_uin: m.from_uin,
                                    time: m.msg_time,
                                })
                                .for_each(async move |m| {
                                    self.handler
                                        .handle(QEvent::FriendMessageRecall(
                                            FriendMessageRecallEvent {
                                                client: self.clone(),
                                                recall: m,
                                            },
                                        ))
                                        .await;
                                })
                                .await;
                        }
                        0x8B => {}
                        0xB3 => {}
                        0xD4 => {}
                        0x27 => {}
                        0x122 => {}
                        0x123 => {}
                        0x44 => {}
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // fn parse_push_info(&self, msg_info: &jce::PushMessageInfo) -> RQResult<PushInfo> {
    //     let info = PushInfo {
    //         msg_seq: raw.msg_seq,
    //         msg_time: raw.msg_time,
    //         msg_uid: raw.msg_uid,
    //         ..Default::default()
    //     };
    //     match raw.msg_type {
    //         732 => {
    //             let mut r = info.v_msg.clone();
    //             let _group_code = r.get_i32() as i64;
    //             let i_type = r.get_u8();
    //             r.get_u8();
    //             match i_type {
    //                 0x0c => {}
    //                 0x10 | 0x11 | 0x14 | 0x15 => {}
    //                 _ => {}
    //             }
    //         }
    //         528 => {
    //             let mut v_msg = raw.v_msg.clone();
    //             let mut jr = jcers::Jce::new(&mut v_msg);
    //             let _sub_type: i64 = jr.get_by_tag(0)?;
    //             // println!("sub_type: {}", sub_type);
    //             // TODO ...
    //         }
    //         _ => {}
    //     }
    //     Ok(info)
    // }
}
