# Demo 1: Slidet — 终端里的 PowerPoint

**痛点**：Markdown 文稿在手，不想打开 PPT 调字体

**方案**：
- 读取 .md → 按 `---` 分隔幻灯片
- ANSI 转义码全屏渲染
- ← → 翻页，q 退出

**技术选型**：
- 解析：正则（不搞完整 AST）
- 渲染：ANSI 序列（不依赖 ncurses）
- 切换：全屏重绘

> 30 分钟搭出最小可用版本
