use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::{Failure, ToolResult};

pub(super) const OUTPUT_LIMIT: u64 = 1_048_576;

pub(super) struct Workspace {
    root: PathBuf,
    files: BTreeSet<PathBuf>,
    serial: u64,
}

fn io_error(error: std::io::Error) -> Failure {
    Failure::new("tool.io", error)
}

impl Workspace {
    pub fn new() -> ToolResult<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for attempt in 0..16 {
            let root =
                std::env::temp_dir().join(format!("lobo-test-{}-{stamp}-{attempt}", std::process::id()));
            let mut directory = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                directory.mode(0o700);
            }
            match directory.create(&root) {
                Ok(()) => {
                    return Ok(Self {
                        root,
                        files: BTreeSet::new(),
                        serial: 0,
                    })
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(io_error(e)),
            }
        }
        Err(Failure::new("tool.io", "cannot reserve session directory"))
    }

    fn create(&mut self, name: &str) -> ToolResult<(PathBuf, File)> {
        let path = self.root.join(name);
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path).map_err(io_error)?;
        self.files.insert(path.clone());
        Ok((path, file))
    }

    pub fn build(&mut self, clang: &Path, source: &str, deadline: Instant) -> ToolResult<PathBuf> {
        self.serial += 1;
        let (source_path, mut file) = self.create(&format!("source-{}.c", self.serial))?;
        file.write_all(source.as_bytes()).map_err(io_error)?;
        drop(file);
        let binary = self.root.join(format!("binary-{}", self.serial));
        self.files.insert(binary.clone());
        let mut command = Command::new(clang);
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LC_ALL", "C")
            .args([
                "-std=c11",
                "-O3",
                "-g0",
                "-ffunction-sections",
                "-fdata-sections",
                "-Wl,--gc-sections",
                "-Wl,-s",
            ])
            .arg(&source_path)
            .arg("-o")
            .arg(&binary);
        let result = self.capture(command, deadline)?;
        if !result.success {
            return Err(Failure::new(
                "tool.compile",
                String::from_utf8_lossy(&result.stderr)
                    .chars()
                    .take(1024)
                    .collect::<String>(),
            ));
        }
        Ok(binary)
    }

    pub fn capture(&mut self, mut command: Command, deadline: Instant) -> ToolResult<Output> {
        if Instant::now() >= deadline {
            return Err(Failure::new("tool.timeout", "process deadline exhausted"));
        }
        self.serial += 1;
        let (out_path, out) = self.create(&format!("job-{}.out", self.serial))?;
        let (err_path, err) = self.create(&format!("job-{}.err", self.serial))?;
        command.stdin(Stdio::null()).stdout(out).stderr(err);
        let result = (|| {
            let mut child = command.spawn().map_err(|e| Failure::new("tool.spawn", e))?;
            let status = loop {
                let excessive = [&out_path, &err_path]
                    .iter()
                    .any(|path| fs::metadata(path).map(|m| m.len() > OUTPUT_LIMIT).unwrap_or(true));
                if excessive || Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(Failure::new(
                        if excessive {
                            "tool.output_limit"
                        } else {
                            "tool.timeout"
                        },
                        "child output or time budget exhausted",
                    ));
                }
                match child.try_wait() {
                    Ok(Some(status)) => break status,
                    Ok(None) => std::thread::sleep(Duration::from_millis(2)),
                    Err(e) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(io_error(e));
                    }
                }
            };
            let read = |path: &Path| -> ToolResult<Vec<u8>> {
                let mut bytes = Vec::new();
                File::open(path)
                    .map_err(io_error)?
                    .take(OUTPUT_LIMIT + 1)
                    .read_to_end(&mut bytes)
                    .map_err(io_error)?;
                if bytes.len() as u64 > OUTPUT_LIMIT {
                    return Err(Failure::new("tool.output_limit", "child output limit exceeded"));
                }
                Ok(bytes)
            };
            Ok(Output {
                success: status.success(),
                stdout: read(&out_path)?,
                stderr: read(&err_path)?,
            })
        })();
        for path in [out_path, err_path] {
            if fs::remove_file(&path).is_ok() {
                self.files.remove(&path);
            }
        }
        result
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        for path in &self.files {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&self.root);
    }
}

pub(super) struct Output {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn session_sources_are_not_readable_by_other_users() {
        let mut workspace = Workspace::new().unwrap_or_else(|_| panic!("workspace creation failed"));
        let (path, file) = workspace
            .create("private.c")
            .unwrap_or_else(|_| panic!("file creation failed"));
        drop(file);
        assert_eq!(
            fs::metadata(&workspace.root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn child_deadlines_output_limits_and_nonzero_exits_are_observable_and_cleanup_is_exact() {
        let mut workspace = Workspace::new().unwrap_or_else(|_| panic!("workspace creation failed"));
        let mut sleeping = Command::new("/bin/sleep");
        sleeping.arg("2");
        let timed = workspace.capture(sleeping, Instant::now() + Duration::from_millis(20));
        assert_eq!(timed.err().map(|e| e.code), Some("tool.timeout"));
        let excessive = workspace.capture(
            Command::new("/usr/bin/yes"),
            Instant::now() + Duration::from_secs(2),
        );
        assert_eq!(excessive.err().map(|e| e.code), Some("tool.output_limit"));
        let failed = workspace
            .capture(
                Command::new("/bin/false"),
                Instant::now() + Duration::from_secs(2),
            )
            .unwrap_or_else(|_| panic!("process capture failed"));
        assert!(!failed.success);
        assert!(workspace.files.is_empty());
        assert_eq!(fs::read_dir(&workspace.root).unwrap().count(), 0);
    }
}
