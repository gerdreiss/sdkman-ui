use eframe::egui::*;
use std::borrow::Cow;

const PADDING: f32 = 8.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);

pub struct Candidate {
    name: String,
    default_version: String,
    url: String,
    description: String,
    installation: String,
    available_versions: String,
}

impl Candidate {
    fn from_model(model: &api::CandidateModel) -> Candidate {
        Candidate {
            name: model.name().clone(),
            default_version: model.default_version().clone(),
            url: model.homepage().clone(),
            description: model.description().clone(),
            installation: format!("$ sdk install {}", model.binary_name()),
            available_versions: model.available_versions().unwrap_or(&String::new()).clone(),
        }
    }
    fn to_model(&self) -> api::CandidateModel {
        api::CandidateModel::new(
            self.name.clone(),
            self.installation
                .split_whitespace()
                .last()
                .unwrap_or(&self.name.to_lowercase())
                .to_owned(),
            self.description.clone(),
            self.url.clone(),
            self.default_version.clone(),
        )
    }
}

pub struct Candidates {
    app_name: &'static str,
    app_heading: &'static str,
    candidates: Vec<Candidate>,
    selected_candidate: Option<Candidate>,
}

impl Candidates {
    pub fn new(models: &Vec<api::CandidateModel>) -> Candidates {
        Candidates {
            app_name: "sdkman-ui",
            app_heading: "candidates",
            candidates: models
                .iter()
                .map(|model| Candidate::from_model(model))
                .collect(),
            selected_candidate: None,
        }
    }

    pub fn app_name(&self) -> &str {
        self.app_name
    }

    pub fn app_heading(&self) -> &str {
        self.app_heading
    }

    pub fn configure_fonts(&self, ctx: &CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            Cow::Borrowed(include_bytes!("../assets/MesloLGS_NF_Regular.ttf")),
        );
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Heading,
            (FontFamily::Proportional, 35.),
        );
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Body,
            (FontFamily::Proportional, 20.),
        );
        font_def
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());
        ctx.set_fonts(font_def);
    }

    pub(crate) fn render_top_panel(&self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📓").text_style(TextStyle::Heading));
                });
                // Candidates
                ui.vertical_centered(|ui| {
                    ui.heading(self.app_heading());
                });
                // controls
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.add_space(10.);
                    let _close_btn = ui.add(Button::new("❌").text_style(TextStyle::Body));
                    if _close_btn.clicked() {
                        frame.quit();
                    }
                    let _refresh_btn = ui.add(Button::new("🔄").text_style(TextStyle::Body));
                    let _theme_btn = ui.add(Button::new("🌙").text_style(TextStyle::Body));
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn render_candidates(&mut self, ui: &mut Ui) {
        for candidate in &self.candidates {
            ui.add_space(PADDING);
            self.render_name_defaultversion_homepage(ui, &candidate);
            ui.add_space(PADDING);
            self.render_description(ui, &candidate);
            ui.add_space(PADDING);
            self.render_installation_instruction(ui, &candidate);
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    fn render_name_defaultversion_homepage(&self, ui: &mut Ui, candidate: &Candidate) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::left_to_right(), |ui| {
                self.render_name_defaultversion(ui, candidate);
            });
            ui.with_layout(Layout::right_to_left(), |ui| {
                self.render_homepage(ui, candidate);
            });
        });
    }

    fn render_name_defaultversion(&self, ui: &mut Ui, candidate: &Candidate) {
        let btn_label = format!("{} {} ⤴", candidate.name, candidate.default_version);
        let title_btn = Button::new(btn_label)
            .text_style(TextStyle::Body)
            .text_color(WHITE);
        let added = ui.add(title_btn).on_hover_ui(|ui| {
            show_tooltip_text(
                ui.ctx(),
                Id::new(&candidate.name),
                "Click to display all available versions",
            );
        });
        if added.clicked() {
            match api::fetch_candidate_versions(&mut candidate.to_model()) {
                Ok(candidate_with_versions) => {
                    //*selected_candidate = Some(Candidate::from_model(candidate_with_versions));
                    let msg = format!(
                        "Displaying all versions for candidate '{}':\n",
                        candidate.name
                    );
                    //self.status_text = msg
                    println!("{}", msg);
                    println!(
                        "{}",
                        candidate_with_versions
                            .available_versions()
                            .unwrap_or(&String::new())
                    )
                }
                Err(e) => {
                    //*selected_candidate = None;
                    let msg = format!(
                        "Loading all versions for candidate '{}' failed",
                        candidate.name
                    );
                    //self.status_text = msg
                    println!("{}:\n{}", msg, e)
                }
            }
        }
    }

    fn render_homepage(&self, ui: &mut Ui, candidate: &Candidate) {
        ui.style_mut().visuals.hyperlink_color = CYAN;
        ui.add(Hyperlink::new(&candidate.url).text(&candidate.url));
    }

    fn render_description(&self, ui: &mut Ui, candidate: &Candidate) {
        let description = Label::new(&candidate.description)
            .wrap(true)
            .text_style(eframe::egui::TextStyle::Body);
        ui.add(description);
    }

    fn render_installation_instruction(&self, ui: &mut Ui, candidate: &Candidate) {
        ui.with_layout(Layout::right_to_left(), |ui| {
            let installation =
                Label::new(&candidate.installation).text_style(eframe::egui::TextStyle::Body);
            ui.add(installation);
        });
    }

    pub fn render_footer(&self, ctx: &CtxRef) {
        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.);
                ui.add(Label::new("API source: https://api.sdkman.io/2").monospace());
                ui.add(
                    Hyperlink::new("https://github.com/emilk/egui")
                        .text("Made with egui")
                        .text_style(TextStyle::Monospace),
                );
                ui.add(
                    Hyperlink::new("https://github.com/gerdreiss/sdkman-ui")
                        .text("gerdreiss/sdkman-ui")
                        .text_style(TextStyle::Monospace),
                );
                ui.add_space(10.);
            })
        });
    }
}
