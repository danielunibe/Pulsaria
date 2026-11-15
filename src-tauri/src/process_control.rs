//! Process-launch policy shared by the native runtime.
//!
//! Windows console applications can open a visible console when they are
//! launched by a GUI process. Keep that policy in one place so every native
//! subprocess follows the same no-window contract.

use std::io;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn hide_std_command(command: &mut std::process::Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        command.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(windows))]
    let _ = command;
}

pub fn hide_tokio_command(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    #[cfg(not(windows))]
    let _ = command;
}

/// Owns a Windows Job Object that terminates a worker and any subprocesses it
/// starts when the worker is dropped after consent revocation or cancellation.
#[cfg(windows)]
pub(crate) struct ProcessTreeGuard {
    _job: std::os::windows::io::OwnedHandle,
}

#[cfg(not(windows))]
pub(crate) struct ProcessTreeGuard;

#[cfg(windows)]
impl ProcessTreeGuard {
    pub(crate) fn attach_to_tokio_child(child: &tokio::process::Child) -> io::Result<Self> {
        let process = child.raw_handle().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "worker process exited before process-tree assignment",
            )
        })?;
        Self::attach_to_process(process)
    }

    pub(crate) fn attach_to_std_child(child: &std::process::Child) -> io::Result<Self> {
        use std::os::windows::io::AsRawHandle;

        Self::attach_to_process(child.as_raw_handle())
    }

    fn attach_to_process(process: std::os::windows::io::RawHandle) -> io::Result<Self> {
        use std::{
            mem::size_of,
            os::windows::io::{FromRawHandle, OwnedHandle},
        };
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let owned_job = unsafe { OwnedHandle::from_raw_handle(job.cast()) };

        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            return Err(io::Error::last_os_error());
        }

        let assigned = unsafe {
            AssignProcessToJobObject(job, process as windows_sys::Win32::Foundation::HANDLE)
        };
        if assigned == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { _job: owned_job })
    }
}

#[cfg(not(windows))]
impl ProcessTreeGuard {
    pub(crate) fn attach_to_tokio_child(_child: &tokio::process::Child) -> io::Result<Self> {
        Ok(Self)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::ProcessTreeGuard;
    use std::{
        fs,
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };

    const PARENT_MODE: &str = "PULSARIA_TEST_PROCESS_TREE_PARENT";
    const GRANDCHILD_MODE: &str = "PULSARIA_TEST_PROCESS_TREE_GRANDCHILD";
    const READY_PATH: &str = "PULSARIA_TEST_PROCESS_TREE_READY";
    const START_PATH: &str = "PULSARIA_TEST_PROCESS_TREE_START";
    const MARKER_PATH: &str = "PULSARIA_TEST_PROCESS_TREE_MARKER";

    #[test]
    fn dropping_the_job_guard_terminates_the_worker_process_tree() {
        if std::env::var_os(GRANDCHILD_MODE).is_some() {
            thread::sleep(Duration::from_millis(750));
            fs::write(
                std::env::var(MARKER_PATH).expect("marker path is set"),
                "ran",
            )
            .expect("grandchild can write its completion marker");
            return;
        }

        if std::env::var_os(PARENT_MODE).is_some() {
            let start_path = std::env::var(START_PATH).expect("start path is set");
            let deadline = Instant::now() + Duration::from_secs(5);
            while !std::path::Path::new(&start_path).exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(20));
            }
            assert!(
                std::path::Path::new(&start_path).exists(),
                "parent helper must wait for the job assignment"
            );

            let executable = std::env::current_exe().expect("test executable path");
            let mut command = Command::new(executable);
            command
                .arg("--exact")
                .arg("process_control::tests::dropping_the_job_guard_terminates_the_worker_process_tree")
                .arg("--nocapture")
                .env(GRANDCHILD_MODE, "1")
                .env(MARKER_PATH, std::env::var(MARKER_PATH).expect("marker path is set"))
                .env_remove(PARENT_MODE)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let _grandchild = command.spawn().expect("grandchild test process starts");
            fs::write(
                std::env::var(READY_PATH).expect("ready path is set"),
                "spawned",
            )
            .expect("parent helper can signal that its child was spawned");
            thread::sleep(Duration::from_secs(10));
            return;
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!(
            "pulsaria-process-tree-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&temp_root).expect("isolated process-tree fixture is created");
        let ready_path = temp_root.join("ready");
        let start_path = temp_root.join("start");
        let marker_path = temp_root.join("grandchild-completed");
        let executable = std::env::current_exe().expect("test executable path");
        let mut command = Command::new(executable);
        command
            .arg("--exact")
            .arg(
                "process_control::tests::dropping_the_job_guard_terminates_the_worker_process_tree",
            )
            .arg("--nocapture")
            .env(PARENT_MODE, "1")
            .env(READY_PATH, &ready_path)
            .env(START_PATH, &start_path)
            .env(MARKER_PATH, &marker_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut parent = command.spawn().expect("parent test process starts");
        let guard = match ProcessTreeGuard::attach_to_std_child(&parent) {
            Ok(guard) => guard,
            Err(error) => {
                let _ = parent.kill();
                let _ = parent.wait();
                let _ = fs::remove_dir_all(&temp_root);
                panic!("worker process tree must be assigned to a job: {error}");
            }
        };

        fs::write(&start_path, "start").expect("parent helper may start its child");
        let ready_deadline = Instant::now() + Duration::from_secs(5);
        while !ready_path.exists() && Instant::now() < ready_deadline {
            thread::sleep(Duration::from_millis(20));
        }
        let child_was_spawned = ready_path.exists();

        drop(guard);
        let exit_deadline = Instant::now() + Duration::from_secs(3);
        let mut parent_exited = false;
        while Instant::now() < exit_deadline {
            if parent
                .try_wait()
                .expect("parent status is readable")
                .is_some()
            {
                parent_exited = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if !parent_exited {
            let _ = parent.kill();
            let _ = parent.wait();
        }
        thread::sleep(Duration::from_millis(900));
        let descendant_was_stopped = !marker_path.exists();
        let _ = fs::remove_dir_all(&temp_root);

        assert!(child_was_spawned, "the test created a synthetic descendant");
        assert!(parent_exited, "closing the job guard terminates its worker");
        assert!(
            descendant_was_stopped,
            "closing the job guard terminates subprocesses that outlive Python"
        );
    }
}
