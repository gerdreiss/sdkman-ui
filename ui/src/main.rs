use eframe::egui::CentralPanel;
use eframe::egui::ScrollArea;
use eframe::egui::Vec2;
use eframe::epi::App;
use eframe::run_native;
use eframe::NativeOptions;

use api::local::retrieve_local_installations;
use api::remote::fetch_remote_candidates;
use candidates::Candidates;

use std::env;

mod candidates;

impl App for Candidates {
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        self.render_top_panel(ctx, frame);
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::auto_sized().show(ui, |ui| {
                self.render_candidates(ctx, ui);
            });
            self.render_footer(ctx);
        });
    }

    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx);
    }

    fn name(&self) -> &str {
        self.app_name()
    }
}

fn main() {
    if cfg!(target_os = "windows") {
        println!("sdkman is not for windows!")
        // for this show a dialog
    } else if env::var("SDKMAN_DIR").is_err() {
        println!("sdkman is not installed!")
    } else {
        tracing_subscriber::fmt::init();

        match retrieve_local_installations() {
            Ok(installations) => {
                dbg!(installations);
            }
            Err(e) => {
                tracing::error!("Failed to start the application with:\n{}", e);
            }
        }

        match fetch_remote_candidates() {
            Ok(candidates) => {
                let app = Candidates::new(&candidates);
                let mut win_option = NativeOptions::default();
                win_option.initial_window_size = Some(Vec2::new(1024., 960.));
                run_native(Box::new(app), win_option);
            }
            Err(e) => {
                tracing::error!("Failed to start the application with:\n{}", e)
            }
        }
    }
}
