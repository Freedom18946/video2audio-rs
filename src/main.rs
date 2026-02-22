//! # Video2Audio-RS 主程序
//!
//! 高性能的批量视频转音频工具，支持多种格式和并行处理。
//!
//! 本程序提供友好的中文命令行界面，支持：
//! - 批量处理视频文件
//! - 多种音频格式输出 (MP3, AAC, Opus)
//! - 多核并行处理
//! - 实时进度显示

use clap::Parser;
use serde::Serialize;
use std::path::Path;
use video2audio_rs::{
    Args, AudioFormat, BatchOptions, BatchSummary, Config, FileProcessor, RuntimeConfig,
    UserInterface, VideoToAudioError,
};

/// 程序主入口点
///
/// 协调各个模块完成完整的视频转音频流程：
/// 1. 解析命令行参数和配置
/// 2. 根据模式选择交互式或批处理流程
/// 3. 执行视频转音频处理
/// 4. 显示处理结果和统计信息
fn main() -> Result<(), VideoToAudioError> {
    // 解析命令行参数
    let args = Args::parse();

    // 加载配置文件
    let mut config = Config::load(args.config_file.as_ref())?;

    // 创建运行时配置
    let runtime_config = RuntimeConfig::from_args_and_config(args, config.clone());

    // 处理特殊命令
    if runtime_config.list_formats {
        show_supported_formats();
        return Ok(());
    }

    // 初始化组件
    let ui = UserInterface::new();
    let processor = FileProcessor::new();

    // 设置并行线程数
    if let Some(jobs) = runtime_config.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .map_err(|e| VideoToAudioError::InvalidInput(format!("无法设置线程池: {e}")))?;
    }

    // 根据模式选择处理流程
    let (source_path, chosen_format, output_dir) = if runtime_config.needs_interaction() {
        // 交互式模式
        interactive_mode(&ui, &processor, &runtime_config)?
    } else {
        // 批处理模式
        batch_mode(&processor, &runtime_config)?
    };

    // 查找视频文件
    let scan_result =
        processor.find_video_files_with_options(&source_path, runtime_config.ignore_scan_errors)?;
    let files_to_process = scan_result.files;
    let total_files = files_to_process.len();

    // 显示扫描结果（除非是静默模式）
    if !runtime_config.quiet {
        ui.show_files_found(total_files, &output_dir);
        if !scan_result.warnings.is_empty() {
            println!("⚠️  扫描警告: {} 项", scan_result.warnings.len());
            for warning in scan_result.warnings.iter().take(5) {
                println!("   - {warning}");
            }
            if scan_result.warnings.len() > 5 {
                println!("   - ... 省略 {} 条", scan_result.warnings.len() - 5);
            }
            println!();
        }
    }

    if total_files == 0 {
        if !runtime_config.quiet {
            println!("未找到任何视频文件，程序退出。");
        }
        return Ok(());
    }

    if runtime_config.dry_run && !runtime_config.quiet {
        println!("🧪 Dry-run 模式: 将只生成处理计划，不执行 FFmpeg 转换。\n");
    }

    let batch_options = BatchOptions {
        skip_existing: runtime_config.skip_existing,
        conflict_strategy: runtime_config.conflict_strategy,
        dry_run: runtime_config.dry_run,
        ffmpeg_timeout: runtime_config.ffmpeg_timeout(),
        max_parallel_ffmpeg: runtime_config.max_parallel_ffmpeg,
        quiet: runtime_config.quiet,
    };

    // 执行批量转换
    let mut summary = processor.batch_convert_with_options(
        &files_to_process,
        &output_dir,
        chosen_format,
        &batch_options,
        |current, total| {
            if !runtime_config.quiet {
                ui.show_progress(current, total);
            }
        },
    )?;
    summary.scan_warnings = scan_result.warnings;

    // 显示完成信息
    if !runtime_config.quiet {
        if runtime_config.dry_run {
            println!();
            println!("✅ Dry-run 完成");
            println!("   共分析 {total_files} 个文件");
            println!("   计划转换: {} 个文件", summary.planned_count);
            println!("   跳过: {} 个文件", summary.skipped_count);
            println!("   失败: {} 个文件", summary.failure_count);
            println!();
        } else {
            ui.show_completion(total_files, &output_dir);
        }

        if summary.failure_count > 0
            || summary.skipped_count > 0
            || runtime_config.verbose
            || runtime_config.dry_run
        {
            println!("📊 处理统计:");
            println!("   ✅ 成功: {} 个文件", summary.success_count);
            println!("   ⏭️  跳过: {} 个文件", summary.skipped_count);
            if runtime_config.dry_run {
                println!("   🧪 计划: {} 个文件", summary.planned_count);
            }
            if summary.failure_count > 0 {
                println!("   ❌ 失败: {} 个文件", summary.failure_count);
                println!("   建议检查失败文件的格式、完整性或冲突策略");
            }
        }
    }

    if let Some(report_file) = runtime_config.report_file.as_ref() {
        write_report(
            report_file,
            &summary,
            &source_path,
            &output_dir,
            chosen_format,
            runtime_config.dry_run,
        )?;

        if !runtime_config.quiet {
            println!("📝 报告已写入: {}", report_file.display());
        }
    }

    // 更新配置（添加最近使用的目录）
    config.add_recent_source_dir(&source_path.to_string_lossy());

    // 保存配置（如果需要）
    if runtime_config.save_config {
        config.set_default_format(chosen_format);
        config.default_jobs = runtime_config.jobs;
        config.skip_existing = runtime_config.skip_existing;
        config.default_conflict_strategy = runtime_config.conflict_strategy;
        config.ignore_scan_errors = runtime_config.ignore_scan_errors;
        config.ffmpeg_timeout_seconds = runtime_config.ffmpeg_timeout_seconds;
        config.max_parallel_ffmpeg = runtime_config.max_parallel_ffmpeg;
        config.verbose = runtime_config.verbose;
        config.quiet = runtime_config.quiet;

        config.save(runtime_config.config_file.as_ref())?;
        if !runtime_config.quiet {
            println!("✅ 配置已保存");
        }
    }

    Ok(())
}

