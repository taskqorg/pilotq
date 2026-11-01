//! Voice and audio input (pilotq.md § 12, #104): ffmpeg decodes to 16 kHz mono WAV, then `sherpa-onnx-offline` with GigaAM v3
//! RNNT-punct int8 recognizes Russian speech on the local CPU. Both run as subprocesses, so the static build needs no onnxruntime.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Longest accepted recording (§ 12).
pub const SECONDS: u32 = 300;

/// Installation config `voice`: the recognizer binary, the GigaAM model directory and the ffmpeg binary.
pub struct Voice {
    pub ffmpeg: String,
    pub recognizer: String,
    pub model: PathBuf,
    pub threads: u32,
}

/// Private per-recording directory; removed with everything in it when the turn's recognition ends.
struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs one child to completion before `until`, stdout into `out`; a late child is killed.
fn run(c: &mut Command, out: &Path, until: Instant) -> Result<(), &'static str> {
    let remaining = until.saturating_duration_since(Instant::now());
    match crate::host::output_with_budget(c,remaining) {
        Ok(o) => std::fs::write(out,o.stdout).map_err(|_|"storage unavailable"),
        Err(e) if e.contains("timeout") || e.contains("deadline") => Err("recognition timed out"),
        Err(_) => Err("failed"),
    }
}

/// A private directory holding `input` with these bytes.
fn workdir(bytes: &[u8]) -> Result<(Dir, PathBuf), &'static str> {
    use std::io::Write;
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|_| "storage unavailable")?.as_nanos();
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    let dir = std::env::temp_dir().join(format!("pilotq-voice-{}-{nonce}", std::process::id()));
    builder.create(&dir).map_err(|_| "storage unavailable")?;
    let dir = Dir(dir);
    let input = dir.0.join("input");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    options.open(&input).and_then(|mut f| f.write_all(bytes)).map_err(|_| "storage unavailable")?;
    Ok((dir, input))
}

/// Frames of a video every 15 s, at most `FRAMES`, as JPEG bytes (#186).
pub const FRAMES: usize = 4;
pub fn frames(v: &Voice, video: &[u8], budget: Duration) -> Result<Vec<Vec<u8>>, String> {
    // Only real video containers reach ffmpeg: a playlist or concat file could make it read other local files or URLs.
    let container = video.get(4..8) == Some(b"ftyp") || video.starts_with(b"\x1a\x45\xdf\xa3")
        || (video.starts_with(b"RIFF") && video.get(8..12) == Some(b"AVI "));
    if !container { return Err("the video container is not MP4, MOV, WebM, MKV or AVI".into()); }
    let until = Instant::now() + budget;
    let (dir, input) = workdir(video).map_err(str::to_string)?;
    // Round partial intervals up so short clips survive; JPEG requires full-range YUV.
    run(Command::new(&v.ffmpeg).args(["-nostdin", "-loglevel", "error", "-protocol_whitelist", "file", "-i"]).arg(&input)
        .args(["-vf", "fps=1/15:round=up,scale='min(1280,iw)':-2", "-frames:v", &FRAMES.to_string(), "-pix_fmt", "yuvj420p", "-q:v", "4"]).arg(dir.0.join("frame%d.jpg")), &dir.0.join("out"), until)
        .map_err(|e| if e == "failed" { "the video could not be decoded".to_string() } else { format!("video {e}") })?;
    let frames: Vec<Vec<u8>> = (1..=FRAMES).map_while(|k| std::fs::read(dir.0.join(format!("frame{k}.jpg"))).ok()).collect();
    if frames.is_empty() { Err("the video has no picture frames".into()) } else { Ok(frames) }
}

