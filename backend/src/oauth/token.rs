use axum::{Form, Json, Router, extract::Query, routing::post};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use centaurus::{
  backend::auth::{jwt_state::JwtInvalidState, oidc::URL_SAFE_CHARS},
  bail,
  db::{init::Connection, tables::ConnectionExt},
  eyre::ContextCompat,
  serde::empty_string_as_none,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt::JwtStateOther,
  db::DBTrait,
  oauth::{
    client_auth::{TokenIssueReq, TokenRefreshReq},
    jwt::RefreshTokenClaims,
    state::CodeChallengeMethod,
  },
};

use super::{
  client_auth::{ClientAuth, Error},
  jwt::OAuthClaims,
  scope::Scope,
  state::{AuthorizeState, ConfigurationState, get_timestamp_10_min},
};

pub fn router() -> Router {
  Router::new()
    .route("/token", post(token))
    .route("/revoke", post(revoke))
}

#[derive(Serialize)]
struct TokenRes {
  access_token: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  id_token: Option<String>,
  token_type: String,
  expires_in: u64,
  scope: Scope,
  refresh_token: String,
}

#[instrument(skip(state, jwt, db, config))]
async fn token(
  state: AuthorizeState,
  jwt: JwtStateOther,
  db: Connection,
  config: ConfigurationState,
  auth: ClientAuth,
) -> Result<Json<TokenRes>, Error> {
  if let Some(body) = auth.body.clone().try_into_issue() {
    issue_token(state, jwt, db, config, body, auth.client_id).await
  } else if let Some(body) = auth.body.clone().try_into_refresh() {
    refresh_token(jwt, db, config, body, auth.client_id).await
  } else {
    tracing::warn!("unsupported grant type: {}", auth.body.grant_type);
    Err(Error::from_str("unsupported_grant_type"))
  }
}

#[instrument(skip(state, jwt, db, config))]
async fn issue_token(
  state: AuthorizeState,
  jwt: JwtStateOther,
  db: Connection,
  config: ConfigurationState,
  body: TokenIssueReq,
  client_id: Uuid,
) -> Result<Json<TokenRes>, Error> {
  let uuid = body.code;

  let Some(auth_code) = state.auth_codes.get(&uuid) else {
    tracing::warn!("authorization code not found: {}", uuid);
    return Err(Error::from_str("invalid_grant"));
  };
  let code_info = &auth_code.value().1;

  if &body.grant_type != "authorization_code" {
    tracing::warn!("unsupported grant type: {}", body.grant_type);
    return Err(Error::from_str("unsupported_grant_type"));
  }
  if code_info.client_id != client_id {
    tracing::warn!(
      "client id mismatch for authorization code: {}, expected {}, got {}",
      uuid,
      code_info.client_id,
      client_id
    );
    return Err(Error::from_str("invalid_client"));
  }

  if let Some(uri) = &code_info.redirect_uri {
    if let Some(req_uri) = body.redirect_uri.clone() {
      if *uri != req_uri {
        tracing::warn!(
          "redirect uri mismatch for authorization code: {}, expected {}, got {}",
          uuid,
          uri,
          req_uri
        );
        return Err(Error::from_str("invalid_request"));
      }
    } else {
      tracing::warn!(
        "missing redirect uri for authorization code: {}, expected {}",
        uuid,
        uri
      );
      return Err(Error::from_str("invalid_request"));
    }
  }

  if let Some(code_challenge) = &code_info.code_challenge {
    if let Some(code_verifier) = &body.code_verifier {
      if !(43..=128).contains(&code_verifier.len())
        || code_verifier
          .chars()
          .any(|c| !URL_SAFE_CHARS.contains(&(c as u8)))
      {
        return Err(Error::from_str("invalid_request"));
      }

      let expected_challenge = match code_challenge.method {
        CodeChallengeMethod::Plain => code_verifier.clone(),
        CodeChallengeMethod::S256 => {
          let ascii_bytes = code_verifier.as_bytes();
          let mut hasher = Sha256::new();
          hasher.update(ascii_bytes);
          BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize())
        }
      };
      if code_challenge.challenge != expected_challenge {
        return Err(Error::from_str("invalid_grant"));
      }
    } else {
      return Err(Error::from_str("invalid_request"));
    }
  }

  drop(auth_code);
  // unwrap is safe because we checked that the code exists with get before
  let (_, (_, code_info)) = state.auth_codes.remove(&uuid).unwrap();

  let exp = Utc::now()
    .checked_add_signed(Duration::seconds(config.refresh_exp))
    .ok_or(Error::from_str("invalid_request"))?
    .timestamp();

  let code_info = RefreshTokenClaims {
    sub: code_info.user,
    aud: code_info.client_id,
    scope: code_info.scope,
    nonce: code_info.nonce,
    exp,
    iss: config.issuer.clone().to_string(),
  };

  let token = create_access_token(&db, &jwt, &code_info, &config, client_id).await?;

  let Ok(refresh_token) = jwt.create_generic_token(&code_info) else {
    tracing::warn!("failed to create refresh token for client: {}", client_id);
    return Err(Error::from_str("unauthorized_client"));
  };

  let id_token = if code_info.scope.contains("openid") {
    Some(token.clone())
  } else {
    None
  };

  Ok(Json(TokenRes {
    access_token: token,
    id_token,
    token_type: "Bearer".into(),
    scope: code_info.scope,
    expires_in: 600,
    refresh_token,
  }))
}

