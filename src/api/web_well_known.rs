use serde_json::Value;

/// Apple associated-domain document for Bitwarden iOS (including Autofill).
pub fn apple_app_site_association_body() -> Value {
    json!({
        "webcredentials": {
            "apps": [
                "LTZ2PFU5D6.com.8bit.bitwarden",
                "LTZ2PFU5D6.com.8bit.bitwarden.beta",
                "LTZ2PFU5D6.com.8bit.bitwarden.autofill"
            ]
        }
    })
}

/// W3C Related Origin Requests document for the vault origin.
/// Clients fetch this when `pm-30529-webauthn-related-origins` is enabled.
pub fn webauthn_related_origins_body(origin: &str) -> Value {
    json!({
        "origins": [origin]
    })
}

/// Dual-mount well-known at `/` when the API lives under a DOMAIN path prefix.
pub fn should_mount_origin_root_well_known(domain_path: &str) -> bool {
    !domain_path.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aasa_includes_autofill_extension() {
        let body = apple_app_site_association_body();
        let apps = body["webcredentials"]["apps"].as_array().unwrap();
        let ids: Vec<&str> = apps.iter().filter_map(Value::as_str).collect();
        assert!(ids.contains(&"LTZ2PFU5D6.com.8bit.bitwarden"));
        assert!(ids.contains(&"LTZ2PFU5D6.com.8bit.bitwarden.autofill"));
    }

    #[test]
    fn related_origins_lists_vault_origin() {
        let body = webauthn_related_origins_body("https://vault.example");
        assert_eq!(body["origins"], json!(["https://vault.example"]));
    }

    #[test]
    fn origin_root_mount_when_domain_has_path() {
        assert!(should_mount_origin_root_well_known("/vw"));
        assert!(!should_mount_origin_root_well_known(""));
    }
}
