# 添加 dns 子命令以更新 hosts
_Exported on 2025/6/29 at GMT+8 23:36:55 from Cursor (1.1.6)_

---

**User**

添加新的子命令 `dns` 用于更新最新的 dns，目前只支持 `github`，实际更新命令为：`sudo sh -c 'sed -i "/# GitHub520 Host Start/Q" /etc/hosts && curl https://raw.hellogithub.com/hosts >> /etc/hosts'`

---

**Cursor**

我需要先了解项目的结构和当前状态，然后为您添加 `dns` 子命令。让我开始分析项目。

好的，我来为您添加 `dns` 子命令来更新 GitHub 的 DNS 配置。让我先分析项目结构并制定实施计划。

## 项目分析

这是一个使用 Rust 和 clap 构建的 CLI 工具项目 `xdev`。项目结构清晰：
- 每个命令都有独立的模块文件
- 使用国际化系统支持中英文
- 采用统一的命令注册和处理模式

## 实施策略

### 方法评估
1. **直接调用系统命令**：使用 `std::process::Command` 执行您提供的 shell 命令
2. **Rust 原生实现**：使用 Rust 代码实现文件操作和网络请求
3. **混合方式**：核心逻辑用 Rust 实现，必要时调用系统命令

### 权衡考虑
- **安全性**：需要 sudo 权限，应明确提示用户
- **可扩展性**：设计时考虑未来支持更多 DNS 源
- **错误处理**：网络、权限、文件系统错误的处理
- **用户体验**：提供清晰的进度反馈和错误信息

## 完整实施计划

我将创建一个结构化的实施计划，遵循项目的规划工作流：

```markdown
# 添加 DNS 子命令计划

## 目标
为 xdev CLI 工具添加新的 `dns` 子命令，用于更新最新的 DNS 配置，目前支持 GitHub DNS 更新。

## 执行步骤

- [ ] 步骤 1: 创建 `src/commands/dns.rs` 模块文件
- [ ] 步骤 2: 在 `src/commands/mod.rs` 中注册 dns 命令
- [ ] 步骤 3: 在国际化文件中添加 DNS 命令相关翻译
- [ ] 步骤 4: 实现核心 DNS 更新逻辑
- [ ] 步骤 5: 添加错误处理和用户提示
- [ ] 步骤 6: 进行代码质量检查（cargo check, clippy）
- [ ] 步骤 7: 测试命令功能

## 技术细节

### 命令设计
```bash
xdev dns [SOURCE]
```
- 默认源：github（当前仅支持此源）
- 未来可扩展：支持其他 DNS 源

### 核心实现
- 执行命令：`sudo sh -c 'sed -i "/# GitHub520 Host Start/Q" /etc/hosts && curl https://raw.hellogithub.com/hosts >> /etc/hosts'`
- 权限检查：提示用户需要 sudo 权限
- 网络检查：验证可以访问更新源
- 错误处理：网络、权限、文件系统错误

### 用户体验
- 清晰的进度提示
- 详细的错误信息
- 成功/失败状态反馈
- 中英文国际化支持
```

## 📋 实施计划已创建

我已经为您制定了完整的添加 DNS 子命令的实施计划。计划包括以下主要步骤：

### 🎯 目标
为 xdev CLI 工具添加新的 `dns` 子命令，用于更新最新的 DNS 配置，目前支持 GitHub DNS 更新。

### 📝 执行步骤
1. **创建 DNS 命令模块** - 新建 `src/commands/dns.rs` 文件
2. **注册命令** - 在命令系统中注册新的 dns 子命令
3. **添加国际化支持** - 为中英文添加相关翻译字符串
4. **实现核心逻辑** - 执行您提供的 DNS 更新命令
5. **完善错误处理** - 处理权限、网络等各种错误情况
6. **代码质量检查** - 运行 cargo check 和 clippy
7. **功能测试** - 验证命令正常工作

### 🛠️ 技术设计
- **命令格式**: `xdev dns [SOURCE]` (默认 github)
- **核心实现**: 执行您提供的 sudo 命令来更新 /etc/hosts
- **用户体验**: 提供清晰的进度提示和错误反馈
- **扩展性**: 为未来支持更多 DNS 源预留接口

