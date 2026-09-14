use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub fn cli() -> Option<Result<(), String>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) != Some("--extract-source") {
        return None;
    }
    Some((|| {
        if args.len() != 5 {
            return Err("Extraction requires input, media type, and output.".into());
        }
        let path = Path::new(&args[2]);
        let ready = std::path::PathBuf::from(format!("{}.ready", args[4]));
        let wait = Instant::now();
        while !ready.exists() {
            if wait.elapsed() > Duration::from_secs(5) {
                return Err("Parser launch was not authorized by its parent.".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 25_000_000 {
            return Err("Source exceeds 25 MB.".into());
        }
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        let result = super::formats::extract(&bytes, &args[3]);
        fs::write(
            &args[4],
            serde_json::to_vec(&result).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })())
}
pub fn extract(
    executable: &Path,
    input: &Path,
    media: &str,
    directory: &Path,
) -> Result<String, String> {
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let output = directory.join(format!("{}.json", uuid::Uuid::new_v4()));
    let mut command = Command::new(executable);
    command
        .args(["--extract-source"])
        .arg(input)
        .arg(media)
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|_| "Could not start the isolated document parser.")?;
    #[cfg(windows)]
    let _job = match Job::attach(&child) {
        Ok(job) => job,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    let started = Instant::now();
    let ready = std::path::PathBuf::from(format!("{}.ready", output.display()));
    if let Err(e) = fs::write(&ready, b"ready") {
        let _ = child.kill();
        let _ = child.wait();
        return Err(e.to_string());
    }
    let result = (|| {
        loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                if !status.success() {
                    return Err("Document parser failed or exceeded its memory limit. Original source remains available.".into());
                }
                break;
            }
            if started.elapsed() > Duration::from_secs(15) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(
                    "Document extraction exceeded 15 seconds. Original source remains available."
                        .into(),
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if fs::metadata(&output).map_err(|e| e.to_string())?.len() > 6_100_000 {
            return Err("Parser output exceeds its limit.".into());
        }
        serde_json::from_slice::<Result<String, String>>(
            &fs::read(&output).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
    })();
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(&ready);
    result
}
#[cfg(windows)]
struct Job(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Job {
    fn attach(child: &std::process::Child) -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        // Closing this job also terminates a parser if the host exits unexpectedly.
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err("Could not create a parser resource limit.".into());
            }
            let job = Self(handle);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags =
                JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            limits.ProcessMemoryLimit = 384 * 1024 * 1024;
            if SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            ) == 0
                || AssignProcessToJobObject(handle, child.as_raw_handle()) == 0
            {
                return Err("Could not enforce the parser memory limit.".into());
            }
            Ok(job)
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
