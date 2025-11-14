use serde::{Deserialize, Serialize};
pub mod accounts;
pub mod login;
pub mod rooms;
pub mod settings;

pub use accounts::*;
pub use login::*;
pub use rooms::*;
pub use settings::*;

use crate::service::{LkService, UiCmd};
use tokio::sync::mpsc::UnboundedSender;
use verdant::services::{VerdantCmd, VerdantService, VerdantUiCmd};

#[derive(Serialize, Deserialize)]
pub enum AppState {
    Room(RoomState),
    Login(LoginState),
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
pub enum ActivePage {
    Login,
    Room,
    Account,
    Settings,
    Discover,
}

pub struct AppPage {
    login: LoginPage,
    room: GridRoom,
    account: AccountPage,
    settings: SettingsPage,
    active: ActivePage,
    pub(crate) service: VerdantService,
}

impl AppPage {
    pub fn new(
        runtime: &tokio::runtime::Runtime,
        cc: &eframe::CreationContext<'_>,
        service: VerdantService,
    ) -> Self {
        let login = LoginPage::new(runtime, cc, service.tx().clone(), "http://localhost");
        let settings = SettingsPage::new(runtime, cc);
        let room = GridRoom::new(runtime, cc, GeneralSettings::default());
        let account = AccountPage::new(runtime, cc);
        let active = ActivePage::Login;
        Self {
            login,
            settings,
            account,
            room,
            active,
            service,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(event) = self.service.try_recv() {
            self.event(event);
        }

        match self.active {
            ActivePage::Room => self.room.update(ctx, frame),
            ActivePage::Login => self.login.update(ctx, frame),
            _ => unimplemented!(),
        }
    }

    pub fn room_event(&mut self, cmd: UiCmd) {
        self.room.event(cmd)
    }

    pub fn event(&mut self, cmd: VerdantUiCmd) {
        match cmd {
            VerdantUiCmd::LkToken(url, response) => {
                self.room.initialize(&url, &response);
                self.active = ActivePage::Room;
            }
            _ => match self.active {
                ActivePage::Login => self.login.event(cmd),
                _ => unimplemented!(),
            },
        }
    }

    pub fn state(&self) -> AppState {
        match self.active {
            ActivePage::Room => AppState::Room(self.room.state().clone()),
            ActivePage::Login => AppState::Login(self.login.state().clone()),
            _ => unimplemented!(),
        }
    }
}
