# 🚀 xdev 自动发布指南

本项目已配置了 GitHub Actions 和 GitLab CI/CD 自动发布功能，可以在推送版本标签时自动构建和发布 release。

## 📋 功能特性

- ✅ 支持多架构交叉编译 (x86_64, aarch64)
- ✅ 自动使用 UPX 压缩二进制文件
- ✅ 同时发布到 GitHub 和 GitLab
- ✅ 自动生成发布说明
- ✅ 提供安装指南

## 🔧 支持的架构

| 架构           | 目标平台          | 说明           |
|----------------|-------------------|----------------|
| x86_64         | Linux x86_64      | 标准 64 位系统 |
| aarch64        | Linux ARM64       | ARM 64 位系统  |

## 📝 发布流程

### 1. 准备发布

在发布前，请确保：
- [ ] 代码已经测试完成
- [ ] 更新了 `Cargo.toml` 中的版本号
- [ ] 如有 `CHANGELOG.md`，请更新变更日志

### 2. 创建版本标签

```bash
# 创建并推送版本标签
git tag v0.1.0
git push origin v0.1.0
```

### 3. 自动构建和发布

推送标签后，将自动触发：

#### GitHub Actions 流程：
1. 🔄 检出代码
2. 🦀 安装 Rust 工具链
3. 🛠️ 安装构建工具 (cross, upx)
4. 🏗️ 交叉编译多架构二进制文件
5. 📦 压缩二进制文件
6. 🚀 创建 GitHub Release
7. 📎 附加二进制文件到 Release

#### GitLab CI/CD 流程：
1. 🔄 并行构建多架构二进制文件
2. 📦 压缩和优化二进制文件
3. 📤 上传到 GitLab Package Registry
4. 🚀 创建 GitLab Release
5. 🔗 生成下载链接

## 📥 用户安装方式

### 从 GitHub 下载

```bash
# 下载最新版本
wget https://github.com/YOUR_USERNAME/xdev/releases/latest/download/xdev-x86_64-unknown-linux-gnu

# 添加执行权限
chmod +x xdev-x86_64-unknown-linux-gnu

# 安装到系统路径
sudo mv xdev-x86_64-unknown-linux-gnu /usr/local/bin/xdev
```

### 从 GitLab 下载

```bash
# 下载最新版本 (替换 YOUR_PROJECT_ID 和 VERSION)
wget "https://gitlab.com/YOUR_USERNAME/xdev/-/packages/generic/xdev/VERSION/xdev-x86_64-unknown-linux-gnu"

# 添加执行权限
chmod +x xdev-x86_64-unknown-linux-gnu

# 安装到系统路径
sudo mv xdev-x86_64-unknown-linux-gnu /usr/local/bin/xdev
```

## 🛠️ 本地构建

如果需要本地构建，可以使用以下脚本：

```bash
# 使用原始构建脚本
./scripts/build.sh

# 使用 CI/CD 构建脚本
./scripts/ci-build.sh

# 构建特定架构
BUILD_TARGET=x86_64-unknown-linux-gnu ./scripts/ci-build.sh
```

## 🐛 故障排除

### GitHub Actions 失败

1. 检查 GitHub Actions 权限设置
2. 确保 `GITHUB_TOKEN` 有足够权限
3. 检查构建日志中的错误信息

### GitLab CI/CD 失败

1. 检查 GitLab CI/CD 权限设置
2. 确保项目启用了 Package Registry
3. 检查 GitLab Runner 配置

### 构建失败

1. 检查 Rust 工具链版本
2. 确保 `cross` 和 `upx` 正确安装
3. 检查目标架构是否支持

## 📚 相关文档

- [Cross 编译文档](https://github.com/cross-rs/cross)
- [UPX 压缩工具](https://upx.github.io/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [GitLab CI/CD 文档](https://docs.gitlab.com/ee/ci/)

## 🔄 版本管理

本项目使用语义化版本控制：
- `v1.0.0` - 主版本
- `v1.1.0` - 次版本
- `v1.1.1` - 补丁版本

推荐的版本标签格式：`v<major>.<minor>.<patch>` 
