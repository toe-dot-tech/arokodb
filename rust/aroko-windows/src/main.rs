#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("ArokoDB Windows is a Windows-only desktop application.");
}

#[cfg(target_os = "windows")]
mod windows_app {
    use aroko_core::index::{DistanceMetric, IndexParams, VectorIndex};
    use eframe::egui::{
        self, Align, Button, Color32, ComboBox, Frame, Layout, RichText, ScrollArea, Stroke,
        TextEdit, Ui, Vec2,
    };
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    const BACKGROUND: Color32 = Color32::from_rgb(12, 18, 31);
    const SIDEBAR: Color32 = Color32::from_rgb(16, 25, 42);
    const PANEL: Color32 = Color32::from_rgb(22, 33, 53);
    const PANEL_LIGHT: Color32 = Color32::from_rgb(29, 43, 66);
    const BORDER: Color32 = Color32::from_rgb(42, 57, 80);
    const TEXT: Color32 = Color32::from_rgb(238, 243, 250);
    const MUTED: Color32 = Color32::from_rgb(151, 167, 190);
    const TEAL: Color32 = Color32::from_rgb(82, 222, 190);
    const BLUE: Color32 = Color32::from_rgb(118, 166, 255);
    const PURPLE: Color32 = Color32::from_rgb(184, 147, 255);
    const ORANGE: Color32 = Color32::from_rgb(255, 190, 112);
    const RED: Color32 = Color32::from_rgb(255, 132, 144);

    #[derive(Clone, Copy, PartialEq)]
    enum Page {
        Studio,
        CaseStudies,
        Case(usize),
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Activity {
        Explorer,
        Industries,
    }

    #[derive(Clone, Copy)]
    struct DemoVector {
        label: &'static str,
        values: [f32; 3],
    }

    struct CaseStudy {
        icon: &'static str,
        name: &'static str,
        summary: &'static str,
        scenario: &'static str,
        what_is_saved: &'static str,
        simple_search: &'static str,
        useful_because: &'static str,
        caution: &'static str,
        examples: [DemoVector; 3],
        query: [f32; 3],
    }