/// The transcript of one recording, or a short `voice …` reason; nothing here sees a token or URL.
pub fn transcribe(v: &Voice, audio: &[u8], budget: Duration) -> Result<String, String> {
    let fail = |e: &str| format!("voice {e}");
    let until = Instant::now() + budget;
    let (dir, input) = workdir(audio).map_err(fail)?;
    let (wav, out) = (dir.0.join("audio.wav"), dir.0.join("out.json"));
    // One second past the cap tells a too-long recording from one exactly at it.
    run(Command::new(&v.ffmpeg).args(["-nostdin", "-loglevel", "error", "-i"]).arg(&input)
        .args(["-t", &(SECONDS + 1).to_string(), "-ac", "1", "-ar", "16000", "-f", "wav"]).arg(&wav), &out, until)
        .map_err(|e| fail(if e == "failed" { "could not be decoded" } else { e }))?;
    let decoded = std::fs::read(&wav).map_err(|_| fail("could not be decoded"))?;
    // ffmpeg may add LIST and other WAV chunks: count PCM, not a guessed 44-byte header.
    let mut at = 12usize;
    let mut data = None;
    while at + 8 <= decoded.len() {
        let len = u32::from_le_bytes(decoded[at+4..at+8].try_into().unwrap()) as usize;
        if &decoded[at..at+4] == b"data" { data = Some((at, (at+8+len).min(decoded.len()))); break; }
        at = at.checked_add(8 + len + len % 2).ok_or_else(|| fail("could not be decoded"))?;
    }
    let samples=data.map(|(header,end)|(end-header-8) as u64 / 2)
        .unwrap_or(decoded.len().saturating_sub(44) as u64 / 2);
    if samples > SECONDS as u64 * 16000 { return Err(fail("is longer than 5 minutes")); }
    // GigaAM's attention tensors must stay bounded; the recording cap is not a model segment cap.
    // Keep the original deadline across every segment and join transcripts only after all succeeded.
    let mut segments = vec![];
    if samples > 25 * 16000 {
        let (header, end) = data.ok_or_else(|| fail("could not be decoded"))?;
        for (n, pcm) in decoded[header+8..end].chunks(25 * 32000).enumerate() {
            let mut part = decoded[..header+8].to_vec();
            part[4..8].copy_from_slice(&((header + pcm.len()) as u32).to_le_bytes());
            part[header+4..header+8].copy_from_slice(&(pcm.len() as u32).to_le_bytes());
            part.extend_from_slice(pcm);
            let path=dir.0.join(format!("segment-{n}.wav"));
            std::fs::write(&path,part).map_err(|_| fail("storage unavailable"))?;
            segments.push(path);
        }
    } else { segments.push(wav); }
    let files = [("tokens", "tokens.txt"), ("encoder", "encoder.int8.onnx"), ("decoder", "decoder.onnx"), ("joiner", "joiner.onnx")];
    let mut transcripts=vec![];
    for segment in segments {
        run(Command::new(&v.recognizer).args(files.map(|(k, f)| format!("--{k}={}", v.model.join(f).display())))
            .args(["--model-type=nemo_transducer", &format!("--num-threads={}", v.threads)]).arg(&segment), &out, until)
            .map_err(|e| fail(if e == "failed" { "recognizer failed" } else { e }))?;
        let printed = std::fs::read_to_string(&out).map_err(|_| fail("recognizer failed"))?;
        let text = printed.lines().rev().find_map(|l| serde_json::from_str::<serde_json::Value>(l).ok()?["text"].as_str().map(str::to_string))
            .ok_or_else(|| { crate::host::log(&v.recognizer,"recognition returned no valid result JSON"); fail("recognizer failed") })?;
        if !text.trim().is_empty() { transcripts.push(text.trim().to_string()); }
    }
    if transcripts.is_empty() { Err(fail("had no recognized speech")) } else { Ok(transcripts.join(" ")) }

}

