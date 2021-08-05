use anyhow::Result;
use log::{info, warn};
use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse};

fn get_username() -> Option<String> {
  users::get_current_username().map(|user| user.to_string_lossy().to_string())
}

pub fn get_stored_token<S: AsRef<str>>(
  service: S,
) -> Option<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
  if let Some(user) = get_username() {
    let token =
      keyring::Keyring::new(&format!("fitsync_{}", service.as_ref()), &user).get_password();
    if let Ok(token) = token {
      let decoded = base64::decode(&token);
      if let Ok(decoded) = decoded {
        let decoded = serde_json::from_slice(&decoded);
        if let Ok(decoded) = decoded {
          info!("Found token in keystore for {}", service.as_ref());
          return Some(decoded);
        } else {
          warn!(
            "Found token in keystore, but failed to decode: {:?}",
            decoded.err()
          )
        }
      } else {
        warn!(
          "Found token in keystore but failed to base64 decode it. {:?}",
          decoded.err()
        )
      }
    }
  } else {
    warn!("Could not determine $USER");
  }
  None
}

pub fn store_token<S: AsRef<str>>(
  service: S,
  token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
) -> Result<()> {
  if let Some(user) = get_username() {
    let json = serde_json::to_string(&token);
    if let Ok(json) = json {
      let encoded = base64::encode(json.as_bytes());
      keyring::Keyring::new(&format!("fitsync_{}", service.as_ref()), &user)
        .set_password(&encoded)?;
    } else {
      warn!("Failed to serialize token: {:?}", json.err());
    }
  } else {
    warn!("Could not determine $USER");
  }

  Ok(())
}