    const CASES: [CaseStudy; 8] = [
        CaseStudy {
            icon: "01",
            name: "Banking & finance",
            summary: "Help people find the right payment, policy, or money guide.",
            scenario: "A customer remembers what they need help with, but not the exact title of the article or transaction.",
            what_is_saved: "Descriptions of help articles, payment notes, or spending categories.",
            simple_search: "“Show me the guide about a card payment I do not recognise.”",
            useful_because: "A meaning-based search can bring related information together even when the wording is different.",
            caution: "An example of finding information; not financial advice, fraud detection, or a live banking integration.",
            examples: [
                DemoVector { label: "Card payment help", values: [0.82, 0.18, 0.06] },
                DemoVector { label: "Monthly budget guide", values: [0.12, 0.86, 0.14] },
                DemoVector { label: "Travel money tips", values: [0.17, 0.18, 0.84] },
            ],
            query: [0.78, 0.22, 0.09],
        },
        CaseStudy {
            icon: "02",
            name: "Cryptocurrency",
            summary: "Bring useful wallet, token, and blockchain explainers closer to a question.",
            scenario: "Someone saves guides and project notes, then later searches by what they remember rather than an exact phrase.",
            what_is_saved: "Descriptions of wallet guides, token explainers, and project notes.",
            simple_search: "“Find my beginner guide to keeping a wallet safe.”",
            useful_because: "A local vector index can organize related saved information on the device.",
            caution: "Illustrative search only; not a wallet, exchange, blockchain connection, or investment recommendation.",
            examples: [
                DemoVector { label: "Wallet safety guide", values: [0.84, 0.14, 0.12] },
                DemoVector { label: "What is a token?", values: [0.14, 0.83, 0.16] },
                DemoVector { label: "Blockchain glossary", values: [0.12, 0.17, 0.86] },
            ],
            query: [0.79, 0.19, 0.13],
        },
        CaseStudy {
            icon: "03",
            name: "Healthcare",
            summary: "Make saved health information easier to find by topic.",
            scenario: "A person has saved clinic instructions or general health reading and wants to find the right note again.",
            what_is_saved: "Descriptions of appointment notes, clinic instructions, or general health articles.",
            simple_search: "“Find the note about getting ready for my appointment.”",
            useful_because: "A search can bring similar saved material together without uploading it to a search service.",
            caution: "A product idea only; not diagnosis, medical advice, a patient-record system, or a deployed health service.",
            examples: [
                DemoVector { label: "Appointment preparation", values: [0.86, 0.11, 0.08] },
                DemoVector { label: "Clinic opening hours", values: [0.13, 0.84, 0.14] },
                DemoVector { label: "General wellbeing article", values: [0.11, 0.16, 0.87] },
            ],
            query: [0.81, 0.18, 0.1],
        },
        CaseStudy {
            icon: "04",
            name: "News & digital media",
            summary: "Rediscover saved stories, clips, and posts by what they were about.",
            scenario: "A reader saved a story or short video, but remembers the subject rather than the headline.",
            what_is_saved: "Descriptions of articles, video captions, podcast notes, or bookmarks.",
            simple_search: "“Show me the clip about the city football final.”",
            useful_because: "Related stories and media can be searched together from a device's local collection.",
            caution: "Demonstration only; no live news feed, media library, recommendation system, or publisher integration.",
            examples: [
                DemoVector { label: "Football final highlights", values: [0.83, 0.17, 0.1] },
                DemoVector { label: "City transport update", values: [0.12, 0.85, 0.12] },
                DemoVector { label: "New music interview", values: [0.15, 0.13, 0.86] },
            ],
            query: [0.79, 0.21, 0.11],
        },
        CaseStudy {
            icon: "05",
            name: "Artificial intelligence",
            summary: "Let an AI app look up related saved notes on the same device.",
            scenario: "A private assistant needs a relevant saved note before it answers a question.",
            what_is_saved: "Descriptions of user-approved notes, documents, or reference material.",
            simple_search: "“Find my notes about preparing a project update.”",
            useful_because: "A local vector search can provide related material to an on-device assistant without requiring cloud search.",
            caution: "A possible building block, not an AI model, chat assistant, or claim that private data is automatically protected.",
            examples: [
                DemoVector { label: "Project update notes", values: [0.85, 0.15, 0.08] },
                DemoVector { label: "Recipe collection", values: [0.11, 0.86, 0.16] },
                DemoVector { label: "Weekend trip plan", values: [0.14, 0.14, 0.85] },
            ],
            query: [0.8, 0.2, 0.12],
        },
        CaseStudy {
            icon: "06",
            name: "Social & community",
            summary: "Reconnect people with saved groups, posts, or local events.",
            scenario: "A community member wants to find an event or discussion they remember from earlier.",
            what_is_saved: "Descriptions of community posts, group topics, or event details.",
            simple_search: "“Find that neighbourhood gardening meet-up.”",
            useful_because: "Related posts can be found by topic instead of scrolling through a long feed.",
            caution: "Illustrative concept only; it does not access social accounts, profiles, or community platforms.",
            examples: [
                DemoVector { label: "Neighbourhood garden meet-up", values: [0.84, 0.16, 0.08] },
                DemoVector { label: "Book club discussion", values: [0.12, 0.83, 0.17] },
                DemoVector { label: "Weekend market event", values: [0.16, 0.13, 0.86] },
            ],
            query: [0.79, 0.2, 0.1],
        },
        CaseStudy {
            icon: "07",
            name: "Fitness & wellness",
            summary: "Find a saved workout or wellness note by the goal it supports.",
            scenario: "Someone has saved a few routines and wants to bring back one that fits their plan.",
            what_is_saved: "Descriptions of personal routines, class notes, or general wellness reading.",
            simple_search: "“Show me the short strength routine I saved.”",
            useful_because: "A local search may make a growing collection of saved routines easier to browse.",
            caution: "Example only; not a coach, medical recommendation, activity tracker, or fitness-results claim.",
            examples: [
                DemoVector { label: "Short strength routine", values: [0.87, 0.12, 0.07] },
                DemoVector { label: "Evening stretch notes", values: [0.13, 0.85, 0.13] },
                DemoVector { label: "Weekend walking route", values: [0.11, 0.15, 0.88] },
            ],
            query: [0.82, 0.18, 0.1],
        },
        CaseStudy {
            icon: "08",
            name: "Education",
            summary: "Help learners find a useful lesson or note by the idea they recall.",
            scenario: "A student remembers the topic from class, but not which file or lesson contains it.",
            what_is_saved: "Descriptions of class notes, practice questions, or learning resources.",
            simple_search: "“Find my notes that explain fractions.”",
            useful_because: "Similar learning material can be found across a collection without remembering file names.",
            caution: "A learning-tool idea only; it does not grade work, replace a teacher, or provide a full learning platform.",
            examples: [
                DemoVector { label: "Fractions practice notes", values: [0.86, 0.13, 0.07] },
                DemoVector { label: "World history lesson", values: [0.13, 0.85, 0.13] },
                DemoVector { label: "Language revision cards", values: [0.12, 0.16, 0.87] },
            ],
            query: [0.81, 0.19, 0.1],
        },
    ];

    struct SearchHit {
        id: usize,
        distance: f32,
        label: Option<String>,
    }

    struct ArokoApp {
        index: VectorIndex,
        labels: HashMap<usize, String>,
        dimension_input: String,
        id_input: String,
        vector_input: String,
        query_input: String,
        top_k_input: String,
        csv_path: String,
        metric: DistanceMetric,
        page: Page,
        activity: Activity,
        sidebar_open: bool,
        status: String,
        status_is_error: bool,
        results: Vec<SearchHit>,
    }

    impl ArokoApp {
        fn new(ctx: &eframe::CreationContext<'_>) -> Self {
            let mut visuals = egui::Visuals::dark();
            visuals.window_fill = BACKGROUND;
            visuals.panel_fill = BACKGROUND;
            visuals.extreme_bg_color = Color32::from_rgb(10, 16, 27);
            visuals.faint_bg_color = PANEL;
            visuals.selection.bg_fill = Color32::from_rgb(40, 91, 96);
            visuals.widgets.noninteractive.bg_fill = PANEL;
            visuals.widgets.noninteractive.fg_stroke.color = TEXT;
            visuals.widgets.inactive.bg_fill = PANEL_LIGHT;
            visuals.widgets.inactive.fg_stroke.color = TEXT;
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(46, 65, 92);
            visuals.widgets.hovered.fg_stroke.color = TEXT;
            visuals.widgets.active.bg_fill = Color32::from_rgb(42, 91, 96);
            visuals.widgets.active.fg_stroke.color = TEXT;
            ctx.egui_ctx.set_visuals(visuals);

            let mut style = (*ctx.egui_ctx.style()).clone();
            style.spacing.item_spacing = Vec2::new(10.0, 10.0);
            style.spacing.button_padding = Vec2::new(14.0, 9.0);
            ctx.egui_ctx.set_style(style);

            Self {
                index: VectorIndex::new(IndexParams { dim: 3 }),
                labels: HashMap::new(),
                dimension_input: "3".to_owned(),
                id_input: "0".to_owned(),
                vector_input: "0.1, 0.2, 0.3".to_owned(),
                query_input: "0.1, 0.2, 0.3".to_owned(),
                top_k_input: "3".to_owned(),
                csv_path: "vectors.csv".to_owned(),
                metric: DistanceMetric::SquaredEuclidean,
                page: Page::Studio,
                activity: Activity::Explorer,
                sidebar_open: true,
                status: "Ready when you are. Add a vector or explore an example.".to_owned(),
                status_is_error: false,
                results: Vec::new(),
            }
        }

