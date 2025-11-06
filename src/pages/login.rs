use crate::services::{VerdantCmd, VerdantService, VerdantUiCmd};
use protocol::auth::LoginResult;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LoginState {
    url: String,
    username: String,
    password: String,
    login_message: Option<String>,
}

impl LoginState {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            username: "".to_string(),
            password: "".to_string(),
            login_message: None,
        }
    }
}

pub struct LoginPage {
    state: LoginState,
    cmd_tx: UnboundedSender<VerdantCmd>,
}
impl LoginPage {
    pub fn new(
        runtime: &tokio::runtime::Runtime,
        cc: &eframe::CreationContext<'_>,
        cmd_tx: UnboundedSender<VerdantCmd>,
        url: &str,
    ) -> Self {
        let state = LoginState::new(url);
        Self { state, cmd_tx }
    }

    pub fn state(&self) -> &LoginState {
        &self.state
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui);
        });

        ctx.request_repaint();
    }

    pub fn central_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Login");

            ui.add_space(10.0);

            ui.label("Verdant Url:");
            ui.text_edit_singleline(&mut self.state.url);

            ui.add_space(5.0);

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.state.username);

            ui.add_space(5.0);

            ui.label("Password:");
            ui.add(
                egui::TextEdit::singleline(&mut self.state.password)
                    .password(true)
                    .hint_text("Enter password"),
            );

            ui.add_space(10.0);

            if ui.button("Submit").clicked() {
                VerdantService::login(
                    &self.cmd_tx,
                    &self.state.url,
                    &self.state.username,
                    &self.state.password,
                );
                self.state.login_message = Some(String::from("logging in..."));
            }

            if let Some(msg) = &self.state.login_message {
                ui.add_space(10.0);
                ui.label(msg);
            }
        });
    }

    pub fn event(&mut self, event: VerdantUiCmd) {
        match event {
            VerdantUiCmd::LoginResult(result) => {
                self.state.login_message = Some(match result {
                    LoginResult::Unauthorized => "incorrect username or password".to_string(),
                    LoginResult::UnknownServer(server) => {
                        format!("couldn't find request server: {}", server)
                    }
                    _ => "login successful".to_string(),
                })
            }
            _ => println!("unhandled event"),
        }
    }
}