/// §9 / #120: local setup probe; silence may return empty text but must decode and produce valid recognizer JSON.
pub fn readiness(v: Option<&Voice>) -> Result<(), String> {
    let v = v.ok_or("voice not configured")?;
    let executable = |bin: &str| {
        let usable = |p: PathBuf| {
            let Ok(m) = std::fs::metadata(p) else { return false; };
            #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; m.is_file() && m.permissions().mode() & 0o111 != 0 }
            #[cfg(not(unix))] { m.is_file() }
        };
        if Path::new(bin).components().count() > 1 { usable(bin.into()) }
        else { std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| usable(d.join(bin)))) }
    };
    if !executable(&v.ffmpeg) { return Err("voice ffmpeg unavailable".into()); }
    if !executable(&v.recognizer) { return Err("voice recognizer unavailable".into()); }
    for f in ["tokens.txt", "encoder.int8.onnx", "decoder.onnx", "joiner.onnx"] {
        if !std::fs::metadata(v.model.join(f)).is_ok_and(|m| m.is_file() && m.len() > 0) {
            return Err("voice model unavailable".into());
        }
    }
    // One second of 16 kHz, mono, 16-bit PCM. No network, user recording or retained transcript.
    let mut wav = Vec::with_capacity(32044);
    wav.extend_from_slice(b"RIFF"); wav.extend_from_slice(&32036u32.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt "); wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&16000u32.to_le_bytes()); wav.extend_from_slice(&32000u32.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data"); wav.extend_from_slice(&32000u32.to_le_bytes()); wav.resize(32044, 0);
    match transcribe(v, &wav, Duration::from_secs(10)) {
        Ok(_) => Ok(()),
        Err(e) if e == "voice had no recognized speech" => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    // #259 A3 / §1: a 4:02 track never sends an oversized tensor to the recognizer.
    #[test]
    #[cfg(unix)]
    fn long_audio_259() {
        use std::{fs,os::unix::fs::PermissionsExt,time::Duration};
        let dir=std::env::temp_dir().join(format!("pilotq-long-audio-test-{}",std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let pcm=vec![0u8;242*32000];
        let mut wav=b"RIFF".to_vec(); wav.extend_from_slice(&((pcm.len()+36) as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt "); wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[1,0,1,0]); wav.extend_from_slice(&16000u32.to_le_bytes());
        wav.extend_from_slice(&32000u32.to_le_bytes()); wav.extend_from_slice(&[2,0,16,0]);
        wav.extend_from_slice(b"data"); wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes()); wav.extend_from_slice(&pcm);
        fs::write(dir.join("fixture.wav"),wav).unwrap();
        let ffmpeg=dir.join("ffmpeg"); let recognizer=dir.join("recognizer");
        fs::write(&ffmpeg,format!("#!/bin/sh\nfor last; do :; done\ncp '{}' \"$last\"\n",dir.join("fixture.wav").display())).unwrap();
        fs::write(&recognizer,"#!/bin/sh\nfor last; do :; done\n[ $(wc -c < \"$last\") -le 1000000 ] || exit 1\necho '{\"text\":\"segment\"}'\n").unwrap();
        for path in [&ffmpeg,&recognizer] {fs::set_permissions(path,fs::Permissions::from_mode(0o700)).unwrap();}
        let v=super::Voice{ffmpeg:ffmpeg.display().to_string(),recognizer:recognizer.display().to_string(),model:dir.clone(),threads:1};
        let result=super::transcribe(&v,b"invented recording",Duration::from_secs(10));
        fs::remove_dir_all(dir).unwrap();
        let result=result.unwrap();
        assert_eq!(result.split_whitespace().count(),10);
    }

    // #186 A3: a 62 s video yields four JPEG frames, one every 15 s. Needs ffmpeg on PATH, which CI lacks:
    // `cargo test --bin pilotq -- --ignored video_frames`.
    #[test]
    #[ignore]
    fn video_frames() {
        let dir = std::env::temp_dir().join(format!("pilotq-video-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("v.mp4");
        let made = std::process::Command::new("ffmpeg").args(["-nostdin", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc=duration=62:size=320x240:rate=5", "-pix_fmt", "yuv420p"]).arg(&path).status().unwrap();
        assert!(made.success());
        let v = super::Voice { ffmpeg: "/usr/bin/ffmpeg".into(), recognizer: "none".into(), model: dir.clone(), threads: 1 };
        let frames = super::frames(&v, &std::fs::read(&path).unwrap(), std::time::Duration::from_secs(30)).unwrap();
        assert_eq!(frames.len(), super::FRAMES);
        assert!(frames.iter().all(|f| f.starts_with(b"\xff\xd8\xff")));
        assert!(super::frames(&v, b"#EXTM3U\nfile:///etc/passwd", std::time::Duration::from_secs(10)).unwrap_err().contains("container"));
        assert!(super::frames(&v, b"\0\0\0\x18ftypmp42 broken", std::time::Duration::from_secs(10)).unwrap_err().contains("could not be decoded"));
        // §1 / #341: real short H264 must yield a picture, without inventing absent audio.
        let short = dir.join("short.mp4");
        assert!(std::process::Command::new("/usr/bin/ffmpeg").args(["-nostdin", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc2=duration=3:size=320x180:rate=10", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-an"]).arg(&short).status().unwrap().success());
        let bytes = std::fs::read(short).unwrap();
        let pictures = super::frames(&v, &bytes, std::time::Duration::from_secs(30)).unwrap();
        assert_eq!(pictures.len(), 1);
        assert!(pictures[0].starts_with(b"\xff\xd8\xff"));
        assert!(super::transcribe(&v, &bytes, std::time::Duration::from_secs(30)).unwrap_err().contains("could not be decoded"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
