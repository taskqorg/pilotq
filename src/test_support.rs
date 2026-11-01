//! Test executables must not expose writable descriptors to concurrent spawns.
use std::{io::Write, path::Path, process::{Command, Stdio}};

pub fn write_script(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    // Linux can return ETXTBSY after fs::write closes: a concurrent fork may
    // still hold its writable descriptor until exec. Write in a child instead,
    // then publish a closed inode, also avoiding truncation of a running script.
    let mut child = Command::new("/bin/sh")
        .args(["-c", "set -eu; tmp=$(mktemp \"$1.XXXXXX\"); trap 'rm -f \"$tmp\"' EXIT; cat > \"$tmp\"; chmod 700 \"$tmp\"; mv -f \"$tmp\" \"$1\"", "write-script"])
        .arg(path.as_ref())
        .stdin(Stdio::piped())
        .spawn()?;
    let written = child.stdin.take().unwrap().write_all(contents.as_ref());
    let status = child.wait()?;
    written?;
    if !status.success() {
        return Err(std::io::Error::other(format!("script writer failed: {status}")));
    }
    Ok(())
}

// #270 A1–A2: a writer retained by a concurrent spawn must not prevent
// execution of the published replacement. Deadline/redaction guards remain
// in release_errors_202 and u106_doctor_deadline_and_redaction.
#[cfg(target_os = "linux")]
#[test]
fn script_publication_with_retained_writer_270() {
    use std::{fs::{self, OpenOptions}, os::unix::fs::PermissionsExt};
    let dir = std::env::temp_dir().join(format!("pilotq-script-270-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for name in ["runner", "doctor"] {
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        // Represents the writable descriptor a fork retained after fs::write.
        let writer = OpenOptions::new().write(true).open(&path).unwrap();
        assert_eq!(Command::new(&path).status().unwrap_err().raw_os_error(), Some(26));
        write_script(&path, "#!/bin/sh\nprintf 'published\\n'\n").unwrap();
        let output = Command::new(&path).output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"published\n");
        drop(writer);
    }
    fs::remove_dir_all(dir).unwrap();
}
