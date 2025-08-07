//! # video2audio-rs 库
//! 
//! 这是一个高性能的视频到音频转换库，提供批量处理和多种音频格式支持。
//! 
//! ## 主要模块
//! 
//! - [`audio_format`] - 音频格式定义和处理
//! - [`file_processor`] - 文件处理和转换逻辑
//! - [`user_interface`] - 用户交互界面
//! - [`error`] - 错误处理类型定义
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use video2audio_rs::{AudioFormat, FileProcessor};
//! use std::path::Path;
//!
//! let processor = FileProcessor::new();
//! let result = processor.convert_single_file(
//!     Path::new("input.mp4"),
//!     Path::new("output"),
//!     AudioFormat::Mp3
//! );
//! ```

pub mod audio_format;
pub mod config;
pub mod error;
pub mod file_processor;
pub mod user_interface;

// 重新导出主要类型，方便外部使用
pub use audio_format::AudioFormat;
pub use config::{Args, Config, RuntimeConfig};
pub use error::{Result, VideoToAudioError};
pub use file_processor::FileProcessor;
pub use user_interface::UserInterface;