### ⚠️ 重要提示
**我将等待您的批准后再开始执行**。请确认以上计划是否符合您的要求，或者提出任何修改建议。

您是否同意开始执行这个计划？

---

**User**

OK，开始实现吧

---

**Cursor**

好的，现在我将开始执行任务：**步骤 1: 创建 `src/commands/dns.rs` 模块文件**

```rust
use crate::core::i18n::t;
use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::process;

pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("dns")
            .about(t!("command.dns.description").to_string())
            .arg(
                Arg::new("source")
                    .help(t!("help.dns_source").to_string())
                    .value_name("SOURCE")
                    .default_value("github")
                    .value_parser(["github"])
            )
    );
}

pub fn execute(matches: &clap::ArgMatches) -> Result<()> {
    let source = matches.get_one::<String>("source").unwrap_or(&"github".to_string());
    
    match source.as_str() {
        "github" => update_github_dns(),
        _ => {
            anyhow::bail!(t!("error.unsupported_dns_source", source = source).to_string());
        }
    }
}

fn update_github_dns() -> Result<()> {
    println!("{}", t!("command.dns.checking_permissions"));
    
    // 检查是否有 sudo 权限
    let sudo_check = process::Command::new("sudo")
        .args(["-n", "true"])
        .output()
        .context(t!("error.sudo_check_failed").to_string())?;
    
    if !sudo_check.status.success() {
        println!("{}", t!("command.dns.sudo_required"));
        println!("{}", t!("command.dns.sudo_instruction"));
    }
    
    println!("{}", t!("command.dns.updating_github_dns"));
    
    // 执行 DNS 更新命令
    let command = r#"sed -i "/# GitHub520 Host Start/Q" /etc/hosts && curl https://raw.hellogithub.com/hosts >> /etc/hosts"#;
    
    let output = process::Command::new("sudo")
        .args(["sh", "-c", command])
        .output()
        .context(t!("error.dns_update_failed").to_string())?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            t!("error.dns_update_command_failed", error = error_msg).to_string()
        );
    }
    
    println!("{}", t!("command.dns.success"));
    println!("{}", t!("command.dns.hosts_updated"));
    
    // 显示更新的主机条目数量（可选）
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().len() > 0 {
            println!("{}", t!("command.dns.update_details"));
        }
    }
    
    Ok(())
}
```

```markdown
- [ ] 步骤 1: 创建 `src/commands/dns.rs` 模块文件

->

- [x] 步骤 1: 创建 `src/commands/dns.rs` 模块文件
```

```rust
pub mod config;
pub mod dns;
pub mod draft;
pub mod install;
pub mod uninstall;
pub mod version;

use crate::core::i18n::t;
use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn register_command() -> Command {
    let mut app = Command::new("xdev")
        .version("0.1.0")
        .author("xleine@qq.com")
        .about(t!("general.app_description").to_string())
        .disable_version_flag(true); // 禁用自动生成的--version参数

    // 各模块注册自己的命令
    version::register_command(&mut app);
    install::register_command(&mut app);
    uninstall::register_command(&mut app);
    config::register_command(&mut app);
    draft::register_command(&mut app);
    dns::register_command(&mut app);

    app
}

pub fn handle_command(app: &mut Command, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("version", _)) => version::execute(),
        Some(("install", _)) => install::execute(),
        Some(("uninstall", _)) => uninstall::execute(),
        Some(("config", config_matches)) => config::execute(config_matches),
        Some(("draft", _)) => draft::execute(),
        Some(("dns", dns_matches)) => dns::execute(dns_matches),
        _ => {
            // 如果没有匹配的子命令，显示帮助信息
            app.print_help()?;
            println!(); // 添加一个换行
            Ok(())
        }
    }
}
```

```markdown
- [ ] 步骤 2: 在 `src/commands/mod.rs` 中注册 dns 命令

->

- [x] 步骤 2: 在 `src/commands/mod.rs` 中注册 dns 命令
```

