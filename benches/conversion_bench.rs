//! # Video2Audio-RS 性能基准测试
//! 
//! 这个基准测试用于评估各个组件的性能表现。
//! 
//! ## 运行基准测试
//! 
//! ```bash
//! cargo bench
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use video2audio_rs::{AudioFormat, FileProcessor};

/// 基准测试：文件发现性能
fn bench_file_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_discovery");
    
    // 测试不同数量的文件
    let file_counts = vec![10, 50, 100, 500];
    
    for count in file_counts {
        let temp_dir = create_test_files(count);
        let processor = FileProcessor::new();
        
        group.bench_with_input(
            BenchmarkId::new("find_video_files", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let files = processor.find_video_files(temp_dir.path()).unwrap();
                    black_box(files);
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：音频格式解析性能
fn bench_audio_format_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_format_parsing");
    
    let test_inputs = vec![
        "1", "2", "3",
        "mp3", "aac", "opus",
        "MP3", "AAC", "OPUS",
    ];
    
    group.bench_function("from_user_input", |b| {
        b.iter(|| {
            for input in &test_inputs {
                let result = AudioFormat::from_user_input(input);
                black_box(result);
            }
        });
    });
    
    group.bench_function("ffmpeg_args", |b| {
        let formats = AudioFormat::all_formats();
        b.iter(|| {
            for format in &formats {
                let args = format.ffmpeg_args();
                black_box(args);
            }
        });
    });
    
    group.finish();
}

/// 基准测试：目录创建性能
fn bench_directory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_operations");
    
    let processor = FileProcessor::new();
    
    group.bench_function("create_output_directory", |b| {
        b.iter_batched(
            || TempDir::new().unwrap(),
            |temp_dir| {
                let result = processor.create_output_directory(temp_dir.path());
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// 基准测试：批量处理性能（不实际转换）
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");
    
    let file_counts = vec![10, 50, 100];
    
    for count in file_counts {
        let temp_dir = create_test_files(count);
        let processor = FileProcessor::new();
        let files = processor.find_video_files(temp_dir.path()).unwrap();
        let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("batch_convert_setup", count),
            &count,
            |b, _| {
                b.iter(|| {
                    // 只测试批量处理的设置开销，不实际转换
                    let (success, failure) = processor.batch_convert(
                        &files,
                        &output_dir,
                        AudioFormat::Mp3,
                        |_current, _total| {
                            // 空的进度回调
                        },
                    );
                    black_box((success, failure));
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：文件扩展名检查性能
fn bench_extension_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("extension_checking");
    
    let processor = FileProcessor::new();
    let test_files = vec![
        "test.mp4", "test.mkv", "test.avi", "test.mov",
        "test.webm", "test.flv", "test.wmv", "test.m4v",
        "test.txt", "test.jpg", "test.mp3", "test.pdf",
    ];
    
    group.bench_function("supported_extensions_check", |b| {
        b.iter(|| {
            for file_name in &test_files {
                let path = Path::new(file_name);
                let is_supported = path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| processor.supported_extensions().contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false);
                black_box(is_supported);
            }
        });
    });
    
    group.finish();
}

/// 基准测试：递归目录遍历性能
fn bench_recursive_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("recursive_traversal");
    
    let depths = vec![1, 3, 5];
    
    for depth in depths {
        let temp_dir = create_nested_structure(depth, 10);
        let processor = FileProcessor::new();
        
        group.bench_with_input(
            BenchmarkId::new("recursive_find", depth),
            &depth,
            |b, _| {
                b.iter(|| {
                    let files = processor.find_video_files(temp_dir.path()).unwrap();
                    black_box(files);
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：内存使用效率
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // 测试大量文件的内存使用
    let large_file_count = 1000;
    let temp_dir = create_test_files(large_file_count);
    let processor = FileProcessor::new();
    
    group.bench_function("large_file_list_processing", |b| {
        b.iter(|| {
            let files = processor.find_video_files(temp_dir.path()).unwrap();
            
            // 模拟对文件列表的处理
            let total_size: usize = files.iter()
                .map(|path| path.to_string_lossy().len())
                .sum();
            
            black_box(total_size);
        });
    });
    
    group.finish();
}

/// 基准测试：并发性能
fn bench_concurrency(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency");
    
    let temp_dir = create_test_files(100);
    let processor = FileProcessor::new();
    let files = processor.find_video_files(temp_dir.path()).unwrap();
    let output_dir = processor.create_output_directory(temp_dir.path()).unwrap();
    
    group.bench_function("parallel_iteration", |b| {
        b.iter(|| {
            use rayon::prelude::*;
            
            let results: Vec<_> = files.par_iter()
                .map(|file| {
                    // 模拟一些计算工作
                    let name_len = file.file_name()
                        .map(|n| n.to_string_lossy().len())
                        .unwrap_or(0);
                    name_len * 2
                })
                .collect();
            
            black_box(results);
        });
    });
    
    group.finish();
}

/// 辅助函数：创建测试文件
fn create_test_files(count: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    let video_extensions = ["mp4", "mkv", "avi", "mov", "webm"];
    let non_video_extensions = ["txt", "jpg", "mp3", "pdf"];
    
    // 创建视频文件
    for i in 0..count {
        let ext = video_extensions[i % video_extensions.len()];
        let file_name = format!("video_{}.{}", i, ext);
        let file_path = temp_dir.path().join(file_name);
        fs::write(file_path, "fake video content").unwrap();
    }
    
    // 创建一些非视频文件
    for i in 0..(count / 4) {
        let ext = non_video_extensions[i % non_video_extensions.len()];
        let file_name = format!("other_{}.{}", i, ext);
        let file_path = temp_dir.path().join(file_name);
        fs::write(file_path, "other content").unwrap();
    }
    
    temp_dir
}

/// 辅助函数：创建嵌套目录结构
fn create_nested_structure(depth: usize, files_per_level: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    fn create_level(base_path: &Path, current_depth: usize, max_depth: usize, files_per_level: usize) {
        if current_depth > max_depth {
            return;
        }
        
        // 在当前层级创建文件
        for i in 0..files_per_level {
            let file_name = format!("video_{}_{}.mp4", current_depth, i);
            let file_path = base_path.join(file_name);
            fs::write(file_path, "fake video content").unwrap();
        }
        
        // 创建子目录并递归
        if current_depth < max_depth {
            for i in 0..2 {
                let sub_dir = base_path.join(format!("level_{}_{}", current_depth, i));
                fs::create_dir_all(&sub_dir).unwrap();
                create_level(&sub_dir, current_depth + 1, max_depth, files_per_level);
            }
        }
    }
    
    create_level(temp_dir.path(), 1, depth, files_per_level);
    temp_dir
}

/// 基准测试：字符串操作性能
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    
    let test_paths = vec![
        "/path/to/video.mp4",
        "/very/long/path/to/some/video/file.mkv",
        "C:\\Windows\\Path\\To\\Video.avi",
        "/path/with/unicode/视频文件.mov",
    ];
    
    group.bench_function("path_extension_extraction", |b| {
        b.iter(|| {
            for path_str in &test_paths {
                let path = Path::new(path_str);
                let extension = path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_lowercase());
                black_box(extension);
            }
        });
    });
    
    group.bench_function("filename_generation", |b| {
        b.iter(|| {
            for path_str in &test_paths {
                let path = Path::new(path_str);
                if let Some(stem) = path.file_stem() {
                    for format in AudioFormat::all_formats() {
                        let output_name = format!("{}.{}", 
                                                stem.to_string_lossy(), 
                                                format.extension());
                        black_box(output_name);
                    }
                }
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_file_discovery,
    bench_audio_format_parsing,
    bench_directory_operations,
    bench_batch_processing,
    bench_extension_checking,
    bench_recursive_traversal,
    bench_memory_efficiency,
    bench_concurrency,
    bench_string_operations
);

criterion_main!(benches);
