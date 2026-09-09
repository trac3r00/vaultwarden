use serde_json::Value;

use crate::{
    api::EmptyResult,
    db::models::cipher_login::normalize_login_type_data,
    util::{LowerCase, validate_and_format_date},
};

/// Same three members `Cipher::to_json` requires for SSH items.
/// Official Bitwarden rejects null/empty; VW used to 200 then drop `sshKey` on read (#7514).
pub fn ssh_key_data_is_complete(type_data: &Value) -> bool {
    ["privateKey", "publicKey", "keyFingerprint"]
        .iter()
        .all(|field| type_data[*field].as_str().is_some_and(|s| !s.is_empty()))
}

pub fn validate_ssh_key_data(type_data: &Value) -> EmptyResult {
    for field in ["privateKey", "publicKey", "keyFingerprint"] {
        if type_data[field].as_str().is_none_or(str::is_empty) {
            err!(format!("SSH key field '{field}' must be a non-empty string"));
        }
    }
    Ok(())
}

/// Parse stored cipher `data` JSON and apply per-type client compatibility fixes.
pub fn normalize_cipher_type_data(atype: i32, data: &str, cipher_uuid: &str) -> Value {
    let mut type_data_json = serde_json::from_str::<LowerCase<Value>>(data)
        .inspect_err(|_| warn!("Error parsing data field for {cipher_uuid}"))
        .map_or_else(|_| Value::Object(serde_json::Map::new()), |d| d.data);

    match atype {
        1 => {
            type_data_json["uri"] = Value::Null;
            if let Some(uris) = type_data_json["uris"].as_array_mut()
                && !uris.is_empty()
            {
                for uri in &mut *uris {
                    if uri["match"].is_string() {
                        let match_value = match uri["match"].as_str().unwrap_or_default().parse::<u8>() {
                            Ok(n) => json!(n),
                            _ => Value::Null,
                        };
                        uri["match"] = match_value;
                    }
                }
                type_data_json["uri"] = uris[0]["uri"].clone();
            }
            if let Some(pw_revision) = type_data_json["passwordRevisionDate"].as_str() {
                type_data_json["passwordRevisionDate"] = json!(validate_and_format_date(pw_revision));
            }
            type_data_json = normalize_login_type_data(type_data_json);
        }
        2 => match type_data_json {
            Value::Object(ref t) if t.get("type").is_some_and(Value::is_number) => {}
            _ => {
                type_data_json = json!({"type": 0});
            }
        },
        5 if !ssh_key_data_is_complete(&type_data_json) => {
            warn!("Error parsing ssh-key, mandatory fields are invalid for {cipher_uuid}");
            type_data_json = Value::Null;
        }
        _ => {}
    }

    type_data_json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_key_validation_table() {
        let cases = [
            ("all fields present", json!({"privateKey": "priv", "publicKey": "pub", "keyFingerprint": "fp"}), true),
            ("null private key", json!({"privateKey": null, "publicKey": "pub", "keyFingerprint": "fp"}), false),
            ("empty public key", json!({"privateKey": "priv", "publicKey": "", "keyFingerprint": "fp"}), false),
            ("missing fingerprint", json!({"privateKey": "priv", "publicKey": "pub"}), false),
            ("non-string fingerprint", json!({"privateKey": "priv", "publicKey": "pub", "keyFingerprint": 1}), false),
            ("no fields", json!({}), false),
        ];
        for (case, type_data, expected_ok) in cases {
            assert_eq!(validate_ssh_key_data(&type_data).is_ok(), expected_ok, "case: {case}");
            assert_eq!(ssh_key_data_is_complete(&type_data), expected_ok, "complete: {case}");
        }
    }

    #[test]
    fn invalid_stored_ssh_type_data_becomes_null_on_read() {
        let out = normalize_cipher_type_data(5, r#"{"privateKey":"x"}"#, "cid");
        assert_eq!(out, Value::Null);
    }
}
