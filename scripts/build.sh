#!/bin/bash
#
# 该脚本使用 `cross` 自动执行针对不同架构的交叉编译过程。
# 它会清理以前的构建，为每个指定的目标进行编译，
# 并将生成的二进制文件放入 `dist` 目录中。

# 如果命令以非零状态退出，则立即退出。
set -e

# --- 工具安装函数 ---

# 检测系统包管理器
detect_package_manager() {
    if command -v apt &> /dev/null; then
        echo "apt"
    elif command -v yum &> /dev/null; then
        echo "yum"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v zypper &> /dev/null; then
        echo "zypper"
    else
        echo "unknown"
    fi
}

# 安装 UPX
install_upx() {
    local pkg_manager=$(detect_package_manager)
    echo "正在安装 UPX..."
    
    case $pkg_manager in
        "apt")
            sudo apt update && sudo apt install -y upx-ucl
            ;;
        "yum")
            sudo yum install -y upx
            ;;
        "dnf")
            sudo dnf install -y upx
            ;;
        "pacman")
            sudo pacman -S --noconfirm upx
            ;;
        "zypper")
            sudo zypper install -y upx
            ;;
        *)
            echo "警告: 无法识别的包管理器，请手动安装 UPX"
            echo "您可以从 https://upx.github.io/ 下载并安装 UPX"
            return 1
            ;;
    esac
}

# 安装 cross
install_cross() {
    echo "正在安装 cross..."
    if command -v cargo &> /dev/null; then
        cargo install cross --git https://github.com/cross-rs/cross
    else
        echo "错误: cargo 未找到。请先安装 Rust 和 Cargo。" >&2
        echo "安装指南: https://rustup.rs/" >&2
        exit 1
    fi
}

# 检查并安装所需工具
check_and_install_tools() {
    echo "--- 检查并安装所需工具 ---"
    
    # 检查并安装 cross
    if ! command -v cross &> /dev/null; then
        echo "'cross' 命令未找到，正在安装..."
        install_cross
        echo "cross 安装完成。"
    else
        echo "✓ cross 已安装"
    fi
    
    # 检查并安装 upx
    if ! command -v upx &> /dev/null; then
        echo "'upx' 命令未找到，正在安装..."
        if install_upx; then
            echo "upx 安装完成。"
        else
            echo "upx 安装失败，构建将继续但不会压缩二进制文件。"
        fi
    else
        echo "✓ upx 已安装"
    fi
    
    echo ""
}

# --- 配置 ---
PROJECT_NAME="xdev"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
)
DIST_DIR="dist"

# --- 主脚本 ---
echo "--- 开始为 $PROJECT_NAME 项目执行构建流程 ---"

# 检查并安装所需工具
check_and_install_tools

# 清理以前的构建并创建 dist 目录
echo "--- 清理旧的构建产物并创建 '$DIST_DIR' 目录 ---"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 循环遍历每个目标并进行构建
for target in "${TARGETS[@]}"; do
    echo ""
    echo "--- 为目标架构编译: $target ---"
    cross build --target "$target" --release

    echo "--- 复制 $target 的二进制文件 ---"
    SOURCE_PATH="target/$target/release/$PROJECT_NAME"
    DEST_PATH="$DIST_DIR/${PROJECT_NAME}-${target}"

    cp "$SOURCE_PATH" "$DEST_PATH"
    echo "已将 $target 的二进制文件复制到 $DEST_PATH"
done

echo ""
echo "--- 所有构建均已成功完成！ ---"
echo "'$DIST_DIR' 目录中的二进制文件:"
ls -l "$DIST_DIR"

# 使用 UPX 压缩二进制文件（如果可用）
if command -v upx &> /dev/null; then
    echo ""
    echo "--- 使用 UPX 压缩二进制文件 ---"
    upx --best "$DIST_DIR"/*

    echo ""
    echo "--- 最终压缩后的二进制文件 ---"
    ls -l "$DIST_DIR"
else
    echo ""
    echo "注意: UPX 不可用，跳过二进制文件压缩步骤。"
fi 
