use windows::core::PCWSTR;
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

use crate::error::{AppError, AppResult, ErrorCode};

// 直接 FFI 声明 LocalFree，因为 windows 0.58 中可能不在预期位置
extern "system" {
    fn LocalFree(hmem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

/// DPAPI CurrentUser 范围加密
pub struct Dpapi;

impl Dpapi {
    /// 使用 Windows DPAPI CurrentUser 加密数据
    pub fn encrypt(plaintext: &[u8]) -> AppResult<Vec<u8>> {
        if plaintext.is_empty() {
            return Ok(b"PMV1".to_vec());
        }

        unsafe {
            let input = CRYPT_INTEGER_BLOB {
                cbData: plaintext.len() as u32,
                pbData: plaintext.as_ptr() as *mut u8,
            };

            let mut output = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };

            let result = CryptProtectData(
                &input,
                PCWSTR::null(),
                None,
                None,
                None,
                0x1, // CRYPTPROTECT_UI_FORBIDDEN
                &mut output,
            );

            if result.is_err() {
                tracing::error!("DPAPI CryptProtectData failed");
                return Err(AppError::new(
                    ErrorCode::VaultEncryptFailed,
                    "DPAPI 加密失败",
                ));
            }

            let ciphertext =
                std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();

            // 使用 LocalFree 释放 DPAPI 分配的内存
            if !output.pbData.is_null() {
                LocalFree(output.pbData as *mut core::ffi::c_void);
            }

            let mut result = b"PMV1".to_vec();
            result.extend_from_slice(&ciphertext);
            Ok(result)
        }
    }

    /// 使用 Windows DPAPI CurrentUser 解密数据
    pub fn decrypt(ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        if ciphertext.len() < 4 || &ciphertext[..4] != b"PMV1" {
            return Err(AppError::new(
                ErrorCode::VaultCiphertextCorrupted,
                "密文格式损坏：缺少或错误的魔数前缀",
            ));
        }

        let actual_ciphertext = &ciphertext[4..];
        if actual_ciphertext.is_empty() {
            return Ok(Vec::new());
        }

        unsafe {
            let input = CRYPT_INTEGER_BLOB {
                cbData: actual_ciphertext.len() as u32,
                pbData: actual_ciphertext.as_ptr() as *mut u8,
            };

            let mut output = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };

            let result = CryptUnprotectData(
                &input,
                None,
                None,
                None,
                None,
                0x1, // CRYPTPROTECT_UI_FORBIDDEN
                &mut output,
            );

            if result.is_err() {
                tracing::error!("DPAPI CryptUnprotectData failed");
                return Err(AppError::new(
                    ErrorCode::VaultDecryptFailed,
                    "DPAPI 解密失败：密文可能已损坏或由其他用户/设备加密",
                ));
            }

            let plaintext =
                std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();

            if !output.pbData.is_null() {
                LocalFree(output.pbData as *mut core::ffi::c_void);
            }

            Ok(plaintext)
        }
    }

    /// 安全清除内存中的敏感数据
    pub fn zero_memory(data: &mut [u8]) {
        for byte in data.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"PM_TEST_SECRET_7f31a9_DO_NOT_USE";
        let encrypted = Dpapi::encrypt(plaintext).unwrap();
        let decrypted = Dpapi::decrypt(&encrypted).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_empty() {
        let encrypted = Dpapi::encrypt(b"").unwrap();
        assert_eq!(&encrypted[..4], b"PMV1");
        assert_eq!(encrypted.len(), 4);
        let decrypted = Dpapi::decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_decrypt_corrupted() {
        let mut encrypted = Dpapi::encrypt(b"test data").unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;
        let result = Dpapi::decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_no_magic() {
        let result = Dpapi::decrypt(b"XXXXrestofdata");
        assert!(result.is_err());
    }

    #[test]
    fn test_ciphertext_does_not_contain_plaintext() {
        let plaintext = b"PM_TEST_SECRET_7f31a9_DO_NOT_USE";
        let encrypted = Dpapi::encrypt(plaintext).unwrap();
        let encrypted_str = String::from_utf8_lossy(&encrypted);
        assert!(!encrypted_str.contains("PM_TEST_SECRET"));
    }

    #[test]
    fn test_zero_memory() {
        let mut data = b"sensitive".to_vec();
        Dpapi::zero_memory(&mut data);
        assert!(data.iter().all(|&b| b == 0));
    }
}
