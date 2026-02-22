//! # 文件处理模块
//!
//! 负责视频文件的发现、验证和转换处理。
//! 提供高性能的并行处理能力和完善的错误处理机制。

use crate::audio_format::AudioFormat;
use crate::config::ConflictStrategy;
use crate::error::{Result, VideoToAudioError};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 批量转换选项
#[derive(Debug, Clone)]
pub struct BatchOptions {
    /// 跳过已存在输出文件
    pub skip_existing: bool,
    /// 冲突处理策略
    pub conflict_strategy: ConflictStrategy,
    /// 仅预览
    pub dry_run: bool,
    /// FFmpeg 超时时间
    pub ffmpeg_timeout: Option<Duration>,
    /// 最大并发 FFmpeg 进程数
    pub max_parallel_ffmpeg: Option<usize>,
    /// 是否静默输出
    pub quiet: bool,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            skip_existing: false,
            conflict_strategy: ConflictStrategy::Overwrite,
            dry_run: false,
            ffmpeg_timeout: None,
            max_parallel_ffmpeg: None,
            quiet: false,
        }
    }
}

/// 文件扫描结果
#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    /// 找到的文件列表
    pub files: Vec<PathBuf>,
    /// 扫描过程中被忽略的警告
    pub warnings: Vec<String>,
}

/// 单文件处理记录
#[derive(Debug, Clone, Serialize)]
pub struct ConversionRecord {
    /// 输入文件
    pub input: String,
    /// 输出文件（可能为空）
    pub output: Option<String>,
    /// 状态：converted/skipped/failed/planned
    pub status: String,
    /// 附加信息
    pub reason: Option<String>,
    /// 处理耗时
    pub duration_ms: u128,
}

/// 批量处理摘要
#[derive(Debug, Clone, Serialize)]
pub struct BatchSummary {
    /// 总文件数
    pub total_files: usize,
    /// 成功数量
    pub success_count: usize,
    /// 失败数量
    pub failure_count: usize,
    /// 跳过数量
    pub skipped_count: usize,
    /// 计划数量（dry-run）
    pub planned_count: usize,
    /// 扫描警告
    pub scan_warnings: Vec<String>,
    /// 详细记录
    pub records: Vec<ConversionRecord>,
}

