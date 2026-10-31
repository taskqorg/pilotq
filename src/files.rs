//! #186: documents sent to the conversation become readable text for the model turn: PDF text, archive listing or plain
//! text, else the exact reason they cannot be read. The content exists only in that turn's input, never in the journal.

/// Text handed to the model per file; the rest is cut with an explicit note.
const TEXT: usize = 50_000;

fn cut(text: &str) -> String {
    if text.chars().count() <= TEXT { return text.to_string(); }
    format!("{}\n[cut: the first {TEXT} characters of a longer text]", text.chars().take(TEXT).collect::<String>())
}

/// Readable content of one document, or why it cannot be read. Bytes are untrusted; nothing here runs or writes them.
pub fn read(name: &str, bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(b"%PDF") { return pdf(bytes); }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") { return zip(bytes); }
    if bytes.starts_with(b"Rar!\x1a\x07") { return Err("RAR archives cannot be opened here; send it as ZIP".into()); }
    if bytes.starts_with(b"7z\xbc\xaf\x27\x1c") { return Err("7z archives cannot be opened here; send it as ZIP".into()); }
    if bytes.starts_with(b"\x1f\x8b") { return Err("gzip/tar archives cannot be opened here; send it as ZIP".into()); }
    match std::str::from_utf8(bytes) {
        Ok(text) if !text.contains('\0') => Ok(cut(text)),
        _ => Err(format!("{name} is a binary file of an unsupported type; send PDF, ZIP or text")),
    }
}

/// Decompressed content read from one PDF; a compression bomb stops here.
const INFLATED: usize = 64 * 1024 * 1024;
const PAGES: usize = 300;

// The parent isolates this parser: data caps alone do not bound parser time.
fn pdf(bytes: &[u8]) -> Result<String, String> {
    let parsed = std::panic::catch_unwind(|| {
        let doc = lopdf::Document::load_mem(bytes).ok()?;
        let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
        let text = doc.extract_text_with_limit(&pages[..pages.len().min(PAGES)], INFLATED).unwrap_or_default();
        Some((pages, text))
    });
    let (pages, text) = parsed.ok().flatten().ok_or("the PDF is damaged or password-protected and cannot be opened")?;
    if text.trim().is_empty() { return Err(format!("the PDF has {} page(s) but no extractable text (scanned images?)", pages.len())); }
    Ok(format!("PDF, {} page(s):\n{}", pages.len(), cut(text.trim())))
}

