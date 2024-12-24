# Litemon (Lightweight Monitor)

一个使用 Rust 编写的轻量级系统资源监控工具。

## ✨ 功能特点

- 📊 实时监控系统资源使用情况
- 🎨 美观的终端用户界面（TUI）
- 🚀 低资源占用
- 📈 支持以下监控项目：
  - CPU 使用率和频率
  - GPU 状态（NVIDIA）
  - 内存使用情况
  - 磁盘使用情况
  - 网络流量

## 🔧 系统要求

- 支持的操作系统：Windows、Linux
- 如需 GPU 监控功能，需要 NVIDIA 显卡和驱动程序

## 📦 安装

### 从源码编译

1. 确保已安装 Rust 工具链：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. 克隆仓库：
```bash
git clone https://github.com/Ecluna/litemon.git
cd litemon
```
3. 编译并安装：
```bash
cargo install --path .
```
## 🚀 使用方法

直接运行程序：
```bash
litemon
```
### 快捷键

- `q`: 退出程序
- `↑/↓`: 滚动查看 CPU 核心信息

## 📊 监控项目说明

### CPU 监控
- CPU 型号信息
- 总体使用率和实时频率
- 每个核心的使用率和频率

### GPU 监控（NVIDIA）
- GPU 型号
- GPU 使用率和温度
- 显存使用情况

### 内存监控
- 物理内存使用情况
- 交换分区使用情况

### 磁盘监控
- 各分区使用情况
- 支持可移动设备

### 网络监控
- 实时网络速率
- 总流量统计

## 🔨 开发说明

### 依赖项目
```
toml
sysinfo = "0.29" # 系统信息收集
nvml-wrapper = "0.9" # NVIDIA GPU 监控
ratatui = "0.24" # 终端界面
crossterm = "0.27" # 终端控制
tokio = "1.32" # 异步运行时
clap = "4.4" # 命令行参数解析
```

### 项目结构
```
src/
├── main.rs # 程序入口
├── cli.rs # 命令行参数处理
├── error.rs # 错误处理
├── monitor/ # 监控模块
│ ├── mod.rs
│ ├── cpu.rs
│ ├── gpu.rs
│ ├── memory.rs
│ ├── disk.rs
│ └── network.rs
└── ui/ # 用户界面
└── mod.rs
```

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件