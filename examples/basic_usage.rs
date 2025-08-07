//! # Video2Audio-RS 基本使用示例
//!
//! 这个示例展示了如何使用 Video2Audio-RS 库进行编程式的视频转音频操作。
//!
//! ## 运行示例
//!
//! ```bash
//! cargo run --example basic_usage
//! ```

use video2audio_rs::{AudioFormat, FileProcessor, UserInterface, VideoToAudioError};

fn main() -> Result<(), VideoToAudioError> {
    println!("=== Video2Audio-RS 基本使用示例 ===\n");

    // 示例 1: 基本的文件处理器使用
    basic_file_processor_example()?;

    // 示例 2: 音频格式操作
    audio_format_example();

    // 示例 3: 用户界面组件使用
    user_interface_example();

    // 示例 4: 错误处理示例
    error_handling_example();

    // 示例 5: 完整的转换流程（模拟）
    complete_workflow_example()?;

    println!("\n=== 所有示例执行完成 ===");
    Ok(())
}

/// 示例 1: 基本的文件处理器使用
fn basic_file_processor_example() -> Result<(), VideoToAudioError> {
    println!("📁 示例 1: 文件处理器基本操作");
    
    let processor = FileProcessor::new();
    
    // 显示支持的文件格式
    println!("支持的视频格式: {:?}", processor.supported_extensions());
    
    // 尝试在当前目录查找视频文件（通常为空）
    let current_dir = std::env::current_dir().unwrap();
    match processor.find_video_files(&current_dir) {
        Ok(files) => {
            println!("在当前目录找到 {} 个视频文件", files.len());
            for (i, file) in files.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, file.display());
            }
            if files.len() > 3 {
                println!("  ... 还有 {} 个文件", files.len() - 3);
            }
        }
        Err(e) => {
            println!("扫描当前目录时出错: {}", e);
        }
    }
    
    println!();
    Ok(())
}

/// 示例 2: 音频格式操作
fn audio_format_example() {
    println!("🎵 示例 2: 音频格式操作");
    
    // 遍历所有支持的音频格式
    println!("支持的音频格式:");
    for (i, format) in AudioFormat::all_formats().iter().enumerate() {
        println!("  {}. {} -> .{}", 
                i + 1, 
                format.description(), 
                format.extension());
        println!("     FFmpeg 参数: {:?}", format.ffmpeg_args());
    }
    
    // 演示格式解析
    println!("\n格式解析示例:");
    let test_inputs = vec!["1", "mp3", "AAC", "opus"];
    for input in test_inputs {
        match AudioFormat::from_user_input(input) {
            Ok(format) => {
                println!("  '{}' -> {}", input, format.description());
            }
            Err(e) => {
                println!("  '{}' -> 错误: {}", input, e);
            }
        }
    }
    
    println!();
}

