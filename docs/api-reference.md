# API 参考文档 | API Reference

## 概述 | Overview

Video2Audio-RS 提供了一套完整的 Rust API，可以作为库集成到其他项目中。本文档详细描述了所有公共 API 的使用方法和示例。

## 核心类型 | Core Types

### AudioFormat

音频格式枚举，定义了所有支持的输出格式。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,      // MP3 格式，VBR 最高质量
    AacCopy,  // AAC 格式，直接复制音频流
    Opus,     // Opus 格式，现代高效编码
}
```

#### 方法 | Methods

##### `extension(&self) -> &'static str`

获取音频格式对应的文件扩展名。

```rust
use video2audio_rs::AudioFormat;

assert_eq!(AudioFormat::Mp3.extension(), "mp3");
assert_eq!(AudioFormat::AacCopy.extension(), "aac");
assert_eq!(AudioFormat::Opus.extension(), "opus");
```

##### `ffmpeg_args(&self) -> Vec<&'static str>`

获取 FFmpeg 编码参数。

```rust
let mp3_args = AudioFormat::Mp3.ffmpeg_args();
// 返回: vec!["-q:a", "0"]

let aac_args = AudioFormat::AacCopy.ffmpeg_args();
// 返回: vec!["-c:a", "copy"]

let opus_args = AudioFormat::Opus.ffmpeg_args();
// 返回: vec!["-c:a", "libopus", "-b:a", "192k"]
```

##### `from_user_input(input: &str) -> Result<Self>`

从用户输入字符串解析音频格式。

```rust
use video2audio_rs::AudioFormat;

// 数字选择
let format1 = AudioFormat::from_user_input("1")?; // Mp3
let format2 = AudioFormat::from_user_input("2")?; // AacCopy
let format3 = AudioFormat::from_user_input("3")?; // Opus

// 格式名称（不区分大小写）
let format_mp3 = AudioFormat::from_user_input("mp3")?;
let format_aac = AudioFormat::from_user_input("AAC")?;
let format_opus = AudioFormat::from_user_input("opus")?;
```

##### `description(&self) -> &'static str`

获取格式的中文描述。

```rust
println!("{}", AudioFormat::Mp3.description());
// 输出: "MP3 (高质量, 最佳兼容性)"
```

##### `all_formats() -> Vec<Self>`

获取所有支持的音频格式。

```rust
let formats = AudioFormat::all_formats();
for format in formats {
    println!("{}: {}", format.extension(), format.description());
}
```

### VideoToAudioError

统一的错误类型，包含所有可能的错误情况。

```rust
#[derive(Debug)]
pub enum VideoToAudioError {
    Io(std::io::Error),           // I/O 操作错误
    FfmpegError(String),          // FFmpeg 执行错误
    InvalidPath(String),          // 文件路径错误
    InvalidInput(String),         // 用户输入错误
    UnsupportedFormat(String),    // 不支持的文件格式
    MissingDependency(String),    // 系统依赖缺失
}
```

#### 错误处理示例

```rust
use video2audio_rs::{FileProcessor, VideoToAudioError};

match processor.find_video_files(path) {
    Ok(files) => println!("找到 {} 个文件", files.len()),
    Err(VideoToAudioError::InvalidPath(msg)) => {
        eprintln!("路径错误: {}", msg);
    }
    Err(VideoToAudioError::Io(err)) => {
        eprintln!("I/O 错误: {}", err);
    }
    Err(err) => {
        eprintln!("其他错误: {}", err);
    }
}
```

### Result<T>

项目的结果类型别名。

```rust
pub type Result<T> = std::result::Result<T, VideoToAudioError>;
```

## 核心组件 | Core Components

### FileProcessor

文件处理器，负责视频文件的发现、验证和转换。

```rust
pub struct FileProcessor {
    // 内部字段...
}
```

#### 构造方法

##### `new() -> Self`

创建新的文件处理器实例。

```rust
use video2audio_rs::FileProcessor;

let processor = FileProcessor::new();
```

