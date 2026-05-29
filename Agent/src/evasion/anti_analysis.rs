#[derive(Default)]
pub struct AntiAnalysis;

impl AntiAnalysis {
    pub fn new() -> Self {
        Self
    }

    /// Run all checks and return tagged results.
    pub fn run_all_checks(&self) -> Vec<(&'static str, bool)> {
        let mut results = Vec::new();

        #[cfg(windows)]
        {
            results.push(("IsDebuggerPresent", self.is_debugger_present()));
            results.push(("CheckRemoteDebugger", self.check_remote_debugger()));
            results.push(("IsVM", self.is_vm()));
            results.push(("TimingAnomaly", self.timing_check()));
        }

        #[cfg(not(windows))]
        {
            results.push(("TracerPid", self.check_tracer_pid()));
        }

        results
    }

    /// Returns true if any check detects analysis environment.
    pub fn is_under_analysis(&self) -> bool {
        self.run_all_checks().iter().any(|(_, detected)| *detected)
    }
}

// ── Windows implementations ──────────────────────────────────────────────────

#[cfg(windows)]
impl AntiAnalysis {
    fn is_debugger_present(&self) -> bool {
        use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;
        unsafe { IsDebuggerPresent().as_bool() }
    }

    fn check_remote_debugger(&self) -> bool {
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::System::Diagnostics::Debug::CheckRemoteDebuggerPresent;
        use windows::Win32::System::Threading::GetCurrentProcess;

        unsafe {
            let mut present = BOOL::default();
            if CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut present).is_ok() {
                present.as_bool()
            } else {
                false
            }
        }
    }

    fn is_vm(&self) -> bool {
        use windows::core::PCWSTR;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
        };

        let vm_indicators: &[&str] = &[
            r"SOFTWARE\Oracle\VirtualBox Guest Additions",
            r"SOFTWARE\VMware, Inc.\VMware Tools",
            r"SOFTWARE\Microsoft\Virtual Machine\Guest\Parameters",
        ];

        for path in vm_indicators {
            let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            let mut key = HKEY::default();
            let status = unsafe {
                RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    PCWSTR::from_raw(wide.as_ptr()),
                    None,
                    KEY_READ,
                    &mut key,
                )
            };
            if status.0 == 0 {
                unsafe { let _ = RegCloseKey(key); }
                return true;
            }
        }

        false
    }

    fn timing_check(&self) -> bool {
        use std::time::Instant;

        let start = Instant::now();
        // A trivial operation — if a debugger single-steps this the elapsed
        // time will be orders of magnitude larger than native execution.
        let mut _dummy: u64 = 0;
        for i in 0..1000u64 {
            _dummy = _dummy.wrapping_add(i);
        }
        let elapsed = start.elapsed();

        // Native execution: < 1 ms.  Debugger single-step: tens of ms+.
        elapsed.as_millis() > 50
    }
}

// ── Linux implementation ─────────────────────────────────────────────────────

#[cfg(not(windows))]
impl AntiAnalysis {
    fn check_tracer_pid(&self) -> bool {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("TracerPid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u32>().ok())
            })
            .map(|pid| pid != 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anti_analysis_new() {
        let _a = AntiAnalysis::new();
    }

    #[test]
    fn test_anti_analysis_default() {
        let _a = AntiAnalysis::default();
    }

    #[test]
    fn test_run_all_checks_returns_results() {
        let a = AntiAnalysis::new();
        let checks = a.run_all_checks();
        assert!(!checks.is_empty());
    }
}
