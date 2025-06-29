# macOS 平台支持

本文档描述了 xdev 项目对 macOS 平台的支持实现。

## 🎯 支持的平台

经过修改，xdev 现在支持以下 4 个平台：

| 平台 | 架构 | 状态 |
|------|------|------|
| **Linux** | x86_64 | ✅ 完全支持 |
| **Linux** | ARM64 | ✅ 完全支持 |
| **macOS** | Intel (x86_64) | ✅ 新增支持 |
| **macOS** | Apple Silicon (ARM64) | ✅ 新增支持 |

## 🔧 构建脚本修改 (scripts/build.sh)

### 核心改进

1. **智能平台检测**
   ```bash
   CURRENT_OS="$(uname -s)"
   
   if [ "$CURRENT_OS" = "Darwin" ]; then
       # macOS 系统：构建 macOS 目标
       TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin")
       USE_CROSS=false
   else
       # Linux 系统：构建 Linux 目标  
       TARGETS=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
       USE_CROSS=true
   fi
   ```

2. **构建工具选择**
   - **Linux**: 使用 `cross` 进行交叉编译
   - **macOS**: 使用原生 `cargo` 构建

3. **包管理器支持**
   - **Linux**: apt, yum, dnf, pacman, zypper
   - **macOS**: Homebrew

4. **压缩优化**
   - 单独处理每个二进制文件
   - 失败时自动回退到未压缩版本

5. **命令行选项**
   - `--all`: 构建所有平台（默认行为）
   - `--linux-only`: 只构建 Linux 平台
   - `--macos-only`: 只构建 macOS 平台
   - `--help`: 显示帮助信息

6. **智能错误处理**
   - 单个目标失败不影响其他目标
   - 详细的错误报告和建议
   - 优雅的跨平台兼容性处理

### 使用方法

```bash
# 默认：尝试构建所有平台的二进制文件（推荐在对应系统上使用）
./scripts/build.sh

# 只构建 Linux 平台的二进制文件（在任何系统上都能成功）
./scripts/build.sh --linux-only

# 只构建 macOS 平台的二进制文件（需要在 macOS 系统上运行）
./scripts/build.sh --macos-only

# 显示帮助信息
./scripts/build.sh --help
```

#### 智能构建逻辑

脚本现在具有智能错误处理：
- **在 Linux 上**: 成功构建 Linux 目标，跳过 macOS 目标并给出提示
- **在 macOS 上**: 成功构建 macOS 目标，可能需要 cross 工具来构建 Linux 目标
- **失败处理**: 单个目标失败不会影响其他目标的构建

## 📦 安装脚本修改 (scripts/install.sh)

### 核心改进

1. **平台识别**
   ```bash
   case "$OSTYPE" in
     Linux)  OSTYPE="unknown-linux-gnu" ;;
     Darwin) OSTYPE="apple-darwin" ;;
   esac
   ```

2. **校验工具适配**
   - **Linux**: 使用 `sha256sum`
   - **macOS**: 使用 `shasum -a 256`

3. **包管理器支持**
   - **Linux**: apt-get, yum, dnf, pacman, apk
   - **macOS**: Homebrew

4. **Shell 配置指导**
   - **Linux**: ~/.bashrc
   - **macOS**: ~/.bash_profile 或 ~/.zshrc

### 使用方法

```bash
# 通用安装命令（自动检测平台）
curl -Lsf https://raw.githubusercontent.com/loveloki/xdev/main/scripts/install.sh | sh
```

## 🚀 GitHub Actions 集成

macOS 二进制文件通过 GitHub Actions 的 macOS runner 构建：

```yaml
build-macos:
  runs-on: macos-latest
  strategy:
    matrix:
      target: [x86_64-apple-darwin, aarch64-apple-darwin]
```

## 🧪 测试验证

### Linux 环境测试
```bash
# 构建测试（默认会尝试所有平台）
./scripts/build.sh

# 预期输出示例:
# ✅ 成功构建的目标 (2):
#    ✓ x86_64-unknown-linux-gnu
#    ✓ aarch64-unknown-linux-gnu
# ❌ 失败的目标 (2):
#    ✗ x86_64-apple-darwin: 需要在 macOS 系统上构建
#    ✗ aarch64-apple-darwin: 需要在 macOS 系统上构建

# 只构建 Linux 目标
./scripts/build.sh --linux-only

# 功能测试  
./dist/xdev-aarch64-unknown-linux-gnu --version
```

### macOS 环境测试
需在实际 macOS 环境中进行测试：

```bash
# 安装依赖
brew install upx

# 构建测试（默认会尝试所有平台）
./scripts/build.sh

# 预期输出示例:
# ✅ 成功构建的目标 (2):
#    ✓ x86_64-apple-darwin
#    ✓ aarch64-apple-darwin
# ❌ 失败的目标 (2):
#    ✗ x86_64-unknown-linux-gnu: cross 构建失败（如果没有安装 cross）
#    ✗ aarch64-unknown-linux-gnu: cross 构建失败（如果没有安装 cross）

# 只构建 macOS 目标
./scripts/build.sh --macos-only

# 功能测试
./dist/xdev-x86_64-apple-darwin --version
./dist/xdev-aarch64-apple-darwin --version
```

## 📋 架构映射

| uname -s | uname -m | 目标架构 |
|----------|----------|----------|
| Linux | x86_64 | x86_64-unknown-linux-gnu |
| Linux | aarch64 | aarch64-unknown-linux-gnu |
| Darwin | x86_64 | x86_64-apple-darwin |
| Darwin | arm64 | aarch64-apple-darwin |

## 🚀 快速参考

| 命令 | 说明 | 适用环境 |
|------|------|----------|
| `./scripts/build.sh` | 构建所有平台（默认） | 任何系统 |
| `./scripts/build.sh --linux-only` | 只构建 Linux 平台 | 任何系统 |
| `./scripts/build.sh --macos-only` | 只构建 macOS 平台 | 推荐 macOS |
| `./scripts/build.sh --help` | 显示帮助信息 | 任何系统 |

## ⚠️ 注意事项

1. **macOS 交叉编译限制**
   - 由于 Apple 许可证限制，无法在 Linux 上交叉编译 macOS 二进制文件
   - macOS 目标必须在 macOS 环境中构建
   - 脚本会智能跳过不兼容的目标并给出提示

2. **UPX 压缩**
   - macOS 上的 UPX 压缩可能不稳定
   - 脚本已添加失败回退机制

3. **依赖管理**
   - macOS 需要 Homebrew 来安装依赖
   - 建议先安装 Xcode Command Line Tools

4. **构建策略**
   - 在对应的系统上构建效果最佳
   - 脚本提供清晰的错误反馈和建议

## 🔗 相关链接

- [GitHub Actions 配置](.github/workflows/release.yml)
- [构建脚本](scripts/build.sh)
- [安装脚本](scripts/install.sh) 
