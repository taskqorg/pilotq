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

// Data caps do not bound parser time. Deadline/cancellation isolation remains open in task #195.
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
