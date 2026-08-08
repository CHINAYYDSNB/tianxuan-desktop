# Tianxuan Desktop

面向运维、站长、个人开发者的跨平台桌面端服务器管理工具，兼容宝塔 & 1Panel。

- **前端**：React 18 + TypeScript + Tailwind CSS + Zustand + React Router
- **桌面壳**：Tauri v2
- **后端**：Rust (Tokio) + russh (SSH/SFTP) + keyring-rs (系统钥匙环) + SQLite
- **目标**：颜值高、交互好、本地化、真正多机批量操作的服务器管理工具

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm tauri build
```

## 路线图

- [x] Phase 0: 项目骨架（Tauri + React + Tailwind + invoke 通联）
- [ ] Phase 1: 主机管理（SQLite + keyring）
- [ ] Phase 2: SSH 终端（xterm.js + russh）
- [ ] Phase 3: 多机总览 Dashboard
- [ ] Phase 4: SFTP 文件管理
- [ ] Phase 5: 批量命令执行
- [ ] Phase 6: WebView 嵌面板
