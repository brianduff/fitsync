use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use oauth2::basic::BasicTokenResponse;
use oauth2::reqwest::http_client;
use oauth2::{basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, TokenUrl};
use oauth2::{ClientSecret, RedirectUrl, TokenResponse};
use serde::Deserialize;

static FITBIT: &str = "fitbit";

lazy_static! {
  static ref URLS: HashMap<&'static str, ServiceUrls> = {
    let mut m = HashMap::new();
    m.insert(
      FITBIT,
      ServiceUrls {
        auth_url: "https://www.fitbit.com/oauth2/authorize".to_owned(),
        token_url: "https://api.fitbit.com/oauth2/token".to_owned(),
        redirect_url_path: "/auth/fitbit".to_owned(),
      },
    );
    m
  };
}

struct ServiceUrls {
  auth_url: String,
  token_url: String,
  redirect_url_path: String,
}

#[derive(Deserialize)]

pub struct ServiceClient {
  pub id: String,
  pub secret: String,
}

pub struct OAuthClient {
  client: BasicClient,
  tokens: Option<BasicTokenResponse>,
  client_id: String,
}

pub struct Tokens {
  token_response: BasicTokenResponse,
}

impl OAuthClient {
  pub fn for_service(service_name: &str, secrets: &ServiceClient) -> Result<Self> {
    let urls = URLS.get(service_name).unwrap();
    let client = BasicClient::new(
      ClientId::new(secrets.id.to_owned()),
      Some(ClientSecret::new(secrets.secret.to_owned())),
      AuthUrl::new(urls.auth_url.to_owned())?,
      Some(TokenUrl::new(urls.token_url.to_owned())?),
    )
    .set_redirect_uri(RedirectUrl::new(
      "http://localhost:8000/auth/fitbit".to_string(),
    )?);

    Ok(Self {
      client,
      client_id: secrets.id.to_owned(),
      tokens: None,
    })
  }

  pub fn obtain_tokens(&mut self, auth_code: String) -> Result<()> {
    let result = self
      .client
      .exchange_code(AuthorizationCode::new(auth_code))
      .request(http_client)?;
    self.tokens = Some(result);
    Ok(())
  }

  pub fn get_secret(&self) -> Result<String> {
    if let Some(ref tokens) = self.tokens {
      Ok(tokens.access_token().secret().to_owned())
    } else {
      Err(anyhow!("No token retrieved. Call obtain_tokens() first."))
    }
  }

  pub fn refresh_tokens(&mut self) -> Result<()> {
    if let Some(ref tokens) = self.tokens {
      let result = self
        .client
        .exchange_refresh_token(tokens.refresh_token().unwrap())
        .request(http_client)?;
      self.tokens = Some(result);
      Ok(())
    } else {
      Err(anyhow!("No token retrieved. Call obtain_tokens() first."))
    }
  }

  pub fn has_secret(&self) -> bool {
    self.tokens.is_some()
  }

  pub fn get_client_id(&self) -> String {
    self.client_id.to_owned()
  }
}
