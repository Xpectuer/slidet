# 结构化内容

> 这是一个引用块。当前实现不会保留 Markdown 外观，但应该保留核心文本。

下面是一个有顺序的信息列表：

1. 先从目录读取 slide
2. 再把 Markdown 切成文本块和图片块
3. 最后把可读内容交给 UI 渲染

接着是一组无序列表：

- loader 关注文件顺序
- markdown 关注块级切分
- image 关注路径和降级
- ui 关注可浏览性

```rust
fn smoke_check() {
    println!("slidet example");
}
```
