# xdev

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS-green.svg)](https://github.com/loveloki/xdev)
[![Architecture](https://img.shields.io/badge/Architecture-x86_64%20%7C%20ARM64-purple.svg)](https://github.com/loveloki/xdev)

一个现代化的、轻量级的 Rust 命令行工具，提供配置管理、自安装/卸载、hosts 文件管理等功能，支持中英双语。

## ✨ 特性

- 🚀 **高性能** - 使用 Rust 2024 Edition 编写，编译后二进制文件经过深度优化
- 🌍 **国际化** - 完整的中英文双语支持
- 🔧 **配置管理** - 交互式和非交互式配置设置
- 📦 **自安装** - 提供便捷的安装和卸载功能
- 🌐 **Hosts 管理** - 支持订阅、更新、备份、恢复 hosts 列表
- 📁 **智能查找** - 自动查找最新的 draft 目录
- 🖥️ **跨平台** - 支持 Linux 和 macOS，x86_64 和 ARM64 架构
- 🛡️ **安全可靠** - 完整的错误处理和用户友好的提示

## 🖥️ 支持的平台

| 平台 | 架构 | 状态 |
|------|------|------|
| **Linux** | x86_64 | ✅ 完全支持 |
| **Linux** | ARM64 | ✅ 完全支持 |
| **macOS** | Intel (x86_64) | ✅ 完全支持 |
| **macOS** | Apple Silicon (ARM64) | ✅ 完全支持 |

## 🚀 快速开始

### 安装

#### 方法一：使用安装脚本（推荐）

```bash
curl -LSsf https://raw.githubusercontent.com/loveloki/xdev/main/scripts/install.sh | bash
```

#### 方法二：手动下载

1. 访问 [Releases](https://github.com/loveloki/xdev/releases) 页面
2. 下载对应平台的二进制文件
3. 解压并移动到 PATH 目录：

```bash
# Linux
sudo mv xdev ~/.local/bin/

# macOS
sudo mv xdev ~/.local/bin/
```

#### 方法三：从源码构建

```bash
git clone https://github.com/loveloki/xdev.git
cd xdev
./scripts/build.sh
```

### 基本使用

```bash
# 查看版本
xdev version

# 查看帮助
xdev --help

# 查看配置
xdev config show

# 设置语言
xdev config set lang en
```

## 📋 命令概览

| 命令 | 描述 | 示例 |
|------|------|------|
| `version` | 显示版本信息 | `xdev version` |
| `install` | 安装到系统 | `xdev install` |
| `uninstall` | 从系统卸载 | `xdev uninstall` |
| `config` | 管理配置 | `xdev config show` |
| `draft` | 查找最新 draft 目录 | `xdev draft` |
| `hosts` | 管理 hosts 文件和订阅 | `xdev hosts list` |

## 📖 详细文档

- [命令文档](docs/COMMANDS.md) - 详细命令使用指南
- [开发指南](docs/DEVELOPMENT.md) - 构建、贡献和架构说明
- [更新日志](CHANGELOG.md) - 版本变更记录

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)。
