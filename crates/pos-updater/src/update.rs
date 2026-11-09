use crate::Error;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub sha256: String,
    pub signature: String,
    pub release_notes: String,
}

pub struct UpdateChecker {
    pub current_version: String,
    pub update_url: String,
    pub public_key: VerifyingKey,
}

impl UpdateChecker {
    pub fn new(current_version: String, update_url: String, public_key_bytes: [u8; 32]) -> Result<Self, Error> {
        let public_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|e| Error::Verification(format!("Clave pública inválida: {}", e)))?;
        
        Ok(Self {
            current_version,
            update_url,
            public_key,
        })
    }

    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>, Error> {
        let response = reqwest::get(&self.update_url)
            .await
            .map_err(|e| Error::Download(format!("Error al consultar actualizaciones: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Download(format!(
                "Error HTTP al consultar actualizaciones: {}",
                response.status()
            )));
        }

        let update_info: UpdateInfo = response
            .json()
            .await
            .map_err(|e| Error::Download(format!("Error al parsear respuesta: {}", e)))?;

        // Comparar versiones
        if update_info.version != self.current_version {
            Ok(Some(update_info))
        } else {
            Ok(None)
        }
    }

    pub async fn download_update(&self, info: &UpdateInfo, dest: &Path) -> Result<(), Error> {
        tracing::info!("Descargando actualización desde: {}", info.download_url);

        let response = reqwest::get(&info.download_url)
            .await
            .map_err(|e| Error::Download(format!("Error al descargar: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Download(format!(
                "Error HTTP al descargar: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::Download(format!("Error al leer bytes: {}", e)))?;

        // Verificar SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());

        if hash != info.sha256 {
            return Err(Error::Verification(format!(
                "Hash SHA-256 no coincide. Esperado: {}, obtenido: {}",
                info.sha256, hash
            )));
        }

        // Verificar firma ed25519
        verify_signature(&info.sha256, &info.signature, &self.public_key)?;

        // Guardar archivo
        std::fs::write(dest, bytes)
            .map_err(|e| Error::Io(format!("Error al guardar archivo: {}", e)))?;

        tracing::info!("Actualización descargada y verificada en: {:?}", dest);
        Ok(())
    }
}

pub fn verify_signature(message: &str, signature_hex: &str, public_key: &VerifyingKey) -> Result<(), Error> {
    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| Error::Verification(format!("Firma hex inválida: {}", e)))?;

    if signature_bytes.len() != 64 {
        return Err(Error::Verification(format!(
            "Firma debe tener 64 bytes, tiene: {}",
            signature_bytes.len()
        )));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&signature_bytes);
    let signature = Signature::from_bytes(&sig_array);

    public_key
        .verify(message.as_bytes(), &signature)
        .map_err(|e| Error::Verification(format!("Firma inválida: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer};
    use std::fs;
    use tempfile::TempDir;

    fn generate_test_keypair() -> (SigningKey, VerifyingKey) {
        use rand::RngCore;
        let mut csprng = rand::rngs::OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        (signing_key, verifying_key)
    }

    #[test]
    fn test_verify_signature_valid() {
        let (signing_key, verifying_key) = generate_test_keypair();
        let message = "test_message";
        let signature = signing_key.sign(message.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_signature(message, &signature_hex, &verifying_key);
        assert!(result.is_ok(), "Firma válida debe verificarse");
    }

    #[test]
    fn test_verify_signature_invalid() {
        let (_, verifying_key) = generate_test_keypair();
        let message = "test_message";
        let fake_signature = hex::encode([0u8; 64]);

        let result = verify_signature(message, &fake_signature, &verifying_key);
        assert!(result.is_err(), "Firma inválida debe fallar");
    }

    #[test]
    fn test_verify_signature_wrong_message() {
        let (signing_key, verifying_key) = generate_test_keypair();
        let message = "original_message";
        let signature = signing_key.sign(message.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_signature("different_message", &signature_hex, &verifying_key);
        assert!(result.is_err(), "Firma con mensaje diferente debe fallar");
    }

    #[tokio::test]
    async fn test_download_with_invalid_url() {
        let (_, verifying_key) = generate_test_keypair();
        
        let checker = UpdateChecker {
            current_version: "1.0.0".to_string(),
            update_url: "http://invalid.url/update".to_string(),
            public_key: verifying_key,
        };

        let info = UpdateInfo {
            version: "1.0.1".to_string(),
            download_url: "http://invalid.url/download".to_string(),
            sha256: "fake_hash".to_string(),
            signature: "fake_sig".to_string(),
            release_notes: "Test".to_string(),
        };

        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("update.bin");

        let result = checker.download_update(&info, &dest).await;
        assert!(result.is_err(), "Descarga con URL inválida debe fallar");
    }
}
