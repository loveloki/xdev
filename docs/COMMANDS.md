# xdev 命令文档

本文档详细介绍了 xdev 的所有可用命令及其使用方法。

## 📋 目录

- [命令概览](#命令概览)
- [基础命令](#基础命令)
- [配置管理](#配置管理)
- [Hosts 管理](#hosts-管理)
- [Draft 查找](#draft-查找)
- [安装管理](#安装管理)

## 🚀 命令概览

| 命令 | 描述 | 语法 |
|------|------|------|
| `version` | 显示版本信息 | `xdev version` |
| `install` | 安装到系统 | `xdev install` |
| `uninstall` | 从系统卸载 | `xdev uninstall` |
| `config` | 管理配置 | `xdev config [subcommand]` |
| `draft` | 查找最新 draft 目录 | `xdev draft` |
| `hosts` | 管理 hosts 文件和订阅 | `xdev hosts [subcommand]` |

## 🔧 基础命令

### `version` - 版本信息

显示当前 xdev 的版本信息。

**语法：**
```bash
xdev version
```

**示例：**
```bash
$ xdev version
xdev 0.1.0
```

**说明：**
- 显示当前安装的 xdev 版本号
- 用于验证安装是否成功
- 帮助用户确认使用的是最新版本

## ⚙️ 配置管理

### `config` - 配置管理

管理 xdev 的配置设置，支持查看、设置和交互式配置。

#### `config show` - 显示配置

显示当前所有配置项。

**语法：**
```bash
xdev config show
```

**示例：**
```bash
$ xdev config show
📋 配置 (~/.config/xdev/config.toml)
┌──────────────┬─────────────────────┐
│ 设置         │ 值                   │
├──────────────┼─────────────────────┤
│ lang         │ zh-Hans              │
│ draft_path   │ /tmp/zdocs           │
└──────────────┴─────────────────────┘
```

#### `config set` - 设置配置

设置配置项的值，支持交互式和非交互式两种模式。

**语法：**
```bash
# 交互式设置
xdev config set

# 非交互式设置
xdev config set <field> <value>
```

**参数：**
- `field`: 配置项名称（`lang` 或 `draft_path`）
- `value`: 配置项的值

**示例：**
```bash
# 交互式设置
$ xdev config set
🔧 交互式配置设置
选择一个配置项进行修改：
  ❯ 语言设置 (lang)
    draft 路径 (draft_path)
    显示当前配置
    退出

# 非交互式设置
$ xdev config set lang en
✅ 设置 lang = en

$ xdev config set draft_path /home/user/documents/draft
✅ 设置 draft_path = /home/user/documents/draft
```

**配置项说明：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `lang` | 字符串 | `zh-Hans` | 界面语言，支持 `zh-Hans` 和 `en` |
| `draft_path` | 字符串 | `/tmp/zdocs` | draft 目录的根路径 |
| `hosts_subscriptions` | 数组 | `[]` | hosts 订阅列表（自动管理） |

**配置文件位置：**
- **Linux/macOS**: `~/.config/xdev/config.toml`

**配置文件格式：**
```toml
# 语言设置 (zh-Hans 或 en)
lang = "zh-Hans"

# draft 目录路径
draft_path = "/tmp/zdocs"

# hosts 订阅列表（自动管理）
hosts_subscriptions = [
    "https://example.com/hosts1.txt",
    "https://example.com/hosts2.txt"
]
```

## 🌐 Hosts 管理

### `hosts` - Hosts 文件管理

管理系统 hosts 文件和订阅，支持多个外部 hosts 列表的订阅管理。

#### `hosts list` - 显示订阅列表

显示当前所有 hosts 订阅及其状态。

**语法：**
```bash
xdev hosts list
```

**示例：**
```bash
$ xdev hosts list
📋 当前订阅列表
┌─────┬─────────────────────────────────────────────────────┬────────────┐
│ 序号 │ 订阅 URL                                            │ 状态       │
├─────┼─────────────────────────────────────────────────────┼────────────┤
│ 1   │ https://raw.githubusercontent.com/StevenBlack/hosts │ ✅ 已应用  │
│     │ /master/hosts                                       │            │
│ 2   │ https://example.com/hosts.txt                       │ ⚠️ 未同步  │
└─────┴─────────────────────────────────────────────────────┴────────────┘

📊 统计信息:
   总订阅数: 2
   已应用:   1 ✅
   未同步:   1 ⚠️
💡 建议: 使用 'xdev hosts update' 来同步所有订阅
```

#### `hosts subscribe` - 订阅 hosts 列表

订阅外部 hosts 列表。

**语法：**
```bash
xdev hosts subscribe <url>
```

**参数：**
- `url`: hosts 文件的 URL，支持 HTTP 和 HTTPS 协议

**示例：**
```bash
$ xdev hosts subscribe https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts
🔄 开始订阅 hosts 列表: https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts
📥 正在下载并验证内容...
🔧 正在更新 hosts 文件...
💾 正在更新配置文件...
✅ 订阅成功: https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts
📊 当前共有 1 个订阅
```

**说明：**
- 自动下载并验证 hosts 文件内容
- 将订阅添加到配置文件中
- 更新系统的 hosts 文件
- 支持标准 hosts 格式（IP 地址 + 域名）

#### `hosts unsubscribe` - 取消订阅

取消指定的 hosts 订阅。

**语法：**
```bash
xdev hosts unsubscribe <url>
```

**参数：**
- `url`: 要取消订阅的 URL

**示例：**
```bash
$ xdev hosts unsubscribe https://example.com/hosts.txt
🗑️ 开始取消订阅: https://example.com/hosts.txt
🔧 正在从 hosts 文件中移除订阅...
💾 正在更新配置文件...
✅ 成功取消订阅: https://example.com/hosts.txt
📊 剩余订阅数量: 1
```

#### `hosts update` - 更新所有订阅

更新所有订阅的 hosts 列表。

**语法：**
```bash
xdev hosts update
```

**示例：**
```bash
$ xdev hosts update
🔄 开始更新所有订阅...
📊 发现 2 个订阅需要更新
🔄 [1/2] 正在更新: https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts
✅ [1/2] 更新成功
🔄 [2/2] 正在更新: https://example.com/hosts.txt
✅ [2/2] 更新成功

🎯 更新完成摘要:
├─ 总订阅数: 2
├─ 成功更新: 2 ✅
└─ 更新失败: 0 ❌
🎉 所有订阅更新成功！
```

#### `hosts backup` - 备份 hosts 文件

备份当前的 hosts 文件。

**语法：**
```bash
xdev hosts backup
```

**示例：**
```bash
$ xdev hosts backup
💾 开始备份 hosts 文件...
✅ 备份完成！
📁 备份文件: /home/user/.local/share/xdev/hosts_backup_20241201_120000.txt
📊 文件大小: 2048 字节
```

**说明：**
- 备份文件包含时间戳
- 备份文件存储在用户目录
- 可用于后续恢复

#### `hosts restore` - 恢复 hosts 文件

恢复 hosts 文件备份。

**语法：**
```bash
# 恢复最新备份
xdev hosts restore

# 恢复指定备份
xdev hosts restore <backup_file>
```

**参数：**
- `backup_file`: 备份文件名（可选）

**示例：**
```bash
# 恢复最新备份
$ xdev hosts restore
🔄 开始恢复 hosts 文件...
🔍 查找最新的备份文件...
🔄 正在恢复最新备份...
✅ hosts 文件恢复完成！
💡 建议: 使用 'xdev hosts list' 检查当前订阅状态

# 恢复指定备份
$ xdev hosts restore hosts_backup_20241201_120000.txt
🔄 开始恢复 hosts 文件...
📁 从指定文件恢复: /home/user/.local/share/xdev/hosts_backup_20241201_120000.txt
📊 备份文件大小: 2048 字节
🕒 备份时间: 1701436800 (UTC timestamp)
✅ hosts 文件恢复完成！
```

## 📁 Draft 查找

### `draft` - 查找最新 draft 目录

智能查找包含所有必需文件的最新 draft 目录。

**语法：**
```bash
xdev draft
```

**示例：**
```bash
$ xdev draft
✅ 找到 draft 目录: /tmp/zdocs/20241201/draft_001
```

**查找逻辑：**
1. 在配置的 `draft_path` 中查找最新修改的子目录
2. 验证目录是否包含所有必需的 zdocs 文件
3. 返回找到的完整路径

**必需文件：**
- `content.json` - 内容数据
- `meta.json` - 元数据
- `numbering.json` - 编号信息
- `relations.json` - 关系数据
- `settings.json` - 设置信息
- `styles.json` - 样式数据

**使用场景：**
```bash
# 查找并进入 draft 目录
cd $(xdev draft)

# 在脚本中使用
DRAFT_PATH=$(xdev draft)
if [ -n "$DRAFT_PATH" ]; then
    echo "当前 draft 目录: $DRAFT_PATH"
fi
```

## 📦 安装管理

### `install` - 安装到系统

将 xdev 二进制文件安装到系统的标准位置。

**语法：**
```bash
xdev install
```

**示例：**
```bash
$ xdev install
📦 正在安装 xdev 到 /home/user/.local/bin...
✅ 成功安装 xdev 到 /home/user/.local/bin
⚠️ 警告：/home/user/.local/bin 不在您的 PATH 中
   请将以下行添加到您的 shell 配置文件 (~/.bashrc, ~/.zshrc 等)：
   export PATH="/home/user/.local/bin:$PATH"
```

**功能特点：**
- 自动检测并创建安装目录
- 检查 PATH 环境变量并给出提示
- 无需 root 权限（用户级安装）
- 安装到 `~/.local/bin` 目录

### `uninstall` - 从系统卸载

从系统中移除 xdev 二进制文件。

**语法：**
```bash
xdev uninstall
```

**示例：**
```bash
$ xdev uninstall
🗑️ 正在从 /home/user/.local/bin 移除 xdev...
✅ 成功卸载 xdev
```

**说明：**
- 从安装目录中删除二进制文件
- 不会删除配置文件（保留用户设置）
- 不会删除备份文件

## 🔧 高级用法

### 脚本自动化

**批量配置脚本：**
```bash
#!/bin/bash
# 自动配置脚本

# 设置语言
xdev config set lang en

# 设置 draft 路径
xdev config set draft_path /home/user/documents/draft

# 添加多个 hosts 订阅
xdev hosts subscribe https://example1.com/hosts.txt
xdev hosts subscribe https://example2.com/hosts.txt

# 更新所有订阅
xdev hosts update

echo "配置完成！"
```

**定时更新脚本：**
```bash
#!/bin/bash
# 定时更新 hosts 订阅

# 备份当前状态
xdev hosts backup

# 更新所有订阅
xdev hosts update

# 记录日志
echo "$(date): Hosts 订阅已更新" >> /var/log/xdev.log
```

### 与其他工具集成

**与 Git 集成：**
```bash
#!/bin/sh
# pre-commit hook

# 检查 draft 目录
DRAFT_PATH=$(xdev draft)
if [ -n "$DRAFT_PATH" ]; then
    echo "当前 draft 目录: $DRAFT_PATH"
fi
```

**与 Shell 别名集成：**
```bash
# 添加到 ~/.bashrc 或 ~/.zshrc
alias draft='cd $(xdev draft)'
alias hosts-update='sudo xdev hosts update'
alias hosts-list='xdev hosts list'
```

## 🛠️ 故障排除

### 常见错误及解决方案

#### 权限问题
```bash
# 错误：无法修改 hosts 文件
❌ Hosts file update failed: Permission denied

# 解决：使用 sudo
sudo xdev hosts update
```

#### PATH 问题
```bash
# 错误：命令未找到
bash: xdev: command not found

# 解决：检查安装路径
ls -la ~/.local/bin/xdev

# 添加到 PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### 配置问题
```bash
# 错误：配置无法读取
❌ Failed to read config file: ~/.config/xdev/config.toml

# 解决：重置配置
rm ~/.config/xdev/config.toml
xdev config show  # 会重新生成默认配置
```

#### 网络问题
```bash
# 错误：订阅更新失败
❌ Update failed: Network error

# 解决：检查网络连接
ping github.com
curl -I https://example.com/hosts.txt
```

### 调试模式

启用详细日志输出：
```bash
# 设置环境变量
export RUST_LOG=debug

# 运行命令查看详细输出
xdev hosts update
```

## 📚 相关文档

- [用户指南](../README.md) - 项目介绍和快速开始
- [开发指南](DEVELOPMENT.md) - 开发者指南
- [更新日志](../CHANGELOG.md) - 版本变更记录

---

*更多帮助信息请运行 `xdev --help` 或 `xdev <command> --help`* 
