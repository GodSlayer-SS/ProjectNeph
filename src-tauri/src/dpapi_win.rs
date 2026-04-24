//! Windows DPAPI helpers for optional memory export encryption (user scope).

#[cfg(windows)]
use anyhow::{anyhow, Result};
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use winapi::shared::minwindef::DWORD;
#[cfg(windows)]
use winapi::um::dpapi::{CryptProtectData, CryptUnprotectData};
#[cfg(windows)]
use winapi::um::wincrypt::DATA_BLOB;
#[cfg(windows)]
use winapi::um::winbase::LocalFree;

#[cfg(windows)]
pub fn protect_user_bytes(plain: &[u8]) -> Result<Vec<u8>> {
    let mut buf = plain.to_vec();
    let mut in_blob = DATA_BLOB {
        cbData: buf.len() as DWORD,
        pbData: buf.as_mut_ptr(),
    };
    let mut out_blob = DATA_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &mut in_blob,
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut out_blob,
        )
    };
    if ok == 0 {
        return Err(anyhow!("CryptProtectData failed"));
    }
    if out_blob.pbData.is_null() || out_blob.cbData == 0 {
        return Err(anyhow!("CryptProtectData returned empty output"));
    }
    let out = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec() };
    unsafe {
        LocalFree(out_blob.pbData as *mut _);
    }
    Ok(out)
}

#[cfg(windows)]
pub fn unprotect_user_bytes(cipher: &[u8]) -> Result<Vec<u8>> {
    let mut buf = cipher.to_vec();
    let mut in_blob = DATA_BLOB {
        cbData: buf.len() as DWORD,
        pbData: buf.as_mut_ptr(),
    };
    let mut out_blob = DATA_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &mut in_blob,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut out_blob,
        )
    };
    if ok == 0 {
        return Err(anyhow!("CryptUnprotectData failed"));
    }
    if out_blob.pbData.is_null() || out_blob.cbData == 0 {
        return Err(anyhow!("CryptUnprotectData returned empty output"));
    }
    let out = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec() };
    unsafe {
        LocalFree(out_blob.pbData as *mut _);
    }
    Ok(out)
}

#[cfg(not(windows))]
pub fn protect_user_bytes(_plain: &[u8]) -> anyhow::Result<Vec<u8>> {
    anyhow::bail!("DPAPI is only available on Windows builds")
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn unprotect_user_bytes(_cipher: &[u8]) -> anyhow::Result<Vec<u8>> {
    anyhow::bail!("DPAPI is only available on Windows builds")
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn round_trip_dpapi() {
        let p = b"neph-diagnostic-test-bytes";
        let c = protect_user_bytes(p).unwrap();
        let d = unprotect_user_bytes(&c).unwrap();
        assert_eq!(d, p);
    }
}
