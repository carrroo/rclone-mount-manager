# Rclone Mount Manager

A macOS menu bar app for managing rclone mounts with a clean GUI. Built with Tauri v2 (Rust) + Vue 3 + TypeScript.

[中文](#中文说明)

## Features

- One-click mount/unmount for rclone remotes
- Auto-detects remotes from `~/.config/rclone/rclone.conf`
- Custom mount configurations with host, user, password, port fields
- Inline editing with direct sync to `rclone.conf`
- Auto-reconnect for custom mounts
- System tray icon with mount status badge
- Dependency check (rclone + macFUSE)
- Bilingual UI: Chinese / English / Follow System

## Screenshots

![Main Window](https://github.com/user-attachments/assets/560a6b40-4248-49ae-95d3-c6e98a3b331e?text=Main+Window)

## Prerequisites

- macOS 10.13+
- [rclone](https://rclone.org/install/) (`brew install rclone`)
- [macFUSE](https://macfuse.io/) (required for FUSE mounts)

## Install

Download the latest `.dmg` from [Releases](../../releases), or build from source:

```bash
git clone https://github.com/yourname/rclone-mount-manager.git
cd rclone-mount-manager
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

The output `.dmg` and `.app` are in `src-tauri/target/release/bundle/`.

## Usage

1. Configure remotes in `~/.config/rclone/rclone.conf`
2. Launch the app — remotes appear automatically
3. Click **Mount** to mount, **Unmount** to unmount
4. Use **+ Add Mount** for custom mount entries
5. Switch language via the menu bar: **Rclone Mount Manager → Language**

## Tech Stack

| Layer | Tech |
|-------|------|
| Frontend | Vue 3, TypeScript, Pinia, vue-i18n |
| Backend | Rust, Tauri v2 |
| Packaging | Tauri bundler (.dmg / .app) |

## License

[MIT](LICENSE)

---

# 中文说明

一个 macOS 菜单栏应用，用于管理 rclone 挂载。使用 Tauri v2 (Rust) + Vue 3 + TypeScript 构建。

## 功能

- 一键挂载/卸载 rclone 远程存储
- 自动读取 `~/.config/rclone/rclone.conf` 中的远程配置
- 支持自定义挂载配置，包含主机、用户、密码、端口等字段
- 行内编辑配置，保存时直接同步到 `rclone.conf`
- 自定义挂载支持断线自动重连
- 系统托盘图标，显示挂载状态
- 依赖检测（rclone + macFUSE）
- 双语界面：中文 / 英文 / 跟随系统

## 前置条件

- macOS 10.13+
- [rclone](https://rclone.org/install/)（`brew install rclone`）
- [macFUSE](https://macfuse.io/)（FUSE 挂载必需）

## 安装

从 [Releases](../../releases) 下载最新的 `.dmg`，或从源码构建：

```bash
git clone https://github.com/yourname/rclone-mount-manager.git
cd rclone-mount-manager
npm install
npm run tauri dev
```

## 构建

```bash
npm run tauri build
```

输出的 `.dmg` 和 `.app` 位于 `src-tauri/target/release/bundle/`。

## 使用

1. 在 `~/.config/rclone/rclone.conf` 中配置远程存储
2. 启动应用 — 远程配置自动显示
3. 点击**挂载**进行挂载，点击**卸载**进行卸载
4. 使用 **+ 添加挂载** 创建自定义挂载条目
5. 通过菜单栏切换语言：**Rclone Mount Manager → 语言**

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3, TypeScript, Pinia, vue-i18n |
| 后端 | Rust, Tauri v2 |
| 打包 | Tauri bundler (.dmg / .app) |

## 许可证

[MIT](LICENSE)