enum PlannedJob {
    Convert {
        source_file: PathBuf,
        output_path: PathBuf,
        overwrite_existing: bool,
    },
    Plan {
        source_file: PathBuf,
        output_path: PathBuf,
    },
    Skip {
        source_file: PathBuf,
        output_path: Option<PathBuf>,
        reason: String,
    },
    Fail {
        source_file: PathBuf,
        output_path: Option<PathBuf>,
        reason: String,
    },
}

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
                "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "3gp", "ts",
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
        self.find_video_files_with_options(source_dir, false)
            .map(|result| result.files)
    }

    /// 在指定目录中查找所有支持的视频文件（可配置是否忽略扫描错误）
    pub fn find_video_files_with_options(
        &self,
        source_dir: &Path,
        ignore_scan_errors: bool,
    ) -> Result<ScanResult> {
        if !source_dir.exists() {
            return Err(VideoToAudioError::InvalidPath(format!(
                "目录不存在: {}",
                source_dir.display()
            )));
        }

        if !source_dir.is_dir() {
            return Err(VideoToAudioError::InvalidPath(format!(
                "路径不是目录: {}",
                source_dir.display()
            )));
        }

        let mut files = Vec::new();
        let mut warnings = Vec::new();

        for entry in walkdir::WalkDir::new(source_dir).into_iter() {
            match entry {
                Ok(e) if e.file_type().is_file() => {
                    if self.is_supported_video_file(e.path()) {
                        files.push(e.into_path());
                    }
                }
                Ok(_) => {}
                Err(err) => {
                    if ignore_scan_errors {
                        warnings.push(Self::format_scan_warning(&err));
                    } else {
                        return Err(VideoToAudioError::Io(std::io::Error::other(err)));
                    }
                }
            }
        }

        files.sort();

        Ok(ScanResult { files, warnings })
    }

    fn format_scan_warning(err: &walkdir::Error) -> String {
        if let Some(path) = err.path() {
            format!("跳过不可访问路径 '{}': {err}", path.display())
        } else {
            format!("跳过不可访问路径: {err}")
        }
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
            .map(|ext| {
                self.supported_extensions
                    .contains(&ext.to_lowercase().as_str())
            })
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

        fs::create_dir_all(&output_dir).map_err(VideoToAudioError::Io)?;

        Ok(output_dir)
    }

    /// 批量并行转换视频文件（兼容旧 API）
    ///
    /// 返回 (成功数, 失败数)
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
        match self.batch_convert_with_options(
            files,
            output_dir,
            format,
            &BatchOptions::default(),
            progress_callback,
        ) {
            Ok(summary) => (summary.success_count, summary.failure_count),
            Err(e) => {
                eprintln!("\n❌ [失败] 批量转换初始化失败: {e}");
                (0, files.len())
            }
        }
    }

    /// 批量并行转换视频文件（带详细选项和记录）
    pub fn batch_convert_with_options<F>(
        &self,
        files: &[PathBuf],
        output_dir: &Path,
        format: AudioFormat,
        options: &BatchOptions,
        progress_callback: F,
    ) -> Result<BatchSummary>
    where
        F: Fn(usize, usize) + Send + Sync,
    {
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        let planned_jobs = self.plan_jobs(files, output_dir, format, options)?;
        let total_files = planned_jobs.len();

        if !options.dry_run
            && planned_jobs
                .iter()
                .any(|job| matches!(job, PlannedJob::Convert { .. }))
        {
            self.check_ffmpeg_availability()?;
        }

        if total_files == 0 {
            return Ok(BatchSummary {
                total_files: 0,
                success_count: 0,
                failure_count: 0,
                skipped_count: 0,
                planned_count: 0,
                scan_warnings: Vec::new(),
                records: Vec::new(),
            });
        }

        let total = total_files;
        let counter = Arc::new(AtomicUsize::new(0));

        let process_job = |job: &PlannedJob| {
            let start = Instant::now();
            let mut record = match job {
                PlannedJob::Convert {
                    source_file,
                    output_path,
                    overwrite_existing,
                } => {
                    match self.execute_ffmpeg_conversion(
                        source_file,
                        output_path,
                        format,
                        *overwrite_existing,
                        options.ffmpeg_timeout,
                    ) {
                        Ok(()) => ConversionRecord {
                            input: source_file.display().to_string(),
                            output: Some(output_path.display().to_string()),
                            status: "converted".to_string(),
                            reason: None,
                            duration_ms: 0,
                        },
                        Err(e) => ConversionRecord {
                            input: source_file.display().to_string(),
                            output: Some(output_path.display().to_string()),
                            status: "failed".to_string(),
                            reason: Some(e.to_string()),
                            duration_ms: 0,
                        },
                    }
                }
                PlannedJob::Plan {
                    source_file,
                    output_path,
                } => ConversionRecord {
                    input: source_file.display().to_string(),
                    output: Some(output_path.display().to_string()),
                    status: "planned".to_string(),
                    reason: Some("dry-run: 未执行转换".to_string()),
                    duration_ms: 0,
                },
                PlannedJob::Skip {
                    source_file,
                    output_path,
                    reason,
                } => ConversionRecord {
                    input: source_file.display().to_string(),
                    output: output_path.as_ref().map(|p| p.display().to_string()),
                    status: "skipped".to_string(),
                    reason: Some(reason.clone()),
                    duration_ms: 0,
                },
                PlannedJob::Fail {
                    source_file,
                    output_path,
                    reason,
                } => ConversionRecord {
                    input: source_file.display().to_string(),
                    output: output_path.as_ref().map(|p| p.display().to_string()),
                    status: "failed".to_string(),
                    reason: Some(reason.clone()),
                    duration_ms: 0,
                },
            };

            record.duration_ms = start.elapsed().as_millis();

            if record.status == "failed" && !options.quiet {
                eprintln!(
                    "\n❌ [失败] 处理文件 '{}' 时出错: {}",
                    record.input,
                    record.reason.as_deref().unwrap_or("未知错误")
                );
            }

            let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
            progress_callback(current, total);

            record
        };

        let records: Vec<ConversionRecord> = if let Some(max_workers) = options.max_parallel_ffmpeg
        {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers.max(1))
                .build()
                .map_err(|e| {
                    VideoToAudioError::InvalidInput(format!("无法创建受限 FFmpeg 线程池: {e}"))
                })?;

            pool.install(|| planned_jobs.par_iter().map(process_job).collect())
        } else {
            planned_jobs.par_iter().map(process_job).collect()
        };

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut skipped_count = 0;
        let mut planned_count = 0;

        for record in &records {
            match record.status.as_str() {
                "converted" => success_count += 1,
                "failed" => failure_count += 1,
                "skipped" => skipped_count += 1,
                "planned" => planned_count += 1,
                _ => {}
            }
        }

        Ok(BatchSummary {
            total_files: records.len(),
            success_count,
            failure_count,
            skipped_count,
            planned_count,
            scan_warnings: Vec::new(),
            records,
        })
    }

    fn plan_jobs(
        &self,
        files: &[PathBuf],
        output_dir: &Path,
        format: AudioFormat,
        options: &BatchOptions,
    ) -> Result<Vec<PlannedJob>> {
        let mut planned = Vec::with_capacity(files.len());
        let mut reserved = HashSet::with_capacity(files.len());

        for source_file in files {
            planned.push(self.plan_single_job(
                source_file,
                output_dir,
                format,
                options,
                &mut reserved,
            )?);
        }

        Ok(planned)
    }

    fn plan_single_job(
        &self,
        source_file: &Path,
        output_dir: &Path,
        format: AudioFormat,
        options: &BatchOptions,
        reserved: &mut HashSet<PathBuf>,
    ) -> Result<PlannedJob> {
        let default_output = self.build_output_path(source_file, output_dir, format)?;
        let conflict_in_batch = reserved.contains(&default_output);
        let conflict_on_disk = default_output.exists();

        if conflict_in_batch {
            if options.skip_existing || options.conflict_strategy == ConflictStrategy::Skip {
                return Ok(PlannedJob::Skip {
                    source_file: source_file.to_path_buf(),
                    output_path: Some(default_output),
                    reason: "输出文件与批次内其他文件冲突，已跳过".to_string(),
                });
            }

            return match options.conflict_strategy {
                ConflictStrategy::Rename => {
                    let renamed = self.generate_unique_output_path(&default_output, reserved)?;
                    reserved.insert(renamed.clone());
                    Ok(self.to_plan_or_convert(source_file, renamed, false, options.dry_run))
                }
                ConflictStrategy::Overwrite => Ok(PlannedJob::Fail {
                    source_file: source_file.to_path_buf(),
                    output_path: Some(default_output),
                    reason: "批次内存在同名输出冲突，overwrite 不允许并发写入同一目标".to_string(),
                }),
                ConflictStrategy::Error => Ok(PlannedJob::Fail {
                    source_file: source_file.to_path_buf(),
                    output_path: Some(default_output),
                    reason: "批次内存在同名输出冲突".to_string(),
                }),
                ConflictStrategy::Skip => unreachable!(),
            };
        }

        if conflict_on_disk {
            if options.skip_existing || options.conflict_strategy == ConflictStrategy::Skip {
                return Ok(PlannedJob::Skip {
                    source_file: source_file.to_path_buf(),
                    output_path: Some(default_output),
                    reason: "输出文件已存在，已跳过".to_string(),
                });
            }

            return match options.conflict_strategy {
                ConflictStrategy::Rename => {
                    let renamed = self.generate_unique_output_path(&default_output, reserved)?;
                    reserved.insert(renamed.clone());
                    Ok(self.to_plan_or_convert(source_file, renamed, false, options.dry_run))
                }
                ConflictStrategy::Overwrite => {
                    reserved.insert(default_output.clone());
                    Ok(self.to_plan_or_convert(source_file, default_output, true, options.dry_run))
                }
                ConflictStrategy::Error => Ok(PlannedJob::Fail {
                    source_file: source_file.to_path_buf(),
                    output_path: Some(default_output),
                    reason: "输出文件已存在".to_string(),
                }),
                ConflictStrategy::Skip => unreachable!(),
            };
        }

        reserved.insert(default_output.clone());
        Ok(self.to_plan_or_convert(source_file, default_output, false, options.dry_run))
    }

    fn to_plan_or_convert(
        &self,
        source_file: &Path,
        output_path: PathBuf,
        overwrite_existing: bool,
        dry_run: bool,
    ) -> PlannedJob {
        if dry_run {
            PlannedJob::Plan {
                source_file: source_file.to_path_buf(),
                output_path,
            }
        } else {
            PlannedJob::Convert {
                source_file: source_file.to_path_buf(),
                output_path,
                overwrite_existing,
            }
        }
    }

    fn generate_unique_output_path(
        &self,
        desired_path: &Path,
        reserved: &HashSet<PathBuf>,
    ) -> Result<PathBuf> {
        let parent = desired_path.parent().ok_or_else(|| {
            VideoToAudioError::InvalidPath(format!("无法获取输出目录: {}", desired_path.display()))
        })?;

        let stem = desired_path
            .file_stem()
            .ok_or_else(|| {
                VideoToAudioError::InvalidPath(format!(
                    "无法获取文件名: {}",
                    desired_path.display()
                ))
            })?
            .to_string_lossy()
            .to_string();

        let ext = desired_path.extension().and_then(|e| e.to_str());

        for index in 1..=u32::MAX {
            let candidate_name = match ext {
                Some(ext) => format!("{stem}_{index}.{ext}"),
                None => format!("{stem}_{index}"),
            };
            let candidate_path = parent.join(candidate_name);

            if !candidate_path.exists() && !reserved.contains(&candidate_path) {
                return Ok(candidate_path);
            }
        }

        Err(VideoToAudioError::InvalidPath(
            "无法生成唯一输出文件名，冲突数量过多".to_string(),
        ))
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
            return Err(VideoToAudioError::InvalidPath(format!(
                "源文件不存在: {}",
                source_file.display()
            )));
        }

        // 构建输出文件路径
        let output_path = self.build_output_path(source_file, output_dir, format)?;

        // 检查 FFmpeg 是否可用
        self.check_ffmpeg_availability()?;

        // 执行转换
        self.execute_ffmpeg_conversion(source_file, &output_path, format, true, None)?;

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
            .ok_or_else(|| {
                VideoToAudioError::InvalidPath(format!("无法获取文件名: {}", source_file.display()))
            })?
            .to_string_lossy();

        let output_filename = format!("{}.{}", file_stem, format.extension());
        Ok(output_dir.join(output_filename))
    }

    /// 检查 FFmpeg 是否可用
    ///
    /// 验证系统中是否安装了 FFmpeg 并且可以正常执行
    fn check_ffmpeg_availability(&self) -> Result<()> {
        let status = Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| {
                VideoToAudioError::MissingDependency(
                    "FFmpeg 未安装或不在系统 PATH 中。请安装 FFmpeg 后重试。".to_string(),
                )
            })?;

        if !status.success() {
            return Err(VideoToAudioError::MissingDependency(
                "FFmpeg 无法正常执行 (ffmpeg -version 失败)".to_string(),
            ));
        }

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
        overwrite_existing: bool,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg(if overwrite_existing { "-y" } else { "-n" })
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-i")
            .arg(source_file)
            .arg("-vn")
            .args(format.ffmpeg_args())
            .arg(output_path);

        if let Some(timeout) = timeout {
            let mut child = cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
            let start = Instant::now();

            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            return Ok(());
                        }
                        return Err(VideoToAudioError::FfmpegError(format!(
                            "转换失败，退出码: {}",
                            status
                                .code()
                                .map(|code| code.to_string())
                                .unwrap_or_else(|| "未知".to_string())
                        )));
                    }
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            let _ = child.kill();
                            let _ = child.wait();
                            return Err(VideoToAudioError::FfmpegError(format!(
                                "转换超时: 超过 {} 秒",
                                timeout.as_secs()
                            )));
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => return Err(VideoToAudioError::Io(e)),
                }
            }
        }

        let output = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoToAudioError::FfmpegError(format!(
                "转换失败: {stderr}"
            )));
        }

        Ok(())
    }
}

