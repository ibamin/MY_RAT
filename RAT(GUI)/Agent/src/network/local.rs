use windows::core::*;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Shell::IShellDispatch2;
use windows::Win32::System::Variant::VARIANT; // 올바른 경로

fn DCOM_EXECUTE() -> Result<()> {
    unsafe {
        // COM 초기화
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

        let clsid = GUID::from_u128(0x13709620_C279_11CE_A49E_444553540000);
        let shell: IShellDispatch2 = CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER)?;

        // ShellExecute를 사용하여 명령 실행
        shell.ShellExecute(
            &BSTR::from("powershell.exe"),  // 실행할 프로그램
            &VARIANT::default(),        // 인자 (None)
            &VARIANT::default(),        // 작업 디렉터리 (None)
            &VARIANT::default(),        // 실행 방식 (None)
            &VARIANT::from(1),          // SW_SHOWNORMAL (1)
        )?;
        CoUninitialize();
    }

    Ok(())
}
