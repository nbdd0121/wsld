use serde::Deserialize;
use uuid::Uuid;

#[serde(rename_all = "PascalCase")]
#[derive(Debug, Deserialize)]
pub struct ComputeSystem {
    pub id: Uuid,
    pub system_type: String,
    pub owner: String,
    pub runtime_id: Uuid,
    pub state: String,
}

pub fn enumerate_compute_systems(query: &str) -> std::io::Result<Vec<ComputeSystem>> {
    use std::ffi::CString;
    use std::io::{Error, ErrorKind};
    use widestring::WideCString;
    use winapi::shared::minwindef::LPVOID;
    use winapi::shared::ntdef::{LPCWSTR, LPWSTR};
    use winapi::um::combaseapi::CoTaskMemFree;
    use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

    unsafe {
        // Load vmcompute.dll and get HcsEnumerateComputeSystems. This cannot yet
        // be done using `#[link] extern {}` as it is semi-documented API.
        let module = LoadLibraryA(CString::new("vmcompute.dll").unwrap().as_ptr());
        if module.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        let func = GetProcAddress(
            module,
            CString::new("HcsEnumerateComputeSystems").unwrap().as_ptr(),
        );
        if func.is_null() {
            FreeLibrary(module);
            return Err(std::io::Error::last_os_error());
        }

        let func: unsafe extern "C" fn(
            query: LPCWSTR,
            compute_systems: &mut LPWSTR,
            result: &mut LPWSTR,
        ) -> i32 = std::mem::transmute(func);

        let query =
            WideCString::from_str(query).map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
        let mut compute_systems: LPWSTR = std::ptr::null_mut();
        let mut result: LPWSTR = std::ptr::null_mut();

        let hr = func(query.as_ptr(), &mut compute_systems, &mut result);

        let compute_systems = if compute_systems.is_null() {
            String::new()
        } else {
            let str = WideCString::from_ptr_str(compute_systems)
                .to_string()
                .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
            CoTaskMemFree(compute_systems as LPVOID);
            str
        };

        CoTaskMemFree(result as LPVOID);
        FreeLibrary(module);

        if hr != 0 {
            return Err(std::io::Error::from_raw_os_error(hr));
        }

        serde_json::from_str(&compute_systems)
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err))
    }
}

pub fn get_wsl_vmid() -> Option<Uuid> {
    let vms = enumerate_compute_systems("{}").unwrap();
    for vm in vms {
        if vm.owner == "WSL" {
            return Some(vm.id);
        }
    }
    None
}
