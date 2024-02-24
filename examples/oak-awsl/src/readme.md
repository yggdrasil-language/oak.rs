# AWSL 文本前端

AWSL（Asgard Web Specification Language）是 **Valkyrie `widget` 机制的 Vue 风格表面语法**，也是
**唯一的单文件组件（SFC）格式**（`.awsl`：顶层 `<widget>` + `<script>` + `<style>` 三大块）。

**一文件一 widget**：`source/pages/counter.awsl` 对应 `<widget counter>`、路由 `counter`；文件名 stem 是 widget 名与路由的唯一来源（
**snake_case**，`interactive-col-plot.awsl` → `<widget interactive_col_plot>`）。若写了 `<widget name>`，必须与 stem 一致。
`<script>` / `<style>` 必须与 `<widget>` 顶层并列，块内不再额外缩进一级。

`<widget>` 下可写多个并列 HTML / widget 标签，编译器会自动包成 fragment（不增加额外 DOM 节点）。也可显式写 `<fragment>`。

其中 **`<template>` 仅作顶层 `<widget>` 的兼容别名**（二者语义等价）。嵌套写法不再支持。
不是独立于 Valkyrie 的平行语言；也不再有单独的 Valkyrie SFC 方言。

## 与 `.v` widget 的关系

|          | AWSL (`.awsl`)                                              | `widget` (`.v`)          |
|:---------|:------------------------------------------------------------|:-------------------------|
| 类比     | Vue SFC                                                     | React                    |
| 模板     | `<widget>` + `@if` / `{expr}`                               | `render() -> Element`    |
| 语义归属 | **Valkyrie widget**（经 voa 前置降级）                      | Valkyrie 主线            |
| 底层     | `<script>` ≡ **vx**（`.vx` 表面）≡ `widget name { script }` | `widget Name { render }` |

二者最终汇入同一 **widget → HIR → MIR** 主线；AWSL 仅在前端多一步模板降级（RenderIR），
不另建编译后端。

## 一句话原则

AWSL 可以有自己的模板语法，但 `<widget>` 与 `<script>` 的语义就是 Valkyrie widget； **`<script>` 正文按 vx（Valkyrie +
X-Grammar）解析**，与 `.vx` 文件相同，不是核心 `.v` 方言。
Vue/React 只是写法差异，不是两套组件模型。

**SFC 根标签 snake_case**；模板内 **Asgard 布局/控件原语仍为 PascalCase**（`Column`、`Button`）。静态 HTML 用 `class="..."`
；动态绑定 `:prop="expr"`；Tailwind 用 `@style="flex w-4"`； **动态 inline CSS 用 `:style="f\"width: {w}\""`**；静态 CSS
规则写在 `<style>` 块；DSL 指令 `@click="expr"`、`@loop="item in items()"`（KV 禁止 `{}`）。

### HTML void 元素（支持，不推荐）

AWSL 识别 HTML5 void 标签（`br` / `img` / `input` / `meta` / `link` / `hr` / …），可直接粘贴 HTML：` <br> `、`<img src="…">`、
`<input type="text">` 均无需闭合标签或 `/>`。这是 **DSL 相对 `.vx` X-Grammar 的便利**：`.vx` 要求显式结构，不支持 HTML void
语义。

新代码仍应优先 widget 原语；void 标签仅用于迁移 HTML 片段或文档站粘贴。

Asgard **平台原生优先**渲染 AWSL UI（DOM / WXML / Compose / SwiftUI 等）；自渲仅在小游戏 Canvas 等场景有限支持。
