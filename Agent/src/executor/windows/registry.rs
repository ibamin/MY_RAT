use std::{
    iter::once,
    slice,
    time::Instant,
};

use async_trait::async_trait;
use windows::{
    core::PCWSTR,
    Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
        HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_ALL_ACCESS,
        KEY_READ, KEY_WRITE, REG_DWORD, REG_SZ, REG_VALUE_TYPE,
    },
};

use crate::executor::{ExecutionResult, Executor};

pub struct RegistryExecutor;

#[async_trait]
impl Executor for RegistryExecutor {
    fn kind(&self) -> &'static str {
        "registry"
    }

    fn is_available(&self) -> bool {
        cfg!(windows)
    }

    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult {
        let started = Instant::now();
        let command = command.to_string();
        let args = args.iter().map(|value| value.to_string()).collect::<Vec<_>>();

        let result = tokio::task::spawn_blocking(move || {
            execute_registry_command(&command, &args)
        })
        .await;

        let duration_ms = started.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((stdout, evidence))) => ExecutionResult {
                success: true,
                stdout,
                stderr: Vec::new(),
                exit_code: Some(0),
                evidence,
                duration_ms,
            },
            Ok(Err(err_text)) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: err_text.into_bytes(),
                exit_code: Some(1),
                evidence: None,
                duration_ms,
            },
            Err(join_err) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: join_err.to_string().into_bytes(),
                exit_code: None,
                evidence: None,
                duration_ms,
            },
        }
    }
}

struct RegKeyGuard(HKEY);

impl Drop for RegKeyGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.0.0.is_null() {
                let _ = RegCloseKey(self.0);
            }
        }
    }
}

fn execute_registry_command(
    command: &str,
    args: &[String],
) -> Result<(Vec<u8>, Option<serde_json::Value>), String> {
    match command {
        "read" => {
            if args.len() < 3 {
                return Err("read requires args: [hive, key_path, value_name]".to_string());
            }

            let hive = parse_hive(&args[0]).ok_or_else(|| format!("Unknown hive: {}", args[0]))?;
            let (data, reg_type) = registry_read(hive, &args[1], &args[2])?;
            let evidence = serde_json::json!({
                "value": registry_value_to_text(&data, reg_type),
                "type": reg_type.0,
            });

            Ok((b"Registry read succeeded".to_vec(), Some(evidence)))
        }
        "write" => {
            if args.len() < 4 {
                return Err("write requires args: [hive, key_path, value_name, data]".to_string());
            }

            let hive = parse_hive(&args[0]).ok_or_else(|| format!("Unknown hive: {}", args[0]))?;
            registry_write(hive, &args[1], &args[2], &args[3])?;
            Ok((b"Registry write succeeded".to_vec(), None))
        }
        "delete" => {
            if args.len() < 3 {
                return Err("delete requires args: [hive, key_path, value_name]".to_string());
            }

            let hive = parse_hive(&args[0]).ok_or_else(|| format!("Unknown hive: {}", args[0]))?;
            registry_delete(hive, &args[1], &args[2])?;
            Ok((b"Registry delete succeeded".to_vec(), None))
        }
        "persist" => {
            if args.len() < 2 {
                return Err("persist requires args: [name, command_to_run]".to_string());
            }

            registry_persist(&args[0], &args[1])?;
            Ok((b"Registry persistence succeeded".to_vec(), None))
        }
        _ => Err(format!("Unsupported registry command: {}", command)),
    }
}

fn parse_hive(name: &str) -> Option<HKEY> {
    match name.to_ascii_uppercase().as_str() {
        "HKLM" | "HKEY_LOCAL_MACHINE" => Some(HKEY_LOCAL_MACHINE),
        "HKCU" | "HKEY_CURRENT_USER" => Some(HKEY_CURRENT_USER),
        "HKCR" | "HKEY_CLASSES_ROOT" => Some(HKEY_CLASSES_ROOT),
        "HKU" | "HKEY_USERS" => Some(HKEY_USERS),
        _ => None,
    }
}

