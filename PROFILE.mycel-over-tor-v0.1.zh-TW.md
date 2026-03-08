# Mycel over Tor Profile v0.1

狀態：profile draft

這份 profile 定義一個收斂的 Mycel over Tor 傳輸部署模型。

這份 profile 聚焦在 transport。

它不聲稱單靠自己就能提供完整匿名性。

它主要收斂：

- transport expectations（傳輸預期）
- peer addressing rules（節點位址規則）
- peer discovery behavior（節點發現行為）
- metadata handling（中繼資料處理）
- role-separation expectations（角色分離預期）

## 0. Scope

這份 profile 假設底層實作已支援：

- Mycel core protocol
- v0.1 wire protocol
- 可代理的 outbound transport
- 可配置的 peer addressing

這份 profile 適用於：

- 經 Tor 路由的 peer transport
- onion-first 的 peer addressing
- 想要先採用狹窄且明確匿名部署設定的第一版系統

## 1. Profile Goals

目標如下：

1. 降低直接 IP 暴露
2. 降低 peer-address 洩漏
3. 讓 transport 假設保持明確
4. 讓匿名性的限制被看見，而不是被暗示

## 2. Transport Requirements

合規節點必須滿足以下條件：

1. 所有 outbound peer traffic MUST 經由 Tor 或同等的本地 Tor proxy
2. direct clearnet peer dialing MUST 預設停用
3. 若存在 onion endpoint，peer addresses SHOULD 優先使用 onion endpoints
4. transport failure 時 MUST NOT 靜默回退到 direct clearnet transport

在這份 profile 中，除非明確跳出這份 profile，否則 direct fallback 應視為 profile violation。

## 3. Peer Addressing

這份 profile 偏好 onion-first addressing。

建議的 address classes：

- `tor:onion-v3`
- `tor:intro-ref`
- 只能透過 Tor-aware configuration 解析的 local bootstrap aliases

這份 profile 不要求公開的 clearnet address。

合規實作應：

- 將 onion endpoints 與可選的 clearnet endpoints 分開保存
- 在 outbound connection attempts 時優先使用 onion endpoints
- 在 profile-conforming mode 下避免發布不必要的 clearnet alternatives

## 4. Peer Discovery

這份 profile 刻意收窄 peer discovery。

允許的 discovery sources：

- 明確的本地 bootstrap list
- 經由 Tor-routed transport 取得的 accepted peer manifests
- out-of-band trusted peer introductions（帶外可信節點介紹）

在這份 profile 中，以下不應成為預設行為：

- public clearnet seed probing
- opportunistic direct-network scanning
- silent clearnet fallback discovery

實作仍可支援 manual peer addition，但合規運作應保持 discovery 有邊界且明確。

## 5. Message Metadata Handling

只有 Tor transport 並不夠。

合規實作也應降低不必要且可關聯的 metadata。

建議規則：

- 避免在使用者可見的 transport 顯示中暴露長期穩定的本地 node labels
- 除非 wire envelope 必要，否則避免暴露不必要的 sender metadata
- 不要把 deployment-specific routing hints（部署特定路由提示）塞進 replicated objects
- timestamp precision 不應高於 active profile 真正需要的程度

這份 profile 不重新定義 wire envelope，但會限制部署在附帶與保留額外 metadata 時的行為。

## 6. Local Logging and Caching

本地行為很容易破壞 transport anonymity。

合規實作應：

- 避免持久化記錄 raw peer IPs
- 在正常運作中避免保存 clearnet fallback attempts
- 將 anonymous-reading caches 與 signer 或 payment-operation caches 分開
- 支援 cache cleanup 與本地 compartment separation（分艙隔離）

這份 profile 不禁止本地診斷，但預設應停用那些會保留直接網路身分的診斷資料。

## 7. Role Separation

這份 profile 建議操作層角色分離。

建議分離：

- anonymous reader nodes 與 governance-maintainer nodes 分開
- governance-maintainer nodes 與 signer nodes 分開
- signer nodes 與 effect 或 payment runtimes 分開

這不會讓 public governance activity 自動變匿名，但可以減少不必要的跨角色關聯。

## 8. Wire Behavior Constraints

合規節點應維持正常 wire 相容性，同時限制 transport 使用方式。

建議規則：

- `HELLO` 與其他 wire messages 仍遵循 v0.1 wire envelope
- peer sessions 應只透過 Tor-routed transport 建立
- repeated failed connection attempts 不可觸發 direct-network fallback
- node operators 應能檢查某個 session 是否符合這份 profile

這份 profile 不修改 message shapes。

它只限制 session 是如何建立的。

## 9. Replication Strategy

這份 profile 對匿名敏感部署建議較窄的 replication 行為。

建議控制方式：

- 只抓取 active role 真正需要的 object families
- 在 reader nodes 上避免對所有敏感 app-layer records 做 universal mirroring
- 保留 accepted-state verification，而不要求所有 artifacts 都在每個節點完整複製

這份 profile 偏好 bounded replication（有邊界的複製），而不是最大可見性。

## 10. Operational Warnings

這份 profile 應明確顯示以下警告：

- Tor transport 不會隱藏穩定的 governance keys
- public signing activity 仍然可被關聯
- payment 或 effect runtimes 可能讓相關活動去匿名化
- 在同一本地環境中混用 anonymous reading 與 identified operations 會削弱匿名性

## 11. Minimal Conforming Flow

最小合規流程如下：

1. 載入 onion-first bootstrap list
2. 僅透過 Tor-routed transport 建立 outbound sessions
3. 交換正常的 v0.1 wire messages
4. 只抓取本地角色需要的 objects
5. 不做 clearnet fallback
6. 依角色將本地狀態保存在對應的 compartment 中

## 12. Non-goals

這份 profile 不定義：

- 完美匿名
- 對全域流量分析的免疫力
- public governance signing 的匿名性
- 匿名支付結算
- 新的 wire-message format

## 13. Minimal First-client Requirements

對第一個可互通 client，我建議：

- 一條明確的 Tor proxy configuration path
- 支援 onion-first peer list
- 不做 automatic clearnet fallback
- 當使用非本 profile transport 時給出明確 UI 警告
- 將 anonymous reading 與 identified operations 分離為不同本地 profiles

## 14. Open Questions

- 後續版本是否應要求所有公開可達 peers 都發布 onion service？
- 在匿名導向 profile 中，timestamp precision 是否應再進一步降低？
- peer discovery 是否應拆成 public-anonymous 與 restricted-anonymous 兩種更窄的 profiles？
