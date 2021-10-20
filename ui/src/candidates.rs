use std::borrow::Cow;

use eframe::egui::*;
use image::GenericImageView;

use api::model::*;
use api::remote::*;

const PADDING: f32 = 8.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);

#[derive(PartialEq)]
pub struct Logo {
    pub size: (usize, usize),
    pub pixels: Vec<Color32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    name: String,
    default_version: String,
    url: String,
    description: String,
    installation_instruction: String,
    versions: Vec<String>,
}

impl Candidate {
    fn from_model(model: &CandidateModel) -> Candidate {
        Candidate {
            name: model.name().clone(),
            default_version: model.default_version().clone(),
            url: model.homepage().clone(),
            description: model.description().clone(),
            installation_instruction: format!("$ sdk install {}", model.binary_name()),
            versions: model.versions().iter().map(|v| v.to_string()).collect(),
        }
    }
    fn to_model(&self) -> CandidateModel {
        CandidateModel::new(
            self.name.clone(),
            self.installation_instruction
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
    logo: Logo,
    candidates: Vec<Candidate>,
    selected_candidate: Option<Candidate>,
    candidate_search_dialog: bool,
    candidate_search_term: String,
}

impl Default for Candidates {
    fn default() -> Self {
        let image = image::load_from_memory(include_bytes!("../assets/logo.png")).unwrap();
        let size = (image.width() as usize, image.height() as usize);
        let pixels: Vec<Color32> = image
            .to_rgba8()
            .into_vec()
            .chunks(4)
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        Self {
            app_name: "sdkman-ui",
            app_heading: "sdkman candidates",
            logo: Logo { size, pixels },
            candidates: Vec::new(),
            selected_candidate: None,
            candidate_search_dialog: false,
            candidate_search_term: String::default(),
        }
    }
}

impl Candidates {
    pub fn new(models: &Vec<CandidateModel>) -> Candidates {
        let mut app = Candidates::default();
        app.candidates = models
            .iter()
            .map(|model| Candidate::from_model(model))
            .collect();
        app
    }

    pub fn app_name(&self) -> &str {
        self.app_name
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

    pub(crate) fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        let Self {
            app_name: _,
            app_heading,
            logo,
            candidates,
            selected_candidate,
            candidate_search_dialog,
            candidate_search_term: _,
        } = self;
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    let texture_id = frame
                        .tex_allocator()
                        .alloc_srgba_premultiplied(logo.size, &logo.pixels);
                    if ui
                        .add(Image::new(texture_id, [56., 56.]).sense(Sense::click()))
                        .on_hover_ui(|ui| {
                            ui.ctx().output().cursor_icon = CursorIcon::PointingHand;
                            show_tooltip_text(ui.ctx(), Id::new("sdkman-logo"), "Go to sdkman.io");
                        })
                        .clicked()
                    {
                        let modifiers = ui.ctx().input().modifiers;
                        ui.ctx().output().open_url = Some(output::OpenUrl {
                            url: "https://sdkman.io/".to_owned(),
                            new_tab: modifiers.any(),
                        });
                    }
                });
                // Candidates
                ui.vertical_centered(|ui| {
                    ui.heading(*app_heading);
                });
                // controls
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.add_space(10.);
                    // Close button
                    if ui
                        .add(Button::new("❌").text_style(TextStyle::Body))
                        .on_hover_text("Close")
                        .clicked()
                    {
                        frame.quit();
                    }
                    // Refresh button
                    if ui
                        .add(Button::new("🔄").text_style(TextStyle::Body))
                        .on_hover_text("Refresh")
                        .clicked()
                    {
                        match fetch_remote_candidates() {
                            Ok(models) => {
                                let cands: Vec<Candidate> = models
                                    .iter()
                                    .map(|model| Candidate::from_model(model))
                                    .collect();
                                *candidates = cands;
                                *selected_candidate = None;
                            }
                            Err(e) => {
                                *selected_candidate = None;
                                tracing::error!(
                                    "Refreshing the list of candidates failed with:\n{}",
                                    e
                                )
                            }
                        }
                    }
                    // Search button
                    if ui
                        .add(Button::new("🔎").text_style(TextStyle::Body))
                        .on_hover_text("Search")
                        .clicked()
                    {
                        *candidate_search_dialog = true;
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn render_candidates(&mut self, ctx: &CtxRef, ui: &mut Ui) {
        let Self {
            app_name: _,
            app_heading: _,
            logo: _,
            candidates,
            selected_candidate,
            candidate_search_dialog,
            candidate_search_term,
        } = self;

        if *candidate_search_dialog {
            Window::new("Search").show(ctx, |ui| {
                ui.add_space(PADDING);
                ui.horizontal(|ui| {
                    ui.label("Candidate:");
                    ui.with_layout(Layout::left_to_right(), |ui| {
                        let text_input = ui.text_edit_singleline(candidate_search_term);
                        if text_input.lost_focus() && ui.input().key_pressed(Key::Enter) {
                            match candidates.into_iter().find(|candidate| {
                                candidate.name == *candidate_search_term
                                    || candidate
                                        .installation_instruction
                                        .ends_with(candidate_search_term.as_str())
                            }) {
                                None => {}
                                Some(found) => {
                                    match fetch_candidate_versions(&mut found.to_model()) {
                                        Ok(candidate_with_versions) => {
                                            *selected_candidate = Some(Candidate::from_model(
                                                candidate_with_versions,
                                            ));
                                        }
                                        Err(e) => {
                                            *selected_candidate = None;
                                            let msg = format!(
                                                "Loading all versions for candidate '{}' failed",
                                                candidate_search_term
                                            );
                                            tracing::error!("{}:\n{}", msg, e)
                                        }
                                    }
                                    *candidate_search_dialog = false;
                                    *candidate_search_term = String::default();
                                }
                            }
                        }
                    });
                });
                ui.add_space(PADDING);
            });
        }

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
                        match fetch_candidate_versions(&mut candidate.to_model()) {
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
                                tracing::error!("{}:\n{}", msg, e)
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
                let installation = Label::new(&candidate.installation_instruction)
                    .text_style(eframe::egui::TextStyle::Body);
                ui.add(installation);
            });

            ui.add_space(PADDING);
            ui.add(Separator::default());

            if selected_candidate.is_some() {
                ui.add_space(PADDING);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(), |ui| {
                        ui.add_space(PADDING);
                        ui.add(
                            Label::new(format!(
                                "Available {} versions",
                                selected_candidate.as_ref().unwrap().name
                            ))
                            .wrap(true)
                            .text_style(eframe::egui::TextStyle::Body),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ui.add_space(PADDING);
                        ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                            let close_btn_label = Label::new("❌")
                                .wrap(true)
                                .text_style(eframe::egui::TextStyle::Body)
                                .sense(Sense::click());
                            if ui
                                .add(close_btn_label)
                                .on_hover_ui(|ui| {
                                    show_tooltip_text(ui.ctx(), Id::new(&candidate.name), "Close");
                                })
                                .clicked()
                            {
                                *selected_candidate = None;
                            }
                        });
                    });
                });
                // render all available versions
                ui.add_space(2. * PADDING);
                let available_versions_text = selected_candidate
                    .as_ref()
                    .map(|c| c.versions.join("\n"))
                    .unwrap_or_default();
                let available_versions =
                    Label::new(available_versions_text).text_style(eframe::egui::TextStyle::Body);
                ui.add(available_versions);
                ui.add_space(3. * PADDING);
            }
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