#[instrument(skip(jwt, db, config))]
async fn refresh_token(
  jwt: JwtStateOther,
  db: Connection,
  config: ConfigurationState,
  body: TokenRefreshReq,
  client_id: Uuid,
) -> Result<Json<TokenRes>, Error> {
  let mut claims = jwt
    .validate_token::<RefreshTokenClaims>(&body.refresh_token)
    .map_err(|_| {
      tracing::warn!("invalid refresh token for client: {}", client_id);
      Error::from_str("invalid_grant")
    })?;

  if claims.aud != client_id {
    tracing::warn!(
      "client id mismatch for refresh token: {}, expected {}, got {}",
      body.refresh_token,
      claims.aud,
      client_id
    );
    return Err(Error::from_str("invalid_client"));
  }

  let token = create_access_token(&db, &jwt, &claims, &config, client_id).await?;

  let exp = Utc::now()
    .checked_add_signed(Duration::seconds(config.refresh_exp))
    .ok_or(Error::from_str("invalid_request"))?
    .timestamp();

  claims.exp = exp;

  let Ok(refresh_token) = jwt.create_generic_token(&claims) else {
    tracing::warn!("failed to create refresh token for client: {}", client_id);
    return Err(Error::from_str("unauthorized_client"));
  };

  let id_token = if claims.scope.contains("openid") {
    Some(token.clone())
  } else {
    None
  };

  Ok(Json(TokenRes {
    access_token: token,
    id_token,
    token_type: "Bearer".into(),
    scope: claims.scope,
    expires_in: 600,
    refresh_token,
  }))
}

