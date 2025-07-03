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
    echo "正在安装 UPX..."
    
    if [ "$CURRENT_OS" = "Darwin" ]; then
        # macOS 使用 Homebrew
        if command -v brew &> /dev/null; then
            brew install upx
        else
            echo "警告: 未找到 Homebrew，请手动安装 UPX"
            echo "安装 Homebrew: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
            echo "或从 https://upx.github.io/ 下载并安装 UPX"
            return 1
        fi
    else
        # Linux 使用各种包管理器
        local pkg_manager=$(detect_package_manager)
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
    fi
}

# 安装 cross (如果需要构建 Linux 目标)
install_cross() {
    # 检查是否有 Linux 目标需要构建
    local has_linux_target=false
    for target in "${TARGETS[@]}"; do
        if [[ $target == *"unknown-linux-gnu" ]]; then
            has_linux_target=true
            break
        fi
    done
    
    if [ "$has_linux_target" = true ]; then
        echo "正在安装 cross（Linux 目标需要）..."
        if command -v cargo &> /dev/null; then
            cargo install cross --git https://github.com/cross-rs/cross
        else
            echo "错误: cargo 未找到。请先安装 Rust 和 Cargo。" >&2
            echo "安装指南: https://rustup.rs/" >&2
            exit 1
        fi
    else
        echo "无 Linux 目标，跳过 cross 安装"
    fi
}

# 检查并安装所需工具
check_and_install_tools() {
    echo "--- 检查并安装所需工具 ---"
    echo "当前系统: $CURRENT_OS"
    
    # 检查是否有 Linux 目标需要 cross
    local has_linux_target=false
    for target in "${TARGETS[@]}"; do
        if [[ $target == *"unknown-linux-gnu" ]]; then
            has_linux_target=true
            break
        fi
    done
    
    # 检查并安装 cross（如果需要）
    if [ "$has_linux_target" = true ]; then
        if ! command -v cross &> /dev/null; then
            echo "'cross' 命令未找到，正在安装..."
            install_cross
            echo "cross 安装完成。"
        else
            echo "✓ cross 已安装"
        fi
    else
        echo "无 Linux 目标，跳过 cross 检查"
    fi
    
    # 检查 Rust 目标
    echo "检查 Rust 目标..."
    for target in "${TARGETS[@]}"; do
        if ! rustup target list --installed | grep -q "$target"; then
            echo "正在添加目标: $target"
            rustup target add "$target"
        else
            echo "✓ 目标 $target 已安装"
        fi
    done
    
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

# 检测当前系统类型
CURRENT_OS="$(uname -s)"

# 解析命令行参数
BUILD_ALL=false
BUILD_LINUX_ONLY=false
BUILD_MACOS_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            BUILD_ALL=true
            shift
            ;;
        --linux-only)
            BUILD_LINUX_ONLY=true
            shift
            ;;
        --macos-only)
            BUILD_MACOS_ONLY=true
            shift
            ;;
        --help|-h)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --all         构建所有平台的二进制文件"
            echo "  --linux-only  只构建 Linux 平台的二进制文件"
            echo "  --macos-only  只构建 macOS 平台的二进制文件"
            echo "  --help, -h    显示此帮助信息"
            echo ""
            echo "默认行为: 构建所有平台的二进制文件"
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            echo "使用 --help 查看可用选项"
            exit 1
            ;;
    esac
done

# 设置构建目标
LINUX_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
)

MACOS_TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# 根据参数和系统类型确定要构建的目标
if [ "$BUILD_LINUX_ONLY" = true ]; then
    TARGETS=("${LINUX_TARGETS[@]}")
    echo "将只构建 Linux 平台的二进制文件"
elif [ "$BUILD_MACOS_ONLY" = true ]; then
    TARGETS=("${MACOS_TARGETS[@]}")
    echo "将只构建 macOS 平台的二进制文件"
else
    # 默认构建所有平台或 --all 被指定
    TARGETS=("${LINUX_TARGETS[@]}" "${MACOS_TARGETS[@]}")
    echo "将构建所有平台的二进制文件"
fi

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
FAILED_TARGETS=()
SUCCESSFUL_TARGETS=()