```yaml
command:
  version:
    description: "显示版本信息"
    output: "xdev %{version}"
  
  install:
    description: "安装 xdev 二进制文件到系统"
    building: "🔨 正在构建 release 版本..."
    installing: "📦 正在安装 xdev 到 %{path}..."
    success: "✅ 成功安装 xdev 到 %{path}"
    path_warning: "⚠️  警告：%{path} 不在您的 PATH 中"
    path_instruction: "   请将以下行添加到您的 shell 配置文件 (~/.bashrc, ~/.zshrc 等)："
    path_export: "   export PATH=\"%{path}:$PATH\""
  
  uninstall:
    description: "从系统中卸载 xdev 二进制文件"
    not_installed: "ℹ️  xdev 未安装在 %{path}"
    removing: "🗑️  正在从 %{path} 移除 xdev..."
    success: "✅ 成功卸载 xdev"
  
  config:
    description: "管理配置"

  draft:
    description: "查找最新的 draft 目录"
    found: "✅ 找到 draft 目录"
    error: "❌ 错误"
    show:
      description: "显示当前配置"
      title: "📋 配置 (%{path})"
      table_header_setting: "设置"
      table_header_value: "值"
    set:
      description: "设置配置值"
      success: "✅ 设置 %{field} = %{value}"
      updated: "✅ 已将 %{field} 从 '%{old}' 更新为 '%{new}'"
      no_changes: "ℹ️  %{field} 没有变更"
      interactive_title: "🔧 交互式配置设置"
      interactive_prompt: "选择一个配置项进行修改："
      show_current: "显示当前配置"
      exit: "退出"
      continue_prompt: "继续配置？"
      completed: "🎉 配置完成！"

  dns:
    description: "更新 DNS 配置"
    checking_permissions: "🔍 检查权限..."
    sudo_required: "⚠️  需要 sudo 权限来修改 /etc/hosts 文件"
    sudo_instruction: "   请在执行此命令时输入管理员密码"
    updating_github_dns: "🌐 正在更新 GitHub DNS 配置..."
    success: "✅ DNS 配置更新成功"
    hosts_updated: "📝 /etc/hosts 文件已更新"
    update_details: "ℹ️  更新详情已显示"

  lang:
    description: "语言设置"
    current_prompt: "%{field} (当前: %{current})"

error:
  cargo_build_failed: "执行 cargo build 失败"
  build_failed: "构建二进制文件失败：\n%{error}"
  copy_failed: "复制二进制文件失败，从 %{from} 到 %{to}"
  remove_failed: "移除二进制文件失败，从 %{path}"
  config_dir_failed: "创建配置目录失败：%{path}"
  config_read_failed: "读取配置文件失败：%{path}"
  config_parse_failed: "解析配置文件失败"
  config_write_failed: "写入配置文件失败：%{path}"
  config_serialize_failed: "序列化配置失败"
  unknown_field: "未知配置字段：%{field}"
  unsupported_language: "不支持的语言：%{lang}"
  current_exe_failed: "获取当前可执行文件路径失败"
  current_dir_failed: "获取当前目录失败"
  config_dir_not_found: "找不到配置目录"
  home_dir_not_found: "无法找到用户主目录"
  create_install_dir_failed: "创建安装目录失败：%{path}"
  zdocs_not_found: "zdocs 目录不存在：%{path}"
  no_subdirs_found: "在 %{path} 中未找到子目录"
  draft_not_found: "在 %{path} 中未找到包含所有必需文件的 draft 目录"
  unsupported_dns_source: "不支持的 DNS 源：%{source}"
  sudo_check_failed: "检查 sudo 权限失败"
  dns_update_failed: "DNS 更新失败"
  dns_update_command_failed: "DNS 更新命令执行失败：%{error}"

general:
  app_description: "开发 CLI 工具"
  choose_option: "选择一个选项"
  draft_path_description: "draft 路径"

help:
  config_field: "要设置的配置字段"
  config_value: "要设置的值"
  dns_source: "DNS 源 (目前支持: github)"

fields:
  draft_path: "草稿路径"
  lang: "语言"
```

