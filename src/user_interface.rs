//! # 用户界面模块
//!
//! 处理所有用户交互逻辑，包括输入获取、格式选择和进度显示。
//! 提供友好的中文界面和清晰的操作提示。

use crate::audio_format::AudioFormat;
use crate::error::{Result, VideoToAudioError};
use std::io::{self, Write};

/// 用户界面管理器
///
/// 负责处理所有与用户的交互，包括：
/// - 获取用户输入
/// - 显示选项菜单
/// - 进度反馈
/// - 错误提示
pub struct UserInterface {}

impl UserInterface {
    /// 创建新的用户界面实例
    pub fn new() -> Self {
        Self {}
    }

    /// 显示程序欢迎信息
    ///
    /// 在程序启动时显示标题和基本信息
    pub fn show_welcome(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                批量视频转音频工具 (高并发版)                 ║");
        println!("║                   Video2Audio-RS v0.1.0                     ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("🎵 支持多种视频格式转换为高质量音频文件");
        println!("⚡ 利用多核 CPU 并行处理，大幅提升转换速度");
        println!("🛠️  基于 FFmpeg 引擎，确保转换质量和兼容性");
        println!();
    }

    /// 获取用户输入
    ///
    /// 显示提示信息并等待用户输入，自动处理输入验证和错误处理
    ///
    /// # 参数
    ///
    /// * `prompt` - 显示给用户的提示信息
    ///
    /// # 返回值
    ///
    /// 用户输入的字符串（已去除首尾空白）
    ///
    /// # 错误
    ///
    /// 当输入操作失败时返回 I/O 错误
    pub fn get_user_input(&self, prompt: &str) -> Result<String> {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        let input = buffer.trim().to_string();

        // 检查空输入
        if input.is_empty() {
            return Err(VideoToAudioError::InvalidInput(
                "输入不能为空，请重新输入".to_string(),
            ));
        }

        Ok(input)
    }

    /// 让用户选择音频格式
    ///
    /// 显示格式选择菜单，处理用户选择并返回对应的音频格式
    ///
    /// # 返回值
    ///
    /// 用户选择的 `AudioFormat`
    ///
    /// # 错误
    ///
    /// 当用户输入无效选项时返回错误
    pub fn select_audio_format(&self) -> Result<AudioFormat> {
        loop {
            println!("┌─────────────────────────────────────────────────────────────┐");
            println!("│                    请选择目标音频格式                        │");
            println!("├─────────────────────────────────────────────────────────────┤");

            // 动态显示所有可用格式
            for (index, format) in AudioFormat::all_formats().iter().enumerate() {
                println!("│  {}. {:<50} │", index + 1, format.description());
            }

            println!("└─────────────────────────────────────────────────────────────┘");
            println!();

            match self.get_user_input("请输入选项 (1-3): ") {
                Ok(choice_str) => match AudioFormat::from_user_input(&choice_str) {
                    Ok(format) => {
                        println!("✓ 已选择格式: {}", format.description());
                        println!();
                        return Ok(format);
                    }
                    Err(_) => {
                        println!("❌ 无效输入，请输入 1, 2, 或 3");
                        println!();
                    }
                },
                Err(e) => {
                    println!("❌ 输入错误: {e}");
                    println!();
                }
            }
        }
    }

    /// 获取并验证源目录路径
    ///
    /// 提示用户输入视频文件夹路径，并验证路径的有效性
    ///
    /// # 返回值
    ///
    /// 验证过的目录路径字符串
    ///
    /// # 错误
    ///
    /// 当路径无效或不是目录时返回错误
    pub fn get_source_directory(&self) -> Result<String> {
        loop {
            println!("📁 请指定要处理的视频文件夹:");
            println!("   提示: 程序会自动扫描该文件夹及其所有子文件夹");
            println!();

            match self.get_user_input("请输入文件夹的完整路径: ") {
                Ok(source_dir) => {
                    let path = std::path::Path::new(&source_dir);

                    if !path.exists() {
                        println!("❌ 错误: 路径 '{source_dir}' 不存在，请检查路径是否正确");
                        println!();
                        continue;
                    }

                    if !path.is_dir() {
                        println!("❌ 错误: '{source_dir}' 不是一个文件夹，请输入文件夹路径");
                        println!();
                        continue;
                    }

                    println!("✓ 源目录验证成功: {source_dir}");
                    println!();
                    return Ok(source_dir);
                }
                Err(e) => {
                    println!("❌ 输入错误: {e}");
                    println!();
                }
            }
        }
    }

    /// 显示文件发现结果
    ///
    /// 显示找到的视频文件数量和即将开始的处理信息
    ///
    /// # 参数
    ///
    /// * `file_count` - 找到的视频文件数量
    /// * `output_dir` - 输出目录路径
    pub fn show_files_found(&self, file_count: usize, output_dir: &std::path::Path) {
        if file_count == 0 {
            println!("📂 扫描完成，但未找到任何支持的视频文件");
            println!("   支持的格式: MP4, MKV, AVI, MOV, WEBM, FLV, WMV");
            return;
        }

        println!("📊 扫描结果:");
        println!("   找到 {file_count} 个视频文件");
        println!("   输出目录: {}", output_dir.display());
        println!("   开始并行转换处理...");
        println!();
    }

    /// 显示处理进度
    ///
    /// 在同一行更新显示当前处理进度
    ///
    /// # 参数
    ///
    /// * `current` - 当前已处理的文件数
    /// * `total` - 总文件数
    pub fn show_progress(&self, current: usize, total: usize) {
        let percentage = if total > 0 {
            (current as f64 / total as f64 * 100.0) as u8
        } else {
            0
        };
        print!("\r🔄 处理进度: {current}/{total} ({percentage}%)");
        io::stdout().flush().unwrap_or(());
    }

    /// 显示处理完成信息
    ///
    /// 显示转换完成的总结信息
    ///
    /// # 参数
    ///
    /// * `total_files` - 总处理文件数
    /// * `output_dir` - 输出目录路径
    pub fn show_completion(&self, total_files: usize, output_dir: &std::path::Path) {
        println!();
        println!("🎉 转换完成!");
        println!("   共处理 {total_files} 个文件");
        println!("   所有音频文件已保存至: {}", output_dir.display());
        println!();
        println!("感谢使用 Video2Audio-RS! 🎵");
    }

    /// 显示错误信息
    ///
    /// 以用户友好的方式显示错误信息
    ///
    /// # 参数
    ///
    /// * `error` - 要显示的错误
    pub fn show_error(&self, error: &VideoToAudioError) {
        println!("❌ 发生错误: {error}");

        // 根据错误类型提供额外的帮助信息
        match error {
            VideoToAudioError::MissingDependency(_) => {
                println!("💡 解决方案:");
                println!("   请安装 FFmpeg 并确保其在系统 PATH 中");
                println!("   macOS: brew install ffmpeg");
                println!("   Windows: choco install ffmpeg");
                println!("   Linux: sudo apt install ffmpeg");
            }
            VideoToAudioError::InvalidPath(_) => {
                println!("💡 请检查路径是否正确，确保使用完整的绝对路径");
            }
            VideoToAudioError::UnsupportedFormat(_) => {
                println!("💡 当前支持的视频格式: MP4, MKV, AVI, MOV, WEBM, FLV, WMV");
            }
            _ => {}
        }
        println!();
    }
}

impl Default for UserInterface {
    fn default() -> Self {
        Self::new()
    }
}
