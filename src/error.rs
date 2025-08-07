//! # 错误处理模块
//! 
//! 定义了项目中使用的所有错误类型和结果类型。
//! 提供统一的错误处理机制，便于错误信息的管理和用户友好的错误提示。

use std::fmt;

/// 项目的主要错误类型
/// 
/// 这个枚举包含了在视频转音频过程中可能遇到的所有错误情况。
/// 每种错误都提供了详细的上下文信息，便于调试和用户理解。
#[derive(Debug)]
pub enum VideoToAudioError {
    /// I/O 操作错误（文件读写、目录创建等）
    Io(std::io::Error),
    
    /// FFmpeg 执行错误
    /// 包含 FFmpeg 的错误输出信息
    FfmpegError(String),
    
    /// 文件路径相关错误
    /// 当文件路径无效或无法处理时抛出
    InvalidPath(String),
    
    /// 用户输入错误
    /// 当用户输入无效数据时抛出
    InvalidInput(String),
    
    /// 不支持的文件格式
    /// 当遇到不支持的视频或音频格式时抛出
    UnsupportedFormat(String),
    
    /// 系统依赖缺失错误
    /// 当系统缺少必要的依赖（如 FFmpeg）时抛出
    MissingDependency(String),
}

impl fmt::Display for VideoToAudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoToAudioError::Io(err) => {
                write!(f, "文件操作错误: {err}")
            }
            VideoToAudioError::FfmpegError(msg) => {
                write!(f, "FFmpeg 执行错误: {msg}")
            }
            VideoToAudioError::InvalidPath(path) => {
                write!(f, "无效的文件路径: {path}")
            }
            VideoToAudioError::InvalidInput(input) => {
                write!(f, "无效的用户输入: {input}")
            }
            VideoToAudioError::UnsupportedFormat(format) => {
                write!(f, "不支持的文件格式: {format}")
            }
            VideoToAudioError::MissingDependency(dep) => {
                write!(f, "缺少系统依赖: {dep}")
            }
        }
    }
}

impl std::error::Error for VideoToAudioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VideoToAudioError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// 实现从标准库错误类型的自动转换
impl From<std::io::Error> for VideoToAudioError {
    fn from(err: std::io::Error) -> Self {
        VideoToAudioError::Io(err)
    }
}

/// 项目的结果类型别名
///
/// 这是一个便利类型，将标准库的 Result 与我们的错误类型结合。
/// 在整个项目中使用这个类型可以保持一致性并简化错误处理。
pub type Result<T> = std::result::Result<T, VideoToAudioError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_error_display() {
        let io_err = VideoToAudioError::Io(io::Error::new(io::ErrorKind::NotFound, "文件未找到"));
        assert!(io_err.to_string().contains("文件操作错误"));

        let ffmpeg_err = VideoToAudioError::FfmpegError("编码失败".to_string());
        assert_eq!(ffmpeg_err.to_string(), "FFmpeg 执行错误: 编码失败");

        let path_err = VideoToAudioError::InvalidPath("/invalid/path".to_string());
        assert_eq!(path_err.to_string(), "无效的文件路径: /invalid/path");

        let input_err = VideoToAudioError::InvalidInput("无效选择".to_string());
        assert_eq!(input_err.to_string(), "无效的用户输入: 无效选择");

        let format_err = VideoToAudioError::UnsupportedFormat("xyz".to_string());
        assert_eq!(format_err.to_string(), "不支持的文件格式: xyz");

        let dep_err = VideoToAudioError::MissingDependency("ffmpeg".to_string());
        assert_eq!(dep_err.to_string(), "缺少系统依赖: ffmpeg");
    }

    #[test]
    fn test_error_source() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "权限不足");
        let video_error = VideoToAudioError::Io(io_error);

        assert!(video_error.source().is_some());

        let ffmpeg_error = VideoToAudioError::FfmpegError("测试错误".to_string());
        assert!(ffmpeg_error.source().is_none());
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "文件不存在");
        let video_error: VideoToAudioError = io_error.into();

        match video_error {
            VideoToAudioError::Io(_) => (),
            _ => panic!("应该转换为 Io 错误"),
        }
    }

    #[test]
    fn test_result_type() {
        fn test_function() -> Result<String> {
            Ok("成功".to_string())
        }

        fn test_error_function() -> Result<String> {
            Err(VideoToAudioError::InvalidInput("测试错误".to_string()))
        }

        assert!(test_function().is_ok());
        assert!(test_error_function().is_err());
    }

    #[test]
    fn test_error_debug() {
        let error = VideoToAudioError::InvalidPath("test".to_string());
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("InvalidPath"));
        assert!(debug_str.contains("test"));
    }
}
