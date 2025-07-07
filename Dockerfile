# --- 构建阶段 ---
FROM python:3.10-slim-bookworm AS builder

WORKDIR /afdko

# 安装构建依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    build-essential && \
    rm -rf /var/lib/apt/lists/*

# 克隆并安装 AFDKO
RUN git clone --depth 1 https://github.com/adobe-type-tools/afdko.git .
RUN pip install --no-cache-dir -r requirements.txt
RUN pip install --no-cache-dir .

# --- 最终阶段 ---
FROM python:3.10-slim-bookworm

# 从构建阶段复制 Python 环境中已安装的包
COPY --from=builder /usr/local/lib/python3.10/site-packages /usr/local/lib/python3.10/site-packages
# 从构建阶段复制 AFDKO 的可执行文件
COPY --from=builder /usr/local/bin /usr/local/bin

# 设置工作目录
WORKDIR /work

# 设置默认命令
CMD ["bash"]

