# 图片与降级

先看一张存在的 PNG 图片资源：

![终端播放流程图](../02-image-demo/assets/terminal-flow.png)

再看一张已存在但暂不支持渲染的 SVG：

![流程图资源](assets/render-path.svg)

最后看一张故意缺失的图片：

![缺失资源](assets/not-found.png)

如果终端支持图片协议，第一张可以走图片路径。

如果终端不支持、格式暂不支持，或者资源缺失，三张都应该退化成可读提示，而不是让播放器崩溃。
