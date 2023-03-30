use serde::Deserialize;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ComputeSystem {
    pub id: Uuid,
    pub system_type: String,
    pub owner: String,
    pub runtime_id: Uuid,
    #[serde(default)]
    pub state: String,
}

fn enumerate_compute_systems(query: &str) -> std::io::Result<Vec<ComputeSystem>> {
    use std::io::{Error, ErrorKind};
    use widestring::WideCString;
    use winapi::shared::minwindef::LPVOID;
    use winapi::shared::ntdef::{LPCWSTR, LPWSTR};
    use winapi::um::combaseapi::CoTaskMemFree;
    use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

    unsafe {
        // Load vmcompute.dll and get HcsEnumerateComputeSystems. This cannot yet
        // be done using `#[link] extern {}` as it is semi-documented API.
        let module = LoadLibraryA(b"vmcompute.dll\0".as_ptr() as _);
        if module.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        let func = GetProcAddress(module, b"HcsEnumerateComputeSystems\0".as_ptr() as _);
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
            let err = Error::from_raw_os_error(hr);
            if hr == 0x8037011Bu32 as i32 {
                // HCS_E_ACCESS_DENIED, this is currently uncategorized in Rust
                return Err(Error::new(ErrorKind::PermissionDenied, err));
            }
            return Err(err);
        }

        serde_json::from_str(&compute_systems)
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err))
    }
}

#[allow(unused)]
fn get_wsl_vmid_by_hcs() -> std::io::Result<Option<Uuid>> {
    let vms = enumerate_compute_systems("{}")?;
    for vm in vms {
        if vm.owner == "WSL" {
            return Ok(Some(vm.id));
        }
    }
    Ok(None)
}

// This is unreliable, so only use this as a backup method.
pub fn get_wsl_vmid_by_reg() -> std::io::Result<Option<Uuid>> {
    let list = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\HostComputeService\VolatileStore\ComputeSystem")?;
    for k in list.enum_keys() {
        let k = k?;
        let subkey = list.open_subkey(&k)?;
        let ty: u32 = match subkey.get_value("ComputeSystemType") {
            Ok(v) => v,
            Err(_) => continue,
        };
        if ty == 2 {
            if let Ok(v) = k.parse() {
                return Ok(Some(v));
            }
        }
    }
    Ok(None)
}

pub fn get_wsl_vmid() -> std::io::Result<Option<Uuid>> {
    match get_wsl_vmid_by_hcs() {
        Ok(v) => return Ok(v),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => (),
        Err(err) => return Err(err),
    }
    get_wsl_vmid_by_reg()
}
