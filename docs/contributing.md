# 贡献指南 | Contributing Guide

## 欢迎贡献 | Welcome Contributors

感谢您对 Video2Audio-RS 项目的关注！我们欢迎各种形式的贡献，包括但不限于：

- 🐛 报告 Bug
- 💡 提出新功能建议
- 📝 改进文档
- 🔧 提交代码修复
- 🎨 优化用户界面
- 🚀 性能优化
- 🧪 添加测试用例

## 开始之前 | Before You Start

### 行为准则 | Code of Conduct

参与本项目即表示您同意遵守我们的行为准则：

- 尊重所有参与者
- 使用友善和包容的语言
- 接受建设性的批评
- 专注于对社区最有利的事情
- 对其他社区成员表现出同理心

### 技术要求 | Technical Requirements

- **Rust**: 1.70.0 或更高版本
- **Git**: 基本的 Git 操作知识
- **FFmpeg**: 用于测试转换功能
- **编辑器**: 推荐使用支持 Rust 的 IDE (VS Code, IntelliJ IDEA, Vim 等)

## 开发环境设置 | Development Setup

### 1. 克隆仓库

```bash
# 克隆主仓库
git clone https://github.com/your-username/video2audio-rs.git
cd video2audio-rs

# 或者克隆您的 fork
git clone https://github.com/your-username/video2audio-rs.git
cd video2audio-rs
git remote add upstream https://github.com/original-owner/video2audio-rs.git
```

### 2. 安装开发工具

```bash
# 安装 Rust 工具链 (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装开发依赖工具
cargo install cargo-watch    # 自动重新编译
cargo install cargo-tarpaulin # 代码覆盖率
cargo install cargo-audit     # 安全审计
cargo install cargo-deny      # 依赖检查
```

### 3. 验证环境

```bash
# 检查编译
cargo check

# 运行测试
cargo test

# 运行 Clippy 检查
cargo clippy

# 格式化代码
cargo fmt
```

### 4. 开发模式运行

```bash
# 监视文件变化，自动重新编译和测试
cargo watch -x check -x test -x run

# 或者手动运行
cargo run
```

## 贡献流程 | Contribution Workflow

### 1. 创建 Issue (可选但推荐)

在开始编码之前，建议先创建一个 Issue 来讨论您的想法：

- 描述问题或功能需求
- 说明预期的解决方案
- 讨论实现方法
- 获得维护者的反馈

### 2. 创建分支

```bash
# 确保主分支是最新的
git checkout main
git pull upstream main

# 创建功能分支
git checkout -b feature/your-feature-name

# 或者修复分支
git checkout -b fix/issue-number-description
```

### 3. 开发和测试

```bash
# 进行您的修改...

# 运行测试确保没有破坏现有功能
cargo test

# 运行 Clippy 检查代码质量
cargo clippy -- -D warnings

# 格式化代码
cargo fmt

# 检查文档
cargo doc --no-deps
```

### 4. 提交更改

```bash
# 添加更改的文件
git add .

# 提交更改 (使用清晰的提交信息)
git commit -m "feat: 添加新的音频格式支持"

# 推送到您的 fork
git push origin feature/your-feature-name
```

### 5. 创建 Pull Request

1. 在 GitHub 上打开您的 fork
2. 点击 "New Pull Request"
3. 选择正确的分支
4. 填写 PR 模板
5. 等待代码审查

## 代码规范 | Code Standards

### Rust 代码风格

我们遵循标准的 Rust 代码风格：

```bash
# 使用 rustfmt 格式化代码
cargo fmt

# 使用 Clippy 检查代码质量
cargo clippy -- -D warnings
```

### 命名约定

- **函数和变量**: `snake_case`
- **类型和结构体**: `PascalCase`
- **常量**: `SCREAMING_SNAKE_CASE`
- **模块**: `snake_case`

### 文档注释

所有公共 API 都必须有文档注释：