```yaml
command:
  version:
    description: "Show version information"
    output: "xdev %{version}"
  
  install:
    description: "Install xdev binary to system"
    building: "🔨 Building release binary..."
    installing: "📦 Installing xdev to %{path}..."
    success: "✅ Successfully installed xdev to %{path}"
    path_warning: "⚠️  Warning: %{path} is not in your PATH"
    path_instruction: "   Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    path_export: "   export PATH=\"%{path}:$PATH\""
  
  uninstall:
    description: "Uninstall xdev binary from system"
    not_installed: "ℹ️  xdev is not installed at %{path}"
    removing: "🗑️  Removing xdev from %{path}..."
    success: "✅ Successfully uninstalled xdev"
  
  config:
    description: "Manage configuration"

  draft:
    description: "Find the latest draft directory"
    found: "✅ Found draft directory"
    error: "❌ Error"
    show:
      description: "Show current configuration"
      title: "📋 Configuration (%{path})"
      table_header_setting: "Setting"
      table_header_value: "Value"
    set:
      description: "Set configuration values"
      success: "✅ Set %{field} = %{value}"
      updated: "✅ Updated %{field} from '%{old}' to '%{new}'"
      no_changes: "ℹ️  No changes made to %{field}"
      interactive_title: "🔧 Interactive Configuration Setup"
      interactive_prompt: "Select a configuration item to modify:"
      show_current: "Show current config"
      exit: "Exit"
      continue_prompt: "Continue configuring?"
      completed: "🎉 Configuration completed!"

  dns:
    description: "Update DNS configuration"
    checking_permissions: "🔍 Checking permissions..."
    sudo_required: "⚠️  sudo permission required to modify /etc/hosts file"
    sudo_instruction: "   Please enter administrator password when prompted"
    updating_github_dns: "🌐 Updating GitHub DNS configuration..."
    success: "✅ DNS configuration updated successfully"
    hosts_updated: "📝 /etc/hosts file has been updated"
    update_details: "ℹ️  Update details shown above"

  lang:
    description: "Language setting"
    current_prompt: "%{field} (current: %{current})"

error:
  cargo_build_failed: "Failed to execute cargo build"
  build_failed: "Failed to build binary:\n%{error}"
  copy_failed: "Failed to copy binary from %{from} to %{to}"
  remove_failed: "Failed to remove binary from %{path}"
  config_dir_failed: "Failed to create config directory: %{path}"
  config_read_failed: "Failed to read config file: %{path}"
  config_parse_failed: "Failed to parse config file"
  config_write_failed: "Failed to write config file: %{path}"
  config_serialize_failed: "Failed to serialize config"
  unknown_field: "Unknown config field: %{field}"
  unsupported_language: "Unsupported language: %{lang}"
  current_exe_failed: "Failed to get current executable path"
  current_dir_failed: "Failed to get current directory"
  config_dir_not_found: "Could not find config directory"
  home_dir_not_found: "Could not find user home directory"
  create_install_dir_failed: "Failed to create install directory: %{path}"
  zdocs_not_found: "zdocs directory does not exist: %{path}"
  no_subdirs_found: "No subdirectories found in %{path}"
  draft_not_found: "No draft directory with all required files found in %{path}"
  unsupported_dns_source: "Unsupported DNS source: %{source}"
  sudo_check_failed: "Failed to check sudo permissions"
  dns_update_failed: "DNS update failed"
  dns_update_command_failed: "DNS update command failed: %{error}"

general:
  app_description: "A development CLI tool"
  choose_option: "Choose an option"
  draft_path_description: "Path for draft documents"

fields:
  draft_path: "Draft Path"
  lang: "Language"

help:
  config_field: "Configuration field to set"
  config_value: "Value to set"
  dns_source: "DNS source (currently supported: github)"
```

```markdown
- [ ] 步骤 3: 在国际化文件中添加 DNS 命令相关翻译
- [ ] 步骤 4: 实现核心 DNS 更新逻辑
- [ ] 步骤 5: 添加错误处理和用户提示

->

- [x] 步骤 3: 在国际化文件中添加 DNS 命令相关翻译
- [x] 步骤 4: 实现核心 DNS 更新逻辑
- [x] 步骤 5: 添加错误处理和用户提示
```

```rust
let source = matches.get_one::<String>("source").unwrap_or(&"github".to_string());
```

发现了编译错误，我需要修复这个生命周期问题：

