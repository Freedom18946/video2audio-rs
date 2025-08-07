//! # 集成测试
//! 
//! 测试各个模块之间的交互和完整的工作流程

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use video2audio_rs::{AudioFormat, FileProcessor, UserInterface, VideoToAudioError};

/// 测试工具模块
mod common;

#[test]
fn test_file_processor_creation() {
    let processor = FileProcessor::new();
    let extensions = processor.supported_extensions();
    
    // 验证支持的扩展名
    assert!(!extensions.is_empty());
    assert!(extensions.contains(&"mp4"));
    assert!(extensions.contains(&"mkv"));
    assert!(extensions.contains(&"avi"));
}

#[test]
fn test_find_video_files_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    
    let files = processor.find_video_files(temp_dir.path()).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_find_video_files_with_videos() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    
    // 创建测试文件
    let video_files = ["test1.mp4", "test2.mkv", "test3.avi"];
    let non_video_files = ["readme.txt", "image.jpg", "audio.mp3"];
    
    for file in &video_files {
        let file_path = temp_dir.path().join(file);
        fs::write(file_path, "fake video content").unwrap();
    }
    
    for file in &non_video_files {
        let file_path = temp_dir.path().join(file);
        fs::write(file_path, "other content").unwrap();
    }
    
    let found_files = processor.find_video_files(temp_dir.path()).unwrap();
    
    // 应该只找到视频文件
    assert_eq!(found_files.len(), video_files.len());
    
    for video_file in &video_files {
        let expected_path = temp_dir.path().join(video_file);
        assert!(found_files.contains(&expected_path));
    }
}

#[test]
fn test_find_video_files_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    
    // 创建嵌套目录结构
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    
    let deep_dir = sub_dir.join("deep");
    fs::create_dir(&deep_dir).unwrap();
    
    // 在不同层级创建视频文件
    fs::write(temp_dir.path().join("root.mp4"), "content").unwrap();
    fs::write(sub_dir.join("sub.mkv"), "content").unwrap();
    fs::write(deep_dir.join("deep.avi"), "content").unwrap();
    
    let found_files = processor.find_video_files(temp_dir.path()).unwrap();
    
    assert_eq!(found_files.len(), 3);
}

#[test]
fn test_find_video_files_nonexistent_directory() {
    let processor = FileProcessor::new();
    let result = processor.find_video_files(Path::new("/nonexistent/directory"));
    
    assert!(result.is_err());
    match result.unwrap_err() {
        VideoToAudioError::InvalidPath(_) => (),
        _ => panic!("应该返回 InvalidPath 错误"),
    }
}

#[test]
fn test_create_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    
    let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();
    
    // 验证输出目录被创建
    assert!(output_dir.exists());
    assert!(output_dir.is_dir());
    assert_eq!(output_dir.file_name().unwrap(), "audio_exports");
    
    // 再次调用应该不会出错
    let output_dir2 = processor.create_output_directory(temp_dir.path()).unwrap();
    assert_eq!(output_dir, output_dir2);
}

#[test]
fn test_audio_format_integration() {
    // 测试所有音频格式的基本功能
    for format in AudioFormat::all_formats() {
        // 验证扩展名不为空
        assert!(!format.extension().is_empty());
        
        // 验证 FFmpeg 参数不为空
        assert!(!format.ffmpeg_args().is_empty());
        
        // 验证描述不为空
        assert!(!format.description().is_empty());
    }
}

#[test]
fn test_user_interface_creation() {
    let ui = UserInterface::new();
    
    // 测试错误显示功能
    let error = VideoToAudioError::InvalidInput("测试错误".to_string());
    ui.show_error(&error); // 不应该 panic
}

#[test]
fn test_batch_convert_empty_list() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();
    
    let files = vec![];
    let (success, failure) = processor.batch_convert(
        &files,
        &output_dir,
        AudioFormat::Mp3,
        |_current, _total| {
            // 进度回调
        },
    );
    
    assert_eq!(success, 0);
    assert_eq!(failure, 0);
}

#[test]
fn test_error_handling_chain() {
    // 测试错误类型转换
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
    let video_error: VideoToAudioError = io_error.into();
    
    match video_error {
        VideoToAudioError::Io(_) => (),
        _ => panic!("应该转换为 Io 错误"),
    }
}

