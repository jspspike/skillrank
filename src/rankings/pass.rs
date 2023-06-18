use sha2::{Digest, Sha256};
use worker::*;

pub async fn get(state: &State) -> Result<bool> {
    let result: Result<Vec<u8>> = state.storage().get("pass").await;
    Ok(result.is_ok())
}

pub async fn set(state: &State, pass: String, salt: String) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", pass, salt));
    let result = hasher.finalize();
    state.storage().put("pass", result.as_slice()).await
}

pub async fn check(state: &State, pass: String, salt: String) -> Result<bool> {
    let salted_pass: Vec<u8> = state.storage().get("pass").await?;
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", pass, salt));
    let result = hasher.finalize();
    Ok(salted_pass == result.as_slice())
}

pub async fn delete(state: &State) -> Result<bool> {
    state.storage().delete("pass").await
}