```rust
pub fn execute(matches: &clap::ArgMatches) -> Result<()> {
    let source = matches.get_one::<String>("source").unwrap_or(&"github".to_string());
    
    match source.as_str() {
        "github" => update_github_dns(),
        _ => {
            anyhow::bail!(t!("error.unsupported_dns_source", source = source).to_string());
        }
    }
}

->

pub fn execute(matches: &clap::ArgMatches) -> Result<()> {
    let source = matches.get_one::<String>("source")
        .map(|s| s.as_str())
        .unwrap_or("github");
    
    match source {
        "github" => update_github_dns(),
        _ => {
            anyhow::bail!(t!("error.unsupported_dns_source", source = source).to_string());
        }
    }
}
```

```rust
    // 显示更新的主机条目数量（可选）
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().len() > 0 {
            println!("{}", t!("command.dns.update_details"));
        }
    }

->

    // 显示更新的主机条目数量（可选）
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", t!("command.dns.update_details"));
        }
    }
```

```markdown
- [ ] 步骤 6: 进行代码质量检查（cargo check, clippy）

->

- [x] 步骤 6: 进行代码质量检查（cargo check, clippy）
```

```markdown
- [ ] 步骤 7: 测试命令功能

->

- [x] 步骤 7: 测试命令功能
```

## 🎉 任务完成！DNS 子命令已成功添加

### ✅ 执行结果报告

我已经成功为 xdev CLI 工具添加了 `dns` 子命令，所有计划任务都已完成：

**✅ 功能测试结果：**
1. **参数验证**：正确拒绝无效的 DNS 源参数
2. **默认参数**：`xdev dns` 默认使用 github 源
3. **权限检查**：正确显示权限检查进度
4. **命令执行**：成功更新了 /etc/hosts 文件 
5. **用户体验**：提供了清晰的 emoji 提示和中文反馈
6. **错误处理**：具备完整的错误处理机制
7. **代码质量**：通过了 cargo check 和 clippy 检查

### 📋 更新后的计划状态

```markdown
# 添加 DNS 子命令计划

## 目标
为 xdev CLI 工具添加新的 `dns` 子命令，用于更新最新的 DNS 配置，目前支持 GitHub DNS 更新。

## 执行步骤

- [x] 步骤 1: 创建 `src/commands/dns.rs` 模块文件
- [x] 步骤 2: 在 `src/commands/mod.rs` 中注册 dns 命令
- [x] 步骤 3: 在国际化文件中添加 DNS 命令相关翻译
- [x] 步骤 4: 实现核心 DNS 更新逻辑
- [x] 步骤 5: 添加错误处理和用户提示
- [x] 步骤 6: 进行代码质量检查（cargo check, clippy）
- [x] 步骤 7: 测试命令功能
```

### 🛠️ 实现的功能

**命令用法：**
```bash
# 使用默认的 github 源更新 DNS
xdev dns

# 显式指定 github 源
xdev dns github

# 查看帮助信息
xdev dns --help
```

**主要特性：**
- ✅ 自动权限检查和提示
- ✅ 执行您提供的 DNS 更新命令
- ✅ 完整的错误处理和用户反馈
- ✅ 中英文国际化支持
- ✅ 为未来扩展其他 DNS 源预留接口

### 🔄 下一步操作

所有任务已完成！是否希望我删除 `PLAN.MD` 文件？

```markdown
# 添加 DNS 子命令计划

## 目标
为 xdev CLI 工具添加新的 `dns` 子命令，用于更新最新的 DNS 配置，目前支持 GitHub DNS 更新。

## 执行步骤

- [x] 步骤 1: 创建 `src/commands/dns.rs` 模块文件
- [x] 步骤 2: 在 `src/commands/mod.rs` 中注册 dns 命令
- [x] 步骤 3: 在国际化文件中添加 DNS 命令相关翻译
- [x] 步骤 4: 实现核心 DNS 更新逻辑
- [x] 步骤 5: 添加错误处理和用户提示
- [x] 步骤 6: 进行代码质量检查（cargo check, clippy）
- [x] 步骤 7: 测试命令功能
```

```shellscript
# 使用默认的 github 源更新 DNS
xdev dns

# 显式指定 github 源
xdev dns github

# 查看帮助信息
xdev dns --help
```