for target in "${TARGETS[@]}"; do
    echo ""
    echo "--- 为目标架构编译: $target ---"
    
    # 标记构建是否成功
    BUILD_SUCCESS=true
    
    # 根据目标类型选择构建方法
    if [[ $target == *"unknown-linux-gnu" ]]; then
        # Linux 目标：使用 cross
        echo "使用 cross 进行交叉编译 (Linux 目标)..."
        if ! command -v cross &> /dev/null; then
            echo "错误: cross 命令未找到，但需要构建 Linux 目标。请先安装 cross。"
            FAILED_TARGETS+=("$target: 缺少 cross 工具")
            BUILD_SUCCESS=false
        else
            if ! cross build --target "$target" --release; then
                echo "错误: cross 构建失败"
                FAILED_TARGETS+=("$target: cross 构建失败")
                BUILD_SUCCESS=false
            fi
        fi
    elif [[ $target == *"apple-darwin" ]]; then
        # macOS 目标：使用原生 cargo
        echo "使用 cargo 进行编译 (macOS 目标)..."
        
        # 检查是否在 macOS 上运行（对于 macOS 目标）
        if [ "$CURRENT_OS" != "Darwin" ]; then
            echo "警告: 在非 macOS 系统上构建 macOS 目标可能失败"
            echo "当前系统: $CURRENT_OS，目标: $target"
            echo "将跳过此目标，建议在 macOS 系统上构建"
            FAILED_TARGETS+=("$target: 需要在 macOS 系统上构建")
            BUILD_SUCCESS=false
        else
            if ! cargo build --target "$target" --release; then
                echo "错误: cargo 构建失败"
                FAILED_TARGETS+=("$target: cargo 构建失败")
                BUILD_SUCCESS=false
            fi
        fi
    else
        echo "未知目标类型: $target，使用默认 cargo 构建..."
        if ! cargo build --target "$target" --release; then
            echo "错误: 构建失败"
            FAILED_TARGETS+=("$target: 构建失败")
            BUILD_SUCCESS=false
        fi
    fi

    # 如果构建成功，复制二进制文件
    if [ "$BUILD_SUCCESS" = true ]; then
        echo "--- 复制 $target 的二进制文件 ---"
        SOURCE_PATH="target/$target/release/$PROJECT_NAME"
        DEST_PATH="$DIST_DIR/${PROJECT_NAME}-${target}"

        if [ ! -f "$SOURCE_PATH" ]; then
            echo "错误: 未找到构建的二进制文件: $SOURCE_PATH"
            FAILED_TARGETS+=("$target: 二进制文件不存在")
        else
            cp "$SOURCE_PATH" "$DEST_PATH"
            echo "已将 $target 的二进制文件复制到 $DEST_PATH"
            SUCCESSFUL_TARGETS+=("$target")
        fi
    fi
done

echo ""
echo "--- 构建总结 ---"

if [ ${#SUCCESSFUL_TARGETS[@]} -gt 0 ]; then
    echo "✅ 成功构建的目标 (${#SUCCESSFUL_TARGETS[@]}):"
    for target in "${SUCCESSFUL_TARGETS[@]}"; do
        echo "   ✓ $target"
    done
    echo ""
    echo "'$DIST_DIR' 目录中的二进制文件:"
    ls -l "$DIST_DIR"
else
    echo "❌ 没有成功构建任何目标"
fi

if [ ${#FAILED_TARGETS[@]} -gt 0 ]; then
    echo ""
    echo "❌ 失败的目标 (${#FAILED_TARGETS[@]}):"
    for target in "${FAILED_TARGETS[@]}"; do
        echo "   ✗ $target"
    done
    echo ""
    echo "💡 建议:"
    echo "   - 对于 macOS 目标：在 macOS 系统上运行构建"
    echo "   - 对于 Linux 目标：确保已安装 cross 工具"
    echo "   - 使用 --linux-only 或 --macos-only 只构建特定平台"
fi

# 使用 UPX 压缩二进制文件（如果可用）
if command -v upx &> /dev/null; then
    echo ""
    echo "--- 使用 UPX 压缩二进制文件 ---"
    
    # 压缩每个二进制文件，单独处理以避免一个失败影响其他
    for binary in "$DIST_DIR"/*; do
        if [ -f "$binary" ]; then
            echo "正在压缩: $(basename "$binary")"
            echo "压缩前大小: $(ls -lh "$binary" | awk '{print $5}')"
            
            if upx --best "$binary" 2>/dev/null; then
                echo "压缩成功: $(basename "$binary")"
                echo "压缩后大小: $(ls -lh "$binary" | awk '{print $5}')"
            else
                echo "警告: 压缩失败 $(basename "$binary")，使用未压缩版本"
            fi
            echo ""
        fi
    done

    echo "--- 最终二进制文件 ---"
    ls -lh "$DIST_DIR"
else
    echo ""
    echo "注意: UPX 不可用，跳过二进制文件压缩步骤。"
fi 