impl Default for FileProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_dry_run_rename_avoids_same_stem_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let dir_a = temp_dir.path().join("a");
        let dir_b = temp_dir.path().join("b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();

        let file_a = dir_a.join("same.mp4");
        let file_b = dir_b.join("same.mkv");
        fs::write(&file_a, "x").unwrap();
        fs::write(&file_b, "x").unwrap();

        let processor = FileProcessor::new();
        let options = BatchOptions {
            dry_run: true,
            conflict_strategy: ConflictStrategy::Rename,
            ..BatchOptions::default()
        };

        let summary = processor
            .batch_convert_with_options(
                &[file_a, file_b],
                &output_dir,
                AudioFormat::Mp3,
                &options,
                |_c, _t| {},
            )
            .unwrap();

        assert_eq!(summary.planned_count, 2);
        let outputs: Vec<String> = summary
            .records
            .iter()
            .map(|record| record.output.clone().unwrap())
            .collect();
        assert_ne!(outputs[0], outputs[1]);
    }

    #[test]
    fn test_dry_run_skip_existing_effective() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let input = temp_dir.path().join("movie.mp4");
        fs::write(&input, "x").unwrap();
        fs::write(output_dir.join("movie.mp3"), "existing").unwrap();

        let processor = FileProcessor::new();
        let options = BatchOptions {
            dry_run: true,
            skip_existing: true,
            conflict_strategy: ConflictStrategy::Overwrite,
            ..BatchOptions::default()
        };

        let summary = processor
            .batch_convert_with_options(
                &[input],
                &output_dir,
                AudioFormat::Mp3,
                &options,
                |_c, _t| {},
            )
            .unwrap();

        assert_eq!(summary.skipped_count, 1);
        assert_eq!(summary.records[0].status, "skipped");
    }

    #[test]
    fn test_dry_run_overwrite_blocks_in_batch_same_target() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let dir_a = temp_dir.path().join("a");
        let dir_b = temp_dir.path().join("b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();

        let file_a = dir_a.join("same.mp4");
        let file_b = dir_b.join("same.mkv");
        fs::write(&file_a, "x").unwrap();
        fs::write(&file_b, "x").unwrap();

        let processor = FileProcessor::new();
        let options = BatchOptions {
            dry_run: true,
            conflict_strategy: ConflictStrategy::Overwrite,
            ..BatchOptions::default()
        };

        let summary = processor
            .batch_convert_with_options(
                &[file_a, file_b],
                &output_dir,
                AudioFormat::Mp3,
                &options,
                |_c, _t| {},
            )
            .unwrap();

        assert_eq!(summary.planned_count, 1);
        assert_eq!(summary.failure_count, 1);
    }
}