```rust
/// 转换单个视频文件为音频
/// 
/// # 参数
/// 
/// * `source_file` - 源视频文件路径
/// * `output_dir` - 输出目录路径
/// * `format` - 目标音频格式
/// 
/// # 返回值
/// 
/// 成功时返回输出文件路径
/// 
/// # 错误
/// 
/// 当转换失败时返回相应的错误信息
/// 
/// # 示例
/// 
/// ```rust
/// use video2audio_rs::{FileProcessor, AudioFormat};
/// use std::path::Path;
/// 
/// let processor = FileProcessor::new();
/// let result = processor.convert_single_file(
///     Path::new("input.mp4"),
///     Path::new("output"),
///     AudioFormat::Mp3
/// )?;
/// ```
pub fn convert_single_file(
    &self,
    source_file: &Path,
    output_dir: &Path,
    format: AudioFormat,
) -> Result<PathBuf> {
    // 实现...
}
```

### 错误处理

- 使用 `Result<T, VideoToAudioError>` 类型
- 提供有意义的错误信息
- 包含足够的上下文信息

```rust
// 好的错误处理
if !source_file.exists() {
    return Err(VideoToAudioError::InvalidPath(
        format!("源文件不存在: {}", source_file.display())
    ));
}

// 不好的错误处理
if !source_file.exists() {
    return Err(VideoToAudioError::InvalidPath("文件不存在".to_string()));
}
```

## 测试指南 | Testing Guidelines

### 测试类型

1. **单元测试**: 测试单个函数或方法
2. **集成测试**: 测试模块间的交互
3. **端到端测试**: 测试完整的用户流程

### 编写测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audio_format_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::AacCopy.extension(), "aac");
        assert_eq!(AudioFormat::Opus.extension(), "opus");
    }

    #[test]
    fn test_file_processor_creation() {
        let processor = FileProcessor::new();
        assert!(!processor.supported_extensions().is_empty());
    }

    #[test]
    fn test_invalid_path_handling() {
        let processor = FileProcessor::new();
        let result = processor.find_video_files(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_audio_format

# 运行集成测试
cargo test --test integration_tests

# 生成代码覆盖率报告
cargo tarpaulin --out Html
```

## 文档贡献 | Documentation Contributions

### 文档类型

- **API 文档**: 代码中的文档注释
- **用户指南**: `docs/user-guide.md`
- **架构文档**: `docs/architecture.md`
- **README**: 项目主页说明

### 文档标准

- 使用简体中文为主，英文技术术语为辅
- 提供清晰的示例代码
- 包含必要的截图或图表
- 保持内容的时效性

### 生成文档

```bash
# 生成 API 文档
cargo doc --no-deps --open

# 检查文档链接
cargo doc --no-deps 2>&1 | grep -i warning
```

## 发布流程 | Release Process

### 版本号规范

我们使用 [语义化版本](https://semver.org/lang/zh-CN/)：

- **主版本号**: 不兼容的 API 修改
- **次版本号**: 向下兼容的功能性新增
- **修订号**: 向下兼容的问题修正

### 发布检查清单

- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] CHANGELOG.md 已更新
- [ ] 版本号已更新
- [ ] 性能基准测试通过
- [ ] 安全审计通过

## 社区交流 | Community

### 获取帮助

- **GitHub Issues**: 报告 Bug 或请求功能
- **GitHub Discussions**: 一般讨论和问答
- **代码审查**: 在 Pull Request 中讨论具体实现

### 提问技巧

1. **搜索现有 Issues**: 避免重复问题
2. **提供详细信息**: 包括系统信息、错误日志等
3. **最小化复现**: 提供能复现问题的最小示例
4. **清晰描述**: 说明期望行为和实际行为

### 代码审查

作为审查者：
- 关注代码质量和设计
- 提供建设性的反馈
- 及时响应审查请求

作为贡献者：
- 响应审查意见
- 解释设计决策
- 保持开放的心态

## 常见贡献场景 | Common Contribution Scenarios

### 添加新的音频格式

1. 在 `AudioFormat` 枚举中添加新变体
2. 实现相关方法 (`extension`, `ffmpeg_args`, `description`)
3. 更新 `from_user_input` 方法
4. 添加测试用例
5. 更新文档

### 修复 Bug

1. 创建复现 Bug 的测试用例
2. 修复问题
3. 确保测试通过
4. 更新相关文档

### 性能优化

1. 添加性能基准测试
2. 实现优化
3. 验证性能提升
4. 确保功能正确性

### 改进用户界面

1. 分析用户体验问题
2. 设计改进方案
3. 实现界面改进
4. 测试用户交互流程

## 致谢 | Acknowledgments

感谢所有为 Video2Audio-RS 项目做出贡献的开发者！您的贡献让这个项目变得更好。

特别感谢：
- 核心维护团队
- 文档贡献者
- Bug 报告者
- 功能建议者
- 测试用户

---

**记住**: 每个贡献都很重要，无论大小。我们期待您的参与！

如果您有任何问题，请随时在 GitHub Issues 中提问。
