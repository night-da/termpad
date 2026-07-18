# termpad 架构说明

轻量 Rust TUI 文本编辑器（v0.2）。参照 Notepad-- 常用能力，在终端实现精简化子集。

## 分层结构

```text
main.rs         CLI 入口
    └── app/    单线程事件循环
            ├── mod.rs       App 结构、run 循环
            ├── handle.rs    命令分发
            ├── mouse.rs     鼠标/滚轮
            ├── edit.rs      列插入
            ├── prompt.rs    goto / open
            └── terminal.rs  终端生命周期
            ├── document[]   多标签状态
            ├── search.rs    查找/替换（App 全局，非 Document 字段）
            └── view/        ratatui 渲染
                    ├── mod.rs     draw 编排
                    ├── layout.rs  坐标映射
                    ├── gutter.rs  行号/标签栏
                    ├── text.rs    正文+选区+高亮
                    └── status.rs  状态栏
```

## 模块职责

| 模块 | 职责 | 行数级 |
| :-: | :-: | :-: |
| `buffer/gap` | 字节 Gap Buffer，UTF-8 编辑 | ~230 |
| `document` | 单标签页聚合、加载/保存 | ~130 |
| `app/` | 事件循环、命令、鼠标 | ~750（6 文件） |
| `view/` | 布局、渲染 | ~760（5 文件） |
| `syntax/*` | C/C++/Rust/Markdown 单遍规则高亮 | ~1600 |
| `search` | 字面量/正则搜索与替换（App 层） | ~220 |
| `cursor` / `selection` | 光标与选区（字符坐标） | ~180 |
| `fold` / `word` / `encoding` | 折叠、单词高亮、编码 | ~400 |

## 坐标约定（重要）

| 概念 | 单位 | 使用处 |
| :-: | :-: | :-: |
| `Cursor.col` | **字符**列 | 移动、插入、选区 |
| `Match.col` / `len` | **字节**偏移（行内） | 搜索、单词高亮、渲染 breakpoints |
| 语法 `Span` | **字节**偏移（行内） | `highlight_line` 输出 |

渲染层在 `view::build_line_spans` 将选区字符列转为字节边界（`char_col_to_byte`）。

## 并发与内存安全

- **单线程**：无 `Arc`/`Mutex`/worker；事件循环在 `app::run`。
- **多标签搜索**：`SearchState` 在 `App` 层全局共享；切换/新建/关闭标签时 `on_active_tab_changed()` 清空搜索并刷新单词高亮。
- **无 `unsafe`**：Gap Buffer 按字节操作，删除/切片使用 UTF-8 边界检查（`safe_byte_range`）。

## 语法高亮设计

- 表驱动关键字 + 共享 `syntax/scan.rs`（数字、字符串、运算符）
- 行内 `WordCtx` 状态机（括号深度、expect_type_name、expect_param_name、after_dot 等）
- **不做** LSP、跨行作用域、全文启发式

## 代码规范

项目根目录 [`rustfmt.toml`](../rustfmt.toml) 与 [`.editorconfig`](../.editorconfig) 统一格式；提交前建议跑 `scripts/check.ps1`（Windows）或 `scripts/check.sh`（Linux/macOS）。

### 格式化（rustfmt）

| 项 | 约定 |
| :-: | :-: |
| 行宽 | 100 列（`max_width = 100`） |
| 缩进 | 4 空格 |
| 换行 | LF（`.editorconfig`；Windows 下 `termpad.cmd` 除外） |
| import | `reorder_imports = true`；长 use 列表按 rustfmt 自动折行 |

```powershell
cargo fmt              # 自动格式化
cargo fmt -- --check   # 仅检查，不改文件
```

### 静态分析（clippy）

```powershell
cargo clippy -- -D warnings
```

警告视为错误；生产代码避免 `.unwrap()` / `.expect()`（测试代码除外）。

### 命名与结构

| 类别 | 风格 | 示例 |
| :-: | :-: | :-: |
| 模块 / 函数 | snake_case | `highlight_line`, `view/layout.rs` |
| 类型 / 枚举变体 | PascalCase | `EditorMode`, `Command::Save` |
| 错误 | 统一枚举 | `EditorError` + `EditorResult<T>` |
| 测试 | 模块内 `#[cfg(test)]` | 无独立 `tests/` 目录 |

### 注释

