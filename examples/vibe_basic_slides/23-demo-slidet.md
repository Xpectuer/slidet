# Demo 2：slidet — 终端版 PPT

一个终端版本的 PPT，读取 Markdown 并渲染成演示内容。

**适合说明：**

- AI 不只会写 CRUD，也能处理"工具型产品"
- Markdown → 解析 → 渲染 → 交互，是典型的多环节需求
- 如果不先拆模块，agent 很容易把解析、渲染、快捷键逻辑搅在一起

**RIPER 应用：**

- Research：终端渲染有哪些库可选？
- Innovation：分页模型、主题系统、快捷键设计怎么选？
- Plan：先支持标题页和普通页，还是一步到位？
- Execute：先做最小渲染闭环
- Review：检查是否过早抽象
