#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryState {
    discoveries: HashMap<String, Discovery>,
}

impl DiscoveryState {
    pub fn new() -> Self {
        Self {
            discoveries: HashMap::new(),
        }
    }
}

pub struct DiscoveryPage<'a> {
    state: DiscoveryState,
    cmd_tx: UnboundedSender<VerdantCmd>,
}

impl<'a> DiscoveryPage<'a> {
    /// used to construct a login page for this specific server instance.
    pub fn create_login_page(&self) -> LoginPage {
        unimplemented!();
    }

    pub fn new(cmd_tx: UnboundedSender<VerdantCmd>) -> Self {
        Self {
            state: DiscoveryState::new(),
            cmd_tx
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui);
        });

        ctx.request_repaint();
    }

    pub fn event(&mut self, event: VerdantUiCmd) {
        match event {
            VerdantUiCmd::ServerDiscovered(discovery) => {
                
                self.state.discoveries.insert(discovery);
            },
            _ => println!("unhandled event")
        }
    }

    pub fn central_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| )
    }
}