fn registry_read(hive: HKEY, path: &str, value_name: &str) -> Result<(Vec<u8>, REG_VALUE_TYPE), String> {
    unsafe {
        let key = open_key(hive, path, KEY_READ)?;
        let value_name_wide = to_wide(value_name);

        let mut reg_type = REG_VALUE_TYPE(0);
        let mut data_len = 0u32;

        check_status(
            RegQueryValueExW(
                key.0,
                PCWSTR::from_raw(value_name_wide.as_ptr()),
                None,
                Some(&mut reg_type),
                None,
                Some(&mut data_len),
            ),
            "RegQueryValueExW(size)",
        )?;

        let mut data = vec![0u8; data_len as usize];
        if data_len > 0 {
            check_status(
                RegQueryValueExW(
                    key.0,
                    PCWSTR::from_raw(value_name_wide.as_ptr()),
                    None,
                    Some(&mut reg_type),
                    Some(data.as_mut_ptr()),
                    Some(&mut data_len),
                ),
                "RegQueryValueExW(data)",
            )?;
            data.truncate(data_len as usize);
        }

        Ok((data, reg_type))
    }
}

fn registry_write(hive: HKEY, path: &str, value_name: &str, data: &str) -> Result<(), String> {
    unsafe {
        let key = open_key(hive, path, KEY_WRITE)?;
        let value_name_wide = to_wide(value_name);
        let data_wide = to_wide(data);

        let data_bytes = slice::from_raw_parts(
            data_wide.as_ptr() as *const u8,
            data_wide.len() * std::mem::size_of::<u16>(),
        );

        check_status(
            RegSetValueExW(
                key.0,
                PCWSTR::from_raw(value_name_wide.as_ptr()),
                None,
                REG_SZ,
                Some(data_bytes),
            ),
            "RegSetValueExW",
        )
    }
}

fn registry_delete(hive: HKEY, path: &str, value_name: &str) -> Result<(), String> {
    unsafe {
        let key = open_key(hive, path, KEY_ALL_ACCESS)?;
        let value_name_wide = to_wide(value_name);

        check_status(
            RegDeleteValueW(key.0, PCWSTR::from_raw(value_name_wide.as_ptr())),
            "RegDeleteValueW",
        )
    }
}

fn registry_persist(name: &str, cmd: &str) -> Result<(), String> {
    registry_write(
        HKEY_CURRENT_USER,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        name,
        cmd,
    )
}

unsafe fn open_key(hive: HKEY, path: &str, access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Result<RegKeyGuard, String> {
    let path_wide = to_wide(path);
    let mut key = HKEY::default();

    check_status(
        RegOpenKeyExW(
            hive,
            PCWSTR::from_raw(path_wide.as_ptr()),
            None,
            access,
            &mut key,
        ),
        "RegOpenKeyExW",
    )?;

    Ok(RegKeyGuard(key))
}

fn check_status(status: windows::Win32::Foundation::WIN32_ERROR, operation: &str) -> Result<(), String> {
    if status.0 == 0 {
        Ok(())
    } else {
        Err(format!("{} failed with code {}", operation, status.0))
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(once(0)).collect()
}

fn registry_value_to_text(data: &[u8], reg_type: REG_VALUE_TYPE) -> String {
    if reg_type == REG_SZ {
        return utf16_bytes_to_string(data);
    }

    if reg_type == REG_DWORD && data.len() >= 4 {
        let raw = [data[0], data[1], data[2], data[3]];
        return u32::from_le_bytes(raw).to_string();
    }

    encode_base64(data)
}

fn utf16_bytes_to_string(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut u16_values = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        u16_values.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }

    while u16_values.last().copied() == Some(0) {
        u16_values.pop();
    }

    String::from_utf16_lossy(&u16_values)
}

fn encode_base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    if data.is_empty() {
        return String::new();
    }

    let mut output = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            output.push(TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}
