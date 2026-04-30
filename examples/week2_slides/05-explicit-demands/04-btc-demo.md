# 实战：BTC 价格监控

**Before（模糊）：**
"帮我监控 BTC 价格"
→ AI 不知道你要 CLI / Dashboard / Slack bot

**After（显化）：**
"命令行脚本，接收币种代码（如 BTC），
调 CoinGecko 免费 API 获取实时美元价格，
格式化：`BTC: $68,420.50 (24h: +3.2%)`，
每 30s 刷新，按 q 退出。
Python，单依赖 requests。"

AI 不需要猜了
