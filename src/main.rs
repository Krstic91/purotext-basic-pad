use eframe::egui;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

fn global_syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn global_theme_set() -> &'static ThemeSet {
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 650.0])
            .with_title("PuroText"),
        ..Default::default()
    };

    eframe::run_native(
        "PuroText",
        options,
        Box::new(|_cc| Box::new(NotepadApp::default())),
    )
}

#[derive(PartialEq)]
enum FontFamilyType {
    Monospace,
    Proportional,
}

struct NotepadApp {
    content: String,
    saved_content: String,
    current_file: Option<PathBuf>,
    status_message: String,
    font_size: f32,
    font_type: FontFamilyType,
    show_about_dialog: bool,
    show_line_numbers: bool,
    dark_mode: bool,
    selected_language: String,
    show_exit_confirmation: bool,
}

impl Default for NotepadApp {
    fn default() -> Self {
        Self {
            content: String::new(),
            saved_content: String::new(),
            current_file: None,
            status_message: String::from("Listo"),
            font_size: 16.0,
            font_type: FontFamilyType::Monospace,
            show_about_dialog: false,
            show_line_numbers: true,
            dark_mode: true,
            selected_language: String::from("Texto Plano"),
            show_exit_confirmation: false,
        }
    }
}

impl NotepadApp {
    fn is_modified(&self) -> bool {
        self.content != self.saved_content
    }

    fn new_file(&mut self) {
        if !self.is_modified() || self.save_file() {
            self.content.clear();
            self.saved_content.clear();
            self.current_file = None;
            self.selected_language = String::from("Texto Plano");
            self.status_message = String::from("Nuevo archivo creado");
        }
    }

    fn detect_language(&mut self, path: &PathBuf) {
        let ps = global_syntax_set();
        if let Some(syntax) = ps.find_syntax_for_file(path).ok().flatten() {
            self.selected_language = syntax.name.clone();
        } else {
            self.selected_language = String::from("Texto Plano");
        }
    }

    fn open_file(&mut self) {
        if !self.is_modified() || self.save_file() {
            if let Some(path) = FileDialog::new().pick_file() {
                match fs::read_to_string(&path) {
                    Ok(text) => {
                        self.content = text.clone();
                        self.saved_content = text;
                        self.detect_language(&path);
                        self.current_file = Some(path.clone());
                        self.status_message =
                            format!("Cargado: {:?}", path.file_name().unwrap_or_default());
                    }
                    Err(err) => {
                        self.status_message = format!("Error al abrir: {}", err);
                    }
                }
            }
        }
    }

    fn save_file(&mut self) -> bool {
        if let Some(ref path) = self.current_file {
            match fs::write(path, &self.content) {
                Ok(_) => {
                    self.saved_content = self.content.clone();
                    self.status_message =
                        format!("Guardado: {:?}", path.file_name().unwrap_or_default());
                    true
                }
                Err(err) => {
                    self.status_message = format!("Error al guardar: {}", err);
                    false
                }
            }
        } else {
            self.save_file_as()
        }
    }

