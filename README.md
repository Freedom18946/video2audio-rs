# Video2Audio-RS | 视频转音频工具

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/your-username/video2audio-rs)

**高性能的批量视频转音频工具 | High-Performance Batch Video-to-Audio Converter**

*基于 Rust 开发，支持多种格式，利用多核并行处理*

[功能特性](#功能特性) • [快速开始](#快速开始) • [使用指南](#使用指南) • [API 文档](#api-文档) • [贡献指南](#贡献指南)

</div>

---

## 📖 项目简介 | Project Overview

Video2Audio-RS 是一个使用 Rust 编写的现代化命令行工具，专门用于批量将视频文件转换为高质量音频文件。该工具采用模块化架构设计，提供友好的中文用户界面，并充分利用多核 CPU 性能进行并行处理。

**核心优势:**
- 🚀 **极致性能**: 基于 Rayon 的并行处理，充分利用多核 CPU
- 🎵 **多格式支持**: 支持 MP3、AAC、Opus 等主流音频格式
- 🛡️ **稳定可靠**: 完善的错误处理和用户友好的提示信息
- 🔧 **易于使用**: 直观的中文交互界面，无需复杂配置
- 📦 **模块化设计**: 清晰的代码架构，便于维护和扩展

## ✨ 功能特性 | Features

### 🎯 核心功能
- **批量处理**: 自动递归扫描目录，支持大规模文件批量转换
- **智能格式检测**: 自动识别支持的视频格式 (MP4, MKV, AVI, MOV, WEBM, FLV, WMV, M4V, 3GP, TS)
- **多音频格式输出**:
  - **MP3**: 高质量 VBR 编码，最佳兼容性
  - **AAC**: 直接复制模式，零损耗快速转换
  - **Opus**: 现代化高效编码，优秀压缩比
- **并行处理**: 利用 Rayon 库实现多线程并发，显著提升转换速度
- **实时进度显示**: 清晰的进度条和统计信息
- **智能输出管理**: 自动创建 `audio_exports` 目录，避免文件混乱
- **冲突安全策略**: 支持 `skip/rename/overwrite/error`，默认 `rename` 避免静默覆盖
- **Dry-run 与审计报告**: 支持 `--dry-run` 预演和 `--report` 导出 JSON/CSV 报告
- **稳健性增强**: 支持扫描容错、FFmpeg 超时和并发上限控制

### 🛠️ 技术特性
- **内存安全**: Rust 语言保证的内存安全和线程安全
- **错误恢复**: 单个文件失败不影响整体处理流程
- **依赖检查**: 自动检测 FFmpeg 可用性并提供安装指导
- **跨平台支持**: 支持 Windows、macOS、Linux 等主流操作系统

## 📁 项目结构 | Project Structure

```
video2audio-rs/
├── 📄 Cargo.toml              # 项目配置和依赖管理
├── 📄 Cargo.lock              # 依赖版本锁定文件
├── 📄 README.md               # 项目说明文档
├── 📁 src/                    # 源代码目录
│   ├── 📄 lib.rs              # 库入口文件
│   ├── 📄 main.rs             # 主程序入口
│   ├── 📄 audio_format.rs     # 音频格式定义模块
│   ├── 📄 error.rs            # 错误处理模块
│   ├── 📄 file_processor.rs   # 文件处理核心模块
│   └── 📄 user_interface.rs   # 用户界面交互模块
├── 📁 docs/                   # 详细文档目录
│   ├── 📄 architecture.md     # 架构设计文档
│   ├── 📄 api-reference.md    # API 参考文档
│   ├── 📄 user-guide.md       # 用户使用指南
│   └── 📄 contributing.md     # 贡献指南
├── 📁 tests/                  # 测试文件目录
│   ├── 📄 integration_tests.rs # 集成测试
│   └── 📄 common/             # 测试工具模块
├── 📁 examples/               # 使用示例目录
│   └── 📄 basic_usage.rs      # 基本使用示例
├── 📁 benches/                # 性能基准测试
│   └── 📄 conversion_bench.rs # 转换性能测试
└── 📁 assets/                 # 资源文件目录
    └── 📄 sample_videos/      # 示例视频文件
```

### 🏗️ 模块架构说明

- **`lib.rs`**: 库的主入口，导出公共 API 和类型定义
- **`main.rs`**: 命令行程序入口，协调各模块完成转换流程
- **`audio_format.rs`**: 音频格式枚举和相关方法，支持 MP3/AAC/Opus
- **`error.rs`**: 统一的错误类型定义和处理机制
- **`file_processor.rs`**: 文件发现、验证和转换的核心逻辑
- **`user_interface.rs`**: 用户交互界面，包括输入获取和进度显示

## 🚀 快速开始 | Quick Start

### 📋 系统要求 | Prerequisites

- **Rust 工具链**: 1.70.0 或更高版本
- **FFmpeg**: 必须安装并添加到系统 PATH 中
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 18.04+)