#### 方法 | Methods

##### `supported_extensions(&self) -> &[&'static str]`

获取支持的视频文件扩展名列表。

```rust
let extensions = processor.supported_extensions();
println!("支持的格式: {:?}", extensions);
// 输出: ["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "3gp", "ts"]
```

##### `find_video_files(&self, source_dir: &Path) -> Result<Vec<PathBuf>>`

在指定目录中查找所有支持的视频文件。

```rust
use std::path::Path;

let source_dir = Path::new("/path/to/videos");
let files = processor.find_video_files(source_dir)?;

println!("找到 {} 个视频文件:", files.len());
for file in &files {
    println!("  {}", file.display());
}
```

**错误情况**:
- `InvalidPath`: 目录不存在或不是有效目录
- `Io`: 目录访问权限不足或其他 I/O 错误

##### `create_output_directory(&self, source_dir: &Path) -> Result<PathBuf>`

创建输出目录。

```rust
let output_dir = processor.create_output_directory(source_dir)?;
println!("输出目录: {}", output_dir.display());
```

**行为**:
- 在源目录下创建 `audio_exports` 子目录
- 如果目录已存在，不会报错
- 自动创建必要的父目录

##### `batch_convert<F>(&self, files: &[PathBuf], output_dir: &Path, format: AudioFormat, progress_callback: F) -> (usize, usize)`

批量并行转换视频文件。

**参数**:
- `files`: 要转换的视频文件路径列表
- `output_dir`: 输出目录路径
- `format`: 目标音频格式
- `progress_callback`: 进度回调函数

**返回值**: `(成功数, 失败数)`

```rust
use video2audio_rs::{AudioFormat, FileProcessor};
use std::path::Path;

let processor = FileProcessor::new();
let files = processor.find_video_files(Path::new("/videos"))?;
let output_dir = processor.create_output_directory(Path::new("/videos"))?;

let (success, failure) = processor.batch_convert(
    &files,
    &output_dir,
    AudioFormat::Mp3,
    |current, total| {
        println!("进度: {}/{} ({:.1}%)", 
                current, total, 
                current as f64 / total as f64 * 100.0);
    },
);

println!("转换完成: 成功 {}, 失败 {}", success, failure);
```

##### `convert_single_file(&self, source_file: &Path, output_dir: &Path, format: AudioFormat) -> Result<PathBuf>`

转换单个视频文件为音频。

```rust
use std::path::Path;

let source_file = Path::new("/videos/sample.mp4");
let output_dir = Path::new("/videos/audio_exports");

match processor.convert_single_file(source_file, output_dir, AudioFormat::Mp3) {
    Ok(output_path) => {
        println!("转换成功: {}", output_path.display());
    }
    Err(err) => {
        eprintln!("转换失败: {}", err);
    }
}
```

**错误情况**:
- `InvalidPath`: 源文件不存在或路径包含无效字符
- `MissingDependency`: FFmpeg 未安装或不可用
- `FfmpegError`: FFmpeg 执行失败

### UserInterface

用户界面管理器，处理所有用户交互逻辑。

```rust
pub struct UserInterface;
```

#### 构造方法

##### `new() -> Self`

创建新的用户界面实例。

```rust
use video2audio_rs::UserInterface;

let ui = UserInterface::new();
```

#### 方法 | Methods

##### `show_welcome(&self)`

显示程序欢迎信息。

```rust
ui.show_welcome();
```

##### `get_user_input(&self, prompt: &str) -> Result<String>`

获取用户输入。

```rust
let input = ui.get_user_input("请输入文件路径: ")?;
println!("用户输入: {}", input);
```

##### `select_audio_format(&self) -> Result<AudioFormat>`

让用户选择音频格式。

```rust
let format = ui.select_audio_format()?;
println!("选择的格式: {}", format.description());
```

##### `get_source_directory(&self) -> Result<String>`

获取并验证源目录路径。

```rust
let source_dir = ui.get_source_directory()?;
println!("源目录: {}", source_dir);
```

