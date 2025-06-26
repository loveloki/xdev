#!/bin/bash
#
# 发布流程测试脚本
# 用于验证 CI/CD 配置是否正确
#

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# 检查配置文件
check_config_files() {
    print_header "检查配置文件"
    
    local files=(
        ".github/workflows/release.yml"
        ".gitlab-ci.yml"
        "scripts/ci-build.sh"
        "CHANGELOG.md"
    )
    
    for file in "${files[@]}"; do
        if [ -f "$file" ]; then
            print_success "找到配置文件: $file"
        else
            print_error "缺少配置文件: $file"
            return 1
        fi
    done
}

# 检查工具依赖
check_tools() {
    print_header "检查构建工具"
    
    local tools=("cargo" "cross")
    local optional_tools=("upx")
    
    for tool in "${tools[@]}"; do
        if command -v "$tool" &> /dev/null; then
            print_success "$tool 已安装"
        else
            print_error "$tool 未安装 (必需)"
            return 1
        fi
    done
    
    for tool in "${optional_tools[@]}"; do
        if command -v "$tool" &> /dev/null; then
            print_success "$tool 已安装"
        else
            print_warning "$tool 未安装 (可选，用于压缩)"
        fi
    done
}

# 验证项目配置
check_project_config() {
    print_header "检查项目配置"
    
    # 检查 Cargo.toml
    if [ -f "Cargo.toml" ]; then
        local project_name
        project_name=$(grep '^name = ' Cargo.toml | cut -d'"' -f2)
        local version
        version=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
        
        print_success "项目名称: $project_name"
        print_success "当前版本: $version"
    else
        print_error "未找到 Cargo.toml 文件"
        return 1
    fi
    
    # 检查 Cross.toml
    if [ -f "Cross.toml" ]; then
        print_success "找到 Cross.toml 配置"
    else
        print_warning "未找到 Cross.toml 配置 (可选)"
    fi
}

# 测试构建流程
test_build() {
    print_header "测试构建流程"
    
    if [ -x "scripts/ci-build.sh" ]; then
        print_info "运行构建测试..."
        if BUILD_TARGET="x86_64-unknown-linux-gnu" ./scripts/ci-build.sh; then
            print_success "构建测试通过"
        else
            print_error "构建测试失败"
            return 1
        fi
    else
        print_error "构建脚本不可执行或不存在"
        return 1
    fi
}

# 检查 Git 配置
check_git_config() {
    print_header "检查 Git 配置"
    
    if git rev-parse --git-dir > /dev/null 2>&1; then
        print_success "Git 仓库已初始化"
        
        # 检查远程仓库
        local remotes
        remotes=$(git remote -v 2>/dev/null || true)
        if [ -n "$remotes" ]; then
            print_success "已配置远程仓库:"
            echo "$remotes" | while IFS= read -r line; do
                print_info "  $line"
            done
        else
            print_warning "未配置远程仓库"
        fi
        
        # 检查当前分支
        local branch
        branch=$(git branch --show-current 2>/dev/null || echo "未知")
        print_info "当前分支: $branch"
        
    else
        print_warning "不是 Git 仓库"
    fi
}

# 显示使用说明
show_usage() {
    print_header "发布使用说明"
    
    cat << EOF
要创建新的发布版本，请按以下步骤操作：

1. 更新版本号：
   编辑 Cargo.toml 中的 version 字段

2. 更新变更日志：
   在 CHANGELOG.md 中记录本版本的变更

3. 创建并推送标签：
   git tag v0.1.0
   git push origin v0.1.0

4. 自动构建和发布：
   推送标签后，GitHub Actions 和 GitLab CI/CD 将自动：
   - 构建多架构二进制文件
   - 创建 Release
   - 上传构建产物

5. 验证发布：
   检查 GitHub/GitLab 的 Releases 页面
EOF
}

# 主函数
main() {
    print_header "xdev 发布流程测试"
    echo ""
    
    local failed=false
    
    # 运行所有检查
    check_config_files || failed=true
    echo ""
    
    check_tools || failed=true
    echo ""
    
    check_project_config || failed=true
    echo ""
    
    check_git_config || failed=true
    echo ""
    
    # 如果前面的检查都通过，运行构建测试
    if [ "$failed" = false ]; then
        test_build || failed=true
        echo ""
    fi
    
    # 显示结果
    if [ "$failed" = false ]; then
        print_success "🎉 所有检查都通过了！发布配置已就绪。"
        echo ""
        show_usage
    else
        print_error "😞 一些检查失败了，请解决上述问题后重新运行。"
        return 1
    fi
}

# 运行主函数
main "$@" 