### 🔧 安装 FFmpeg | Installing FFmpeg

FFmpeg 是本工具的核心依赖，用于执行实际的媒体转换操作。

#### macOS
```bash
# 使用 Homebrew (推荐)
brew install ffmpeg

# 使用 MacPorts
sudo port install ffmpeg
```

#### Windows
```powershell
# 使用 Chocolatey (推荐)
choco install ffmpeg

# 使用 Scoop
scoop install ffmpeg

# 使用 winget
winget install Gyan.FFmpeg
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install ffmpeg

# CentOS/RHEL/Fedora
sudo dnf install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg
```

### 📦 安装和编译 | Installation & Build

#### 方法一：从源码编译 (推荐)

```bash
# 1. 克隆仓库
git clone https://github.com/your-username/video2audio-rs.git
cd video2audio-rs

# 2. 编译项目 (Release 模式获得最佳性能)
cargo build --release

# 3. 运行程序
./target/release/video2audio-rs
```

#### 方法二：直接安装

```bash
# 从 crates.io 安装 (如果已发布)
cargo install video2audio-rs

# 从 Git 仓库安装
cargo install --git https://github.com/your-username/video2audio-rs.git
```

### 🎮 基本使用 | Basic Usage

启动程序后，按照中文提示进行操作：

```bash
$ ./target/release/video2audio-rs

╔══════════════════════════════════════════════════════════════╗
║                批量视频转音频工具 (高并发版)                 ║
║                   Video2Audio-RS v0.1.0                     ║
╚══════════════════════════════════════════════════════════════╝

🎵 支持多种视频格式转换为高质量音频文件
⚡ 利用多核 CPU 并行处理，大幅提升转换速度
🛠️  基于 FFmpeg 引擎，确保转换质量和兼容性

📁 请指定要处理的视频文件夹:
   提示: 程序会自动扫描该文件夹及其所有子文件夹

请输入文件夹的完整路径: /path/to/your/videos

┌─────────────────────────────────────────────────────────────┐
│                    请选择目标音频格式                        │
├─────────────────────────────────────────────────────────────┤
│  1. MP3 (高质量, 最佳兼容性)                                │
│  2. AAC (直接复制, 速度最快, 零损耗)                        │
│  3. Opus (现代化, 高效率)                                   │
└─────────────────────────────────────────────────────────────┘

请输入选项 (1-3): 1
```

## 📚 使用指南 | User Guide

### 🎯 支持的文件格式

**输入格式 (视频)**:
- MP4, MKV, AVI, MOV, WEBM, FLV, WMV
- M4V, 3GP, TS 等主流视频格式

**输出格式 (音频)**:
- **MP3**: 使用 VBR 最高质量设置，兼容性最佳
- **AAC**: 直接复制音频流，速度最快，零损耗
- **Opus**: 现代化编码，压缩效率高，适合网络传输

### 📊 性能优化建议

1. **硬件配置**: 多核 CPU 能显著提升并行处理性能
2. **存储设备**: SSD 硬盘能加快文件读写速度
3. **内存容量**: 建议至少 4GB RAM 用于大批量处理
4. **格式选择**: AAC 复制模式速度最快，适合快速提取

### 🔧 高级用法

#### CLI 高级参数