##### `show_progress(&self, current: usize, total: usize)`

显示处理进度。

```rust
for i in 0..=100 {
    ui.show_progress(i, 100);
    std::thread::sleep(std::time::Duration::from_millis(50));
}
```

##### `show_completion(&self, total_files: usize, output_dir: &Path)`

显示处理完成信息。

```rust
ui.show_completion(files.len(), &output_dir);
```

##### `show_error(&self, error: &VideoToAudioError)`

显示错误信息。

```rust
if let Err(err) = some_operation() {
    ui.show_error(&err);
}
```

## 完整使用示例 | Complete Usage Example

```rust
use video2audio_rs::{AudioFormat, FileProcessor, UserInterface, VideoToAudioError};
use std::path::Path;

fn main() -> Result<(), VideoToAudioError> {
    // 初始化组件
    let ui = UserInterface::new();
    let processor = FileProcessor::new();

    // 显示欢迎界面
    ui.show_welcome();

    // 获取用户输入
    let source_dir = ui.get_source_directory()?;
    let format = ui.select_audio_format()?;

    // 处理文件
    let source_path = Path::new(&source_dir);
    let files = processor.find_video_files(source_path)?;
    let output_dir = processor.create_output_directory(source_path)?;

    // 显示发现的文件
    ui.show_files_found(files.len(), &output_dir);

    if files.is_empty() {
        return Ok(());
    }

    // 执行转换
    let (success, failure) = processor.batch_convert(
        &files,
        &output_dir,
        format,
        |current, total| ui.show_progress(current, total),
    );

    // 显示结果
    ui.show_completion(files.len(), &output_dir);
    
    if failure > 0 {
        println!("统计: 成功 {}, 失败 {}", success, failure);
    }

    Ok(())
}
```

## 错误处理最佳实践 | Error Handling Best Practices

### 1. 使用 ? 操作符

```rust
fn process_videos() -> Result<()> {
    let processor = FileProcessor::new();
    let files = processor.find_video_files(Path::new("/videos"))?;
    let output_dir = processor.create_output_directory(Path::new("/videos"))?;
    // ... 其他操作
    Ok(())
}
```

### 2. 模式匹配处理特定错误

```rust
match processor.convert_single_file(source, output, format) {
    Ok(path) => println!("成功: {}", path.display()),
    Err(VideoToAudioError::MissingDependency(msg)) => {
        eprintln!("依赖缺失: {}", msg);
        eprintln!("请安装 FFmpeg");
    }
    Err(VideoToAudioError::FfmpegError(msg)) => {
        eprintln!("转换失败: {}", msg);
    }
    Err(err) => eprintln!("其他错误: {}", err),
}
```

### 3. 错误链传播

```rust
use std::error::Error;

fn handle_error(err: &VideoToAudioError) {
    eprintln!("错误: {}", err);
    
    // 打印错误链
    let mut source = err.source();
    while let Some(err) = source {
        eprintln!("原因: {}", err);
        source = err.source();
    }
}
```

## 性能优化建议 | Performance Tips

### 1. 批量处理

```rust
// 推荐：批量处理
let (success, failure) = processor.batch_convert(&files, &output_dir, format, callback);

// 不推荐：逐个处理
for file in files {
    processor.convert_single_file(&file, &output_dir, format)?;
}
```

### 2. 选择合适的音频格式

```rust
// 最快速度（如果源文件是 AAC）
let format = AudioFormat::AacCopy;

// 最佳兼容性
let format = AudioFormat::Mp3;

// 最小文件大小
let format = AudioFormat::Opus;
```

### 3. 预先验证

```rust
// 在开始转换前验证所有输入
let files = processor.find_video_files(source_dir)?;
if files.is_empty() {
    return Err(VideoToAudioError::InvalidInput("未找到视频文件".to_string()));
}
```

这个 API 设计注重易用性、类型安全和错误处理，为开发者提供了灵活而强大的视频转音频功能。
