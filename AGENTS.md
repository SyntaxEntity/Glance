# AGENTS.md — Glance 上手契约

> 项目根 = 仓库根 = `~/Projects/Glance`。
> **工作区统一规范（红线）见 `~/Projects/Atlas/prompts/PROMPT-00-workspace-norms.md` 与 `~/Projects/Atlas/CONVENTIONS.md`，本文件只补项目特有的部分。**

---

## 0. 30 秒认知

**Glance** = 低饱和护眼的 **Markdown + LaTeX** 桌面阅读器，面向 GNOME / Linux。Rust + GTK4 + WebKit，MIT。

| 项 | 值 |
|---|---|
| 分支 / 远端 | `master` ← SSH `git@<host>:<owner>/<repo>.git`（**2026-10-17 由 `ssh://…:443` 改回**） |
| 版本 | `0.1.2`，`Cargo.toml` 与 `packaging/glance.spec` 两处须一致 |
| 技术栈 | gtk4-rs 0.11 (`v4_22`) · libadwaita 0.9 · webkit6 0.6 · sourceview5 0.11 · serde |
| 渲染链 | Markdown → Rust 侧解析大纲 → JS 注入 → WebView 内 marked.js → KaTeX（**完全离线**，字体本地打包） |
| 文档 | `PROJECT.md`（现状与架构）· `docs/DEVELOPMENT.md`（构建/打包/发布）· 旧 `HANDOFF.md` 已删除 |

---

## 1. 🔴 待 Atlas / 用户决策的遗留问题

### 1.1 历史中的真实个人邮箱 ✅ 已由 Atlas 清理（2026-10-17）

一个**真实个人邮箱**曾出现在 **5 个已推送提交**的 author + committer 字段，**已在 public 仓库公开**。

**Atlas 已用 `git filter-repo` 改写全部 10 个提交并强推覆盖**：

- ✅ 全部提交身份归一为 `SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com>`
- ✅ 全部 10 个 **tree SHA 逐字节未变** —— 只有元数据被改写，代码零改动
- ✅ `master` + `v0.1.1` + `v0.1.2` 三个 ref 全部保留
- ✅ 改写期间工作区的未提交改动**逐字节完好**（Atlas 在裸镜像上操作，再 `git reset --soft` 同步本地）
- ⚠️ 旧 SHA 在 GitHub GC 前**仍可被 API 直接寻址**；已 fork / 克隆的副本不受影响

**新提交必须使用** `265272538+SyntaxEntity@users.noreply.github.com`（已配好）。

### 1.2 maintainer 身份在**文件内容**里已统一 ✅（2026-10-17）

原 `Esperanta <esperanta@…>` 陈旧身份已从当前树清除，全部统一为 `SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com>`：

| 位置 | 状态 |
|---|---|
| `packaging/glance.spec` `%changelog`（3 处） | ✅ 已改 |
| `packaging/build-deb.sh` `Maintainer:` | ✅ 已改 |
| git 提交身份 | ✅ 已由 Atlas 统一 |

> 自测：`grep -rn "esperanta" packaging/ \| wc -l` → `0`（构建产物已随之重新生成）。

### 1.3 仓库元信息

- description 仍带 `[Vibe]` 前缀 → 按规范 §4.4 去掉。
- 无 topics、无 homepage、README 无截图。
- `master` 无分支保护。

### 1.4 无 CI（D4）

`.github/` 不存在，四包靠本机脚本（aarch64 走 qemu 约 25 分钟）。
**前置阻塞**：`gh` token 缺 `workflow` scope → 无法经 API 推送 workflow 文件，需用户扩 scope 或改走 SSH push。

---

## 2. 构建与验证

```bash
cargo check && cargo build --release && cargo test        # 应为 16 passed / 0 failed

# 无头环境跑 GTK 测试（broadwayd 已装）
broadwayd -p 8085 :5 & GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 cargo test

# 版本一致性（三处应一致）
grep '^version' Cargo.toml; grep -E '^Version:' packaging/glance.spec
grep -oE '^\* [A-Z][a-z]{2} [A-Z][a-z]{2} [0-9]{2} [0-9]{4}' packaging/glance.spec  # 星期须与日期自洽
```

**沙箱限制**：`/var/tmp` 只读 → `rpmbuild` 需 `--define "_tmppath <可写路径>"`；`/run/user/1000` 只读 → dconf / WebKit socket 告警（无害）；无 GPU → Mesa ZINK 回退告警（无害）。

---

## 3. 硬性约束

1. **不上传任何文件或列表到网络** —— 离线阅读器是核心卖点，KaTeX / marked.js 必须本地打包，不依赖 CDN。
2. **护眼配色双端一致** —— GTK 主题与 WebView 主题必须同步改。
3. **禁止 `git clean` / `git reset --hard` / `git checkout -- .`** —— 工作区可能有多 agent 改动。
4. **无 `sudo`** —— 需提权的操作打印命令交用户。
5. **不在用户实时桌面盲启 GUI** —— 无隔离显示环境时只做静态验证 + 交用户真机验收。

---

## 4. 文档地图

| 文件 | 什么时候读 |
|---|---|
| `README.md` | 项目定位、功能、构建安装 |
| `PROJECT.md` | 项目现状与架构 |
| `docs/DEVELOPMENT.md` | 构建 / 打包 / 发布 / 约束 |

---

## 5. 汇报

跨项目事务（推送、发版、仓库元信息、需用户提权、跨项目约定）→ 上报 **Atlas**。
报告写到 `~/Projects/Atlas/reports/inbox/`，命名 `AGENT-REPORT-Glance-<YYYYMMDD>-<NN>.md`。
**不要把任何文件写到 `~/Projects/` 根目录。**