```bash
# 1) 批处理 + 冲突自动重命名（默认）
video2audio-rs --batch --source /videos --format mp3 --on-conflict rename

# 2) 只做预演，不实际转码，并输出 JSON 报告
video2audio-rs --batch --source /videos --format opus --dry-run --report ./plan.json

# 3) 跳过已存在文件，并输出 CSV 报告
video2audio-rs --batch --source /videos --format mp3 --skip-existing --report ./result.csv

# 4) 扫描容错 + 限制 FFmpeg 并发 + 超时保护
video2audio-rs --batch --source /videos --format aac \
  --ignore-scan-errors --max-parallel-ffmpeg 4 --ffmpeg-timeout 120

# 5) 保存当前参数到配置文件
video2audio-rs --batch --source /videos --format mp3 \
  --on-conflict rename --config ~/.config/video2audio-rs/config.json --save-config
```

#### 作为库使用

```rust
use video2audio_rs::{AudioFormat, FileProcessor};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = FileProcessor::new();

    // 查找视频文件
    let files = processor.find_video_files(Path::new("/path/to/videos"))?;

    // 创建输出目录
    let output_dir = processor.create_output_directory(Path::new("/path/to/videos"))?;

    // 批量转换
    let (success, failure) = processor.batch_convert(
        &files,
        &output_dir,
        AudioFormat::Mp3,
        |current, total| {
            println!("进度: {}/{}", current, total);
        },
    );

    println!("转换完成: 成功 {}, 失败 {}", success, failure);
    Ok(())
}
```

## 🔍 故障排除 | Troubleshooting

### 常见问题

**Q: 提示 "FFmpeg 未安装或不在系统 PATH 中"**
A: 请按照上述说明安装 FFmpeg，并确保可以在命令行中运行 `ffmpeg -version`

**Q: 转换速度很慢**
A: 检查 CPU 核心数和硬盘类型，考虑使用 AAC 复制模式以获得最快速度

**Q: 某些文件转换失败**
A: 检查源文件是否损坏，或尝试使用其他音频格式；也可增加 `--ffmpeg-timeout`、`--report` 定位问题

**Q: 内存使用过高**
A: 大批量处理时属于正常现象，程序会自动管理内存使用

### 错误代码说明

- **文件操作错误**: 检查文件权限和磁盘空间
- **FFmpeg 执行错误**: 检查源文件格式和完整性
- **无效路径**: 确保输入的路径存在且可访问
- **不支持的格式**: 查看支持的文件格式列表

## 📖 API 文档 | API Documentation

详细的 API 文档请查看：
- [在线文档](https://docs.rs/video2audio-rs) (如果已发布到 crates.io)
- [本地文档](#生成本地文档)

### 生成本地文档

```bash
# 生成并打开文档
cargo doc --open

# 仅生成文档
cargo doc --no-deps
```

## 🧪 测试 | Testing

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration_tests

# 运行性能基准测试
cargo bench
```

## 🤝 贡献指南 | Contributing

我们欢迎各种形式的贡献！请查看 [CONTRIBUTING.md](docs/contributing.md) 了解详细信息。

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/your-username/video2audio-rs.git
cd video2audio-rs

# 安装开发依赖
cargo install cargo-watch cargo-tarpaulin

# 运行开发模式 (自动重新编译)
cargo watch -x check -x test -x run
```

### 提交规范

- 使用清晰的提交信息
- 遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范
- 确保所有测试通过
- 添加必要的文档和测试

## 📄 许可证 | License

本项目采用双许可证：

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

您可以选择其中任一许可证使用本项目。

## 🙏 致谢 | Acknowledgments

- [FFmpeg](https://ffmpeg.org/) - 强大的多媒体处理框架
- [Rayon](https://github.com/rayon-rs/rayon) - 数据并行处理库
- [Walkdir](https://github.com/BurntSushi/walkdir) - 目录遍历工具
- Rust 社区的所有贡献者

---

<div align="center">

**如果这个项目对您有帮助，请考虑给它一个 ⭐**

[报告问题](https://github.com/your-username/video2audio-rs/issues) • [功能请求](https://github.com/your-username/video2audio-rs/issues) • [讨论](https://github.com/your-username/video2audio-rs/discussions)

</div>
