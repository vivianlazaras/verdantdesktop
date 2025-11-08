use keycast::discovery::{Beacon, Discovery, ServiceIdent, WaitFor}; // replace with your crate name

use verdant::services::*;
use crate::{
    pages::*,
    service::{AsyncCmd, LkService, UiCmd},
    
    video_grid::VideoGrid,
    video_renderer::VideoRenderer,
};
use egui::{CornerRadius, Stroke};
use livekit::{e2ee::EncryptionType, prelude::*, track::VideoQuality, SimulateScenario};
use std::collections::HashMap;

pub struct LkApp {
    async_runtime: tokio::runtime::Runtime,
    page: AppPage,
}

impl LkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let service =
            VerdantService::new(&async_runtime, true).expect("failed to create verdant service");

        let discovery = service.discoveries().get(0).unwrap();
        let room_settings = RoomSettings::from_discovery(discovery);
        println!("room settings: {:?}", room_settings);

        let page = AppPage::new(&async_runtime, cc, room_settings, service);

        Self {
            async_runtime,
            page,
        }
    }

    fn event(&mut self, cmd: UiCmd) {
        self.page.room_event(cmd)
    }
}

impl eframe::App for LkApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.page.state());
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // this is now handled in respective pages until VerdantService has proper event receiving
        //if let Some(event) = self.service.try_recv() {
        //    self.page.event(event);
        //}

        self.page.update(ctx, frame)
    }
}
