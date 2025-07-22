# 视频到音频转换器 (video2audio-rs)

这是一个使用 Rust 编写的高性能命令行工具，用于批量将指定目录下的视频文件转换为音频文件。它利用 `rayon` 库进行多线程并发处理，以最大化转换效率，并调用外部 `ffmpeg` 程序来执行实际的媒体转码操作。

## 功能特性

- **批量处理**: 自动扫描指定目录及其所有子目录下的视频文件。
- **多种格式支持**: 支持多种常见的视频输入格式 (如 .mp4, .mkv, .avi, .mov 等)。
- **音频格式选择**: 允许用户选择输出的音频格式，包括 MP3 (高质量)、AAC (直接复制/无损) 和 Opus (高效率)。
- **高并发**: 使用 Rayon 库并行处理文件，充分利用多核 CPU 性能，显著加快转换速度。
- **清晰的进度反馈**: 在处理过程中实时显示进度。
- **自动创建目录**: 转换后的音频文件会自动保存在源目录下的 `audio_exports` 子目录中。

## 项目结构解析

```
.
├── Cargo.lock          # (不可编译) Cargo 依赖锁文件，确保构建的可复现性。
├── Cargo.toml          # (不可编译) 项目配置文件，定义了元数据和依赖项。
├── .gitignore          # (不可编译) Git 忽略文件配置。
├── src/
│   └── main.rs         # (可编译) 核心源代码，包含了程序的所有逻辑。
└── target/             # (编译产物) 编译后生成的目标文件、缓存和最终的可执行程序。
```

- **可编译部分**:
  - `src/main.rs`: 这是项目唯一的源代码文件，包含了所有的业务逻辑，是 Cargo 的主要编译对象。

- **不可编译的配置/元数据**:
  - `Cargo.toml` / `Cargo.lock`: 这些是 Cargo 的配置文件，用于管理项目和依赖，它们指导编译过程，但本身不是源代码。
  - `.gitignore`: Git 的配置文件，与编译无关。

- **编译产物 (非迁移部分)**:
  - `target/`: 这个目录由 `cargo build` 命令生成，包含了所有中间编译文件和最终的可执行文件。在迁移或分发项目时，通常会忽略此目录，因为它可以在任何支持的环境中重新生成。

## 外部依赖

本项目依赖于 **`ffmpeg`**。您必须在您的操作系统上安装 `ffmpeg`，并确保其路径已添加到系统的 `PATH` 环境变量中，否则程序将无法执行转换。

### 如何安装 ffmpeg

- **macOS (使用 Homebrew)**:
  ```sh
  brew install ffmpeg
  ```
- **Windows (使用 Chocolatey)**:
  ```sh
  choco install ffmpeg
  ```
- **Linux (Ubuntu/Debian)**:
  ```sh
  sudo apt update && sudo apt install ffmpeg
  ```

## 如何编译和运行

### 1. 环境准备

确保您的系统上已安装 [Rust 工具链](https://www.rust-lang.org/tools/install) 和 `ffmpeg`。

### 2. 克隆项目 (如果适用)

```sh
git clone <repository_url>
cd video2audio-rs
```

### 3. 编译项目

在项目根目录下，运行以下命令进行编译。推荐使用 `release` 模式以获得最佳性能。

```sh
cargo build --release
```

编译成功后，可执行文件将位于 `./target/release/video2audio-rs`。

### 4. 运行程序

直接运行编译好的可执行文件：

```sh
./target/release/video2audio-rs
```

程序启动后，会提示您输入：
1.  **视频文件夹的绝对路径**: 您想要扫描的包含视频文件的文件夹路径。
2.  **目标音频格式**: 根据菜单提示输入数字 (1-3) 选择一种输出格式。

程序将开始转换，并在完成后将所有音频文件保存在源目录下的 `audio_exports` 文件夹中。
