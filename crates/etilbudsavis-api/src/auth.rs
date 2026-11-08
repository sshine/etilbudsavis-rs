//! Account creation, email OTP login, and profile updates.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{AuthSession, AuthState, OtpStart, Person, PersonUpdate};

impl Client {
    /// The `auth` data key: the current person, or no person for an empty session.
    pub async fn auth_state(&self) -> Result<AuthState> {
        self.data("auth", Value::Null).await
    }

    /// The current person, when signed in.
    pub async fn me(&self) -> Result<Option<Person>> {
        Ok(self.auth_state().await?.person)
    }

    /// Creates an anonymous account and returns its token.
    ///
    /// This is what the app does before the first shopping list or favourite, so
    /// the caller does not have to sign up to use those features.
    pub async fn create_anonymous(&self) -> Result<AuthSession> {
        self.call_api("auth-create-person", json!({ "apiKey": self.api_key() }))
            .await
    }

    /// Starts an email OTP login. The service mails a one-time code.
    pub async fn otp_initialize(&self, email: &str) -> Result<OtpStart> {
        self.call_api(
            "auth-otp-initialize",
            json!({ "apiKey": self.api_key(), "email": email }),
        )
        .await
    }

    /// Exchanges the OTP id and mailed code for an account token.
    pub async fn otp_finalize(&self, otp_id: &str, otp: &str) -> Result<AuthSession> {
        self.call_api(
            "auth-otp-finalize",
            json!({ "apiKey": self.api_key(), "otpId": otp_id, "otp": otp }),
        )
        .await
    }

    /// Updates the signed-in person's country, consent, or notification kinds.
    pub async fn update_person(&self, update: &PersonUpdate) -> Result<Value> {
        let mut body = serde_json::to_value(update)?;
        if let Value::Object(map) = &mut body {
            map.insert("apiKey".to_owned(), json!(self.api_key()));
        }
        self.call_api_value("auth-update-person", body).await
    }

    /// Ends the current session server-side.
    pub async fn terminate(&self) -> Result<()> {
        self.call_api_unit("auth-terminate", Value::Null).await
    }
}
