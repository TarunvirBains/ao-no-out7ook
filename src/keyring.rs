use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_DEVOPS: &str = "ao-no-out7ook-devops";

/// Store a credential in the system keyring
pub fn store_credential(service: &str, username: &str, password: &str) -> Result<()> {
    let entry = Entry::new(service, username).context("Failed to create keyring entry")?;

    entry
        .set_password(password)
        .context("Failed to store credential in keyring")?;

    Ok(())
}

/// Retrieve a credential from the system keyring
pub fn get_credential(service: &str, username: &str) -> Result<String> {
    let entry = Entry::new(service, username).context("Failed to create keyring entry")?;

    entry
        .get_password()
        .context("Failed to retrieve credential from keyring")
}

/// Delete a credential from the system keyring
pub fn delete_credential(service: &str, username: &str) -> Result<()> {
    let entry = Entry::new(service, username).context("Failed to create keyring entry")?;

    entry
        .delete_credential()
        .context("Failed to delete credential from keyring")?;

    Ok(())
}

/// Store DevOps PAT in keyring
pub fn store_devops_pat(pat: &str) -> Result<()> {
    store_credential(SERVICE_DEVOPS, "default", pat)
}

/// Retrieve DevOps PAT from keyring
pub fn get_devops_pat() -> Result<String> {
    get_credential(SERVICE_DEVOPS, "default")
}

/// Delete DevOps PAT from keyring
pub fn delete_devops_pat() -> Result<()> {
    delete_credential(SERVICE_DEVOPS, "default")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual keyring backend
    fn test_store_and_retrieve() {
        let test_service = "ao-no-out7ook-test";
        let test_username = "test_user";
        let test_password = "test_password_123";

        // Store
        store_credential(test_service, test_username, test_password).unwrap();

        // Retrieve
        let retrieved = get_credential(test_service, test_username).unwrap();
        assert_eq!(retrieved, test_password);

        // Cleanup
        delete_credential(test_service, test_username).unwrap();
    }
}
