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
use verdant::services::{VerdantCmd, VerdantService, VerdantUiCmd};
use tokio::sync::mpsc::UnboundedSender;

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
}

pub struct AppPage {
    login: LoginPage,
    room: GridRoom,
    account: AccountPage,
    settings: SettingsPage,
    active: ActivePage,
    service: VerdantService,
}

impl AppPage {
    pub fn new(
        runtime: &tokio::runtime::Runtime,
        cc: &eframe::CreationContext<'_>,
        room_settings: RoomSettings,
        service: VerdantService,
    ) -> Self {
        let url = match service.discoveries().get(0) {
            Some(discovery) => discovery.urls().get(0).unwrap().to_string(),
            None => "".to_string(),
        };
        println!("found url: {}", url);
        let login = LoginPage::new(runtime, cc, service.tx().clone(), &url);
        let settings = SettingsPage::new(runtime, cc);
        let room = GridRoom::new(runtime, cc, room_settings);
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
            VerdantUiCmd::LkToken(response) => {
                self.room.initialize(response);
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
