# 更新日志

## v0.1.0-20250707

xdev 是一款专为开发者设计的命令行工具，帮助你高效管理本地开发环境和配置。

### 主要功能与改进

- **一键安装与卸载**
  你可以通过 `xdev install` 命令将工具快速安装到系统，无需手动复制或配置。卸载同样便捷，`xdev uninstall` 一条命令即可彻底移除。

- **版本信息随时可查**
  使用 `xdev version`，你可以随时查看当前工具版本，确保环境一致性。

- **配置管理更简单**
  通过 `xdev config show` 查看当前配置，`xdev config set` 支持交互式和命令行两种方式，轻松管理如 draft 目录等核心参数。

- **draft 目录一键定位**
  使用 `xdev draft`，快速查找和打开你的最新 draft 目录，无需手动查找路径。

- **Hosts 文件智能管理**
  通过 `xdev hosts subscribe` 订阅外部 hosts 列表，`xdev hosts update` 一键更新所有订阅，`xdev hosts list` 查看订阅状态。支持自动备份和恢复，确保系统安全。

- **极简命令架构，易扩展**
  所有命令均模块化实现，主入口自动分发，帮助信息清晰友好。无论是新手还是资深开发者，都能快速上手。

### 使用示例

```bash
# 安装 xdev 到系统
xdev install

# 卸载 xdev
xdev uninstall

# 查看版本
xdev version

# 管理配置
xdev config show
xdev config set draft_path /your/path

# 查找 draft 目录
xdev draft

# Hosts 管理
xdev hosts subscribe https://example.com/hosts.txt
xdev hosts list
xdev hosts update
```

---

本次发布专注于提升开发者的日常体验，让常用操作一条命令即可完成。欢迎体验并提出宝贵建议！
