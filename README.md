# RS-QQ

![](https://socialify.git.ci/lz1998/rs-qq/image?forks=1&issues=1&language=1&owner=1&pattern=Circuit%20Board&pulls=1&stargazers=1&theme=Dark)

qq-android 协议的 rust 实现 移植于 MiraiGo、OICQ、Mirai

## 如何使用

本项目是协议 lib, 不推荐直接基于本项目开发。

Rust 用户推荐使用 [RQ-Tower](https://github.com/lz1998/rq-tower),
基于 [Demo](https://github.com/lz1998/rq-tower/tree/main/examples/demo) 修改，或 [rust_proc_qq](https://github.com/niuhuan/rust_proc_qq)。

其他语言用户推荐使用 [(WIP)Walle-Q](https://github.com/abrahum/walle-q), 基于 OneBot 协议开发。

如果一定要基于本项目开发，可以参考 `examples/password_login.rs` 或 `examples/qrcode_login.rs`。

> 本项目是一个年轻的项目，请使用 nightly channel 构建本项目哦（正经人谁用 stable 啊）

## 已完成功能/开发计划

### 登录

- [x] 账号密码登录
- [x] 二维码登录
- [x] 验证码提交
- [x] 设备锁验证
- [x] 错误信息解析

### 消息类型

- [x] 文本
- [x] 表情
- [x] At
- [x] 回复
- [x] 匿名
- [x] 骰子
- [x] 石头剪刀布
- [x] 图片
- [x] 语音
- [ ] 长消息(仅群聊/私聊)
- [ ] 链接分享
- [ ] 小程序(暂只支持RAW)
- [ ] 短视频
- [ ] 合并转发
- [ ] 群文件(上传与接收信息)

### 事件

- [x] 群消息
- [x] 好友消息
- [x] 新好友请求
- [x] 收到其他用户进群请求
- [x] 新好友
- [x] 群禁言
- [x] 好友消息撤回
- [x] 群消息撤回
- [x] 收到邀请进群请求
- [x] 群名称变更
- [x] 好友删除
- [x] 群成员权限变更
- [x] 新成员进群/退群
- [x] 登录号加群
- [x] 临时会话消息
- [ ] 登录号退群(包含T出)
- [ ] 客户端离线
- [ ] 群提示 (戳一戳/运气王等)

### 主动操作

> 为防止滥用，将不支持主动邀请新成员进群

- [x] 修改昵称
- [x] 发送群消息
- [x] 获取/刷新群列表
- [x] 获取/刷新群成员列表
- [x] 获取/刷新好友列表
- [x] 群成员禁言/解除禁言
- [x] 踢出群成员
- [x] 戳一戳群友
- [x] 戳一戳好友
- [x] 设置群管理员
- [x] 设置群公告
- [x] 设置群名称
- [x] 全员禁言
- [x] 获取群@全体剩余次数
- [x] 翻译
- [x] 修改群成员头衔
- [x] 设置群精华消息
- [x] 发送好友消息
- [x] 发送临时会话消息
- [x] 修改群成员Card
- [x] 撤回群消息
- [x] 撤回好友消息
- [x] 处理被邀请加群请求
- [x] 处理加群请求
- [x] 处理好友请求
- [x] 删除好友
- [x] 获取陌生人信息
- [x] 设置在线状态
- [x] 修改个人资料
- [x] 修改个性签名
- [ ] 获取群荣誉 (龙王/群聊火焰等)
- [ ] 获取群文件下载链接
- [ ] ~~群成员邀请~~

### 敏感操作

> 由于[QQ钱包支付用户服务协议](https://www.tenpay.com/v2/html5/basic/public/agreement/protocol_mqq_pay.shtml), 将不支持一切有关QQ钱包的协议

> 4.13 您不得利用本服务实施下列任一的行为：
> \
> （9） **侵害QQ钱包支付服务系統；**

- [ ] ~~QQ钱包协议(收款/付款等)~~