async fn create_access_token(
  db: &Connection,
  jwt: &JwtStateOther,
  code_info: &RefreshTokenClaims,
  config: &ConfigurationState,
  client_id: Uuid,
) -> Result<String, Error> {
  let Ok(user) = db.user().get_user_by_id(code_info.sub).await else {
    tracing::warn!("user not found: {}", code_info.sub);
    return Err(Error::from_str("unauthorized_client"));
  };
  let Ok(groups) = db.user().get_user_groups(user.id).await else {
    tracing::warn!("failed to get groups for user: {}", user.id);
    return Err(Error::from_str("unauthorized_client"));
  };

  let group_ids: Vec<Uuid> = groups.iter().map(|g| g.uuid).collect();
  let Ok(rest) = db
    .oauth_scope()
    .get_values_for_user(code_info.scope.inner(), &group_ids)
    .await
  else {
    tracing::warn!("failed to get scope values for user: {}", user.id);
    return Err(Error::from_str("unauthorized_client"));
  };

  let groups = groups.into_iter().map(|group| group.name).collect();

  let name = if code_info.scope.contains("profile") {
    Some(user.name.clone())
  } else {
    None
  };
  let email = if code_info.scope.contains("email") {
    Some(user.email)
  } else {
    None
  };
  let picture = if code_info.scope.contains("image")
    && db.user_ext().has_avatar(code_info.sub).await.map_err(|_| {
      tracing::warn!("failed to check avatar for user: {}", code_info.sub);
      Error::from_str("unauthorized_client")
    })? {
    Some(format!("{}/picture/{}", config.issuer, code_info.sub))
  } else {
    None
  };

  let time = Utc::now().timestamp();
  let claims = OAuthClaims {
    sub: code_info.sub,
    exp: get_timestamp_10_min(),
    iss: config.issuer.clone().to_string(),
    aud: code_info.aud,
    iat: time,
    auth_time: time,
    nonce: code_info.nonce.clone(),
    scope: code_info.scope.clone(),
    email,
    preferred_username: name.clone(),
    picture,
    name,
    groups,
    rest,
  };

  let Ok(token) = jwt.create_generic_token(&claims) else {
    tracing::warn!("failed to create token for client: {}", client_id);
    return Err(Error::from_str("unauthorized_client"));
  };

  tracing::info!("Client {} got token for {}", client_id, user.name);
  Ok(token)
}

#[derive(Deserialize, Debug)]
struct RevokeReqOption {
  #[serde(default, deserialize_with = "empty_string_as_none")]
  token: Option<String>,
}

impl RevokeReqOption {
  fn try_into(self) -> Option<RevokeReq> {
    let token = self.token?;
    Some(RevokeReq { token })
  }
}

#[derive(Deserialize)]
struct RevokeReq {
  token: String,
}

#[instrument(skip(state, db, invalidate))]
async fn revoke(
  Query(req_p): Query<RevokeReqOption>,
  db: Connection,
  state: JwtStateOther,
  invalidate: JwtInvalidState,
  Form(req_b): Form<RevokeReqOption>,
) -> centaurus::error::Result<()> {
  let req = if let Some(req) = req_p.try_into() {
    req
  } else if let Some(req) = req_b.try_into() {
    req
  } else {
    bail!("invalid_request");
  };

  let claims = state.validate_token::<OAuthClaims>(&req.token)?;
  let exp = DateTime::from_timestamp(claims.exp, 0).context("Invalid timestamp")?;

  db.invalid_jwt()
    .invalidate_jwt(req.token, exp, invalidate.count.clone())
    .await?;

  Ok(())
}

#[cfg(test)]
mod test {
  use super::{RevokeReqOption, create_access_token, issue_token, refresh_token, revoke, token};
  use crate::{
    config::Config,
    db::test::{insert_user, jwt_state, test_db},
    oauth::{
      client_auth::{ClientAuth, Error, TokenIssueReq, TokenRefreshReq, TokenReq},
      jwt::{OAuthClaims, RefreshTokenClaims},
      scope::Scope,
      state::{
        AuthorizeState, CodeChallenge, CodeChallengeMethod, CodeReq, ConfigurationState,
        get_timestamp_10_min,
      },
    },
  };
  use axum::{Form, extract::Query};
  use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
  use centaurus::backend::auth::jwt_state::JwtInvalidState;
  use sha2::{Digest, Sha256};
  use std::{collections::HashMap, time::Instant};
  use uuid::Uuid;

  // ---- helpers -----------------------------------------------------------

  fn err_code(e: Error) -> String {
    serde_json::to_value(e).unwrap()["error"]
      .as_str()
      .unwrap()
      .to_string()
  }

