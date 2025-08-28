# ytframes

Download a YouTube video and extract frames to images 
Built on [yt-dlp](https://github.com/yt-dlp/yt-dlp) (via the [`youtube_dl`](https://crates.io/crates/youtube_dl) crate) and (via the [`ffmpeg_cli`](https://crates.io/crates/ffmpeg_cli) crate).

> just glueing things together 


## 🚀 Quick Start

1. **Install dependencies**

```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt update && sudo apt install -y ffmpeg
```

Install ytframes

```
cargo install ytframes
```

Run:
```
ytframes "https://www.youtube.com/watch?v=VIDEO_ID"
```


From source:

```
git clone https://github.com/<yourname>/ytframes
cd ytframes
cargo build --release
./target/release/ytframes --help
```

⚙️ Requirements
```
ffmpeg (must be on PATH)

yt-dlp (on PATH) — or use --fetch-yt-dlp to auto-download a local copy

Rust 1.74+
```




Usage: ytframes [OPTIONS] <URL>

Arguments:
  <URL>  YouTube URL

```
Options:
  -o, --out-dir <OUT_DIR>   Output directory for frames [default: frames]
  -f, --fps <FPS>           FPS for frame extraction [default: 10]
      --pattern <PATTERN>   Output filename pattern [default: frame_%06d.png]
      --scale <SCALE>       Optional scale (e.g., 1280:-1, 720:-2)
      --start <START>       Optional start time (e.g., 00:00:05)
      --duration <DURATION> Optional duration (e.g., 10, or 00:00:10)
      --keep-video          Keep the downloaded video file
      --video-path <PATH>   Path to save video if --keep-video [default: video.mp4]
      --fetch-yt-dlp        Download yt-dlp automatically if missing
  -h, --help                Print help
  -V, --version             Print version

```


🔍 Examples

Extract frames at 5 fps into shots/:

ytframes -f 5 -o shots "https://www.youtube.com/watch?v=VIDEO_ID"


Extract first 10 seconds, scaled to 720px width:

ytframes --start 00:00:00 --duration 10 --scale 720:-1 "https://youtu.be/VIDEO_ID"
