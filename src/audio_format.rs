//! # 音频格式模块
//! 
//! 定义和管理支持的音频格式，包括格式特性、编码参数和文件扩展名。
//! 每种格式都针对不同的使用场景进行了优化。

use crate::error::{Result, VideoToAudioError};

/// 支持的音频格式枚举
/// 
/// 每种格式都有其特定的用途和优势：
/// - MP3: 最广泛兼容，适合一般用途
/// - AAC: 高效压缩，适合移动设备
/// - Opus: 现代化编码，适合网络传输
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// MP3 格式 - 使用 VBR 最高质量设置
    /// 
    /// 优点：
    /// - 最广泛的兼容性
    /// - 成熟的编码技术
    /// - 良好的质量/大小平衡
    Mp3,
    
    /// AAC 格式 - 直接复制音频流
    /// 
    /// 优点：
    /// - 零损耗转换（如果源文件已是 AAC）
    /// - 转换速度最快
    /// - 现代设备广泛支持
    AacCopy,
    
    /// Opus 格式 - 现代高效编码
    /// 
    /// 优点：
    /// - 最先进的音频编码技术
    /// - 优秀的压缩效率
    /// - 低延迟特性
    Opus,
}

impl AudioFormat {
    /// 获取音频格式对应的文件扩展名
    /// 
    /// # 返回值
    /// 
    /// 返回不带点号的文件扩展名字符串
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use video2audio_rs::AudioFormat;
    /// 
    /// assert_eq!(AudioFormat::Mp3.extension(), "mp3");
    /// assert_eq!(AudioFormat::AacCopy.extension(), "aac");
    /// assert_eq!(AudioFormat::Opus.extension(), "opus");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::AacCopy => "aac",
            AudioFormat::Opus => "opus",
        }
    }

    /// 获取 FFmpeg 编码参数
    /// 
    /// 返回适用于当前音频格式的 FFmpeg 命令行参数。
    /// 这些参数经过优化，在质量和文件大小之间取得平衡。
    /// 
    /// # 返回值
    /// 
    /// 返回 FFmpeg 参数的字符串向量
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use video2audio_rs::AudioFormat;
    /// 
    /// let mp3_args = AudioFormat::Mp3.ffmpeg_args();
    /// assert_eq!(mp3_args, vec!["-q:a", "0"]);
    /// ```
    pub fn ffmpeg_args(&self) -> Vec<&'static str> {
        match self {
            // VBR 最高质量设置 - 可变比特率，质量优先
            AudioFormat::Mp3 => vec!["-q:a", "0"],
            
            // 直接复制音频流，不重新编码 - 最快速度，零损耗
            AudioFormat::AacCopy => vec!["-c:a", "copy"],
            
            // 使用 libopus 编码器，192k 码率 - 现代化高效编码
            AudioFormat::Opus => vec!["-c:a", "libopus", "-b:a", "192k"],
        }
    }

    /// 从用户输入字符串解析音频格式
    /// 
    /// 支持数字选择（1-3）和格式名称（不区分大小写）
    /// 
    /// # 参数
    /// 
    /// * `input` - 用户输入的字符串
    /// 
    /// # 返回值
    /// 
    /// 成功时返回对应的 `AudioFormat`，失败时返回错误
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use video2audio_rs::AudioFormat;
    /// 
    /// assert_eq!(AudioFormat::from_user_input("1").unwrap(), AudioFormat::Mp3);
    /// assert_eq!(AudioFormat::from_user_input("mp3").unwrap(), AudioFormat::Mp3);
    /// assert_eq!(AudioFormat::from_user_input("MP3").unwrap(), AudioFormat::Mp3);
    /// ```
    pub fn from_user_input(input: &str) -> Result<Self> {
        let input = input.trim().to_lowercase();
        match input.as_str() {
            "1" | "mp3" => Ok(AudioFormat::Mp3),
            "2" | "aac" | "aac-copy" => Ok(AudioFormat::AacCopy),
            "3" | "opus" => Ok(AudioFormat::Opus),
            _ => Err(VideoToAudioError::InvalidInput(format!(
                "不支持的音频格式选择: '{input}'. 请选择 1-3 或格式名称 (mp3/aac/opus)"
            ))),
        }
    }

    /// 获取格式的中文描述
    /// 
    /// 返回用户友好的中文格式描述，用于界面显示
    /// 
    /// # 返回值
    /// 
    /// 格式的中文描述字符串
    pub fn description(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "MP3 (高质量, 最佳兼容性)",
            AudioFormat::AacCopy => "AAC (直接复制, 速度最快, 零损耗)",
            AudioFormat::Opus => "Opus (现代化, 高效率)",
        }
    }

    /// 获取所有支持的音频格式
    ///
    /// 返回包含所有可用音频格式的向量，用于遍历或显示选项
    ///
    /// # 返回值
    ///
    /// 包含所有 `AudioFormat` 变体的向量
    pub fn all_formats() -> Vec<Self> {
        vec![AudioFormat::Mp3, AudioFormat::AacCopy, AudioFormat::Opus]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::AacCopy.extension(), "aac");
        assert_eq!(AudioFormat::Opus.extension(), "opus");
    }

    #[test]
    fn test_ffmpeg_args() {
        assert_eq!(AudioFormat::Mp3.ffmpeg_args(), vec!["-q:a", "0"]);
        assert_eq!(AudioFormat::AacCopy.ffmpeg_args(), vec!["-c:a", "copy"]);
        assert_eq!(AudioFormat::Opus.ffmpeg_args(), vec!["-c:a", "libopus", "-b:a", "192k"]);
    }

    #[test]
    fn test_from_user_input_numbers() {
        assert_eq!(AudioFormat::from_user_input("1").unwrap(), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_user_input("2").unwrap(), AudioFormat::AacCopy);
        assert_eq!(AudioFormat::from_user_input("3").unwrap(), AudioFormat::Opus);
    }

    #[test]
    fn test_from_user_input_names() {
        assert_eq!(AudioFormat::from_user_input("mp3").unwrap(), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_user_input("MP3").unwrap(), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_user_input("aac").unwrap(), AudioFormat::AacCopy);
        assert_eq!(AudioFormat::from_user_input("AAC").unwrap(), AudioFormat::AacCopy);
        assert_eq!(AudioFormat::from_user_input("aac-copy").unwrap(), AudioFormat::AacCopy);
        assert_eq!(AudioFormat::from_user_input("opus").unwrap(), AudioFormat::Opus);
        assert_eq!(AudioFormat::from_user_input("OPUS").unwrap(), AudioFormat::Opus);
    }

    #[test]
    fn test_from_user_input_invalid() {
        assert!(AudioFormat::from_user_input("4").is_err());
        assert!(AudioFormat::from_user_input("invalid").is_err());
        assert!(AudioFormat::from_user_input("").is_err());
        assert!(AudioFormat::from_user_input("   ").is_err());
    }

    #[test]
    fn test_description() {
        assert_eq!(AudioFormat::Mp3.description(), "MP3 (高质量, 最佳兼容性)");
        assert_eq!(AudioFormat::AacCopy.description(), "AAC (直接复制, 速度最快, 零损耗)");
        assert_eq!(AudioFormat::Opus.description(), "Opus (现代化, 高效率)");
    }

    #[test]
    fn test_all_formats() {
        let formats = AudioFormat::all_formats();
        assert_eq!(formats.len(), 3);
        assert!(formats.contains(&AudioFormat::Mp3));
        assert!(formats.contains(&AudioFormat::AacCopy));
        assert!(formats.contains(&AudioFormat::Opus));
    }

    #[test]
    fn test_format_equality() {
        assert_eq!(AudioFormat::Mp3, AudioFormat::Mp3);
        assert_ne!(AudioFormat::Mp3, AudioFormat::AacCopy);
    }

    #[test]
    fn test_format_clone_copy() {
        let format = AudioFormat::Mp3;
        let cloned = format; // Copy trait automatically used
        let copied = format;

        assert_eq!(format, cloned);
        assert_eq!(format, copied);
    }
}
