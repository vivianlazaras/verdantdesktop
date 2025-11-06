#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct AccountState {
    firstname: String,
    lastname: String,
    username: String,
    email: String,
}

/*#[derive(serde::Serialize, serde::Deserialize)]
pub enum AccountState {
    Creating(AccountCreationState),
}*/

pub struct AccountPage {
    state: AccountState,
}

impl AccountPage {
    pub fn new(runtime: &tokio::runtime::Runtime, cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AccountState::default(),
        }
    }
}
