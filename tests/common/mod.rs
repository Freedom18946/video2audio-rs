//! # 测试工具模块
//! 
//! 提供测试中使用的公共工具函数和辅助类型

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// 测试文件创建器
/// 
/// 用于在临时目录中创建测试文件的辅助结构
pub struct TestFileBuilder {
    temp_dir: TempDir,
}

impl TestFileBuilder {
    /// 创建新的测试文件构建器
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("无法创建临时目录"),
        }
    }

    /// 获取临时目录路径
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// 创建视频文件
    /// 
    /// # 参数
    /// 
    /// * `name` - 文件名（包含扩展名）
    /// * `content` - 文件内容（可选，默认为 "fake video content"）
    pub fn create_video_file(&self, name: &str, content: Option<&str>) -> PathBuf {
        let file_path = self.temp_dir.path().join(name);
        let content = content.unwrap_or("fake video content");
        fs::write(&file_path, content).expect("无法创建测试文件");
        file_path
    }

    /// 创建非视频文件
    pub fn create_non_video_file(&self, name: &str, content: Option<&str>) -> PathBuf {
        let file_path = self.temp_dir.path().join(name);
        let content = content.unwrap_or("other content");
        fs::write(&file_path, content).expect("无法创建测试文件");
        file_path
    }

    /// 创建子目录
    pub fn create_subdirectory(&self, name: &str) -> PathBuf {
        let dir_path = self.temp_dir.path().join(name);
        fs::create_dir_all(&dir_path).expect("无法创建子目录");
        dir_path
    }

    /// 在子目录中创建视频文件
    pub fn create_video_file_in_subdir(&self, subdir: &str, name: &str) -> PathBuf {
        let subdir_path = self.create_subdirectory(subdir);
        let file_path = subdir_path.join(name);
        fs::write(&file_path, "fake video content").expect("无法创建测试文件");
        file_path
    }

    /// 批量创建测试文件
    /// 
    /// # 参数
    /// 
    /// * `video_files` - 视频文件名列表
    /// * `other_files` - 其他文件名列表
    pub fn create_test_files(&self, video_files: &[&str], other_files: &[&str]) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let video_paths: Vec<PathBuf> = video_files
            .iter()
            .map(|name| self.create_video_file(name, None))
            .collect();

        let other_paths: Vec<PathBuf> = other_files
            .iter()
            .map(|name| self.create_non_video_file(name, None))
            .collect();

        (video_paths, other_paths)
    }
}

impl Default for TestFileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试断言辅助函数
pub mod assertions {
    use std::path::Path;

    /// 断言路径存在且是文件
    pub fn assert_file_exists(path: &Path) {
        assert!(path.exists(), "文件应该存在: {}", path.display());
        assert!(path.is_file(), "路径应该是文件: {}", path.display());
    }

    /// 断言路径存在且是目录
    pub fn assert_dir_exists(path: &Path) {
        assert!(path.exists(), "目录应该存在: {}", path.display());
        assert!(path.is_dir(), "路径应该是目录: {}", path.display());
    }

    /// 断言路径不存在
    pub fn assert_not_exists(path: &Path) {
        assert!(!path.exists(), "路径不应该存在: {}", path.display());
    }

    /// 断言向量包含指定路径
    pub fn assert_contains_path(paths: &[std::path::PathBuf], expected: &Path) {
        assert!(
            paths.iter().any(|p| p == expected),
            "路径列表应该包含: {}",
            expected.display()
        );
    }

    /// 断言向量不包含指定路径
    pub fn assert_not_contains_path(paths: &[std::path::PathBuf], unexpected: &Path) {
        assert!(
            !paths.iter().any(|p| p == unexpected),
            "路径列表不应该包含: {}",
            unexpected.display()
        );
    }
}

