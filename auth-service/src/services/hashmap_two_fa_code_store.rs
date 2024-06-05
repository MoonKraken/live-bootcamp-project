use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(code) => Ok(code.clone()),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore}, email::Email};

    use super::HashmapTwoFACodeStore;

    #[tokio::test]
    async fn should_add_code() {
        let email = Email::parse("ken@cttm.io".to_string()).expect("email should be parsed");
        let mut store = HashmapTwoFACodeStore::default();
        let res = store.add_code(email, LoginAttemptId::default(), TwoFACode::default()).await;
        assert_eq!(res, Ok(()));
    }
}
