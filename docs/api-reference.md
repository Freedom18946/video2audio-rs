# API 参考文档 | API Reference

## 概述 | Overview

Video2Audio-RS 提供完整的 Rust API，可作为库集成到其他项目。当前版本新增了更安全的冲突处理策略、dry-run 计划模式和结构化处理报告能力。

## 核心类型 | Core Types

### `AudioFormat`

支持的输出格式：

```rust
pub enum AudioFormat {
    Mp3,
    AacCopy,
    Opus,
}
```

常用方法：

- `extension(&self) -> &'static str`
- `ffmpeg_args(&self) -> Vec<&'static str>`
- `from_user_input(input: &str) -> Result<Self>`
- `description(&self) -> &'static str`
- `all_formats() -> Vec<Self>`

### `ConflictStrategy`

输出冲突处理策略：

```rust
pub enum ConflictStrategy {
    Skip,
    Rename,
    Overwrite,
    Error,
}
```

默认策略为 `Rename`（用于 CLI 配置），可避免静默覆盖。

### `VideoToAudioError`

统一错误类型：

```rust
pub enum VideoToAudioError {
    Io(std::io::Error),
    FfmpegError(String),
    InvalidPath(String),
    InvalidInput(String),
    UnsupportedFormat(String),
    MissingDependency(String),
}
```

## 配置类型 | Config Types

### `Args`

命令行参数结构，支持以下关键能力：

- `--on-conflict` 冲突策略
- `--skip-existing` 跳过已存在输出
- `--dry-run` 仅预演
- `--report` 输出 JSON/CSV 报告
- `--ignore-scan-errors` 扫描容错
- `--max-parallel-ffmpeg` 限制 FFmpeg 并发
- `--ffmpeg-timeout` 单文件超时（秒）

### `Config`

可序列化的配置结构（JSON），支持保存和加载：

- `load(config_path: Option<&PathBuf>) -> Result<Config>`
- `save(config_path: Option<&PathBuf>) -> Result<()>`
- `add_recent_source_dir(dir: &str)`
- `get_default_format() -> Result<AudioFormat>`
- `set_default_format(format: AudioFormat)`

### `RuntimeConfig`

命令行参数与配置文件合并后的运行时配置：

- `from_args_and_config(args: Args, config: Config) -> RuntimeConfig`
- `needs_interaction(&self) -> bool`
- `get_thread_count(&self) -> usize`
- `ffmpeg_timeout(&self) -> Option<Duration>`

## 文件处理 API | File Processing API

### `FileProcessor`

主处理器，负责扫描、规划、转换。

#### 构造与基础方法

```rust
let processor = FileProcessor::new();
let exts = processor.supported_extensions();
```

- `new() -> FileProcessor`
- `supported_extensions(&self) -> &[&'static str]`
- `create_output_directory(&self, source_dir: &Path) -> Result<PathBuf>`

#### 扫描方法

```rust
let files = processor.find_video_files(Path::new("/videos"))?;
```

- `find_video_files(&self, source_dir: &Path) -> Result<Vec<PathBuf>>`

可配置容错扫描：

```rust
let scan = processor.find_video_files_with_options(Path::new("/videos"), true)?;
println!("files={}, warnings={}", scan.files.len(), scan.warnings.len());
```

- `find_video_files_with_options(&self, source_dir: &Path, ignore_scan_errors: bool) -> Result<ScanResult>`

`ScanResult`:

```rust
pub struct ScanResult {
    pub files: Vec<PathBuf>,
    pub warnings: Vec<String>,
}
```

#### 批量转换（兼容旧 API）

```rust
let (success, failure) = processor.batch_convert(
    &files,
    &output_dir,
    AudioFormat::Mp3,
    |current, total| println!("{current}/{total}"),
);
```

- `batch_convert(&self, files: &[PathBuf], output_dir: &Path, format: AudioFormat, progress_callback) -> (usize, usize)`

#### 批量转换（新 API）

```rust
use std::time::Duration;
use video2audio_rs::{BatchOptions, ConflictStrategy};

let options = BatchOptions {
    skip_existing: true,
    conflict_strategy: ConflictStrategy::Rename,
    dry_run: false,
    ffmpeg_timeout: Some(Duration::from_secs(120)),
    max_parallel_ffmpeg: Some(4),
    quiet: false,
};

let summary = processor.batch_convert_with_options(
    &files,
    &output_dir,
    AudioFormat::Mp3,
    &options,
    |current, total| println!("{current}/{total}"),
)?;
```

- `batch_convert_with_options(&self, files: &[PathBuf], output_dir: &Path, format: AudioFormat, options: &BatchOptions, progress_callback) -> Result<BatchSummary>`

`BatchOptions`:

```rust
pub struct BatchOptions {
    pub skip_existing: bool,
    pub conflict_strategy: ConflictStrategy,
    pub dry_run: bool,
    pub ffmpeg_timeout: Option<Duration>,
    pub max_parallel_ffmpeg: Option<usize>,
    pub quiet: bool,
}
```

`BatchSummary`:

```rust
pub struct BatchSummary {
    pub total_files: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub skipped_count: usize,
    pub planned_count: usize,
    pub scan_warnings: Vec<String>,
    pub records: Vec<ConversionRecord>,
}
```

`ConversionRecord`:

```rust
pub struct ConversionRecord {
    pub input: String,
    pub output: Option<String>,
    pub status: String,      // converted/skipped/failed/planned
    pub reason: Option<String>,
    pub duration_ms: u128,
}
```

#### 单文件转换

```rust
let out = processor.convert_single_file(
    Path::new("/videos/demo.mp4"),
    Path::new("/videos/audio_exports"),
    AudioFormat::Mp3,
)?;
```

- `convert_single_file(&self, source_file: &Path, output_dir: &Path, format: AudioFormat) -> Result<PathBuf>`

## UserInterface API

`UserInterface` 负责交互和输出展示：

- `new() -> UserInterface`
- `show_welcome(&self)`
- `get_user_input(&self, prompt: &str) -> Result<String>`
- `select_audio_format(&self) -> Result<AudioFormat>`
- `get_source_directory(&self) -> Result<String>`
- `show_files_found(&self, file_count: usize, output_dir: &Path)`
- `show_progress(&self, current: usize, total: usize)`
- `show_completion(&self, total_files: usize, output_dir: &Path)`
- `show_error(&self, error: &VideoToAudioError)`

## 错误处理建议 | Error Handling Tips

- 对 `find_video_files_with_options(..., true)` 的返回结果，建议记录 `warnings` 到日志或报告。
- 对 `batch_convert_with_options`，建议始终保存 `BatchSummary.records` 便于后续审计。
- 若用于自动化任务，建议同时设置：`skip_existing`、`ffmpeg_timeout`、`max_parallel_ffmpeg`。
