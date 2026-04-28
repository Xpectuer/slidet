# Demo 1：BTC 价格监控

一个"一键直出"的 BTC 价格监控最小版本：

- 获取 BTC 当前价格
- 支持设定阈值
- 到达阈值时提醒
- 先做本地可运行版本

**RIPER 节奏：**

1. **Research**：用什么 API？轮询还是 WebSocket？CLI 还是网页？
2. **Innovation**：公开 API + 命令行输出 + 阈值提醒（最小闭环）
3. **Plan**：fetch_price → check_threshold → main loop → 测试 → README
4. **Execute**：先写测试 → 获取价格 → 阈值判断 → 主循环 → 运行验证
5. **Review**：API 失败处理、输入异常、配置硬编码、扩展性

> demo 越小，越适合完整走 RIPER。真正的能力不是让 AI 一次写 500 行，而是让它始终不偏航。
