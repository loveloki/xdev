#!/bin/bash
#
# 发布脚本 - 自动版本发布工具
# 功能：
# 1. 更新 CHANGELOG.md
# 2. 创建 Git 标签
# 3. 推送到 GitHub
# 4. 生成发布说明

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目配置
PROJECT_NAME="xdev"
CHANGELOG_FILE="CHANGELOG.md"
CARGO_FILE="Cargo.toml"

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 显示帮助信息
show_help() {
    cat << EOF
用法: $0 [选项]

选项:
  -v, --version VERSION    指定版本号 (例如: 0.1.1)
  -m, --message MESSAGE    发布消息 (可选)
  -d, --dry-run           试运行模式，不实际执行
  -h, --help              显示此帮助信息

示例:
  $0 -v 0.1.1 -m "修复重要bug"
  $0 -v 0.1.1
  $0 --dry-run -v 0.1.1

注意:
  - 版本号必须符合语义化版本规范 (例如: 0.1.1, 1.0.0, 2.1.3)
  - 脚本会自动生成带时间戳的标签 (例如: v0.1.1-20241201-143022)
EOF
}

# 验证版本号格式
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_error "无效的版本号格式: $version"
        log_error "版本号必须符合语义化版本规范 (例如: 0.1.1, 1.0.0)"
        exit 1
    fi
}

# 检查 CHANGELOG 中是否有指定版本的更新信息
check_changelog_updates() {
    local version=$1
    
    log_info "检查 CHANGELOG 中版本 $version 的更新信息..."
    
    # 检查是否已经存在该版本的条目（支持 v0.0.1-20250101 格式）
    if grep -q "## v$version-" "$CHANGELOG_FILE"; then
        log_error "版本 $version 在 CHANGELOG 中已存在，请检查版本号"
        exit 1
    fi
    
    # 检查 [未发布] 部分是否有内容
    local unreleased_content=$(awk '/## \[未发布\]/,/## v/' "$CHANGELOG_FILE" | grep -v "## \[未发布\]" | grep -v "## v" | grep -v "^$" | grep -v "^---" | head -n 1)
    
    if [ -z "$unreleased_content" ]; then
        log_error "CHANGELOG 的 [未发布] 部分没有更新信息"
        log_error "请在 CHANGELOG.md 的 [未发布] 部分添加本次发布的更新内容"
        log_error "格式示例："
        log_error "## [未发布]"
        log_error ""
        log_error "### 新增"
        log_error "- 新功能描述"
        log_error ""
        log_error "### 变更"
        log_error "- 变更描述"
        log_error ""
        log_error "### 修复"
        log_error "- 修复描述"
        exit 1
    fi
    
    log_success "CHANGELOG 检查通过，发现更新信息"
}

# 检查 Git 状态
check_git_status() {
    log_info "检查 Git 状态..."
    
    # 检查是否在 Git 仓库中
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "当前目录不是 Git 仓库"
        exit 1
    fi
    
    # 检查是否有未提交的更改
    if ! git diff-index --quiet HEAD --; then
        log_warning "检测到未提交的更改"
        echo "未提交的文件:"
        git status --porcelain
        echo ""
        read -p "是否继续发布? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "发布已取消"
            exit 0
        fi
    fi
    
    # 检查远程仓库
    if ! git remote get-url origin > /dev/null 2>&1; then
        log_warning "未找到 origin 远程仓库"
        read -p "是否继续发布? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "发布已取消"
            exit 0
        fi
    fi
}

# 生成时间戳
generate_timestamp() {
    date +"%Y%m%d-%H%M%S"
}

# 更新 Cargo.toml 版本
update_cargo_version() {
    local version=$1
    log_info "更新 Cargo.toml 版本为 $version"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] 将更新 Cargo.toml 版本为 $version"
        return
    fi
    
    # 使用 sed 更新版本号
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS 需要特殊处理
        sed -i '' "s/^version = \".*\"/version = \"$version\"/" "$CARGO_FILE"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$version\"/" "$CARGO_FILE"
    fi
    
    log_success "Cargo.toml 版本已更新"
}

