pub enum BannedTokenError {
    AddFailure,
}

pub trait BannedTokenStore {
    fn add_token(&mut self, token: String) -> ();
    fn contains_token(&self, token: &str) -> bool;
}