  fn s256(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize())
  }

  fn code_req(client_id: Uuid, user: Uuid, scope: &[&str]) -> CodeReq {
    CodeReq {
      client_id,
      redirect_uri: None,
      scope: Scope::from(scope.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
      user,
      nonce: None,
      code_challenge: None,
    }
  }

  struct Ctx {
    db: centaurus::db::init::Connection,
    jwt: crate::auth::jwt::JwtStateOther,
    state: AuthorizeState,
    config: ConfigurationState,
    client_id: Uuid,
    user: Uuid,
  }

  async fn ctx() -> Ctx {
    let db = test_db().await;
    let user = insert_user(&db, "user", "user@x.com").await;
    let jwt = jwt_state(&db).await;
    let cfg = Config::default();
    Ctx {
      state: AuthorizeState::init(&cfg),
      config: ConfigurationState::init(&cfg),
      client_id: Uuid::new_v4(),
      user,
      jwt,
      db,
    }
  }

  // ---- RevokeReqOption ---------------------------------------------------

  #[test]
  fn revoke_option_none_when_token_absent() {
    assert!(RevokeReqOption { token: None }.try_into().is_none());
  }

  #[test]
  fn revoke_option_some_when_token_present() {
    let opt = RevokeReqOption {
      token: Some("tok".into()),
    };
    assert_eq!(opt.try_into().unwrap().token, "tok");
  }

  // ---- issue_token -------------------------------------------------------

  #[tokio::test]
  async fn issue_token_success_with_openid_emits_id_token() {
    let c = ctx().await;
    let code = Uuid::new_v4();
    c.state.auth_codes.insert(
      code,
      (Instant::now(), code_req(c.client_id, c.user, &["openid"])),
    );

    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let axum::Json(res) = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .unwrap();

    assert!(!res.access_token.is_empty());
    assert!(!res.refresh_token.is_empty());
    assert!(res.id_token.is_some());
    assert_eq!(res.token_type, "Bearer");
    assert_eq!(res.expires_in, 600);
    assert_eq!(res.scope.to_string(), "openid");
  }

  #[tokio::test]
  async fn issue_token_without_openid_has_no_id_token() {
    let c = ctx().await;
    let code = Uuid::new_v4();
    c.state.auth_codes.insert(
      code,
      (Instant::now(), code_req(c.client_id, c.user, &["email"])),
    );

    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let axum::Json(res) = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .unwrap();
    assert!(res.id_token.is_none());
  }

  #[tokio::test]
  async fn issue_token_unknown_code_is_invalid_grant() {
    let c = ctx().await;
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code: Uuid::new_v4(),
      redirect_uri: None,
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_grant");
  }

  #[tokio::test]
  async fn issue_token_wrong_grant_type_is_unsupported() {
    let c = ctx().await;
    let code = Uuid::new_v4();
    c.state.auth_codes.insert(
      code,
      (Instant::now(), code_req(c.client_id, c.user, &["openid"])),
    );
    let body = TokenIssueReq {
      grant_type: "client_credentials".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "unsupported_grant_type");
  }

  #[tokio::test]
  async fn issue_token_client_id_mismatch() {
    let c = ctx().await;
    let code = Uuid::new_v4();
    c.state.auth_codes.insert(
      code,
      (
        Instant::now(),
        code_req(Uuid::new_v4(), c.user, &["openid"]),
      ),
    );
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_client");
  }

  #[tokio::test]
  async fn issue_token_redirect_uri_mismatch_and_missing() {
    // mismatch
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.redirect_uri = Some("https://app/cb".into());
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: Some("https://evil/cb".into()),
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_request");

    // missing in request but required by the stored code
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.redirect_uri = Some("https://app/cb".into());
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_request");
  }

  #[tokio::test]
  async fn issue_token_matching_redirect_uri_succeeds() {
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.redirect_uri = Some("https://app/cb".into());
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: Some("https://app/cb".into()),
      code_verifier: None,
    };
    assert!(
      issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
        .await
        .is_ok()
    );
  }

  #[tokio::test]
  async fn issue_token_pkce_s256_success_and_failure() {
    let verifier = "a".repeat(43);

    // success
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.code_challenge = Some(CodeChallenge {
      challenge: s256(&verifier),
      method: CodeChallengeMethod::S256,
    });
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: Some(verifier.clone()),
    };
    assert!(
      issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
        .await
        .is_ok()
    );

    // wrong verifier -> invalid_grant
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.code_challenge = Some(CodeChallenge {
      challenge: s256(&verifier),
      method: CodeChallengeMethod::S256,
    });
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: Some("b".repeat(43)),
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_grant");
  }

  #[tokio::test]
  async fn issue_token_pkce_plain_success() {
    let verifier = "a".repeat(43);
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.code_challenge = Some(CodeChallenge {
      challenge: verifier.clone(),
      method: CodeChallengeMethod::Plain,
    });
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: Some(verifier),
    };
    assert!(
      issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
        .await
        .is_ok()
    );
  }

  #[tokio::test]
  async fn issue_token_pkce_invalid_verifier_length_and_missing() {
    // verifier too short
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.code_challenge = Some(CodeChallenge {
      challenge: "x".into(),
      method: CodeChallengeMethod::Plain,
    });
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: Some("short".into()),
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_request");

    // verifier missing entirely
    let c = ctx().await;
    let code = Uuid::new_v4();
    let mut req = code_req(c.client_id, c.user, &["openid"]);
    req.code_challenge = Some(CodeChallenge {
      challenge: "a".repeat(43),
      method: CodeChallengeMethod::Plain,
    });
    c.state.auth_codes.insert(code, (Instant::now(), req));
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_request");
  }

  #[tokio::test]
  async fn issue_token_unknown_user_is_unauthorized_client() {
    let c = ctx().await;
    let code = Uuid::new_v4();
    // user id that does not exist in the db
    c.state.auth_codes.insert(
      code,
      (
        Instant::now(),
        code_req(c.client_id, Uuid::new_v4(), &["openid"]),
      ),
    );
    let body = TokenIssueReq {
      grant_type: "authorization_code".into(),
      code,
      redirect_uri: None,
      code_verifier: None,
    };
    let err = issue_token(c.state, c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "unauthorized_client");
  }

  // ---- create_access_token scope-driven claims ---------------------------

  #[tokio::test]
  async fn create_access_token_includes_profile_email_and_picture() {
    use entity::user_avatar;
    use sea_orm::{ActiveValue::Set, EntityTrait};

    let c = ctx().await;
    // give the user an avatar so the `image` scope yields a picture claim
    user_avatar::Entity::insert(user_avatar::ActiveModel {
      user_id: Set(c.user),
      data: Set(vec![1, 2, 3]),
    })
    .exec(&c.db.0)
    .await
    .unwrap();

    let claims = RefreshTokenClaims {
      exp: get_timestamp_10_min(),
      sub: c.user,
      iss: c.config.issuer.to_string(),
      aud: c.client_id,
      scope: Scope::from(vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
        "image".to_string(),
      ]),
      nonce: None,
    };

    let token = create_access_token(&c.db, &c.jwt, &claims, &c.config, c.client_id)
      .await
      .unwrap();
    let decoded: OAuthClaims = c.jwt.validate_token(&token).unwrap();

    assert_eq!(decoded.name.as_deref(), Some("user"));
    assert_eq!(decoded.email.as_deref(), Some("user@x.com"));
    assert!(decoded.picture.is_some());
    let _ = decoded.scope; // ensure full struct decoded
  }

  #[tokio::test]
  async fn create_access_token_omits_optional_claims_without_scopes() {
    let c = ctx().await;
    let claims = RefreshTokenClaims {
      exp: get_timestamp_10_min(),
      sub: c.user,
      iss: c.config.issuer.to_string(),
      aud: c.client_id,
      scope: Scope::from(vec!["openid".to_string()]),
      nonce: None,
    };
    let token = create_access_token(&c.db, &c.jwt, &claims, &c.config, c.client_id)
      .await
      .unwrap();
    let decoded: OAuthClaims = c.jwt.validate_token(&token).unwrap();
    assert!(decoded.name.is_none());
    assert!(decoded.email.is_none());
    assert!(decoded.picture.is_none());
  }

  // ---- refresh_token -----------------------------------------------------

  fn refresh_claims(aud: Uuid, sub: Uuid, issuer: String) -> RefreshTokenClaims {
    RefreshTokenClaims {
      exp: get_timestamp_10_min(),
      sub,
      iss: issuer,
      aud,
      scope: Scope::from(vec!["openid".to_string()]),
      nonce: None,
    }
  }

  #[tokio::test]
  async fn refresh_token_success() {
    let c = ctx().await;
    let claims = refresh_claims(c.client_id, c.user, c.config.issuer.to_string());
    let rt = c.jwt.create_generic_token(&claims).unwrap();
    let body = TokenRefreshReq { refresh_token: rt };

    let axum::Json(res) = refresh_token(c.jwt, c.db, c.config, body, c.client_id)
      .await
      .unwrap();
    assert!(!res.access_token.is_empty());
    assert!(res.id_token.is_some());
  }

  #[tokio::test]
  async fn refresh_token_invalid_token_is_invalid_grant() {
    let c = ctx().await;
    let body = TokenRefreshReq {
      refresh_token: "garbage".into(),
    };
    let err = refresh_token(c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_grant");
  }

  #[tokio::test]
  async fn refresh_token_aud_mismatch_is_invalid_client() {
    let c = ctx().await;
    // token minted for a different audience
    let claims = refresh_claims(Uuid::new_v4(), c.user, c.config.issuer.to_string());
    let rt = c.jwt.create_generic_token(&claims).unwrap();
    let body = TokenRefreshReq { refresh_token: rt };
    let err = refresh_token(c.jwt, c.db, c.config, body, c.client_id)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "invalid_client");
  }

  // ---- token dispatcher --------------------------------------------------

  #[tokio::test]
  async fn token_dispatches_issue_refresh_and_rejects_unknown_grant() {
    // issue path
    let c = ctx().await;
    let code = Uuid::new_v4();
    c.state.auth_codes.insert(
      code,
      (Instant::now(), code_req(c.client_id, c.user, &["openid"])),
    );
    let auth = ClientAuth {
      client_id: c.client_id,
      body: TokenReq {
        grant_type: "authorization_code".into(),
        code: Some(code),
        redirect_uri: None,
        client_id: None,
        client_secret: None,
        refresh_token: None,
        code_verifier: None,
      },
    };
    assert!(token(c.state, c.jwt, c.db, c.config, auth).await.is_ok());

    // unsupported grant type
    let c = ctx().await;
    let auth = ClientAuth {
      client_id: c.client_id,
      body: TokenReq {
        grant_type: "password".into(),
        code: None,
        redirect_uri: None,
        client_id: None,
        client_secret: None,
        refresh_token: None,
        code_verifier: None,
      },
    };
    let err = token(c.state, c.jwt, c.db, c.config, auth)
      .await
      .map(|_| ())
      .unwrap_err();
    assert_eq!(err_code(err), "unsupported_grant_type");
  }

  // ---- revoke ------------------------------------------------------------

  #[tokio::test]
  async fn revoke_from_body_token_succeeds() {
    let c = ctx().await;
    let claims = OAuthClaims {
      sub: c.user,
      exp: get_timestamp_10_min(),
      iss: c.config.issuer.to_string(),
      aud: c.client_id,
      iat: 0,
      auth_time: 0,
      nonce: None,
      scope: Scope::from(vec!["openid".to_string()]),
      email: None,
      name: None,
      preferred_username: None,
      picture: None,
      groups: vec![],
      rest: HashMap::new(),
    };
    let tok = c.jwt.create_generic_token(&claims).unwrap();

    revoke(
      Query(RevokeReqOption { token: None }),
      c.db,
      c.jwt,
      JwtInvalidState::default(),
      Form(RevokeReqOption { token: Some(tok) }),
    )
    .await
    .unwrap();
  }

  #[tokio::test]
  async fn revoke_without_token_anywhere_errors() {
    let c = ctx().await;
    let res = revoke(
      Query(RevokeReqOption { token: None }),
      c.db,
      c.jwt,
      JwtInvalidState::default(),
      Form(RevokeReqOption { token: None }),
    )
    .await;
    assert!(res.is_err());
  }
}
