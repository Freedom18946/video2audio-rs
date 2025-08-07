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
use video2audio_rs::{Args, AudioFormat, Config, FileProcessor, RuntimeConfig, UserInterface, VideoToAudioError};

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
            .map_err(|e| VideoToAudioError::InvalidInput(
                format!("无法设置线程池: {e}")
            ))?;
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
    let files_to_process = processor.find_video_files(&source_path)?;
    let total_files = files_to_process.len();

    // 显示扫描结果（除非是静默模式）
    if !runtime_config.quiet {
        ui.show_files_found(total_files, &output_dir);
    }

    if total_files == 0 {
        if !runtime_config.quiet {
            println!("未找到任何视频文件，程序退出。");
        }
        return Ok(());
    }

    // 执行批量转换
    let (success_count, failure_count) = processor.batch_convert(
        &files_to_process,
        &output_dir,
        chosen_format,
        |current, total| {
            if !runtime_config.quiet {
                ui.show_progress(current, total);
            }
        },
    );

    // 显示完成信息
    if !runtime_config.quiet {
        ui.show_completion(total_files, &output_dir);

        // 显示详细统计信息
        if failure_count > 0 || runtime_config.verbose {
            println!("📊 处理统计:");
            println!("   ✅ 成功: {success_count} 个文件");
            if failure_count > 0 {
                println!("   ❌ 失败: {failure_count} 个文件");
                println!("   建议检查失败文件的格式或完整性");
            }
        }
    }

    // 更新配置（添加最近使用的目录）
    config.add_recent_source_dir(&source_path.to_string_lossy());

    // 保存配置（如果需要）
    if runtime_config.save_config {
        config.save(runtime_config.output_dir.as_ref())?;
        if !runtime_config.quiet {
            println!("✅ 配置已保存");
        }
    }

    Ok(())
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
        println!("  {} - {}",
                format.extension().to_uppercase(),
                format.description());
    }
    println!();
}

/// 交互式模式处理
fn interactive_mode(
    ui: &UserInterface,
    processor: &FileProcessor,
    config: &RuntimeConfig
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
    config: &RuntimeConfig
) -> Result<(std::path::PathBuf, AudioFormat, std::path::PathBuf), VideoToAudioError> {
    // 验证必需的参数
    let source_path = config.source_dir.as_ref()
        .ok_or_else(|| VideoToAudioError::InvalidInput(
            "批处理模式需要指定源目录 (--source)".to_string()
        ))?
        .clone();

    let chosen_format = config.format
        .ok_or_else(|| VideoToAudioError::InvalidInput(
            "批处理模式需要指定音频格式 (--format)".to_string()
        ))?;

    // 创建输出目录
    let output_dir = if let Some(ref dir) = config.output_dir {
        std::fs::create_dir_all(dir)?;
        dir.clone()
    } else {
        processor.create_output_directory(&source_path)?
    };

    Ok((source_path, chosen_format, output_dir))
}


