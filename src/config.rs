//! # 配置管理模块
//! 
//! 处理程序配置，包括命令行参数解析、配置文件管理和用户偏好设置。
//! 支持多种运行模式和自定义选项。

use crate::audio_format::AudioFormat;
use crate::error::{Result, VideoToAudioError};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 命令行参数定义
/// 
/// 使用 clap 库解析命令行参数，支持交互式和批处理模式
#[derive(Parser, Debug)]
#[command(
    name = "video2audio-rs",
    version = "0.1.0",
    about = "高性能的批量视频转音频工具",
    long_about = "Video2Audio-RS 是一个基于 Rust 开发的高性能批量视频转音频工具。\n支持多种视频格式，利用多核并行处理，提供友好的中文用户界面。"
)]
pub struct Args {
    /// 源视频文件夹路径
    #[arg(
        short = 's',
        long = "source",
        help = "指定包含视频文件的源目录路径"
    )]
    pub source_dir: Option<PathBuf>,

    /// 目标音频格式
    #[arg(
        short = 'f',
        long = "format",
        value_enum,
        help = "指定输出音频格式 [可选值: mp3, aac, opus]"
    )]
    pub format: Option<CliAudioFormat>,

    /// 输出目录（可选，默认为源目录下的 audio_exports）
    #[arg(
        short = 'o',
        long = "output",
        help = "指定音频文件输出目录"
    )]
    pub output_dir: Option<PathBuf>,

    /// 批处理模式（非交互式）
    #[arg(
        short = 'b',
        long = "batch",
        help = "启用批处理模式，跳过所有交互式提示"
    )]
    pub batch_mode: bool,

    /// 详细输出模式
    #[arg(
        short = 'v',
        long = "verbose",
        help = "启用详细输出，显示更多处理信息"
    )]
    pub verbose: bool,

    /// 静默模式
    #[arg(
        short = 'q',
        long = "quiet",
        help = "启用静默模式，只显示错误信息",
        conflicts_with = "verbose"
    )]
    pub quiet: bool,

    /// 并行处理线程数
    #[arg(
        short = 'j',
        long = "jobs",
        help = "指定并行处理的线程数 (默认为 CPU 核心数)"
    )]
    pub jobs: Option<usize>,

    /// 跳过已存在的文件
    #[arg(
        long = "skip-existing",
        help = "跳过已存在的输出文件，避免重复转换"
    )]
    pub skip_existing: bool,

    /// 显示支持的格式列表
    #[arg(
        long = "list-formats",
        help = "显示所有支持的视频和音频格式"
    )]
    pub list_formats: bool,

    /// 配置文件路径
    #[arg(
        short = 'c',
        long = "config",
        help = "指定配置文件路径"
    )]
    pub config_file: Option<PathBuf>,

    /// 保存当前设置为默认配置
    #[arg(
        long = "save-config",
        help = "将当前设置保存为默认配置"
    )]
    pub save_config: bool,
}

/// 命令行音频格式枚举
/// 
/// 用于 clap 的 ValueEnum，支持命令行参数解析
#[derive(ValueEnum, Clone, Debug)]
pub enum CliAudioFormat {
    /// MP3 格式
    Mp3,
    /// AAC 格式
    Aac,
    /// Opus 格式
    Opus,
}

impl From<CliAudioFormat> for AudioFormat {
    fn from(cli_format: CliAudioFormat) -> Self {
        match cli_format {
            CliAudioFormat::Mp3 => AudioFormat::Mp3,
            CliAudioFormat::Aac => AudioFormat::AacCopy,
            CliAudioFormat::Opus => AudioFormat::Opus,
        }
    }
}

/// 程序配置结构
/// 
/// 包含所有可配置的程序选项，支持序列化和反序列化
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// 默认音频格式
    pub default_format: String,
    
    /// 默认并行线程数
    pub default_jobs: Option<usize>,
    
    /// 是否跳过已存在的文件
    pub skip_existing: bool,
    
    /// 详细输出模式
    pub verbose: bool,
    
    /// 静默模式
    pub quiet: bool,
    
    /// 最近使用的源目录
    pub recent_source_dirs: Vec<String>,
    
    /// 用户界面语言
    pub language: String,
    
    /// 进度显示样式
    pub progress_style: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_format: "mp3".to_string(),
            default_jobs: None,
            skip_existing: false,
            verbose: false,
            quiet: false,
            recent_source_dirs: Vec::new(),
            language: "zh-CN".to_string(),
            progress_style: "detailed".to_string(),
        }
    }
}