- 语言：模块 //! 与必要的 /// 用中文；保留 API 名、类型名等英文标识符
- 格式：纯文本，不用 Markdown 标记（反引号、链接语法、加粗等）
- 模块 doc：写职责、坐标/MVP 局限、跨模块约定（如字节 vs 字符列）；避免与函数 doc 重复
- 函数 doc：只标注 non-obvious 行为（倒序替换、poll 边缘滚动、列插入语义等）；自解释 getter/setter 不写
- 行内注释：仅解释「为什么」或易错 invariant；不写逐行翻译
- 示例：buffer/gap.rs 坐标约定、search.rs 字节匹配、syntax/* 启发式局限

### 提交前检查

```text
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
```

或使用 `scripts/check.ps1` / `scripts/check.sh`。

## 测试

- 49 个单元测试，模块内 `#[cfg(test)]`（含 `input` 键位映射、`view/layout` 拖动自动滚动、`demos/*` 语法 fixture）
- 无 `tests/` 集成测试；TUI 路径靠手工验证
- `demos/demo.c`、`demo.cpp`：高亮回归 fixture，**不要求可编译**；`demo.rs` 为规范 Rust 演示样本
- 门禁：`cargo test` + `cargo clippy -- -D warnings`。

## 已知局限

### 1. Gap Buffer 读路径 O(n)

`buffer/gap.rs` 中 `GapBuffer::as_text()` 每次调用都会把 gap 两侧的字节拼成完整 `String`（O(n)）。以下 API 在实现上依赖或间接触发该路径：

| API | 影响 |
| :-: | :-: |
| `line(row)` / `line_len(row)` | 取行、渲染、光标 clamp 时频繁调用 |
| `line_count()` | 滚动、行号 gutter、折叠映射 |
| `position_to_offset()` | 光标与 buffer 偏移互转 |

**表现**：文件全量读入内存；打开或编辑超大文件时，滚动、高亮、搜索可能明显变慢。MVP 阶段未做行索引缓存或增量 `as_text`。

**改进方向**：维护行首字节偏移表；或 `line()` 只扫描 gap 附近而非重建全文；大文件可选只读/分块加载（超出当前范围）。

### 2. `Document` ↔ `view` 层耦合

`document::Document` 内嵌 `view::ViewState`（`scroll_row`、`follow_cursor`、布局 `Rect`），而 `view::draw` 又接收 `&mut Document` 渲染——形成 **document → view 类型依赖** 与 **view → document 数据依赖** 的双向关系。

```text
document.rs   持有 ViewState（视图状态）
view/mod.rs   读取 Document 并写回 scroll / layout
```

**原因**：多标签下每个 tab 需要独立的滚动与视口；实现时把视口状态与 buffer 绑在同一 struct，减少 `App` 层字段。

**代价**：严格分层上「领域层」依赖了「展示层」模块；若将来抽离 headless 核心或单元测试 document 逻辑，需 mock 或迁移 `ViewState`。

**改进方向**：将 `ViewState` 移到 `document` 子模块或独立 `state/viewport.rs`；`view` 只接受 `(buffer, cursor, ViewState)` 快照渲染，不反向定义被 domain 引用的类型。

### 3. 其他（简述）

| 项 | 说明 |
| :-: | :-: |
| 命令分发集中 | `app/handle.rs` 单一大 `match`，功能增多时可按主题拆分 |
| 无集成测试 | 键盘/鼠标/TUI 回归依赖手工；可后续加 snapshot 或 scripted input |
| 语法高亮 | 单遍行内规则，无 LSP/跨行作用域 |

## 与 Notepad-- 的差异

| 项 | Notepad-- | termpad |
| :-: | :-: | :-: |
| UI | Qt GUI | ratatui TUI |
| 语言 | 100+ | C/C++/Rust/Markdown + Plain |
| 大文件 | 流式 | 全量加载；大文件打开/编辑较慢 |
| 选区 | 渲染 + 删除/输入覆盖 | Shift+方向键 / 鼠标拖拽；Backspace/Delete/输入替换选区 |
| 关标签 | `w`，dirty 时确认 | 同 Quit 的 y/n 流程 |

## 参考

- [ccpp_theme](https://github.com/xenkuo/ccpp_theme) — 配色
- [Notepad--](https://github.com/cxasm/notepad--) — 功能清单参考
- [kilo](https://github.com/antirez/kilo) / [helix](https://github.com/helix-editor/helix) — TUI 结构参考
