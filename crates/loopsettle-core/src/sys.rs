//! Peak resident memory of the current process, without dependencies.
//!
//! On Windows we declare the one Win32 function we need by hand (`extern "system"` is the
//! stdcall/Win64 calling convention used by the Windows API) instead of pulling `winapi`.
//! On Linux we read `VmHWM` ("high water mark") from `/proc/self/status`.

#[cfg(windows)]
mod imp {
    #[repr(C)]
    #[allow(non_snake_case)]
    struct ProcessMemoryCounters {
        cb: u32,
        PageFaultCount: u32,
        PeakWorkingSetSize: usize,
        WorkingSetSize: usize,
        QuotaPeakPagedPoolUsage: usize,
        QuotaPagedPoolUsage: usize,
        QuotaPeakNonPagedPoolUsage: usize,
        QuotaNonPagedPoolUsage: usize,
        PagefileUsage: usize,
        PeakPagefileUsage: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(
            process: isize,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }
    pub fn peak_rss_bytes() -> Option<u64> {
        let mut c: ProcessMemoryCounters = unsafe { std::mem::zeroed() };
        c.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
        // SAFETY: `c` is a properly sized, writable struct and `cb` holds its size.
        let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
        if ok != 0 {
            Some(c.PeakWorkingSetSize as u64)
        } else {
            None
        }
    }
}

#[cfg(all(not(windows), target_os = "linux"))]
mod imp {
    pub fn peak_rss_bytes() -> Option<u64> {
        let s = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = s.lines().find(|l| l.starts_with("VmHWM:"))?;
        let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
        Some(kb * 1024)
    }
}

#[cfg(all(not(windows), not(target_os = "linux")))]
mod imp {
    pub fn peak_rss_bytes() -> Option<u64> {
        None
    }
}

/// Peak resident set (working set) of this process in bytes, if the OS exposes it.
pub fn peak_rss_bytes() -> Option<u64> {
    imp::peak_rss_bytes()
}

/// Peak RSS in MiB (0.0 if unknown).
pub fn peak_rss_mib() -> f64 {
    peak_rss_bytes()
        .map(|b| b as f64 / (1024.0 * 1024.0))
        .unwrap_or(0.0)
}