impl Config {
    /// 从配置文件加载配置
    /// 
    /// # 参数
    /// 
    /// * `config_path` - 配置文件路径，如果为 None 则使用默认路径
    /// 
    /// # 返回值
    /// 
    /// 加载的配置或默认配置
    pub fn load(config_path: Option<&PathBuf>) -> Result<Self> {
        let config_file = match config_path {
            Some(path) => path.clone(),
            None => Self::default_config_path()?,
        };

        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            let config: Config = serde_json::from_str(&content)
                .map_err(|e| VideoToAudioError::InvalidInput(
                    format!("配置文件格式错误: {e}")
                ))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// 保存配置到文件
    /// 
    /// # 参数
    /// 
    /// * `config_path` - 配置文件路径，如果为 None 则使用默认路径
    pub fn save(&self, config_path: Option<&PathBuf>) -> Result<()> {
        let config_file = match config_path {
            Some(path) => path.clone(),
            None => Self::default_config_path()?,
        };

        // 确保配置目录存在
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| VideoToAudioError::InvalidInput(
                format!("配置序列化失败: {e}")
            ))?;

        std::fs::write(&config_file, content)?;
        Ok(())
    }

    /// 获取默认配置文件路径
    fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| VideoToAudioError::InvalidPath(
                "无法获取配置目录".to_string()
            ))?;
        
        Ok(config_dir.join("video2audio-rs").join("config.json"))
    }

    /// 添加最近使用的源目录
    /// 
    /// # 参数
    /// 
    /// * `dir` - 要添加的目录路径
    pub fn add_recent_source_dir(&mut self, dir: &str) {
        // 移除已存在的相同路径
        self.recent_source_dirs.retain(|d| d != dir);
        
        // 添加到列表开头
        self.recent_source_dirs.insert(0, dir.to_string());
        
        // 限制列表长度
        if self.recent_source_dirs.len() > 10 {
            self.recent_source_dirs.truncate(10);
        }
    }

    /// 获取默认音频格式
    pub fn get_default_format(&self) -> Result<AudioFormat> {
        AudioFormat::from_user_input(&self.default_format)
    }

    /// 设置默认音频格式
    /// 
    /// # 参数
    /// 
    /// * `format` - 要设置的音频格式
    pub fn set_default_format(&mut self, format: AudioFormat) {
        self.default_format = match format {
            AudioFormat::Mp3 => "mp3".to_string(),
            AudioFormat::AacCopy => "aac".to_string(),
            AudioFormat::Opus => "opus".to_string(),
        };
    }
}

/// 运行时配置
/// 
/// 结合命令行参数和配置文件的最终运行配置
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// 源目录路径
    pub source_dir: Option<PathBuf>,
    
    /// 音频格式
    pub format: Option<AudioFormat>,
    
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    
    /// 是否为批处理模式
    pub batch_mode: bool,
    
    /// 详细输出
    pub verbose: bool,
    
    /// 静默模式
    pub quiet: bool,
    
    /// 并行线程数
    pub jobs: Option<usize>,
    
    /// 跳过已存在文件
    pub skip_existing: bool,
    
    /// 显示格式列表
    pub list_formats: bool,
    
    /// 保存配置
    pub save_config: bool,
}

impl RuntimeConfig {
    /// 从命令行参数和配置文件创建运行时配置
    /// 
    /// # 参数
    /// 
    /// * `args` - 命令行参数
    /// * `config` - 配置文件内容
    /// 
    /// # 返回值
    /// 
    /// 合并后的运行时配置
    pub fn from_args_and_config(args: Args, config: Config) -> Self {
        Self {
            source_dir: args.source_dir,
            format: args.format.map(AudioFormat::from),
            output_dir: args.output_dir,
            batch_mode: args.batch_mode,
            verbose: args.verbose || config.verbose,
            quiet: args.quiet || config.quiet,
            jobs: args.jobs.or(config.default_jobs),
            skip_existing: args.skip_existing || config.skip_existing,
            list_formats: args.list_formats,
            save_config: args.save_config,
        }
    }

    /// 检查是否需要交互式输入
    /// 
    /// # 返回值
    /// 
    /// 如果需要交互式输入返回 true
    pub fn needs_interaction(&self) -> bool {
        !self.batch_mode && (self.source_dir.is_none() || self.format.is_none())
    }

    /// 获取并行线程数
    /// 
    /// # 返回值
    /// 
    /// 要使用的线程数，如果未指定则返回 CPU 核心数
    pub fn get_thread_count(&self) -> usize {
        self.jobs.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        })
    }
}
