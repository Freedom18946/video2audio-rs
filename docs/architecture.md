# 架构设计文档 | Architecture Design

## 概述 | Overview

Video2Audio-RS 采用模块化架构设计，将功能按职责分离到不同的模块中。这种设计提高了代码的可维护性、可测试性和可扩展性。

## 整体架构 | Overall Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        用户界面层                            │
│                    (User Interface)                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              UserInterface                          │    │
│  │  - 欢迎界面显示                                      │    │
│  │  - 用户输入获取                                      │    │
│  │  - 格式选择菜单                                      │    │
│  │  - 进度显示                                         │    │
│  │  - 错误信息展示                                      │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                        业务逻辑层                            │
│                    (Business Logic)                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              FileProcessor                          │    │
│  │  - 文件发现和过滤                                    │    │
│  │  - 批量并行处理                                      │    │
│  │  - 转换任务调度                                      │    │
│  │  - 进度跟踪                                         │    │
│  │  - 错误处理                                         │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                        数据模型层                            │
│                     (Data Models)                          │
│                                                             │
│  ┌─────────────────┐  ┌─────────────────────────────────┐   │
│  │  AudioFormat    │  │         Error Types             │   │
│  │  - 格式定义     │  │  - VideoToAudioError           │   │
│  │  - 编码参数     │  │  - 统一错误处理                 │   │
│  │  - 扩展名映射   │  │  - 错误分类                     │   │
│  │  - 用户输入解析 │  │  - 上下文信息                   │   │
│  └─────────────────┘  └─────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                        外部依赖层                            │
│                   (External Dependencies)                  │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   FFmpeg    │  │   Rayon     │  │      Walkdir        │  │
│  │  - 媒体转换 │  │  - 并行处理 │  │  - 目录遍历         │  │
│  │  - 格式支持 │  │  - 线程池   │  │  - 文件过滤         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 模块详细设计 | Module Design Details

### 1. 用户界面模块 (UserInterface)

**职责**:
- 管理所有用户交互逻辑
- 提供友好的中文界面
- 处理用户输入验证
- 显示进度和状态信息

**核心方法**:
```rust
impl UserInterface {
    pub fn show_welcome(&self)                          // 显示欢迎界面
    pub fn get_source_directory(&self) -> Result<String> // 获取源目录
    pub fn select_audio_format(&self) -> Result<AudioFormat> // 选择音频格式
    pub fn show_progress(&self, current: usize, total: usize) // 显示进度
    pub fn show_completion(&self, total: usize, output_dir: &Path) // 显示完成信息
}
```

**设计原则**:
- 单一职责：只负责用户交互
- 无状态设计：不保存业务数据
- 错误友好：提供清晰的错误提示

### 2. 文件处理模块 (FileProcessor)

**职责**:
- 视频文件的发现和验证
- 并行转换任务的调度和执行
- 进度跟踪和错误恢复
- 输出目录管理

**核心方法**:
```rust
impl FileProcessor {
    pub fn find_video_files(&self, source_dir: &Path) -> Result<Vec<PathBuf>>
    pub fn create_output_directory(&self, source_dir: &Path) -> Result<PathBuf>
    pub fn batch_convert<F>(&self, files: &[PathBuf], output_dir: &Path, 
                           format: AudioFormat, progress_callback: F) -> (usize, usize)
    pub fn convert_single_file(&self, source_file: &Path, output_dir: &Path, 
                              format: AudioFormat) -> Result<PathBuf>
}
```

**并行处理设计**:
- 使用 Rayon 的 `par_iter()` 实现数据并行
- 每个文件独立处理，失败不影响其他文件
- 线程安全的进度计数器
- 自动负载均衡

### 3. 音频格式模块 (AudioFormat)

**职责**:
- 定义支持的音频格式
- 提供格式特定的编码参数
- 处理用户输入到格式的映射

**设计特点**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,      // VBR 最高质量
    AacCopy,  // 直接复制，零损耗
    Opus,     // 现代高效编码
}
```

**扩展性**:
- 新增格式只需添加枚举变体
- 自动支持格式验证和参数生成
- 类型安全的格式处理

### 4. 错误处理模块 (Error)

**职责**:
- 统一的错误类型定义
- 错误分类和上下文信息
- 用户友好的错误消息

**错误分类**:
```rust
pub enum VideoToAudioError {
    Io(std::io::Error),           // I/O 操作错误
    FfmpegError(String),          // FFmpeg 执行错误
    InvalidPath(String),          // 路径错误
    InvalidInput(String),         // 用户输入错误
    UnsupportedFormat(String),    // 不支持的格式
    MissingDependency(String),    // 依赖缺失
}
```

## 数据流设计 | Data Flow Design

```
用户启动程序
    │
    ▼
显示欢迎界面
    │
    ▼
获取源目录路径 ──────────────┐
    │                      │
    ▼                      │
验证目录有效性 ◄─────────────┘ (循环直到有效)
    │
    ▼
选择音频格式 ───────────────┐
    │                      │
    ▼                      │
验证格式选择 ◄──────────────┘ (循环直到有效)
    │
    ▼
创建输出目录
    │
    ▼
扫描视频文件 ──► 文件过滤 ──► 生成文件列表
    │
    ▼
显示扫描结果
    │
    ▼
并行转换处理 ──► 进度回调 ──► 实时进度显示
    │
    ▼
收集转换结果
    │
    ▼
显示完成统计
```

## 并发模型 | Concurrency Model

### 线程模型
- **主线程**: 用户界面和程序控制流
- **工作线程池**: Rayon 管理的并行处理线程
- **进度线程**: 实时更新处理进度

### 同步机制
```rust
// 进度计数器 - 线程安全
let progress_counter = Arc<Mutex<usize>>;

// 结果计数器 - 分别统计成功和失败
let success_counter = Arc<Mutex<usize>>;
let failure_counter = Arc<Mutex<usize>>;
```

### 错误隔离
- 单个文件转换失败不影响其他文件
- 错误信息异步输出到 stderr
- 主流程继续执行直到所有文件处理完成

## 性能优化策略 | Performance Optimization

### 1. I/O 优化
- 异步文件发现和过滤
- 批量目录操作
- 智能缓存机制

### 2. 内存管理
- 流式处理，避免大量文件同时加载到内存
- 及时释放不需要的资源
- 使用 Rust 的零成本抽象

### 3. 并行优化
- 自动检测 CPU 核心数
- 动态负载均衡
- 避免线程争用

### 4. FFmpeg 优化
- 最小化 FFmpeg 启动开销
- 优化编码参数
- 错误输出重定向

## 扩展性设计 | Extensibility Design

### 新增音频格式
1. 在 `AudioFormat` 枚举中添加新变体
2. 实现对应的 `ffmpeg_args()` 方法
3. 更新 `from_user_input()` 解析逻辑
4. 添加相应的测试用例

### 新增输入格式
1. 在 `FileProcessor` 中更新 `supported_extensions`
2. 添加格式特定的验证逻辑
3. 更新文档和帮助信息

### 新增处理模式
1. 扩展 `FileProcessor` 接口
2. 实现新的处理策略
3. 更新用户界面选项

## 测试策略 | Testing Strategy

### 单元测试
- 每个模块的核心功能
- 错误处理路径
- 边界条件验证

### 集成测试
- 端到端转换流程
- 多格式兼容性
- 并发安全性

### 性能测试
- 大批量文件处理
- 内存使用监控
- 转换速度基准

这种架构设计确保了代码的清晰性、可维护性和可扩展性，同时保持了高性能和用户友好性。
