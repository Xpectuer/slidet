# 设计取舍

实现里最关键的点有三个。

1. 目录中的 `.md` 文件按文件名排序。
2. Markdown 被拆成稳定文本块和图片块。
3. 图片不可用时必须有降级内容。

```text
load_slides(dir)
  -> sort markdown files
  -> parse blocks
  -> render or fallback
```

---

如果这三点成立，最小可用版本就能成立。

