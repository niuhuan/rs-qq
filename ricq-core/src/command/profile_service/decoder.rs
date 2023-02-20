use std::collections::HashMap;

use bytes::{Buf, Bytes};
use prost::Message;

use crate::command::profile_service::*;
use crate::{jce, RQResult};
use crate::{pb, RQError};

impl super::super::super::Engine {
    // ProfileService.Pb.ReqSystemMsgNew.Group
    pub fn decode_system_msg_group_packet(&self, payload: Bytes) -> RQResult<GroupSystemMessages> {
        let rsp = pb::structmsg::RspSystemMsgNew::decode(&*payload);
        let mut join_group_requests = Vec::new();
        let mut self_invited = Vec::new();
        match rsp {
            Ok(rsp) => {
                for st in rsp
                    .groupmsgs
                    .into_iter()
                    .filter_map(|st| st.msg.map(|m| (st.msg_seq, st.msg_time, st.req_uin, m)))
                {
                    let msg_seq = st.0;
                    let msg_time = st.1;
                    let req_uin = st.2;
                    let msg = st.3;
                    match msg.sub_type {
                        // 1 进群申请
                        1 => match msg.group_msg_type {
                            1 => join_group_requests.push(JoinGroupRequest {
                                msg_seq,
                                msg_time,
                                message: msg.msg_additional,
                                req_uin,
                                req_nick: msg.req_uin_nick,
                                group_code: msg.group_code,
                                group_name: msg.group_name,
                                actor_uin: msg.actor_uin,
                                suspicious: !msg.warning_tips.is_empty(),
                                ..Default::default()
                            }),
                            2 => self_invited.push(SelfInvited {
                                msg_seq,
                                msg_time,
                                invitor_uin: msg.action_uin,
                                invitor_nick: msg.action_uin_nick,
                                group_code: msg.group_code,
                                group_name: msg.group_name,
                                actor_uin: msg.actor_uin,
                                actor_nick: msg.actor_uin_nick,
                            }),
                            22 => join_group_requests.push(JoinGroupRequest {
                                msg_seq,
                                msg_time,
                                message: msg.msg_additional,
                                req_uin,
                                req_nick: msg.req_uin_nick,
                                group_code: msg.group_code,
                                group_name: msg.group_name,
                                actor_uin: msg.actor_uin,
                                suspicious: !msg.warning_tips.is_empty(),
                                invitor_uin: Some(msg.action_uin),
                                invitor_nick: Some(msg.action_uin_qq_nick),
                            }),
                            _ => {}
                        },
                        // 2 被邀请，不需要处理
                        2 => {}
                        // ?
                        3 => {}
                        // 自身状态变更(管理员/加群退群)
                        5 => {}
                        _ => {}
                    }
                }
                Ok(GroupSystemMessages {
                    self_invited,
                    join_group_requests,
                })
            }
            Err(_) => Err(RQError::Decode(
                "failed to decode RspSystemMsgNew".to_string(),
            )),
        }
    }

    // ProfileService.Pb.ReqSystemMsgNew.Friend
    pub fn decode_system_msg_friend_packet(
        &self,
        payload: Bytes,
    ) -> RQResult<FriendSystemMessages> {
        let rsp = pb::structmsg::RspSystemMsgNew::decode(&*payload)
            .map_err(|_| RQError::Decode("RspSystemMsgNew".into()))?;
        Ok(FriendSystemMessages {
            requests: rsp
                .friendmsgs
                .into_iter()
                .map(|m| {
                    let msg = m.msg.as_ref();
                    NewFriendRequest {
                        msg_seq: m.msg_seq,
                        message: msg
                            .map(|msg| msg.msg_additional.to_owned())
                            .unwrap_or_default(),
                        req_uin: m.req_uin,
                        req_nick: msg
                            .map(|msg| msg.req_uin_nick.to_owned())
                            .unwrap_or_default(),
                    }
                })
                .collect(),
        })
    }

    pub fn decode_get_rich_sig_response_packet(
        &self,
        mut payload: Bytes,
    ) -> RQResult<Vec<RichSigInfo>> {
        let mut request: jce::RequestPacket = jcers::from_buf(&mut payload)?;
        let mut data: jce::RequestDataVersion2 = jcers::from_buf(&mut request.s_buffer)?;
        let mut a = data
            .map
            .remove("GetRichSigRes")
            .ok_or_else(|| RQError::Decode("missing GetRichSigRes".into()))?;
        let mut b = a
            .remove("KQQ.GetRichSigRes")
            .ok_or_else(|| RQError::Decode("missing KQQ.GetRichSigRes".into()))?;
        b.advance(1);
        let resp: jce::GetRichSigRes = jcers::from_buf(&mut b)?;
        Ok(resp
            .sig_infos
            .into_iter()
            .map(|mut info| RichSigInfo {
                status: info.status,
                uin: info.uin,
                dw_time: info.dw_time,
                infos: {
                    let mut infos = HashMap::new();
                    while info.sig_info.remaining() > 2 {
                        let tag = info.sig_info.get_u8();
                        let len = info.sig_info.get_u8();
                        if info.sig_info.len() < len as usize {
                            break;
                        }
                        let value = info.sig_info.copy_to_bytes(len as usize);
                        infos.insert(tag, value);
                    }
                    infos
                },
            })
            .collect())
    }
}
