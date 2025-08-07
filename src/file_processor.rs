//! # 文件处理模块
//! 
//! 负责视频文件的发现、验证和转换处理。
//! 提供高性能的并行处理能力和完善的错误处理机制。

use crate::audio_format::AudioFormat;
use crate::error::{Result, VideoToAudioError};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::fs;

/// 文件处理器
/// 
/// 负责管理整个文件转换流程，包括：
/// - 视频文件发现和过滤
/// - 并行转换处理
/// - 进度跟踪和错误处理
/// - 输出目录管理
pub struct FileProcessor {
    /// 支持的视频文件扩展名列表
    supported_extensions: Vec<&'static str>,
}

impl FileProcessor {
    /// 创建新的文件处理器实例
    /// 
    /// 初始化支持的视频格式列表，包括常见的视频文件格式
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "3gp", "ts"
            ],
        }
    }

    /// 获取支持的视频文件扩展名列表
    /// 
    /// # 返回值
    /// 
    /// 包含所有支持的文件扩展名的向量
    pub fn supported_extensions(&self) -> &[&'static str] {
        &self.supported_extensions
    }

    /// 在指定目录中查找所有支持的视频文件
    /// 
    /// 递归扫描目录及其子目录，查找所有支持格式的视频文件
    /// 
    /// # 参数
    /// 
    /// * `source_dir` - 要扫描的源目录路径
    /// 
    /// # 返回值
    /// 
    /// 包含所有找到的视频文件路径的向量
    /// 
    /// # 错误
    /// 
    /// 当目录访问失败或路径无效时返回错误
    pub fn find_video_files(&self, source_dir: &Path) -> Result<Vec<PathBuf>> {
        if !source_dir.exists() {
            return Err(VideoToAudioError::InvalidPath(
                format!("目录不存在: {}", source_dir.display())
            ));
        }

        if !source_dir.is_dir() {
            return Err(VideoToAudioError::InvalidPath(
                format!("路径不是目录: {}", source_dir.display())
            ));
        }

        let files: Result<Vec<PathBuf>> = walkdir::WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|entry| {
                match entry {
                    Ok(e) if e.file_type().is_file() => Some(Ok(e.into_path())),
                    Ok(_) => None, // 跳过目录
                    Err(err) => Some(Err(VideoToAudioError::Io(
                        std::io::Error::other(err)
                    ))),
                }
            })
            .filter(|result| {
                match result {
                    Ok(path) => self.is_supported_video_file(path),
                    Err(_) => true, // 保留错误以便传播
                }
            })
            .collect();

        files
    }

    /// 检查文件是否为支持的视频格式
    /// 
    /// 通过文件扩展名判断是否为支持的视频文件
    /// 
    /// # 参数
    /// 
    /// * `path` - 要检查的文件路径
    /// 
    /// # 返回值
    /// 
    /// 如果是支持的视频文件返回 `true`，否则返回 `false`
    fn is_supported_video_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.supported_extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// 创建输出目录
    /// 
    /// 在源目录下创建 `audio_exports` 子目录用于存放转换后的音频文件
    /// 
    /// # 参数
    /// 
    /// * `source_dir` - 源目录路径
    /// 
    /// # 返回值
    /// 
    /// 创建的输出目录路径
    /// 
    /// # 错误
    /// 
    /// 当目录创建失败时返回错误
    pub fn create_output_directory(&self, source_dir: &Path) -> Result<PathBuf> {
        let output_dir = source_dir.join("audio_exports");
        
        fs::create_dir_all(&output_dir)
            .map_err(VideoToAudioError::Io)?;
            
        Ok(output_dir)
    }

    /// 批量并行转换视频文件
    /// 
    /// 使用 Rayon 库进行并行处理，最大化利用多核 CPU 性能
    /// 
    /// # 参数
    /// 
    /// * `files` - 要转换的视频文件路径列表
    /// * `output_dir` - 输出目录路径
    /// * `format` - 目标音频格式
    /// * `progress_callback` - 进度回调函数，接收 (当前进度, 总数) 参数
    /// 
    /// # 返回值
    /// 
    /// 返回转换结果的统计信息 (成功数, 失败数)
    pub fn batch_convert<F>(
        &self,
        files: &[PathBuf],
        output_dir: &Path,
        format: AudioFormat,
        progress_callback: F,
    ) -> (usize, usize)
    where
        F: Fn(usize, usize) + Send + Sync,
    {
        let total_files = files.len();
        let progress_counter = Arc::new(Mutex::new(0));
        let success_counter = Arc::new(Mutex::new(0));
        let failure_counter = Arc::new(Mutex::new(0));

        // 使用 Rayon 进行并行处理
        files.par_iter().for_each(|source_file| {
            match self.convert_single_file(source_file, output_dir, format) {
                Ok(_) => {
                    let mut success_count = success_counter.lock().unwrap();
                    *success_count += 1;
                }
                Err(e) => {
                    let mut failure_count = failure_counter.lock().unwrap();
                    *failure_count += 1;
                    
                    // 输出错误信息到标准错误流
                    eprintln!(
                        "\n❌ [失败] 处理文件 '{}' 时出错: {}",
                        source_file.display(),
                        e
                    );
                }
            }

            // 更新进度
            let mut count = progress_counter.lock().unwrap();
            *count += 1;
            progress_callback(*count, total_files);
        });

        let success_count = *success_counter.lock().unwrap();
        let failure_count = *failure_counter.lock().unwrap();
        
        (success_count, failure_count)
    }

    /// 转换单个视频文件为音频
    /// 
    /// 调用 FFmpeg 执行实际的媒体转换操作
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
    pub fn convert_single_file(
        &self,
        source_file: &Path,
        output_dir: &Path,
        format: AudioFormat,
    ) -> Result<PathBuf> {
        // 验证源文件
        if !source_file.exists() {
            return Err(VideoToAudioError::InvalidPath(
                format!("源文件不存在: {}", source_file.display())
            ));
        }

        // 构建输出文件路径
        let output_path = self.build_output_path(source_file, output_dir, format)?;

        // 检查 FFmpeg 是否可用
        self.check_ffmpeg_availability()?;

        // 执行转换
        self.execute_ffmpeg_conversion(source_file, &output_path, format)?;

        Ok(output_path)
    }

    /// 构建输出文件路径
    /// 
    /// 根据源文件名和目标格式生成输出文件的完整路径
    fn build_output_path(
        &self,
        source_file: &Path,
        output_dir: &Path,
        format: AudioFormat,
    ) -> Result<PathBuf> {
        let file_stem = source_file
            .file_stem()
            .ok_or_else(|| VideoToAudioError::InvalidPath(
                format!("无法获取文件名: {}", source_file.display())
            ))?
            .to_string_lossy();

        let output_filename = format!("{}.{}", file_stem, format.extension());
        Ok(output_dir.join(output_filename))
    }

    /// 检查 FFmpeg 是否可用
    /// 
    /// 验证系统中是否安装了 FFmpeg 并且可以正常执行
    fn check_ffmpeg_availability(&self) -> Result<()> {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| VideoToAudioError::MissingDependency(
                "FFmpeg 未安装或不在系统 PATH 中。请安装 FFmpeg 后重试。".to_string()
            ))?;
        
        Ok(())
    }

    /// 执行 FFmpeg 转换命令
    /// 
    /// 构建并执行 FFmpeg 命令进行实际的媒体转换
    fn execute_ffmpeg_conversion(
        &self,
        source_file: &Path,
        output_path: &Path,
        format: AudioFormat,
    ) -> Result<()> {
        let source_str = source_file.to_str()
            .ok_or_else(|| VideoToAudioError::InvalidPath(
                "源文件路径包含无效字符".to_string()
            ))?;

        let output_str = output_path.to_str()
            .ok_or_else(|| VideoToAudioError::InvalidPath(
                "输出文件路径包含无效字符".to_string()
            ))?;

        // 构建 FFmpeg 命令参数
        let mut args = vec![
            "-y",                    // 覆盖已存在的文件
            "-hide_banner",          // 隐藏版本信息
            "-loglevel", "error",    // 只显示错误信息
            "-i", source_str,        // 输入文件
            "-vn",                   // 不包含视频流
        ];

        // 添加格式特定的参数
        args.extend(format.ffmpeg_args());
        args.push(output_str);

        // 执行 FFmpeg 命令
        let output = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(VideoToAudioError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoToAudioError::FfmpegError(
                format!("转换失败: {stderr}")
            ));
        }

        Ok(())
    }
}

impl Default for FileProcessor {
    fn default() -> Self {
        Self::new()
    }
}
