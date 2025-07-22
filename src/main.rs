use anyhow::{Context, Result};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

// 使用枚举来清晰地表示和管理可选的音频格式
#[derive(Debug, Clone, Copy)]
enum AudioFormat {
    Mp3,
    AacCopy,
    Opus,
}

impl AudioFormat {
    // 获取文件扩展名
    fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::AacCopy => "aac",
            AudioFormat::Opus => "opus",
        }
    }

    // 获取对应的 ffmpeg 音频编解码参数
    fn ffmpeg_args(&self) -> Vec<&'static str> {
        match self {
            // VBR 最高质量设置
            AudioFormat::Mp3 => vec!["-q:a", "0"],
            // 直接复制音频流，不重新编码
            AudioFormat::AacCopy => vec!["-c:a", "copy"],
            // 使用 libopus 编码器，设置一个不错的码率
            AudioFormat::Opus => vec!["-c:a", "libopus", "-b:a", "192k"],
        }
    }
}

fn main() -> Result<()> {
    println!("--- 批量视频转音频工具 (高并发版) ---");

    // 1. 获取用户输入的源目录
    let source_dir = get_user_input("请输入要扫描的视频文件夹绝对路径: ")?;
    let source_path = Path::new(&source_dir);
    if !source_path.is_dir() {
        anyhow::bail!("错误: '{}' 不是一个有效的目录。", source_dir);
    }

    // 2. 让用户选择输出格式
    let chosen_format = select_audio_format()?;

    // 3. 创建输出目录
    let output_dir = source_path.join("audio_exports");
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("创建输出目录 '{}' 失败", output_dir.display()))?;

    println!("输出目录: {}", output_dir.display());

    // 4. 查找所有视频文件
    let video_extensions: Vec<&str> = vec!["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv"];
    let files_to_process: Vec<PathBuf> = walkdir::WalkDir::new(source_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| video_extensions.contains(&s.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    let total_files = files_to_process.len();
    if total_files == 0 {
        println!("在 '{}' 中未找到任何支持的视频文件。", source_dir);
        return Ok(());
    }
    println!("找到 {} 个视频文件，开始转换...", total_files);

    // 5. 使用 Rayon 并发处理
    let progress_counter = Arc::new(Mutex::new(0));

    files_to_process.par_iter().for_each(|source_file| {
        match convert_file_to_audio(source_file, &output_dir, chosen_format) {
            Ok(output_path) => {
                // 成功转换，可以在这里记录成功信息（如果需要的话）
                // println!("✓ 成功转换: {}", output_path.display());
                let _ = output_path; // 明确表示我们知道这个变量但选择不使用
            }
            Err(e) => {
                // 打印错误但继续执行
                eprintln!(
                    "\n[失败] 处理文件 '{}' 时出错: {}",
                    source_file.display(),
                    e
                );
            }
        }

        // 更新并打印进度
        let mut count = progress_counter.lock().unwrap();
        *count += 1;
        print!("\r处理进度: {}/{}...", *count, total_files);
        io::stdout().flush().unwrap();
    });

    println!(
        "\n\n转换完成！所有音频文件已保存至 '{}' 目录。",
        output_dir.display()
    );

    Ok(())
}

/// 提示用户输入并获取字符串
fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

/// 显示菜单并让用户选择音频格式
fn select_audio_format() -> Result<AudioFormat> {
    loop {
        println!("\n请选择要转换的目标音频格式:");
        println!("  1. MP3  (高质量, 最佳兼容性)");
        println!("  2. AAC  (直接复制, 速度最快, 零损耗)");
        println!("  3. Opus (现代化, 高效率)");

        let choice_str = get_user_input("请输入选项 (1-3): ")?;
        match choice_str.as_str() {
            "1" => return Ok(AudioFormat::Mp3),
            "2" => return Ok(AudioFormat::AacCopy),
            "3" => return Ok(AudioFormat::Opus),
            _ => println!("无效输入，请输入 1, 2, 或 3。"),
        }
    }
}

/// 调用 ffmpeg 将单个视频文件转换为音频
fn convert_file_to_audio(
    source_file: &Path,
    output_dir: &Path,
    format: AudioFormat,
) -> Result<PathBuf> {
    // 构建输出文件名
    let file_stem = source_file
        .file_stem()
        .context("无法获取文件名")?
        .to_string_lossy();

    let output_filename = format!("{}.{}", file_stem, format.extension());
    let output_path = output_dir.join(output_filename);

    // 构建 ffmpeg 命令
    let mut args = vec!["-i", source_file.to_str().unwrap(), "-vn"];
    args.extend(format.ffmpeg_args());
    args.push(output_path.to_str().unwrap());

    let output = Command::new("ffmpeg")
        // -y: 覆盖已存在的文件
        // -hide_banner: 隐藏版本信息等
        // -loglevel error: 只在发生错误时打印日志
        .args(["-y", "-hide_banner", "-loglevel", "error"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("执行 ffmpeg 命令失败。请确保 ffmpeg 已安装并在系统 PATH 中。")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("ffmpeg 执行出错: {}", stderr));
    }

    Ok(output_path)
}
