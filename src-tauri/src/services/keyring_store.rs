use keyring::Entry;

const SERVICE: &str = "tianxuan-desktop";

pub fn set_password(key: &str, password: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
    entry.set_password(password).map_err(|e| e.to_string())
}

pub fn get_password(key: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn delete_password(key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "test-keyring-store-unit";

    fn cleanup() {
        let _ = delete_password(TEST_KEY);
    }

    #[test]
    fn test_set_get_delete() {
        cleanup();
        // initially empty
        assert_eq!(get_password(TEST_KEY).unwrap(), None);

        set_password(TEST_KEY, "s3cret-value").unwrap();
        assert_eq!(get_password(TEST_KEY).unwrap().as_deref(), Some("s3cret-value"));

        delete_password(TEST_KEY).unwrap();
        assert_eq!(get_password(TEST_KEY).unwrap(), None);
        cleanup();
    }

    #[test]
    fn test_overwrite() {
        cleanup();
        set_password(TEST_KEY, "first").unwrap();
        set_password(TEST_KEY, "second").unwrap();
        assert_eq!(get_password(TEST_KEY).unwrap().as_deref(), Some("second"));
        cleanup();
    }
}