/// 测试数据生成器
pub mod generators {
    /// 生成测试视频文件名
    pub fn video_filenames() -> Vec<&'static str> {
        vec![
            "test1.mp4",
            "test2.mkv",
            "test3.avi",
            "test4.mov",
            "test5.webm",
            "test6.flv",
            "test7.wmv",
            "test8.m4v",
            "test9.3gp",
            "test10.ts",
        ]
    }

    /// 生成非视频文件名
    pub fn non_video_filenames() -> Vec<&'static str> {
        vec![
            "readme.txt",
            "image.jpg",
            "audio.mp3",
            "document.pdf",
            "archive.zip",
            "script.sh",
            "config.json",
            "data.csv",
        ]
    }

    /// 生成混合大小写的视频文件名
    pub fn mixed_case_video_filenames() -> Vec<&'static str> {
        vec![
            "Test1.MP4",
            "TEST2.MKV",
            "test3.Avi",
            "Test4.MOV",
            "TEST5.webm",
        ]
    }

    /// 生成无效的音频格式输入
    pub fn invalid_audio_format_inputs() -> Vec<&'static str> {
        vec![
            "0",
            "4",
            "invalid",
            "",
            "   ",
            "mp4",
            "video",
            "audio",
            "-1",
            "1.5",
        ]
    }

    /// 生成有效的音频格式输入
    pub fn valid_audio_format_inputs() -> Vec<(&'static str, video2audio_rs::AudioFormat)> {
        vec![
            ("1", video2audio_rs::AudioFormat::Mp3),
            ("2", video2audio_rs::AudioFormat::AacCopy),
            ("3", video2audio_rs::AudioFormat::Opus),
            ("mp3", video2audio_rs::AudioFormat::Mp3),
            ("MP3", video2audio_rs::AudioFormat::Mp3),
            ("aac", video2audio_rs::AudioFormat::AacCopy),
            ("AAC", video2audio_rs::AudioFormat::AacCopy),
            ("opus", video2audio_rs::AudioFormat::Opus),
            ("OPUS", video2audio_rs::AudioFormat::Opus),
        ]
    }
}

/// 模拟进度回调
pub struct MockProgressCallback {
    pub calls: std::sync::Mutex<Vec<(usize, usize)>>,
}

impl MockProgressCallback {
    pub fn new() -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn callback(&self) -> impl Fn(usize, usize) + '_ {
        move |current, total| {
            self.calls.lock().unwrap().push((current, total));
        }
    }

    pub fn get_calls(&self) -> Vec<(usize, usize)> {
        self.calls.lock().unwrap().clone()
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

impl Default for MockProgressCallback {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试环境设置
pub struct TestEnvironment {
    pub file_builder: TestFileBuilder,
    pub progress_callback: MockProgressCallback,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            file_builder: TestFileBuilder::new(),
            progress_callback: MockProgressCallback::new(),
        }
    }

    /// 设置标准测试场景
    /// 
    /// 创建包含视频文件和非视频文件的测试目录结构
    pub fn setup_standard_scenario(&self) -> (Vec<std::path::PathBuf>, Vec<std::path::PathBuf>) {
        let video_files = generators::video_filenames();
        let non_video_files = generators::non_video_filenames();
        
        self.file_builder.create_test_files(&video_files[..3], &non_video_files[..3])
    }

    /// 设置嵌套目录场景
    pub fn setup_nested_scenario(&self) -> Vec<std::path::PathBuf> {
        let mut created_files = Vec::new();
        
        // 根目录文件
        created_files.push(self.file_builder.create_video_file("root.mp4", None));
        
        // 子目录文件
        created_files.push(self.file_builder.create_video_file_in_subdir("subdir", "sub.mkv"));
        
        // 深层嵌套文件
        let deep_dir = self.file_builder.create_subdirectory("subdir/deep");
        let deep_file = deep_dir.join("deep.avi");
        std::fs::write(&deep_file, "fake video content").unwrap();
        created_files.push(deep_file);
        
        created_files
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