    fn save_file_as(&mut self) -> bool {
        if let Some(path) = FileDialog::new()
            .add_filter("Texto / Código", &["txt", "md", "rs", "py", "js", "cpp", "c", "html", "json"])
            .save_file()
        {
            match fs::write(&path, &self.content) {
                Ok(_) => {
                    self.status_message =
                        format!("Guardado como: {:?}", path.file_name().unwrap_or_default());
                    self.detect_language(&path);
                    self.current_file = Some(path);
                    self.saved_content = self.content.clone();
                    true
                }
                Err(err) => {
                    self.status_message = format!("Error al guardar: {}", err);
                    false
                }
            }
        } else {
            false
        }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        let ps = global_syntax_set();
        let ts = global_theme_set();

        // --- BARRA SUPERIOR (Menú) ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Archivo", |ui| {
                    if ui.button("Nuevo (Ctrl+N)").clicked() { self.new_file(); }
                    if ui.button("Abrir... (Ctrl+O)").clicked() { self.open_file(); }
                    if ui.button("Guardar (Ctrl+S)").clicked() { self.save_file(); }
                    if ui.button("Guardar como...").clicked() { self.save_file_as(); }
                    ui.separator();
                    if ui.button("Salir").clicked() {
                        if self.is_modified() {
                            self.show_exit_confirmation = true;
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });

                ui.menu_button("Lenguaje", |ui| {
                    if ui.selectable_label(self.selected_language == "Texto Plano", "Texto Plano").clicked() {
                        self.selected_language = String::from("Texto Plano");
                    }
                    ui.separator();

                    for syntax in ps.syntaxes().iter().take(15) {
                        if ui.selectable_label(self.selected_language == syntax.name, &syntax.name).clicked() {
                            self.selected_language = syntax.name.clone();
                        }
                    }
                });

                ui.menu_button("Ver", |ui| {
                    ui.checkbox(&mut self.show_line_numbers, "Mostrar números de línea");
                    ui.separator();
                    let theme_label = if self.dark_mode { "🌙 Modo Oscuro" } else { "☀️ Modo Claro" };
                    ui.checkbox(&mut self.dark_mode, theme_label);
                });

                ui.menu_button("Fuente", |ui| {
                    ui.radio_value(&mut self.font_type, FontFamilyType::Monospace, "Monoespaciada");
                    ui.radio_value(&mut self.font_type, FontFamilyType::Proportional, "Proporcional");
                    ui.separator();
                    ui.add(egui::Slider::new(&mut self.font_size, 10.0..=40.0).text("px"));
                });

                ui.menu_button("Ayuda", |ui| {
                    if ui.button("Acerca de PuroText...").clicked() {
                        self.show_about_dialog = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // --- BARRA INFERIOR (Estado) ---
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status = if self.is_modified() {
                    format!("{} *", self.status_message)
                } else {
                    self.status_message.clone()
                };
                ui.label(status);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let line_count = self.content.lines().count().max(1);
                    let char_count = self.content.chars().count();
                    ui.label(format!(
                        "Lenguaje: {} | Líneas: {} | Caracteres: {}",
                        self.selected_language, line_count, char_count
                    ));
                });
            });
        });

        // --- ÁREA CENTRAL (Editor) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let font_style = match self.font_type {
                FontFamilyType::Monospace => egui::FontId::new(self.font_size, egui::FontFamily::Monospace),
                FontFamilyType::Proportional => egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
            };

            let theme_name = if self.dark_mode { "base16-mocha.dark" } else { "base16-ocean.light" };
            let theme = ts.themes.get(theme_name).unwrap_or(&ts.themes["base16-ocean.dark"]);
            let syntax = ps.find_syntax_by_name(&self.selected_language)
                .unwrap_or_else(|| ps.find_syntax_plain_text());

            let font_style_layout = font_style.clone();
            let is_dark = self.dark_mode;

            let mut layouter = move |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                let mut layout_job = egui::text::LayoutJob::default();
                let mut highlighter = HighlightLines::new(syntax, theme);

                for line in string.split_inclusive('\n') {
                    if let Ok(ranges) = highlighter.highlight_line(line, ps) {
                        for (style, text) in ranges {
                            let color = egui::Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                            layout_job.append(
                                text,
                                0.0,
                                egui::TextFormat {
                                    font_id: font_style_layout.clone(),
                                    color,
                                    ..Default::default()
                                },
                            );
                        }
                    } else {
                        layout_job.append(
                            line,
                            0.0,
                            egui::TextFormat {
                                font_id: font_style_layout.clone(),
                                color: if is_dark { egui::Color32::WHITE } else { egui::Color32::BLACK },
                                ..Default::default()
                            },
                        );
                    }
                }
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.show_line_numbers {
                        let total_lines = self.content.lines().count().max(1);
                        let mut line_str = String::new();
                        for i in 1..=total_lines {
                            line_str.push_str(&format!("{}\n", i));
                        }

                        let line_color = if self.dark_mode { egui::Color32::from_gray(120) } else { egui::Color32::from_gray(140) };

                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(line_str)
                                    .font(font_style.clone())
                                    .color(line_color),
                            )
                            .selectable(false),
                        );
                        ui.add(egui::Separator::default().vertical());
                    }

                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.content)
                            .font(font_style)
                            .layouter(&mut layouter)
                            .frame(false),
                    );
                });
            });
        });

    
        // --- VENTANA MODAL: ACERCA DE ---
        if self.show_about_dialog {
            let mut close_dialog = false;

            egui::Window::new("Acerca de PuroText")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about_dialog)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("📝 PuroText");
                        ui.label("Versión 1.0.0");
                        ui.add_space(8.0);
                        
                        ui.label("Un editor de texto ligero y rápido construido con Rust y egui.");
                        ui.label("Soporta resaltado de sintaxis mediante syntect.");
                        ui.add_space(10.0);
                        
                        ui.separator();
                        ui.add_space(5.0);
                        
                        ui.label(egui::RichText::new("Desarrollado por:").strong());
                        ui.label("• Cristhian Cruz");
                        ui.label("• William F. Aguilar");
                        
                        ui.add_space(12.0);

                        if ui.button("Cerrar").clicked() {
                            close_dialog = true;
                        }
                    });
                });

            if close_dialog {
                self.show_about_dialog = false;
            }
        }
    }
}