#[derive(Serialize)]
struct ReportDocument<'a> {
    source_dir: String,
    output_dir: String,
    format: String,
    dry_run: bool,
    summary: &'a BatchSummary,
}

fn write_report(
    report_path: &Path,
    summary: &BatchSummary,
    source_path: &Path,
    output_dir: &Path,
    format: AudioFormat,
    dry_run: bool,
) -> Result<(), VideoToAudioError> {
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let extension = report_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.to_lowercase());

    match extension.as_deref() {
        Some("json") => {
            let doc = ReportDocument {
                source_dir: source_path.display().to_string(),
                output_dir: output_dir.display().to_string(),
                format: format.extension().to_string(),
                dry_run,
                summary,
            };

            let content = serde_json::to_string_pretty(&doc)
                .map_err(|e| VideoToAudioError::InvalidInput(format!("报告序列化失败: {e}")))?;
            std::fs::write(report_path, content)?;
            Ok(())
        }
        Some("csv") => {
            let mut content = String::from("input,output,status,reason,duration_ms\n");
            for record in &summary.records {
                content.push_str(&format!(
                    "{},{},{},{},{}\n",
                    csv_escape(&record.input),
                    csv_escape(record.output.as_deref().unwrap_or("")),
                    csv_escape(&record.status),
                    csv_escape(record.reason.as_deref().unwrap_or("")),
                    record.duration_ms
                ));
            }

            std::fs::write(report_path, content)?;
            Ok(())
        }
        _ => Err(VideoToAudioError::InvalidInput(
            "报告文件扩展名仅支持 .json 或 .csv".to_string(),
        )),
    }
}

fn csv_escape(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

/// 显示支持的格式列表
fn show_supported_formats() {
    println!("📋 支持的文件格式:");
    println!();

    println!("🎬 输入格式 (视频):");
    let processor = FileProcessor::new();
    let extensions = processor.supported_extensions();
    for (i, ext) in extensions.iter().enumerate() {
        if i % 5 == 0 && i > 0 {
            println!();
        }
        print!("  {:<8}", ext.to_uppercase());
    }
    println!();
    println!();

    println!("🎵 输出格式 (音频):");
    for format in AudioFormat::all_formats() {
        println!(
            "  {} - {}",
            format.extension().to_uppercase(),
            format.description()
        );
    }
    println!();
}

/// 交互式模式处理
fn interactive_mode(
    ui: &UserInterface,
    processor: &FileProcessor,
    config: &RuntimeConfig,
) -> Result<(std::path::PathBuf, AudioFormat, std::path::PathBuf), VideoToAudioError> {
    // 显示欢迎信息
    if !config.quiet {
        ui.show_welcome();
    }

    // 获取源目录
    let source_dir = if let Some(ref dir) = config.source_dir {
        dir.to_string_lossy().to_string()
    } else {
        ui.get_source_directory()?
    };
    let source_path = std::path::PathBuf::from(&source_dir);

    // 获取音频格式
    let chosen_format = if let Some(format) = config.format {
        format
    } else {
        ui.select_audio_format()?
    };

    // 创建输出目录
    let output_dir = if let Some(ref dir) = config.output_dir {
        std::fs::create_dir_all(dir)?;
        dir.clone()
    } else {
        processor.create_output_directory(&source_path)?
    };

    Ok((source_path, chosen_format, output_dir))
}

/// 批处理模式处理
fn batch_mode(
    processor: &FileProcessor,
    config: &RuntimeConfig,
) -> Result<(std::path::PathBuf, AudioFormat, std::path::PathBuf), VideoToAudioError> {
    // 验证必需的参数
    let source_path = config
        .source_dir
        .as_ref()
        .ok_or_else(|| {
            VideoToAudioError::InvalidInput("批处理模式需要指定源目录 (--source)".to_string())
        })?
        .clone();

    let chosen_format = config.format.ok_or_else(|| {
        VideoToAudioError::InvalidInput("批处理模式需要指定音频格式 (--format)".to_string())
    })?;

    // 创建输出目录
    let output_dir = if let Some(ref dir) = config.output_dir {
        std::fs::create_dir_all(dir)?;
        dir.clone()
    } else {
        processor.create_output_directory(&source_path)?
    };

    Ok((source_path, chosen_format, output_dir))
}