        fn set_status(&mut self, message: impl Into<String>) {
            self.status = message.into();
            self.status_is_error = false;
        }

        fn set_error(&mut self, message: impl Into<String>) {
            self.status = message.into();
            self.status_is_error = true;
        }

        fn apply_dimension(&mut self) {
            let dimension = match self.dimension_input.trim().parse::<usize>() {
                Ok(dimension) if (1..=65_536).contains(&dimension) => dimension,
                _ => {
                    self.set_error("Choose a whole-number dimension from 1 to 65,536.");
                    return;
                }
            };
            if !self.index.is_empty() {
                self.set_error("Clear the saved vectors before changing their dimension.");
                return;
            }

            self.index = VectorIndex::new(IndexParams { dim: dimension });
            self.results.clear();
            self.vector_input = vec!["0"; dimension].join(", ");
            self.query_input = vec!["0"; dimension].join(", ");
            self.set_status(format!(
                "Dimension set to {dimension}. Each vector needs {dimension} numbers."
            ));
        }

        fn add_vector(&mut self) {
            let id = match self.id_input.trim().parse::<usize>() {
                Ok(id) => id,
                Err(_) => {
                    self.set_error("Vector ID must be a non-negative whole number.");
                    return;
                }
            };
            let vector = match parse_vector(&self.vector_input, self.index.params().dim) {
                Ok(vector) => vector,
                Err(error) => {
                    self.set_error(error);
                    return;
                }
            };

            match self.index.insert_with_id(id, vector) {
                Ok(()) => {
                    self.labels.remove(&id);
                    self.id_input = id
                        .checked_add(1)
                        .map_or_else(|| id.to_string(), |next| next.to_string());
                    self.vector_input = vec!["0"; self.index.params().dim].join(", ");
                    self.set_status(format!(
                        "Vector {id} added. Your collection now has {} vector(s).",
                        self.index.len()
                    ));
                }
                Err(error) => self.set_error(error),
            }
        }

        fn generate_sample_vector(&mut self) {
            let seed = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => duration
                    .as_secs()
                    .wrapping_mul(1_000_000_000)
                    .wrapping_add(u64::from(duration.subsec_nanos())),
                Err(error) => {
                    self.set_error(format!("Could not generate sample values: {error}"));
                    return;
                }
            };
            self.vector_input = generate_sample_vector(self.index.params().dim, seed);
            self.set_status(format!(
                "Generated {} sample values. These are test numbers, not AI embeddings.",
                self.index.params().dim
            ));
        }

        fn search(&mut self) {
            let query = match parse_vector(&self.query_input, self.index.params().dim) {
                Ok(query) => query,
                Err(error) => {
                    self.results.clear();
                    self.set_error(error);
                    return;
                }
            };
            let k = match self.top_k_input.trim().parse::<usize>() {
                Ok(k) if k > 0 => k,
                _ => {
                    self.results.clear();
                    self.set_error("Top K must be a positive whole number.");
                    return;
                }
            };

            self.results = self
                .index
                .search_with_metric(&query, k, self.metric)
                .into_iter()
                .map(|(id, distance)| SearchHit {
                    id,
                    distance,
                    label: self.labels.get(&id).cloned(),
                })
                .collect();
            if self.results.is_empty() {
                self.set_status("No vectors are saved yet. Add one or try an industry example.");
            } else {
                self.set_status(format!(
                    "Found {} nearby item(s), ordered from closest to furthest.",
                    self.results.len()
                ));
            }
        }

        fn clear_vectors(&mut self) {
            let dimension = self.index.params().dim;
            self.index = VectorIndex::new(IndexParams { dim: dimension });
            self.labels.clear();
            self.results.clear();
            self.set_status("Your in-memory vector collection has been cleared.");
        }

        fn load_case_example(&mut self, case_index: usize) {
            let Some(case) = CASES.get(case_index) else {
                self.set_error("That example could not be found.");
                return;
            };

            if self.index.is_empty() && self.index.params().dim != 3 {
                self.index = VectorIndex::new(IndexParams { dim: 3 });
                self.dimension_input = "3".to_owned();
                self.vector_input = "0, 0, 0".to_owned();
                self.query_input = "0, 0, 0".to_owned();
            } else if self.index.params().dim != 3 {
                self.set_error(
                    "This small example uses 3 numbers. Clear the current collection to switch.",
                );
                return;
            }

            let mut next_id = 1;
            for example in case.examples {
                while self.index.contains_id(next_id) {
                    next_id += 1;
                }
                match self.index.insert_with_id(next_id, example.values.to_vec()) {
                    Ok(()) => {
                        self.labels.insert(next_id, example.label.to_owned());
                        next_id += 1;
                    }
                    Err(error) => {
                        self.set_error(format!("Could not load the example: {error}"));
                        return;
                    }
                }
            }
            self.query_input = case
                .query
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            self.top_k_input = "3".to_owned();
            self.page = Page::Studio;
            self.search();
            self.set_status(format!(
                "{} example loaded. These are sample numbers, not real text embeddings.",
                case.name
            ));
        }