# 更新 CHANGELOG.md
update_changelog() {
    local version=$1
    local timestamp=$(generate_timestamp)
    local tag_name="v$version-$timestamp"
    
    log_info "更新 CHANGELOG.md..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] 将更新 CHANGELOG.md，标签: $tag_name"
        echo "$tag_name"
        return
    fi
    
    # 创建临时文件
    local temp_file=$(mktemp)
    
    # 读取当前 CHANGELOG 内容
    local changelog_content=$(cat "$CHANGELOG_FILE")
    
    # 替换 [未发布] 部分
    local new_release_section="## v$version-$timestamp

### 新增
- 发布版本 $version

### 变更
- 无

### 修复
- 无

## [未发布]

### 新增
- 添加了 GitHub Actions 自动发布工作流
- 添加了 GitLab CI/CD 自动发布配置
- 新增多架构交叉编译支持 (x86_64, aarch64)
- 集成 UPX 二进制文件压缩

### 变更
- 优化了构建脚本，增加了 CI/CD 专用版本

### 修复
- 无"
    
    # 替换内容
    echo "$changelog_content" | sed "s/## \[未发布\]/$new_release_section/" > "$temp_file"
    
    # 替换原文件
    mv "$temp_file" "$CHANGELOG_FILE"
    
    log_success "CHANGELOG.md 已更新"
    echo "$tag_name"
}

# 创建 Git 标签
create_git_tag() {
    local version=$1
    local tag_name=$2
    local message=${3:-"Release version $version"}
    
    log_info "创建 Git 标签: $tag_name"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] 将创建标签: $tag_name"
        log_info "[DRY RUN] 标签消息: $message"
        return
    fi
    
    # 添加所有更改
    git add .
    
    # 提交更改
    git commit -m "chore: release version $version"
    
    # 创建标签
    git tag -a "$tag_name" -m "$message"
    
    log_success "Git 标签已创建: $tag_name"
}

# 推送到 GitHub
push_to_github() {
    local tag_name=$1
    
    log_info "推送到 GitHub..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] 将推送标签到 GitHub: $tag_name"
        return
    fi
    
    # 推送提交
    git push origin main
    
    # 推送标签
    git push origin "$tag_name"
    
    log_success "已推送到 GitHub"
}

# 生成发布说明
generate_release_notes() {
    local version=$1
    local tag_name=$2
    
    log_info "生成发布说明..."
    
    # 从 CHANGELOG 提取版本信息（支持 v0.0.1-20250101 格式）
    local release_notes=$(awk "/## v$version-/,/## v/" "$CHANGELOG_FILE" | head -n -1)
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] 发布说明预览:"
        echo "$release_notes"
        echo "release_notes_${version}.md"
        return
    fi
    
    # 保存到文件
    local notes_file="release_notes_${version}.md"
    echo "$release_notes" > "$notes_file"
    
    log_success "发布说明已保存到: $notes_file"
    echo "$notes_file"
}

# 主函数
main() {
    local version=""
    local message=""
    local dry_run=false
    
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                version="$2"
                shift 2
                ;;
            -m|--message)
                message="$2"
                shift 2
                ;;
            -d|--dry-run)
                dry_run=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 检查必需参数
    if [ -z "$version" ]; then
        log_error "必须指定版本号"
        show_help
        exit 1
    fi
    
    # 设置全局变量
    DRY_RUN=$dry_run
    
    # 验证版本号
    validate_version "$version"
    
    # 检查 CHANGELOG 更新信息
    check_changelog_updates "$version"
    
    # 显示发布信息
    log_info "开始发布流程..."
    log_info "版本号: $version"
    log_info "发布消息: ${message:-"Release version $version"}"
    if [ "$dry_run" = true ]; then
        log_warning "试运行模式 - 不会实际执行更改"
    fi
    echo
    
    # 检查 Git 状态
    check_git_status
    
    # 更新 Cargo.toml
    update_cargo_version "$version"
    
    # 更新 CHANGELOG
    local tag_name=$(update_changelog "$version")
    
    # 创建 Git 标签
    create_git_tag "$version" "$tag_name" "$message"
    
    # 推送到 GitHub
    push_to_github "$tag_name"
    
    # 生成发布说明
    local notes_file=$(generate_release_notes "$version" "$tag_name")
    
    # 完成
    echo
    log_success "发布完成!"
    log_info "版本: $version"
    log_info "标签: $tag_name"
    if [ "$dry_run" = false ]; then
        log_info "发布说明: $notes_file"
        log_info "GitHub 标签: https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\([^/]*\/[^/]*\).*/\1/')/releases/tag/$tag_name"
    fi
}

# 执行主函数
main "$@" 