/// 示例 3: 用户界面组件使用
fn user_interface_example() {
    println!("🖥️  示例 3: 用户界面组件");
    
    let ui = UserInterface::new();
    
    // 显示欢迎信息
    println!("显示欢迎界面:");
    ui.show_welcome();
    
    // 模拟进度显示
    println!("进度显示示例:");
    for i in 0..=10 {
        ui.show_progress(i, 10);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!(); // 换行
    
    // 模拟完成信息
    let temp_dir = std::env::temp_dir().join("audio_exports");
    ui.show_completion(10, &temp_dir);
    
    println!();
}

/// 示例 4: 错误处理示例
fn error_handling_example() {
    println!("⚠️  示例 4: 错误处理");
    
    let ui = UserInterface::new();
    
    // 演示不同类型的错误
    let errors = vec![
        VideoToAudioError::InvalidPath("/nonexistent/path".to_string()),
        VideoToAudioError::InvalidInput("invalid_choice".to_string()),
        VideoToAudioError::FfmpegError("编码失败".to_string()),
        VideoToAudioError::UnsupportedFormat("xyz".to_string()),
        VideoToAudioError::MissingDependency("ffmpeg".to_string()),
    ];
    
    for (i, error) in errors.iter().enumerate() {
        println!("错误类型 {}: {}", i + 1, error);
        ui.show_error(error);
        println!();
    }
}

/// 示例 5: 完整的转换流程（模拟）
fn complete_workflow_example() -> Result<(), VideoToAudioError> {
    println!("🔄 示例 5: 完整转换流程（模拟）");
    
    // 创建临时目录和模拟文件
    let temp_dir = create_demo_environment()?;
    
    let processor = FileProcessor::new();
    let ui = UserInterface::new();
    
    println!("1. 扫描视频文件...");
    let files = processor.find_video_files(&temp_dir)?;
    println!("   找到 {} 个视频文件", files.len());
    
    println!("2. 创建输出目录...");
    let output_dir = processor.create_output_directory(&temp_dir)?;
    println!("   输出目录: {}", output_dir.display());
    
    println!("3. 显示发现的文件...");
    ui.show_files_found(files.len(), &output_dir);
    
    if !files.is_empty() {
        println!("4. 模拟批量转换...");
        
        // 模拟转换过程（不实际调用 FFmpeg）
        let total_files = files.len();
        for i in 1..=total_files {
            ui.show_progress(i, total_files);
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        println!(); // 换行
        
        println!("5. 显示完成信息...");
        ui.show_completion(total_files, &output_dir);
    }
    
    // 清理临时文件
    std::fs::remove_dir_all(&temp_dir).ok();
    
    println!();
    Ok(())
}

/// 创建演示环境
fn create_demo_environment() -> Result<std::path::PathBuf, VideoToAudioError> {
    let temp_dir = std::env::temp_dir().join("video2audio_demo");
    
    // 创建目录
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| VideoToAudioError::Io(e))?;
    
    // 创建一些模拟的视频文件
    let demo_files = vec![
        "demo1.mp4",
        "demo2.mkv", 
        "demo3.avi",
        "presentation.mov",
        "tutorial.webm",
    ];
    
    for file_name in demo_files {
        let file_path = temp_dir.join(file_name);
        std::fs::write(&file_path, "这是一个模拟的视频文件内容")
            .map_err(|e| VideoToAudioError::Io(e))?;
    }
    
    // 创建一些非视频文件（应该被忽略）
    let non_video_files = vec![
        "readme.txt",
        "image.jpg",
        "audio.mp3",
    ];
    
    for file_name in non_video_files {
        let file_path = temp_dir.join(file_name);
        std::fs::write(&file_path, "其他类型的文件内容")
            .map_err(|e| VideoToAudioError::Io(e))?;
    }
    
    // 创建子目录和嵌套文件
    let sub_dir = temp_dir.join("subfolder");
    std::fs::create_dir_all(&sub_dir)
        .map_err(|e| VideoToAudioError::Io(e))?;
    
    let nested_file = sub_dir.join("nested_video.mp4");
    std::fs::write(&nested_file, "嵌套的视频文件")
        .map_err(|e| VideoToAudioError::Io(e))?;
    
    Ok(temp_dir)
}



/// 演示库的配置和自定义
#[allow(dead_code)]
fn configuration_example() {
    println!("⚙️  配置示例: 库的自定义使用");
    
    // 演示如何检查系统环境
    println!("系统信息:");
    println!("  当前目录: {:?}", std::env::current_dir().unwrap_or_default());
    println!("  临时目录: {:?}", std::env::temp_dir());
    
    // 演示格式特性比较
    println!("\n格式特性比较:");
    println!("┌─────────┬──────────┬─────────────┬─────────────┐");
    println!("│ 格式    │ 扩展名   │ 编码参数    │ 特点        │");
    println!("├─────────┼──────────┼─────────────┼─────────────┤");
    
    for format in AudioFormat::all_formats() {
        let args_str = format.ffmpeg_args().join(" ");
        let feature = match format {
            AudioFormat::Mp3 => "高兼容性",
            AudioFormat::AacCopy => "最快速度",
            AudioFormat::Opus => "最小体积",
        };
        
        println!(
            "│ {:7} │ {:8} │ {:11} │ {:11} │",
            format!("{:?}", format),
            format.extension(),
            args_str,
            feature
        );
    }
    println!("└─────────┴──────────┴─────────────┴─────────────┘");
}

/// 性能测试示例
#[allow(dead_code)]
fn performance_example() -> Result<(), VideoToAudioError> {
    println!("📊 性能测试示例");
    
    let processor = FileProcessor::new();
    
    // 测试文件发现性能
    let start = std::time::Instant::now();
    let current_dir = std::env::current_dir().unwrap();
    let files = processor.find_video_files(&current_dir)?;
    let scan_duration = start.elapsed();
    
    println!("文件扫描性能:");
    println!("  扫描目录: {}", current_dir.display());
    println!("  找到文件: {} 个", files.len());
    println!("  扫描耗时: {:.2}ms", scan_duration.as_millis());
    
    if !files.is_empty() {
        println!("  平均每文件: {:.2}ms", 
                scan_duration.as_millis() as f64 / files.len() as f64);
    }
    
    Ok(())
}