        fn import_csv(&mut self) {
            let contents = match fs::read_to_string(self.csv_path.trim()) {
                Ok(contents) => contents,
                Err(error) => {
                    self.set_error(format!("Could not open CSV: {error}"));
                    return;
                }
            };
            let records = match parse_csv(&contents, self.index.params().dim) {
                Ok(records) => records,
                Err(error) => {
                    self.set_error(format!("CSV import failed: {error}"));
                    return;
                }
            };
            if let Some((id, _)) = records.iter().find(|(id, _)| self.index.contains_id(*id)) {
                self.set_error(format!(
                    "Import stopped: vector ID {id} is already in this collection."
                ));
                return;
            }

            let count = records.len();
            for (id, vector) in records {
                if let Err(error) = self.index.insert_with_id(id, vector) {
                    self.set_error(format!("CSV import failed: {error}"));
                    return;
                }
                self.labels.remove(&id);
            }
            self.set_status(format!(
                "Imported {count} vector(s). The collection has {} in total.",
                self.index.len()
            ));
        }

        fn export_csv(&mut self) {
            let csv = format_csv(&self.index);
            match fs::write(self.csv_path.trim(), csv) {
                Ok(()) => self.set_status(format!(
                    "Saved {} vector(s) to {}.",
                    self.index.len(),
                    self.csv_path.trim()
                )),
                Err(error) => self.set_error(format!("Could not save CSV: {error}")),
            }
        }

