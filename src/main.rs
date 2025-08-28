use anyhow::{Context, Result};
use clap::{ArgAction, Parser, ValueHint};
use ffmpeg_cli::{FfmpegBuilder, File as FfmpegFile, Parameter};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use tokio::fs;
use which::which;
use youtube_dl::{YoutubeDl, YoutubeDlOutput, YoutubeDlFetcher};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// YouTube URL
    url: String,

    /// Output directory for frames (will be created)
    #[arg(short, long, value_hint = ValueHint::DirPath, default_value = "frames")]
    out_dir: PathBuf,

    /// FPS for frame extraction
    #[arg(short, long, default_value_t = 10)]
    fps: u32,

    /// Optional output image pattern (default: frame_%06d.png)
    #[arg(long, default_value = "frame_%06d.png")]
    pattern: String,

    /// Optional re-scale (e.g., 1280:-1 or 720:-2). If provided, applied after fps.
    #[arg(long)]
    scale: Option<String>,

    /// Optional start time (e.g., 00:00:05)
    #[arg(long)]
    start: Option<String>,

    /// Optional duration (e.g., 10 for 10 seconds; or 00:00:10)
    #[arg(long)]
    duration: Option<String>,

    /// Keep the downloaded video file (otherwise stored in a temp dir)
    #[arg(long, action = ArgAction::SetTrue)]
    keep_video: bool,

    /// Path to save the downloaded MP4 if --keep-video (defaults to ./video.mp4)
    #[arg(long, requires = "keep_video", default_value = "video.mp4")]
    video_path: PathBuf,

    /// Force download of yt-dlp if not present
    #[arg(long, action = ArgAction::SetTrue)]
    fetch_yt_dlp: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    ensure_ffmpeg_available()?;
    ensure_yt_dlp_available(cli.fetch_yt_dlp).await?;

    let (video_path, _tmp_guard) = match cli.keep_video {
        true => (cli.video_path.clone(), None),
        false => {
            let tmp = tempdir().context("create temp dir")?;
            let path = tmp.path().join("video.mp4");
            (path, Some(tmp))
        }
    };

    download_video_as_mp4(&cli.url, &video_path).await
        .with_context(|| "downloading video with yt-dlp failed")?;

    extract_frames_with_ffmpeg(
        &video_path,
        &cli.out_dir,
        &cli.pattern,
        cli.fps,
        cli.scale.as_deref(),
        cli.start.as_deref(),
        cli.duration.as_deref(),
    ).await.with_context(|| "ffmpeg frame extraction failed")?;

    println!(" Done. Frames in: {}", cli.out_dir.display());
    Ok(())
}

fn ensure_ffmpeg_available() -> Result<()> {
    which("ffmpeg").context(
        "ffmpeg not found on PATH. ",
    )?;
    Ok(())
}

async fn ensure_yt_dlp_available(fetch_if_missing: bool) -> Result<()> {
    if which("yt-dlp").is_ok() {
        return Ok(());
    }
    if !fetch_if_missing {
        anyhow::bail!(
            "yt-dlp not found on PATH. \
             Install it (`pip install yt-dlp`, package manager) or run with --fetch-yt-dlp."
        );
    }

    // Download a local yt-dlp binary next to the executable (or CWD).
    let bin = "yt-dlp";
    let fetcher = YoutubeDlFetcher::new().with_name(bin.to_string());
    let path = fetcher.fetch().await.context("downloading yt-dlp")?;
    std::env::set_var("PATH", format!(
        "{}:{}",
        path.parent().unwrap_or(Path::new(".")) .display(),
        std::env::var("PATH").unwrap_or_default()
    ));
    Ok(())
}

async fn download_video_as_mp4(url: &str, output_path: &Path) -> Result<PathBuf> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    //ask yt-dlp for mp4 if possible, falling back to best.
    //   yt-dlp -o <output> -f "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]/best" --remux-video mp4 <URL>
    let mut ytdl = YoutubeDl::new(url);
    ytdl
        .extra_arg("-o").extra_arg(output_path.to_string_lossy())
        .extra_arg("-f").extra_arg("bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]/best")
        .extra_arg("--remux-video").extra_arg("mp4");

    let out: YoutubeDlOutput = ytdl.run_async().await?;

    if !output_path.exists() {

        anyhow::bail!("yt-dlp did not produce expected file: {}", output_path.display());
    }
    Ok(output_path.to_path_buf())
}

async fn extract_frames_with_ffmpeg(
    input: &Path,
    out_dir: &Path,
    pattern: &str,
    fps: u32,
    scale: Option<&str>,
    start: Option<&str>,
    duration: Option<&str>,
) -> Result<()> {
    fs::create_dir_all(out_dir).await.context("creating output directory")?;
    let output_pattern = out_dir.join(pattern);

    //-vf chain
    let mut vf_parts = vec![format!("fps={}", fps)];
    if let Some(s) = scale {
        vf_parts.push(format!("scale={}", s));
    }
    let vf_chain = vf_parts.join(",");

    let mut b = FfmpegBuilder::new();
    b = b.option("-hide_banner").option("-y");

    if let Some(t) = start {
        b = b.option("-ss").option(t);
    }

    b = b.input(FfmpegFile::new(input.to_string_lossy()));

    if let Some(d) = duration {
        b = b.option("-t").option(d);
    }

    b = b.output(
        FfmpegFile::new(output_pattern.to_string_lossy())
            .option(Parameter::new("-vf", &vf_chain))
            .option(Parameter::new("-vsync", "vfr")) 
            .option(Parameter::new("-frame_pts", "1")) 
    );

    b.run().await?;
    Ok(())
}
