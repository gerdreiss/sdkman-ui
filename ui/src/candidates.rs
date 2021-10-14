use eframe::egui::*;
use std::borrow::Cow;

const PADDING: f32 = 8.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);

#[derive(PartialEq)]
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

#[derive(PartialEq)]
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
        let Self {
            app_name: _,
            app_heading: _,
            candidates,
            selected_candidate,
        } = self;

        // render candidates
        for curr in candidates {
            // check whether to display the selected candidate only
            let candidate = if selected_candidate.is_none()
                || curr.name == selected_candidate.as_ref().unwrap().name
            {
                curr
            } else {
                continue;
            };

            ui.add_space(PADDING);

            // render name, default version, and homepage URL
            ui.horizontal(|ui| {
                // render name and default version
                ui.with_layout(Layout::left_to_right(), |ui| {
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
                    // handle clicks on the name and default version
                    if added.clicked() {
                        match api::fetch_candidate_versions(&mut candidate.to_model()) {
                            Ok(candidate_with_versions) => {
                                *selected_candidate =
                                    Some(Candidate::from_model(candidate_with_versions));
                            }
                            Err(e) => {
                                *selected_candidate = None;
                                let msg = format!(
                                    "Loading all versions for candidate '{}' failed",
                                    candidate.name
                                );
                                println!("{}:\n{}", msg, e)
                            }
                        }
                    }
                });

                // render homepage URL
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.style_mut().visuals.hyperlink_color = CYAN;
                    ui.add(Hyperlink::new(&candidate.url).text(&candidate.url));
                });
            });

            // render description
            ui.add_space(PADDING);
            let description = Label::new(&candidate.description)
                .wrap(true)
                .text_style(eframe::egui::TextStyle::Body);
            ui.add(description);
            ui.add_space(PADDING);
            // render installation instruction
            ui.with_layout(Layout::right_to_left(), |ui| {
                let installation =
                    Label::new(&candidate.installation).text_style(eframe::egui::TextStyle::Body);
                ui.add(installation);
            });

            ui.add_space(PADDING);

            if selected_candidate.is_some() {
                ui.add_space(PADDING);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(), |ui| {
                        // render all available versions
                        ui.add_space(PADDING);
                        let available_versions =
                            Label::new(&selected_candidate.as_ref().unwrap().available_versions)
                                .wrap(true)
                                .text_style(eframe::egui::TextStyle::Body);
                        ui.add(available_versions);
                        ui.add_space(PADDING);
                    });
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ui.add_space(10.);
                        ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                            let _close_btn = ui
                                .add(
                                    Label::new("❌")
                                        .wrap(true)
                                        .text_style(eframe::egui::TextStyle::Body)
                                        .sense(Sense::click()),
                                )
                                .on_hover_ui(|ui| {
                                    show_tooltip_text(
                                        ui.ctx(),
                                        Id::new(&candidate.name),
                                        "Click to close all available versions",
                                    );
                                });
                            if _close_btn.clicked() {
                                *selected_candidate = None;
                            }
                        });
                    });
                });
            }
            ui.add(Separator::default());
        }

        ui.add_space(7. * PADDING);
    }

    pub fn render_footer(&self, ctx: &CtxRef) {
        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.);
                ui.add(Label::new("API: https://api.sdkman.io/2").monospace());
                ui.add(
                    Hyperlink::new("https://github.com/emilk/egui")
                        .text("Made with egui")
                        .text_style(TextStyle::Monospace),
                );
                ui.add(
                    Hyperlink::new("https://github.com/gerdreiss/sdkman-ui")
                        .text("Hosted on Github")
                        .text_style(TextStyle::Monospace),
                );
                ui.add_space(10.);
            })
        });
    }
}