        fn activity_bar(&mut self, ctx: &egui::Context) {
            egui::SidePanel::left("activity_bar")
                .exact_width(56.0)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(SIDEBAR)
                        .inner_margin(egui::Margin::symmetric(5, 10)),
                )
                .show(ctx, |ui| {
                    ui.add_space(3.0);
                    self.activity_button(ui, "▦", "Vector studio", Activity::Explorer);
                    ui.add_space(4.0);
                    self.activity_button(ui, "✳", "Industry ideas", Activity::Industries);
                    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                        ui.label(RichText::new("A").size(20.0).strong().color(TEAL))
                            .on_hover_text("ArokoDB");
                    });
                });
        }

        fn activity_button(&mut self, ui: &mut Ui, icon: &str, tooltip: &str, activity: Activity) {
            let selected = self.activity == activity && self.sidebar_open;
            let response = ui
                .add_sized(
                    [42.0, 44.0],
                    Button::new(RichText::new(icon).size(22.0).color(if selected {
                        TEAL
                    } else {
                        MUTED
                    }))
                    .fill(if selected { PANEL_LIGHT } else { SIDEBAR })
                    .stroke(Stroke::NONE),
                )
                .on_hover_text(tooltip);
            if selected {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        response.rect.left_top(),
                        Vec2::new(3.0, response.rect.height()),
                    ),
                    2.0,
                    TEAL,
                );
            }
            if response.clicked() {
                if self.activity == activity && self.sidebar_open {
                    self.sidebar_open = false;
                } else {
                    self.activity = activity;
                    self.sidebar_open = true;
                    self.page = match activity {
                        Activity::Explorer => Page::Studio,
                        Activity::Industries => Page::CaseStudies,
                    };
                }
            }
        }

        fn sidebar(&mut self, ctx: &egui::Context) {
            if !self.sidebar_open {
                return;
            }
            egui::SidePanel::left("navigation")
                .exact_width(238.0)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(SIDEBAR)
                        .inner_margin(egui::Margin::symmetric(15, 16)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [34.0, 34.0],
                            Button::new(RichText::new("A").size(20.0).strong().color(BACKGROUND))
                                .fill(TEAL),
                        );
                        ui.vertical(|ui| {
                            ui.label(RichText::new("ArokoDB").size(19.0).strong().color(TEXT));
                            ui.label(RichText::new("VECTOR WORKSPACE").size(9.0).color(MUTED));
                        });
                    });
                    ui.add_space(24.0);
                    match self.activity {
                        Activity::Explorer => {
                            ui.label(RichText::new("EXPLORER").size(10.0).strong().color(MUTED));
                            ui.add_space(8.0);
                            self.nav_button(ui, "▦", "Vector studio", Page::Studio);
                            self.nav_button(ui, "✳", "Industry ideas", Page::CaseStudies);
                            ui.add_space(18.0);
                            ui.label(
                                RichText::new("INDUSTRY EXAMPLES")
                                    .size(10.0)
                                    .strong()
                                    .color(MUTED),
                            );
                            ui.add_space(8.0);
                            for (index, case) in CASES.iter().enumerate() {
                                self.nav_button(ui, case.icon, case.name, Page::Case(index));
                            }
                        }
                        Activity::Industries => {
                            ui.label(
                                RichText::new("INDUSTRY IDEAS")
                                    .size(10.0)
                                    .strong()
                                    .color(MUTED),
                            );
                            ui.add_space(8.0);
                            self.nav_button(ui, "✳", "All industries", Page::CaseStudies);
                            for (index, case) in CASES.iter().enumerate() {
                                self.nav_button(ui, case.icon, case.name, Page::Case(index));
                            }
                        }
                    }
                    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                        ui.label(
                            RichText::new("Local demo · no account needed")
                                .size(11.0)
                                .color(MUTED),
                        );
                        ui.label(
                            RichText::new("ArokoDB  ·  early preview")
                                .size(11.0)
                                .color(MUTED),
                        );
                    });
                });
        }

        fn nav_button(&mut self, ui: &mut Ui, icon: &str, label: &str, target: Page) {
            let selected = self.page == target;
            let background = if selected { PANEL_LIGHT } else { SIDEBAR };
            let text_color = if selected { TEAL } else { TEXT };
            let response = ui.add_sized(
                [ui.available_width(), 38.0],
                Button::new(
                    RichText::new(format!("{icon}     {label}"))
                        .size(13.0)
                        .color(text_color),
                )
                .fill(background)
                .stroke(Stroke::NONE),
            );
            if response.clicked() {
                self.page = target;
                self.activity = if matches!(target, Page::Studio) {
                    Activity::Explorer
                } else {
                    Activity::Industries
                };
            }
        }

        fn studio_page(&mut self, ui: &mut Ui) {
            ui.label(
                RichText::new("YOUR PRIVATE SEARCH WORKSPACE")
                    .size(11.0)
                    .strong()
                    .color(TEAL),
            );
            ui.add_space(4.0);
            ui.heading(
                RichText::new("Vector studio")
                    .size(31.0)
                    .strong()
                    .color(TEXT),
            );
            ui.label(
                RichText::new(
                    "Save number-based descriptions, then find the ones most like your search.",
                )
                .size(14.0)
                .color(MUTED),
            );
            ui.add_space(18.0);

            let stat_width = ((ui.available_width() - 20.0) / 3.0).max(120.0);
            ui.horizontal(|ui| {
                self.stat_card(
                    ui,
                    stat_width,
                    "SAVED ITEMS",
                    &self.index.len().to_string(),
                    "Stored in this session",
                    TEAL,
                );
                self.stat_card(
                    ui,
                    stat_width,
                    "VECTOR SIZE",
                    &self.index.params().dim.to_string(),
                    "Numbers per item",
                    BLUE,
                );
                self.stat_card(
                    ui,
                    stat_width,
                    "SEARCH STYLE",
                    "Exact",
                    "Checks every saved item",
                    PURPLE,
                );
            });
            ui.add_space(16.0);

            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    self.vector_entry_card(ui);
                    ui.add_space(14.0);
                    self.csv_card(ui);
                });
                columns[1].vertical(|ui| {
                    self.search_card(ui);
                    ui.add_space(14.0);
                    self.results_card(ui);
                });
            });

            ui.add_space(18.0);
            self.status_card(ui);
            ui.add_space(18.0);
            self.studio_cases(ui);
        }

        fn stat_card(
            &self,
            ui: &mut Ui,
            width: f32,
            eyebrow: &str,
            value: &str,
            detail: &str,
            accent: Color32,
        ) {
            Frame::new()
                .fill(PANEL)
                .stroke(Stroke::new(1.0_f32, BORDER))
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.label(RichText::new(eyebrow).size(10.0).strong().color(MUTED));
                    ui.add_space(7.0);
                    ui.label(RichText::new(value).size(22.0).strong().color(accent));
                    ui.label(RichText::new(detail).size(11.0).color(MUTED));
                });
        }

        fn vector_entry_card(&mut self, ui: &mut Ui) {
            card(ui, |ui| {
                card_heading(
                    ui,
                    "Add a vector",
                    "A vector is a short list of numbers describing one item.",
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Dimension").color(TEXT));
                    ui.add_sized(
                        [72.0, 30.0],
                        TextEdit::singleline(&mut self.dimension_input),
                    );
                    if ui.button("Apply").clicked() {
                        self.apply_dimension();
                    }
                    if ui.button("Clear all").clicked() {
                        self.clear_vectors();
                    }
                });
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Vector ID").color(TEXT));
                    ui.add_sized([88.0, 30.0], TextEdit::singleline(&mut self.id_input));
                });
                ui.label(RichText::new("Vector values").size(12.0).color(MUTED));
                ui.horizontal(|ui| {
                    let input_width = (ui.available_width() - 42.0).max(100.0);
                    ui.add_sized(
                        [input_width, 32.0],
                        TextEdit::singleline(&mut self.vector_input)
                            .hint_text("For example: 0.1, 0.2, 0.3"),
                    );
                    if ui
                        .add_sized(
                            [34.0, 32.0],
                            Button::new(RichText::new("✦").size(17.0).color(TEAL))
                                .fill(PANEL_LIGHT)
                                .stroke(Stroke::new(1.0_f32, BORDER)),
                        )
                        .on_hover_text("Generate random sample values (not AI embeddings)")
                        .clicked()
                    {
                        self.generate_sample_vector();
                    }
                });
                ui.label(
                    RichText::new("Use ✦ to fill this with random sample numbers.")
                        .size(10.0)
                        .color(MUTED),
                );
                ui.add_space(5.0);
                if accent_button(ui, "＋   Add vector", TEAL).clicked() {
                    self.add_vector();
                }
            });
        }

        fn search_card(&mut self, ui: &mut Ui) {
            card(ui, |ui| {
                card_heading(
                    ui,
                    "Find similar items",
                    "Enter a search vector to compare it with saved items.",
                );
                ui.add_space(10.0);
                ui.label(RichText::new("Your search vector").size(12.0).color(MUTED));
                ui.add(
                    TextEdit::singleline(&mut self.query_input)
                        .hint_text("Enter the same number of values as the dimension")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("How to compare").color(TEXT));
                    ComboBox::from_id_salt("distance_metric")
                        .selected_text(metric_short_label(self.metric))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.metric,
                                DistanceMetric::SquaredEuclidean,
                                "Squared Euclidean",
                            );
                            ui.selectable_value(&mut self.metric, DistanceMetric::Cosine, "Cosine");
                            ui.selectable_value(
                                &mut self.metric,
                                DistanceMetric::Manhattan,
                                "Manhattan",
                            );
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Show closest").color(TEXT));
                    ui.add_sized([54.0, 28.0], TextEdit::singleline(&mut self.top_k_input));
                    ui.label(RichText::new("matches").color(TEXT));
                });
                ui.add_space(5.0);
                if accent_button(ui, "⌕   Search vectors", BLUE).clicked() {
                    self.search();
                }
            });
        }

        fn results_card(&self, ui: &mut Ui) {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Nearest matches")
                            .size(17.0)
                            .strong()
                            .color(TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} saved", self.index.len()))
                                .size(11.0)
                                .color(MUTED),
                        );
                    });
                });
                ui.add_space(4.0);
                if self.results.is_empty() {
                    ui.label(
                        RichText::new("Your closest results will show up here after a search.")
                            .size(12.0)
                            .color(MUTED),
                    );
                } else {
                    for (rank, hit) in self.results.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [28.0, 28.0],
                                Button::new(
                                    RichText::new(format!("{:02}", rank + 1))
                                        .size(11.0)
                                        .strong()
                                        .color(TEAL),
                                )
                                .fill(PANEL_LIGHT)
                                .stroke(Stroke::NONE),
                            );
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(hit.label.as_deref().unwrap_or("Saved vector"))
                                        .strong()
                                        .color(TEXT),
                                );
                                ui.label(
                                    RichText::new(format!("Vector ID {}", hit.id))
                                        .size(11.0)
                                        .color(MUTED),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("{:.5}", hit.distance))
                                        .monospace()
                                        .color(TEAL),
                                );
                            });
                        });
                        ui.add_space(4.0);
                    }
                    ui.label(
                        RichText::new("Smaller distance = a closer match.")
                            .size(11.0)
                            .color(MUTED),
                    );
                }
            });
        }

        fn csv_card(&mut self, ui: &mut Ui) {
            card(ui, |ui| {
                card_heading(
                    ui,
                    "Bring your own vectors",
                    "Save or reload a simple CSV file on this computer.",
                );
                ui.add_space(8.0);
                ui.add(
                    TextEdit::singleline(&mut self.csv_path)
                        .hint_text("vectors.csv")
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    if ui.button("Import CSV").clicked() {
                        self.import_csv();
                    }
                    if ui.button("Export CSV").clicked() {
                        self.export_csv();
                    }
                });
            });
        }

        fn status_card(&self, ui: &mut Ui) {
            let color = if self.status_is_error { RED } else { TEAL };
            Frame::new()
                .fill(Color32::from_rgb(19, 42, 47))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(42, 84, 85)))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(if self.status_is_error { "!" } else { "●" })
                                .strong()
                                .color(color),
                        );
                        ui.label(RichText::new(&self.status).color(TEXT));
                    });
                });
        }

        fn studio_cases(&mut self, ui: &mut Ui) {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(
                        RichText::new("What could local search help with?")
                            .size(20.0)
                            .strong()
                            .color(TEXT),
                    );
                    ui.label(
                        RichText::new("Choose an industry to see the idea in everyday language.")
                            .color(MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Browse all ideas  →").clicked() {
                        self.page = Page::CaseStudies;
                    }
                });
            });
            ui.add_space(10.0);
            let width = ((ui.available_width() - 24.0) / 4.0).max(175.0);
            for row_start in (0..CASES.len()).step_by(4) {
                ui.horizontal(|ui| {
                    for (index, case) in CASES.iter().enumerate().skip(row_start).take(4) {
                        let response = ui.add_sized(
                            [width, 90.0],
                            Button::new(
                                RichText::new(format!(
                                    "{}   {}\n\n{}",
                                    case.icon, case.name, case.summary
                                ))
                                .size(12.0)
                                .color(TEXT),
                            )
                            .fill(PANEL)
                            .stroke(Stroke::new(1.0_f32, BORDER)),
                        );
                        if response.clicked() {
                            self.page = Page::Case(index);
                        }
                    }
                });
                ui.add_space(6.0);
            }
        }

        fn case_studies_page(&mut self, ui: &mut Ui) {
            ui.label(
                RichText::new("REAL-WORLD IDEAS, PLAINLY EXPLAINED")
                    .size(11.0)
                    .strong()
                    .color(TEAL),
            );
            ui.add_space(5.0);
            ui.heading(
                RichText::new("One search idea. Many useful places.")
                    .size(30.0)
                    .strong()
                    .color(TEXT),
            );
            ui.label(
                RichText::new("Pick an industry to see what people might save, what they could search for, and where local vector search could fit.")
                    .size(14.0)
                    .color(MUTED),
            );
            ui.add_space(18.0);
            let width = ((ui.available_width() - 24.0) / 4.0).max(180.0);
            for row_start in (0..CASES.len()).step_by(4) {
                ui.horizontal(|ui| {
                    for (index, case) in CASES.iter().enumerate().skip(row_start).take(4) {
                        Frame::new()
                            .fill(PANEL)
                            .stroke(Stroke::new(1.0_f32, BORDER))
                            .inner_margin(15.0)
                            .show(ui, |ui| {
                                ui.set_min_width(width - 30.0);
                                ui.set_min_height(166.0);
                                ui.label(RichText::new(case.icon).size(12.0).strong().color(TEAL));
                                ui.add_space(7.0);
                                ui.label(RichText::new(case.name).size(16.0).strong().color(TEXT));
                                ui.add_space(3.0);
                                ui.label(RichText::new(case.summary).size(12.0).color(MUTED));
                                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                                    if ui.button("See the idea  →").clicked() {
                                        self.page = Page::Case(index);
                                    }
                                });
                            });
                    }
                });
                ui.add_space(12.0);
            }
            ui.add_space(8.0);
            card(ui, |ui| {
                ui.label(
                    RichText::new("A quick note about these examples")
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new("These are possible product ideas, not claims that ArokoDB is already connected to these industries. The sample search uses tiny made-up number lists so you can try the controls.")
                        .size(12.0)
                        .color(MUTED),
                );
            });
        }

        fn case_detail_page(&mut self, ui: &mut Ui, case_index: usize) {
            let Some(case) = CASES.get(case_index) else {
                self.page = Page::CaseStudies;
                return;
            };
            if ui.button("←  All industry ideas").clicked() {
                self.page = Page::CaseStudies;
            }
            ui.add_space(10.0);
            ui.label(
                RichText::new("INDUSTRY IDEA  ·  PLAIN ENGLISH")
                    .size(11.0)
                    .strong()
                    .color(TEAL),
            );
            ui.add_space(4.0);
            ui.heading(RichText::new(case.name).size(32.0).strong().color(TEXT));
            ui.label(RichText::new(case.summary).size(15.0).color(MUTED));
            ui.add_space(18.0);

            Frame::new()
                .fill(PANEL)
                .stroke(Stroke::new(1.0_f32, BORDER))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("THE EVERYDAY SITUATION")
                            .size(10.0)
                            .strong()
                            .color(BLUE),
                    );
                    ui.add_space(7.0);
                    ui.label(RichText::new(case.scenario).size(17.0).color(TEXT));
                    ui.add_space(14.0);
                    ui.columns(3, |columns| {
                        explanation_block(
                            &mut columns[0],
                            "01  What might be saved",
                            case.what_is_saved,
                        );
                        explanation_block(
                            &mut columns[1],
                            "02  What could be asked",
                            case.simple_search,
                        );
                        explanation_block(
                            &mut columns[2],
                            "03  Why it may help",
                            case.useful_because,
                        );
                    });
                });

            ui.add_space(14.0);
            ui.columns(2, |columns| {
                card(&mut columns[0], |ui| {
                    card_heading(ui, "What ArokoDB does", "A small helper behind an app, not the whole app.");
                    ui.add_space(9.0);
                    ui.label(RichText::new("1. An app turns saved content into number lists.\n2. ArokoDB keeps those lists on the device.\n3. A search returns the closest saved items.").color(TEXT));
                    ui.add_space(8.0);
                    ui.label(RichText::new(case.caution).size(12.0).color(ORANGE));
                });
                card(&mut columns[1], |ui| {
                    card_heading(ui, "Try a tiny example", "See the vector studio find a related sample.");
                    ui.add_space(8.0);
                    ui.label(RichText::new("We'll add three made-up 3-number vectors, run a search, and show the closest result. These numbers are teaching examples—not real descriptions or AI embeddings.").size(12.0).color(MUTED));
                    ui.add_space(9.0);
                    if accent_button(ui, "Try this example  →", TEAL).clicked() {
                        self.load_case_example(case_index);
                    }
                });
            });
        }
    }

    impl eframe::App for ArokoApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.activity_bar(ctx);
            self.sidebar(ctx);
            egui::CentralPanel::default()
                .frame(Frame::new().fill(BACKGROUND).inner_margin(22.0))
                .show(ctx, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| match self.page {
                            Page::Studio => self.studio_page(ui),
                            Page::CaseStudies => self.case_studies_page(ui),
                            Page::Case(index) => self.case_detail_page(ui, index),
                        });
                });
        }
    }

    fn card(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        Frame::new()
            .fill(PANEL)
            .stroke(Stroke::new(1.0_f32, BORDER))
            .inner_margin(16.0)
            .show(ui, content);
    }

    fn card_heading(ui: &mut Ui, heading: &str, description: &str) {
        ui.heading(RichText::new(heading).size(17.0).strong().color(TEXT));
        ui.label(RichText::new(description).size(12.0).color(MUTED));
    }

    fn explanation_block(ui: &mut Ui, heading: &str, body: &str) {
        ui.label(RichText::new(heading).size(11.0).strong().color(TEAL));
        ui.add_space(5.0);
        ui.label(RichText::new(body).size(13.0).color(TEXT));
    }

    fn accent_button(ui: &mut Ui, label: &str, color: Color32) -> egui::Response {
        ui.add(
            Button::new(RichText::new(label).strong().color(BACKGROUND))
                .fill(color)
                .stroke(Stroke::NONE),
        )
    }

    fn metric_short_label(metric: DistanceMetric) -> &'static str {
        match metric {
            DistanceMetric::SquaredEuclidean => "Squared Euclidean",
            DistanceMetric::Cosine => "Cosine distance",
            DistanceMetric::Manhattan => "Manhattan (L1)",
        }
    }

    fn parse_vector(text: &str, dimension: usize) -> Result<Vec<f32>, String> {
        let values: Result<Vec<f32>, _> = text
            .split(',')
            .map(|part| part.trim().parse::<f32>())
            .collect();
        let values = values.map_err(|_| "Enter comma-separated numbers.".to_owned())?;
        if values.len() != dimension || values.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "Enter exactly {dimension} finite numbers separated by commas."
            ));
        }
        Ok(values)
    }

    fn generate_sample_vector(dimension: usize, seed: u64) -> String {
        let mut state = seed.max(1);
        (0..dimension)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let value = (state % 2001) as i32 - 1000;
                format!("{:.3}", value as f32 / 1000.0)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn parse_csv(text: &str, dimension: usize) -> Result<Vec<(usize, Vec<f32>)>, String> {
        let mut lines = text.lines();
        let header = lines
            .next()
            .ok_or_else(|| "CSV file is empty.".to_owned())?
            .trim_start_matches('\u{feff}')
            .trim();
        let columns: Vec<_> = header.split(',').map(str::trim).collect();
        if columns.len() != dimension + 1
            || columns[0] != "id"
            || columns[1..]
                .iter()
                .enumerate()
                .any(|(i, column)| *column != format!("v{}", i + 1))
        {
            return Err(format!(
                "Expected CSV header: id,{}",
                (1..=dimension)
                    .map(|i| format!("v{i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }

        let mut records = Vec::new();
        let mut ids = HashSet::new();
        for (line_number, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split(',').map(str::trim).collect();
            if fields.len() != dimension + 1 {
                return Err(format!(
                    "CSV line {} has {} fields; expected {}.",
                    line_number + 2,
                    fields.len(),
                    dimension + 1
                ));
            }
            let id = fields[0]
                .parse::<usize>()
                .map_err(|_| format!("CSV line {} has an invalid vector ID.", line_number + 2))?;
            if !ids.insert(id) {
                return Err(format!(
                    "CSV contains duplicate vector ID {id} (line {}).",
                    line_number + 2
                ));
            }
            let values: Result<Vec<f32>, _> = fields[1..]
                .iter()
                .map(|value| value.parse::<f32>())
                .collect();
            let values = values
                .map_err(|_| format!("CSV line {} contains an invalid number.", line_number + 2))?;
            if values.iter().any(|value| !value.is_finite()) {
                return Err(format!(
                    "CSV line {} contains a non-finite number.",
                    line_number + 2
                ));
            }
            records.push((id, values));
        }
        Ok(records)
    }

    fn format_csv(index: &VectorIndex) -> String {
        let mut csv = format!(
            "id,{}\r\n",
            (1..=index.params().dim)
                .map(|i| format!("v{i}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        for (id, vector) in index.records() {
            csv.push_str(&id.to_string());
            for value in vector {
                csv.push(',');
                csv.push_str(&value.to_string());
            }
            csv.push_str("\r\n");
        }
        csv
    }

    pub fn run() -> eframe::Result {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1420.0, 930.0])
                .with_min_inner_size([980.0, 720.0])
                .with_title("ArokoDB · Vector Studio"),
            ..Default::default()
        };
        eframe::run_native(
            "ArokoDB · Vector Studio",
            options,
            Box::new(|ctx| Ok(Box::new(ArokoApp::new(ctx)))),
        )
    }

    #[cfg(test)]
    mod tests {
        use super::{format_csv, generate_sample_vector, parse_csv, parse_vector};
        use aroko_core::index::{IndexParams, VectorIndex};

        #[test]
        fn parses_vectors_for_selected_dimension() {
            assert_eq!(parse_vector("1, 2.5, -3", 3).unwrap(), vec![1.0, 2.5, -3.0]);
            assert!(parse_vector("1, 2", 3).is_err());
            assert!(parse_vector("1, NaN, 3", 3).is_err());
        }

        #[test]
        fn generated_sample_values_match_dimension_and_are_finite() {
            let generated = generate_sample_vector(8, 12345);
            let values = parse_vector(&generated, 8).unwrap();
            assert!(values.iter().all(|value| (-1.0..=1.0).contains(value)));
            assert_eq!(generated, generate_sample_vector(8, 12345));
        }

        #[test]
        fn csv_round_trip_preserves_ids_and_values() {
            let mut index = VectorIndex::new(IndexParams { dim: 2 });
            index.insert_with_id(42, vec![1.25, -2.0]).unwrap();
            let csv = format_csv(&index);
            assert_eq!(csv, "id,v1,v2\r\n42,1.25,-2\r\n");
            assert_eq!(parse_csv(&csv, 2).unwrap(), vec![(42, vec![1.25, -2.0])]);
        }

        #[test]
        fn csv_rejects_malformed_rows_and_duplicate_ids() {
            assert!(parse_csv("id,v1,v2\n1,2\n", 2).is_err());
            assert!(parse_csv("id,v1,v2\n1,2,3\n1,4,5\n", 2).is_err());
            assert!(parse_csv("id,v1\n1,NaN\n", 1).is_err());
        }

        #[test]
        fn each_industry_example_returns_its_featured_match_first() {
            for case in super::CASES {
                let mut index = VectorIndex::new(IndexParams { dim: 3 });
                for (id, example) in case.examples.into_iter().enumerate() {
                    index.insert_with_id(id, example.values.to_vec()).unwrap();
                }

                let results = index.search(&case.query, case.examples.len());
                assert_eq!(results[0].0, 0, "{}", case.name);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn main() -> eframe::Result {
    windows_app::run()
}
