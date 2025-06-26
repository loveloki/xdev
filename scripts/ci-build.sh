#!/bin/bash
#
# CI/CD 专用构建脚本
# 用于在持续集成环境中构建和打包二进制文件
#

set -e

# --- 配置 ---
PROJECT_NAME="xdev"
DIST_DIR="dist"

# 默认目标架构（如果没有通过环境变量指定）
DEFAULT_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
)

# 从环境变量获取目标架构，如果未设置则使用默认值
if [ -n "$BUILD_TARGET" ]; then
    TARGETS=("$BUILD_TARGET")
else
    TARGETS=("${DEFAULT_TARGETS[@]}")
fi

# --- 函数定义 ---

# 打印信息
print_info() {
    echo "🔧 $1"
}

print_success() {
    echo "✅ $1"
}

print_error() {
    echo "❌ $1" >&2
}

# 检查必要工具
check_tools() {
    print_info "检查必要工具..."
    
    if ! command -v cross &> /dev/null; then
        print_error "cross 未安装"
        exit 1
    fi
    
    print_success "所有必要工具已就绪"
}

# 构建单个目标
build_target() {
    local target=$1
    print_info "开始构建目标: $target"
    
    # 构建
    cross build --target "$target" --release
    
    # 创建 dist 目录
    mkdir -p "$DIST_DIR"
    
    # 复制二进制文件
    local source_path="target/$target/release/$PROJECT_NAME"
    local dest_path="$DIST_DIR/${PROJECT_NAME}-${target}"
    
    if [ -f "$source_path" ]; then
        cp "$source_path" "$dest_path"
        print_success "已复制 $target 的二进制文件到 $dest_path"
    else
        print_error "构建失败：找不到 $source_path"
        exit 1
    fi
    
    # 如果有 UPX，进行压缩
    if command -v upx &> /dev/null; then
        print_info "使用 UPX 压缩 $target 的二进制文件..."
        upx --best "$dest_path"
        print_success "$target 的二进制文件压缩完成"
    else
        print_info "UPX 不可用，跳过压缩"
    fi
}

# 显示构建结果
show_results() {
    print_success "构建完成！"
    echo ""
    echo "📦 构建产物:"
    if [ -d "$DIST_DIR" ]; then
        ls -lh "$DIST_DIR"/
    else
        print_error "dist 目录不存在"
    fi
}

# --- 主逻辑 ---
main() {
    print_info "开始 CI/CD 构建流程..."
    print_info "项目名称: $PROJECT_NAME"
    print_info "目标架构: ${TARGETS[*]}"
    
    # 检查工具
    check_tools
    
    # 清理旧的构建产物
    print_info "清理旧的构建产物..."
    rm -rf "$DIST_DIR"
    
    # 构建所有目标
    for target in "${TARGETS[@]}"; do
        build_target "$target"
    done
    
    # 显示结果
    show_results
    
    print_success "🎉 所有构建任务完成！"
}

# 如果直接执行脚本，运行主函数
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi 
