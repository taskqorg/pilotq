//! Voice and audio input (pilotq.md § 12, #104): ffmpeg decodes to 16 kHz mono WAV, then `sherpa-onnx-offline` with GigaAM v3
//! RNNT-punct int8 recognizes Russian speech on the local CPU. Both run as subprocesses, so the static build needs no onnxruntime.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
    let out = std::fs::File::create(out).map_err(|_| "storage unavailable")?;
    let mut child = c.stdin(Stdio::null()).stdout(out).stderr(Stdio::null()).spawn().map_err(|_| "recognizer unavailable")?;
    loop {
        if let Some(status) = child.try_wait().map_err(|_| "recognizer unavailable")? {
            return if status.success() { Ok(()) } else { Err("failed") };
        }
        if Instant::now() >= until {
            let _ = child.kill();
            let _ = child.wait();
            return Err("recognition timed out");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The transcript of one recording, or a short `voice …` reason; nothing here sees a token or URL.
pub fn transcribe(v: &Voice, audio: &[u8], budget: Duration) -> Result<String, String> {
    use std::io::Write;
    let fail = |e: &str| format!("voice {e}");
    let until = Instant::now() + budget;
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|_| fail("storage unavailable"))?.as_nanos();
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    let dir = std::env::temp_dir().join(format!("pilotq-voice-{}-{nonce}", std::process::id()));
    builder.create(&dir).map_err(|_| fail("storage unavailable"))?;
    let dir = Dir(dir);
    let (input, wav, out) = (dir.0.join("input"), dir.0.join("audio.wav"), dir.0.join("out.json"));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    options.open(&input).and_then(|mut f| f.write_all(audio)).map_err(|_| fail("storage unavailable"))?;
    // One second past the cap tells a too-long recording from one exactly at it.
    run(Command::new(&v.ffmpeg).args(["-nostdin", "-loglevel", "error", "-i"]).arg(&input)
        .args(["-t", &(SECONDS + 1).to_string(), "-ac", "1", "-ar", "16000", "-f", "wav"]).arg(&wav), &out, until)
        .map_err(|e| fail(if e == "failed" { "could not be decoded" } else { e }))?;
    let samples = std::fs::metadata(&wav).map_err(|_| fail("could not be decoded"))?.len().saturating_sub(44) / 2;
    if samples > SECONDS as u64 * 16000 { return Err(fail("is longer than 5 minutes")); }
    let files = [("tokens", "tokens.txt"), ("encoder", "encoder.int8.onnx"), ("decoder", "decoder.onnx"), ("joiner", "joiner.onnx")];
    run(Command::new(&v.recognizer).args(files.map(|(k, f)| format!("--{k}={}", v.model.join(f).display())))
        .args(["--model-type=nemo_transducer", &format!("--num-threads={}", v.threads)]).arg(&wav), &out, until)
        .map_err(|e| fail(if e == "failed" { "recognizer failed" } else { e }))?;
    // The recognizer prints its result as one JSON line after its configuration dump.
    let printed = std::fs::read_to_string(&out).map_err(|_| fail("recognizer failed"))?;
    let text = printed.lines().rev().find_map(|l| serde_json::from_str::<serde_json::Value>(l).ok()?["text"].as_str().map(str::to_string))
        .ok_or_else(|| fail("recognizer failed"))?;
    let text = text.trim();
    if text.is_empty() { Err(fail("had no recognized speech")) } else { Ok(text.to_string()) }
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
