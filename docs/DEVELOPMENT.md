# xdev 开发者指南

本文档为 xdev 项目的开发者提供完整的开发环境设置、构建指南、贡献流程和技术架构说明。

## 📋 目录

- [开发环境设置](#开发环境设置)
- [构建指南](#构建指南)
- [项目架构](#项目架构)
- [代码结构](#代码结构)
- [贡献指南](#贡献指南)
- [发布流程](#发布流程)

## 🛠️ 开发环境设置

### 系统要求

- **Rust**: 1.70+ (推荐最新稳定版)
- **Cargo**: 随 Rust 一起安装
- **Git**: 用于版本控制
- **构建工具**:
  - **Linux**: gcc, make
  - **macOS**: Xcode Command Line Tools
- **可选工具**:
  - **cross**: 用于交叉编译
  - **upx**: 用于二进制文件压缩

### 环境准备

#### 1. 安装 Rust

```bash
# 使用 rustup 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### 2. 克隆项目

```bash
git clone https://github.com/loveloki/xdev.git
cd xdev

# 检查项目状态
cargo check
```

#### 3. 安装开发依赖

```bash
# 安装 cross (用于交叉编译)
cargo install cross --git https://github.com/cross-rs/cross

# 安装 upx (用于二进制压缩)
# Linux
sudo apt install upx-ucl  # Ubuntu/Debian
sudo yum install upx      # CentOS/RHEL

# macOS
brew install upx
```

## 🔨 构建指南

### 基本构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo check
cargo clippy
```

### 多平台构建

#### 使用构建脚本（推荐）

```bash
# 构建所有平台
./scripts/build.sh

# 只构建 Linux 平台
./scripts/build.sh --linux-only

# 只构建 macOS 平台
./scripts/build.sh --macos-only
```

#### 手动交叉编译

```bash
# 添加目标平台
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# 使用 cross 构建 Linux 目标
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu

# 使用 cargo 构建 macOS 目标（需要在 macOS 上）
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### 构建优化

项目在 `Cargo.toml` 中配置了发布优化：

```toml
[profile.release]
lto = true          # 链接时优化
opt-level = "z"     # 为最小体积优化
strip = true        # 自动剥离符号信息
panic = "abort"     # panic 时直接终止程序
codegen-units = 1   # 减少codegen单元以获得更好的优化
```

### 构建产物

构建完成后，二进制文件位于：
- **开发构建**: `target/debug/xdev`
- **发布构建**: `target/release/xdev`
- **交叉编译**: `target/{target}/release/xdev`

## 🏗️ 项目架构

### 整体架构

xdev 采用模块化架构设计，主要分为以下几个层次：

```
xdev
├── 应用层 (Application Layer)
│   ├── main.rs              # 程序入口
│   └── commands/            # 命令模块
├── 核心层 (Core Layer)
│   ├── core/                # 核心功能
│   └── globals.rs           # 全局配置
└── 基础设施层 (Infrastructure Layer)
    ├── i18n/                # 国际化
    ├── filesystem/          # 文件系统操作
    └── http/                # 网络请求
```

### 核心设计原则

1. **模块化设计** - 每个功能模块独立，便于维护和扩展
2. **错误处理** - 使用 `anyhow` 进行统一的错误处理
3. **国际化** - 完整的中英文双语支持
4. **配置管理** - 基于 TOML 的配置文件管理
5. **用户友好** - 提供交互式和非交互式两种操作模式

### 技术栈

- **语言**: Rust 2024 Edition
- **命令行解析**: clap 4.5
- **配置管理**: toml + serde
- **错误处理**: anyhow + thiserror
- **国际化**: rust-i18n
- **HTTP 客户端**: reqwest
- **交互式提示**: inquire
- **表格显示**: comfy-table

## 📁 代码结构

### 目录结构

```
src/
├── main.rs                  # 程序入口点
├── commands/                # 命令模块
│   ├── mod.rs              # 命令注册和分发
│   ├── version.rs          # 版本信息命令
│   ├── install.rs          # 安装命令
│   ├── uninstall.rs        # 卸载命令
│   ├── config/             # 配置管理
│   │   ├── mod.rs          # 配置命令入口
│   │   ├── model.rs        # 配置数据模型
│   │   ├── file.rs         # 配置文件操作
│   │   └── interactive.rs  # 交互式配置
│   ├── draft.rs            # draft 目录查找
│   └── hosts/              # hosts 文件管理
│       ├── mod.rs          # hosts 命令入口
│       ├── core.rs         # hosts 核心逻辑
│       ├── operations.rs   # 订阅操作
│       ├── backup.rs       # 备份恢复
│       ├── list.rs         # 列表显示
│       ├── helpers.rs      # 辅助函数
│       └── validation.rs   # 数据验证
└── core/                   # 核心功能模块
    ├── mod.rs              # 核心模块声明
    ├── globals.rs          # 全局常量和配置
    ├── i18n.rs             # 国际化支持
    ├── filesystem.rs       # 文件系统操作
    ├── http.rs             # HTTP 客户端
    ├── permission.rs       # 权限检查
    ├── table.rs            # 表格显示
    └── validation.rs       # 数据验证
```

### 核心模块详解

#### 1. 命令系统 (`commands/`)

**设计模式**: 采用注册-分发模式

```rust
// 命令注册
pub fn register_command() -> Command {
    let mut app = Command::new(APP_NAME)
        .version("0.1.0")
        .about(t!("general.app_description").to_string());
    
    // 注册子命令
    version::register_command(&mut app);
    config::register_command(&mut app);
    // ...
    
    app
}

// 命令分发
pub fn handle_command(app: &mut Command, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("version", _)) => version::execute(),
        Some(("config", config_matches)) => config::execute(config_matches),
        // ...
    }
}
```

#### 2. 配置管理 (`commands/config/`)

**特性**:
- 支持 TOML 格式配置文件
- 交互式和非交互式配置
- 自动创建默认配置
- 语言设置实时生效

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub draft_path: String,
    pub lang: String,
    pub hosts_subscriptions: Option<Vec<String>>,
}
```

#### 3. Hosts 管理 (`commands/hosts/`)

**核心功能**:
- 订阅管理（添加、删除、更新）
- 文件结构解析和重构
- 自动备份和恢复
- 状态检查和统计

**文件结构处理**:
```rust
pub struct HostsFileStructure {
    pub other_content: Vec<String>,
    pub subscription_blocks: HashMap<String, Vec<String>>,
}
```

#### 4. 国际化系统 (`core/i18n.rs`)

**特性**:
- 支持中英文双语
- 运行时语言切换
- 线程安全的语言状态管理

```rust
// 语言设置
pub fn set_language(lang: &str) -> Result<()> {
    rust_i18n::set_locale(lang);
    let _ = CURRENT_LANGUAGE.set(lang.to_string());
    Ok(())
}
```

### 扩展新命令

添加新命令的步骤：

1. **创建命令文件** (`src/commands/new_command.rs`)
```rust
use clap::Command;
use anyhow::Result;

pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("new_command")
            .about("新命令描述")
    );
}

pub fn execute() -> Result<()> {
    // 命令实现
    Ok(())
}
```

2. **注册命令** (`src/commands/mod.rs`)
```rust
pub mod new_command;

// 在 register_command 中调用
new_command::register_command(&mut app);

// 在 handle_command 中处理
Some(("new_command", _)) => new_command::execute(),
```

3. **添加国际化支持** (`locales/zh-Hans.yml`)
```yaml
command:
  new_command:
    description: "新命令描述"
    success: "✅ 操作成功"
```

## 🤝 贡献指南

### 贡献流程

1. **Fork 项目**
   ```bash
   git clone https://github.com/your-username/xdev.git
   cd xdev
   ```

2. **创建功能分支**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **开发功能**
   - 遵循代码规范
   - 添加必要的测试
   - 更新相关文档

4. **测试验证**
   ```bash
   cargo check
   cargo clippy
   cargo test
   ```

5. **提交代码**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   git push origin feature/your-feature-name
   ```

6. **创建 Pull Request**

### 代码规范

#### Rust 代码规范

- 遵循 [Rust 官方编码规范](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 函数和变量使用 snake_case
- 类型和模块使用 snake_case

#### 提交信息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**类型**:
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

**示例**:
```
feat(hosts): add automatic backup before update

fix(config): resolve language setting not persisting

docs: update installation guide for macOS
```

### 测试指南

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        // 测试实现
        assert_eq!(expected, actual);
    }
}
```

#### 集成测试

在 `tests/` 目录下创建集成测试：

```rust
// tests/integration_test.rs
use xdev::commands::config::Config;

#[test]
fn test_config_loading() {
    let config = Config::load().unwrap();
    assert_eq!(config.lang, "zh-Hans");
}
```

#### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_function_name

# 运行集成测试
cargo test --test integration_test
```

### 文档规范

#### 代码文档

- 所有公共 API 必须有文档注释
- 使用 `///` 进行文档注释
- 包含使用示例

```rust
/// 设置当前语言
/// 
/// # Arguments
/// 
/// * `lang` - 语言代码，支持 "zh-Hans" 和 "en"
/// 
/// # Returns
/// 
/// 返回 Result，成功时返回 Ok(())
/// 
/// # Examples
/// 
/// ```
/// use xdev::core::i18n::set_language;
/// 
/// set_language("en")?;
/// ```
pub fn set_language(lang: &str) -> Result<()> {
    // 实现
}
```

#### 项目文档

- README.md 面向用户
- DEVELOPMENT.md 面向开发者
- 保持文档与代码同步
- 使用清晰的 Markdown 格式

## 🚀 发布流程

### 版本管理

项目使用 [语义化版本控制](https://semver.org/)：

- **主版本号**: 不兼容的 API 修改
- **次版本号**: 向下兼容的功能性新增
- **修订号**: 向下兼容的问题修正

### 发布步骤

1. **更新版本号** (`Cargo.toml`)
   ```toml
   [package]
   version = "0.2.0"
   ```

2. **更新 CHANGELOG.md**
   ```markdown
   ## [0.2.0] - 2024-XX-XX
   
   ### 新增
   - 新功能描述
   
   ### 修复
   - Bug 修复描述
   ```

3. **构建发布版本**
   ```bash
   ./scripts/build.sh
   ```

4. **创建 Git 标签**
   ```bash
   git tag -a v0.2.0 -m "Release version 0.2.0"
   git push origin v0.2.0
   ```

5. **发布到 GitHub Releases**
   - 上传构建的二进制文件
   - 添加发布说明
   - 标记为最新版本

### CI/CD 集成

项目配置了 GitHub Actions 自动构建和发布：

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        target: [x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin]
```

## 📚 参考资料

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Cargo 手册](https://doc.rust-lang.org/cargo/)
- [clap 文档](https://docs.rs/clap/)
- [anyhow 文档](https://docs.rs/anyhow/)
- [serde 文档](https://serde.rs/)

## 🆘 获取帮助

- **Issues**: [GitHub Issues](https://github.com/loveloki/xdev/issues)
- **讨论**: [GitHub Discussions](https://github.com/loveloki/xdev/discussions)
- **文档**: [项目文档](https://github.com/loveloki/xdev/tree/main/docs)

---

感谢您对 xdev 项目的贡献！🎉 