/// File list from the ZIP central directory; entries are listed, never extracted.
// ponytail: listing only; read entries (docx/xlsx text) when a person needs them.
fn zip(bytes: &[u8]) -> Result<String, String> {
    let bad = || "the ZIP archive is damaged and cannot be listed".to_string();
    let u16at = |i: usize| bytes.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize).ok_or_else(bad);
    let u32at = |i: usize| bytes.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).ok_or_else(bad);
    let end = (0..=bytes.len().saturating_sub(22)).rev().take(65_557).find(|&i| bytes[i..].starts_with(b"PK\x05\x06")).ok_or_else(bad)?;
    let (count, mut at) = (u16at(end + 10)?, u32at(end + 16)?);
    let mut lines = vec![format!("ZIP archive, {count} entr{}:", if count == 1 { "y" } else { "ies" })];
    for _ in 0..count {
        if !bytes.get(at..).is_some_and(|b| b.starts_with(b"PK\x01\x02")) { return Err(bad()); }
        let (size, n, extra, comment) = (u32at(at + 24)?, u16at(at + 28)?, u16at(at + 30)?, u16at(at + 32)?);
        let name = String::from_utf8_lossy(bytes.get(at + 46..at + 46 + n).ok_or_else(bad)?);
        lines.push(format!("{name}  {size} bytes"));
        at += 46 + n + extra + comment;
    }
    Ok(cut(&lines.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::read;

    // #186 A2: a PDF, a ZIP and plain text reach the model as their content; RAR and binaries name the exact reason.
    #[test]
    fn documents_become_text_or_a_reason() {
        use lopdf::{Object, Stream, dictionary};
        let mut doc = lopdf::Document::with_version("1.5");
        let pages = doc.new_object_id();
        let font = doc.add_object(dictionary! {"Type"=>"Font","Subtype"=>"Type1","BaseFont"=>"Helvetica"});
        let content = doc.add_object(Stream::new(dictionary! {}, b"BT /F1 12 Tf 10 10 Td (Invented invoice 42) Tj ET".to_vec()));
        let page = doc.add_object(dictionary! {"Type"=>"Page","Parent"=>pages,"Contents"=>content,"Resources"=>dictionary!{"Font"=>dictionary!{"F1"=>font}}});
        doc.objects.insert(pages, Object::Dictionary(dictionary! {"Type"=>"Pages","Kids"=>vec![page.into()],"Count"=>1}));
        let catalog = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
        doc.trailer.set("Root", catalog);
        let mut pdf = Vec::new();
        doc.save_to(&mut pdf).unwrap();
        let text = read("invoice.pdf", &pdf).unwrap();
        assert!(text.starts_with("PDF, 1 page(s):") && text.contains("Invented invoice 42"), "{text}");

        doc.objects.insert(content, Stream::new(dictionary! {}, Vec::new()).into());
        let mut scanned = Vec::new();
        doc.save_to(&mut scanned).unwrap();
        assert_eq!(read("scan.pdf", &scanned).unwrap_err(),
            "the PDF has 1 page(s) but no extractable text (scanned images?)");

        // stored ZIP with one entry "a.txt" (5 bytes), built by hand
        let mut zip = Vec::new();
        zip.extend_from_slice(b"PK\x03\x04"); zip.extend_from_slice(&[0; 22]); zip.extend_from_slice(&5u16.to_le_bytes()); zip.extend_from_slice(&[0; 2]);
        zip.extend_from_slice(b"a.txthello");
        let cd = zip.len() as u32;
        zip.extend_from_slice(b"PK\x01\x02"); zip.extend_from_slice(&[0; 20]); zip.extend_from_slice(&5u32.to_le_bytes());
        zip.extend_from_slice(&5u16.to_le_bytes()); zip.extend_from_slice(&[0; 16]); zip.extend_from_slice(b"a.txt");
        zip.extend_from_slice(b"PK\x05\x06"); zip.extend_from_slice(&[0; 4]); zip.extend_from_slice(&1u16.to_le_bytes());
        zip.extend_from_slice(&1u16.to_le_bytes()); zip.extend_from_slice(&51u32.to_le_bytes()); zip.extend_from_slice(&cd.to_le_bytes()); zip.extend_from_slice(&[0; 2]);
        assert_eq!(read("x.zip", &zip).unwrap(), "ZIP archive, 1 entry:\na.txt  5 bytes");
        assert!(read("x.zip", &zip[..zip.len() - 22]).unwrap_err().contains("damaged"));

        assert_eq!(read("notes.txt", "Привет".as_bytes()).unwrap(), "Привет");
        assert!(read("x.rar", b"Rar!\x1a\x07\x01\x00").unwrap_err().contains("RAR"));
        assert!(read("blob.bin", b"\x00\x01\x02").unwrap_err().contains("binary"));
        assert!(read("x.pdf", b"%PDF-1.4 broken").unwrap_err().contains("PDF"));
    }
}

use std::{process::Command, time::Instant};

/// Isolate untrusted parsing so a stuck parser can be terminated and reaped.
pub fn bounded(name: &str, bytes: &[u8], end: Instant, active: impl Fn() -> bool) -> Result<String, String> {
    use std::io::{Seek, Write};
    if !active() { return Err("document parsing cancelled".into()); }
    if Instant::now() >= end { return Err("document parsing timed out".into()); }
    // Unlink private input before dispatch: no retained attachment or blocked writer thread.
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|_| "document storage unavailable")?.as_nanos();
    let path = std::env::temp_dir().join(format!("pilotq-document-{}-{nonce}", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    let mut input = options.open(&path).map_err(|_| "document storage unavailable")?;
    std::fs::remove_file(&path).map_err(|_| "document storage unavailable")?;
    input.write_all(bytes).and_then(|_| input.rewind()).map_err(|_| "document storage unavailable")?;
    let exe = std::env::current_exe().map_err(|_| "document parser unavailable")?;
    let mut command = crate::commands::command(exe, ".", &["--document-parser", name]);
    command.stdin(input);
    supervised(command, end, active)
}

/// Private child protocol; no configuration, journal, model or external executable.
pub fn worker(name: &str) {
    use std::io::Read;
    std::panic::set_hook(Box::new(|_| {}));
    let mut bytes = Vec::new();
    let result = std::io::stdin().take(20 * 1024 * 1024 + 1).read_to_end(&mut bytes)
        .map_err(|_| "document input could not be read".to_string())
        .and_then(|_| if bytes.len() > 20 * 1024 * 1024 { Err("document exceeds 20 MiB".into()) } else { read(name, &bytes) });
    println!("{}", serde_json::to_string(&result).unwrap());
}

/// Supervise an isolated parser with the original media deadline and live journal fence.
fn supervised(command: Command, end: Instant, active: impl Fn() -> bool) -> Result<String, String> {
    // JSON can escape each of 50,000 text characters to six bytes; leave room for metadata.
    let output = crate::commands::execute_bounded(command, end, active, 1024 * 1024);
    match output["status"].as_str().unwrap_or("unknown") {
        "success" => serde_json::from_str::<Result<String, String>>(output["stdout"].as_str().unwrap_or(""))
            .map_err(|_| "document parser returned an invalid result".to_string())?,
        "timeout" => Err("document parsing timed out".into()),
        "cancelled" => Err("document parsing cancelled".into()),
        "output_limit" => Err("document parser exceeded its output limit".into()),
        "missing" | "spawn_failed" => Err("document parser could not be started".into()),
        _ => Err("document parser failed before returning readable content".into()),
    }
}

#[cfg(all(test, unix))]
mod budget_tests {
    use super::*;
    use std::time::{Duration, Instant};

    // §1 / §12, #195 A2: a stuck parser must release its process on the media deadline.
    #[test]
    fn stuck_document_deadline() {
        let pidfile = std::env::temp_dir().join(format!("pilotq-parser-deadline-{}.pid", std::process::id()));
        let mut command = crate::commands::command("/bin/sh", ".", &["-c",
            "echo $$ > \"$1\"; exec sleep 30", "parser"]);
        command.arg(&pidfile);
        let start = Instant::now();
        let result = supervised(command, start + Duration::from_millis(300), || true);
        assert_eq!(result.unwrap_err(), "document parsing timed out");
        assert!(start.elapsed() < Duration::from_secs(2));
        let pid = std::fs::read_to_string(&pidfile).unwrap();
        assert!(!std::process::Command::new("/bin/kill").args(["-0", pid.trim()]).status().unwrap().success(),
            "parser is still alive");
        std::fs::remove_file(pidfile).unwrap();
    }
    // §12 / R5: /stop must free the parser, not merely discard its eventual reply.
    #[test]
    fn stop_reaps_document_parser() {
        use crate::journal::{Input, Journal};
        let dir = std::env::temp_dir().join(format!("pilotq-parser-stop-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut j = Journal::open(&dir.join("journal.sqlite")).unwrap();
        j.bootstrap(7, 0).unwrap();
        j.admit(&[(1, Some(Input {
            files: vec![], callback: None, media_group: None, quote: None,
            image: None, voice: None, language: None, chat: 7, topic: None,
            sender: Some(7), name: None, message: 1, kind: "text",
            text: Some("read the attached document".into()), reply_to: None,
        }))], 1).unwrap();
        let run = j.claim_next(1, 1).unwrap().remove(0);
        let journal = std::cell::RefCell::new(j);
        let pidfile = dir.join("parser.pid");
        let mut command = crate::commands::command("/bin/sh", ".", &["-c",
            "echo $$ > \"$1\"; exec sleep 30", "parser"]);
        command.arg(&pidfile);
        let stopped = std::cell::Cell::new(false);
        let active = || {
            if pidfile.exists() && !stopped.get() {
                journal.borrow_mut().admit(&[(2, Some(Input {
                    files: vec![], callback: None, media_group: None, quote: None,
                    image: None, voice: None, language: None, chat: 7, topic: None,
                    sender: Some(7), name: None, message: 2, kind: "text",
                    text: Some("/stop".into()), reply_to: None,
                }))], 2).unwrap();
                stopped.set(true);
            }
            !journal.borrow().fenced(&run).unwrap()
        };
        let start = Instant::now();
        assert_eq!(supervised(command, start + Duration::from_secs(5), active).unwrap_err(),
            "document parsing cancelled");
        assert!(start.elapsed() < Duration::from_secs(2));
        let pid = std::fs::read_to_string(&pidfile).unwrap();
        assert!(!std::process::Command::new("/bin/kill").args(["-0", pid.trim()]).status().unwrap().success(),
            "parser is still alive");
        drop(journal);
        std::fs::remove_dir_all(dir).unwrap();
    }

    // §1: bounded transport preserves content/reasons and cannot accept invalid parser output.
    #[test]
    fn parser_results_and_preflight() {
        for (output, expected) in [
            (r#"{"Ok":"Привет"}"#, Ok("Привет".to_string())),
            (r#"{"Err":"the ZIP archive is damaged and cannot be listed"}"#,
                Err("the ZIP archive is damaged and cannot be listed".to_string())),
            ("invalid", Err("document parser returned an invalid result".to_string())),
        ] {
            let mut command = crate::commands::command("/usr/bin/printf", ".", &["%s"]);
            command.arg(output);
            assert_eq!(supervised(command, Instant::now() + Duration::from_secs(2), || true), expected);
        }
        assert_eq!(bounded("x", b"hello", Instant::now(), || true).unwrap_err(), "document parsing timed out");
        assert_eq!(bounded("x", b"hello", Instant::now() + Duration::from_secs(2), || false).unwrap_err(),
            "document parsing cancelled");
    }

}