#[test]
fn test_format_parsing_comprehensive() {
    // 测试各种输入格式
    let test_cases = vec![
        ("1", AudioFormat::Mp3),
        ("2", AudioFormat::AacCopy),
        ("3", AudioFormat::Opus),
        ("mp3", AudioFormat::Mp3),
        ("MP3", AudioFormat::Mp3),
        ("aac", AudioFormat::AacCopy),
        ("AAC", AudioFormat::AacCopy),
        ("opus", AudioFormat::Opus),
        ("OPUS", AudioFormat::Opus),
    ];
    
    for (input, expected) in test_cases {
        let result = AudioFormat::from_user_input(input).unwrap();
        assert_eq!(result, expected, "输入 '{input}' 应该解析为 {expected:?}");
    }
    
    // 测试无效输入
    let invalid_inputs = vec!["0", "4", "invalid", "", "   ", "mp4"];
    for input in invalid_inputs {
        assert!(AudioFormat::from_user_input(input).is_err(), "输入 '{input}' 应该返回错误");
    }
}

#[test]
fn test_file_processor_supported_extensions() {
    let processor = FileProcessor::new();
    let extensions = processor.supported_extensions();
    
    // 验证包含常见的视频格式
    let expected_formats = ["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv"];
    for format in &expected_formats {
        assert!(extensions.contains(format), "应该支持 {format} 格式");
    }
}

#[test]
fn test_progress_callback() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();

    use std::sync::{Arc, Mutex};
    let progress_calls = Arc::new(Mutex::new(Vec::new()));
    let progress_calls_clone = progress_calls.clone();

    let files = vec![];
    processor.batch_convert(
        &files,
        &output_dir,
        AudioFormat::Mp3,
        move |current, total| {
            progress_calls_clone.lock().unwrap().push((current, total));
        },
    );

    // 空文件列表不应该调用进度回调
    assert!(progress_calls.lock().unwrap().is_empty());
}

#[test]
fn test_output_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    
    let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();
    
    // 验证输出目录的结构
    assert!(output_dir.starts_with(temp_dir.path()));
    assert_eq!(output_dir.file_name().unwrap(), "audio_exports");
    
    // 验证可以在输出目录中创建文件
    let test_file = output_dir.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    assert!(test_file.exists());
}

#[test]
fn test_case_insensitive_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    
    // 创建不同大小写的视频文件
    let files = ["test.MP4", "test.MKV", "test.Avi"];
    for file in &files {
        let file_path = temp_dir.path().join(file);
        fs::write(file_path, "content").unwrap();
    }
    
    let found_files = processor.find_video_files(temp_dir.path()).unwrap();
    assert_eq!(found_files.len(), files.len());
}

#[test]
fn test_ui_progress_display() {
    let ui = UserInterface::new();
    
    // 测试进度显示不会 panic
    ui.show_progress(0, 100);
    ui.show_progress(50, 100);
    ui.show_progress(100, 100);
    
    // 测试边界情况
    ui.show_progress(0, 0); // 应该处理除零情况
    ui.show_progress(1, 1);
}

#[test]
fn test_comprehensive_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let processor = FileProcessor::new();
    let ui = UserInterface::new();
    
    // 1. 创建测试视频文件
    let video_file = temp_dir.path().join("test.mp4");
    fs::write(&video_file, "fake video content").unwrap();
    
    // 2. 查找视频文件
    let files = processor.find_video_files(temp_dir.path()).unwrap();
    assert_eq!(files.len(), 1);
    
    // 3. 创建输出目录
    let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();
    assert!(output_dir.exists());
    
    // 4. 显示文件发现结果
    ui.show_files_found(files.len(), &output_dir);
    
    // 5. 测试进度显示
    ui.show_progress(1, 1);
    
    // 6. 显示完成信息
    ui.show_completion(files.len(), &output_dir);
    
    // 整个流程应该没有 panic
}

#[test]
fn test_error_display_formatting() {
    let ui = UserInterface::new();
    
    // 测试不同类型的错误显示
    let errors = vec![
        VideoToAudioError::InvalidPath("测试路径".to_string()),
        VideoToAudioError::InvalidInput("测试输入".to_string()),
        VideoToAudioError::FfmpegError("测试FFmpeg错误".to_string()),
        VideoToAudioError::UnsupportedFormat("测试格式".to_string()),
        VideoToAudioError::MissingDependency("测试依赖".to_string()),
    ];
    
    for error in errors {
        ui.show_error(&error); // 不应该 panic
    }
}
