use std::env;
use std::thread;

use eframe::egui::CentralPanel;
use eframe::egui::ScrollArea;
use eframe::egui::Vec2;
use eframe::epi::App;
use eframe::run_native;
use eframe::NativeOptions;

use api::local::retrieve_local_candidates;
use api::remote::fetch_remote_candidates;
use candidates::SdkmanApp;

mod candidates;

impl App for SdkmanApp {
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        self.render_top_panel(ctx, frame);
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
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
        let remote_candidates_handle = thread::spawn(|| match fetch_remote_candidates() {
            Ok(candidates) => candidates,
            Err(e) => {
                println!("Failed to retrieve remote candidates: {}", e);
                Vec::new()
            }
        });
        let local_candidates_handle = thread::spawn(|| match retrieve_local_candidates() {
            Ok(candidates) => candidates,
            Err(e) => {
                println!("Failed to retrieve local candidates: {}", e);
                Vec::new()
            }
        });

        match (
            remote_candidates_handle.join(),
            local_candidates_handle.join(),
        ) {
            (Ok(remote_candidates), Ok(local_candidates)) => {
                let app = SdkmanApp::new(&remote_candidates, &local_candidates);
                let win_option = NativeOptions {
                    initial_window_size: Some(Vec2::new(1024., 960.)),
                    ..Default::default()
                };
                run_native(Box::new(app), win_option);
            }
            (Err(_), _) => {
                println!("Remote candidates retrieval thread failed.");
            }
            (_, Err(_)) => {
                println!("Local candidates retrieval thread failed.");
            }
        }
    }
}
