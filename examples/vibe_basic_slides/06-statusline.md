# statusline：让 agent 知道自己在哪

给 Claude Code 增加一个稳定可见的状态条：

- 当前 **git 分支**
- 当前 **工作路径**
- 当前 **剩余 context window**

如果 agent 不知道这些信息，就容易：

- 改错目录
- 在错误分支上工作
- 上下文溢出后失忆
- 在长会话中重复探索
