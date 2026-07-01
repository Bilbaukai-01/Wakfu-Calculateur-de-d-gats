use eframe::egui;
use crate::model::{AppState, Tab};

// ==================================================================================
// SOUS-MODULES (tabs)
// ==================================================================================
#[path = "tab_aide.rs"]
pub mod tab_aide;
#[path = "tab_stats.rs"]
pub mod tab_stats;
#[path = "tab_combats.rs"]
pub mod tab_combats;
#[path = "tab_archivage.rs"]
pub mod tab_archivage;
#[path = "tab_settings.rs"]
pub mod tab_settings;

// ==================================================================================
// UI WIDGETS
// ==================================================================================

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    // Correction ici : on ne passe que 3 arguments (Type, Valeur, Label)
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    let how_on = ui.ctx().animate_bool(response.id, *on);
    let visuals = ui.style().interact_selectable(&response, *on);
    let rect = rect.expand(visuals.expansion);
    let radius = 0.5 * rect.height();

    // Couleur de fond (Vert si On, Gris si Off)
    let bg_color = if *on {
        egui::Color32::from_rgb(50, 200, 50) // vert
    } else {
        egui::Color32::from_rgb(80, 80, 80) // gris
    };

    ui.painter().add(egui::Shape::rect_filled(rect, radius, bg_color));

    // Le cercle blanc qui bouge
    let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
    let center = egui::pos2(circle_x, rect.center().y);
    ui.painter().add(egui::Shape::circle_filled(center, radius * 0.75, egui::Color32::WHITE));

    response
}


// ==================================================================================
// MAIN APPLICATION (MyApp) — requis par main.rs pour eframe::run_native
// ==================================================================================

pub struct MyApp {
    pub state: std::sync::Arc<std::sync::Mutex<AppState>>,
}

impl eframe::App for MyApp {

    // FORCAGE DU FOND TRANSPARENT (la fenêtre est en mode transparent)
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let mut s = match self.state.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        // --- 1. APPLICATION DU ZOOM ET DE LA TAILLE (PRIORITAIRE) ---
        let applying_change = (s.config.zoom_factor - s.last_applied_zoom).abs() > 0.01;

        if applying_change {
            ctx.set_zoom_factor(s.config.zoom_factor);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(s.config.window_width, s.config.window_height)));
            s.last_applied_zoom = s.config.zoom_factor;
        }

        // --- 2. SAUVEGARDE DE LA TAILLE MANUELLE ---
        if !s.is_compact && !applying_change {
            let current_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
            if let Some(size) = current_size {
                if size.x > 250.0 && size.y > 350.0 {
                    if (size.x - s.config.window_width).abs() > 10.0 || (size.y - s.config.window_height).abs() > 10.0 {
                        s.config.window_width = size.x;
                        s.config.window_height = size.y;
                        crate::persistence::save_config(&s.config);

                    }
                }
            }
        }

        // --- FENÊTRE TOUJOURS AU-DESSUS ---
        let window_level = if s.config.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(window_level));

        // --- IMAGES (logo + bouton réduire) ---
        let img_logo = egui::include_image!("../logo.png");
        let img_reduce = egui::include_image!("../reduction_window.png");

        // --- VISUELS (couleur de texte configurable) ---
        let mut visuals = egui::Visuals::dark();
        let r = s.config.text_color[0];
        let g = s.config.text_color[1];
        let b = s.config.text_color[2];
        let ma_couleur_globale = egui::Color32::from_rgb(r, g, b);
        visuals.override_text_color = Some(ma_couleur_globale);
        visuals.widgets.noninteractive.fg_stroke.color = ma_couleur_globale;
        visuals.widgets.inactive.fg_stroke.color = ma_couleur_globale;
        visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
        visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        visuals.panel_fill = egui::Color32::TRANSPARENT;
        ctx.set_visuals(visuals);

        // =================================================================
        // DÉGRADÉ DE FOND : noir → violet avec opacité
        // =================================================================
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let rect = ui.max_rect();
            let mut mesh = egui::Mesh::default();

            // On applique l'opacité utilisateur (s.config.opacity) aux deux couleurs du dégradé de fond
            let color_top = egui::Color32::BLACK.linear_multiply(s.config.opacity);
            let color_bottom = egui::Color32::from_rgb(79, 0, 111).linear_multiply(s.config.opacity);

            mesh.vertices.push(egui::epaint::Vertex { pos: rect.left_top(), uv: egui::Pos2::ZERO, color: color_top });
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.right_top(), uv: egui::Pos2::ZERO, color: color_top });
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.right_bottom(), uv: egui::Pos2::ZERO, color: color_bottom });
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.left_bottom(), uv: egui::Pos2::ZERO, color: color_bottom });
            mesh.indices.extend([0, 1, 2, 0, 2, 3]);
            ui.painter().add(egui::Shape::mesh(mesh));

            ui.add_space(5.0);

            // --- MODE COMPACT (fenêtre réduite à un bouton) ---
            if s.is_compact {
                ui.centered_and_justified(|ui| {
                    if ui.add(egui::ImageButton::new(egui::Image::new(img_logo).max_width(45.0))).clicked() {
                        s.is_compact = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(s.config.window_width, s.config.window_height)));
                    }
                });
                return;
            }

            // --- BARRE HAUTE : titre + logo + bouton réduire ---
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.add(egui::Image::new(img_logo).max_width(32.0));
                ui.heading(egui::RichText::new("Wakfu calculateur de dégats").strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.add(egui::ImageButton::new(egui::Image::new(img_reduce).max_width(24.0))).clicked() {
                        s.is_compact = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(80.0, 80.0)));
                    }
                });
            });

            ui.add_space(10.0);

            // --- ONGLETS ---
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                if ui.selectable_label(s.current_tab == Tab::Aide, "ℹ Aide").clicked() { s.current_tab = Tab::Aide; }
                if ui.selectable_label(s.current_tab == Tab::Stats, "📊 Stats").clicked() { s.current_tab = Tab::Stats; }
                if ui.selectable_label(s.current_tab == Tab::Combats, "🕒 Combats").clicked() { s.current_tab = Tab::Combats; }
                if ui.selectable_label(s.current_tab == Tab::Archivage, "📜 Archivage").clicked() { s.current_tab = Tab::Archivage; }
                if ui.selectable_label(s.current_tab == Tab::Settings, "⚙ Réglages").clicked() { s.current_tab = Tab::Settings; }
            });

            ui.separator();

            // --- CONTENU DE L'ONGLET SÉLECTIONNÉ ---
            egui::ScrollArea::vertical()
                .id_source("main_scroll_area")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    match s.current_tab {
                        Tab::Aide => tab_aide::render(&mut s, ui),
                        Tab::Stats => tab_stats::render(&mut s, ui),
                        Tab::Combats => tab_combats::render(&mut s, ui),
                        Tab::Archivage => tab_archivage::render(&mut s, ui),
                        Tab::Settings => tab_settings::render(&mut s, ui),
                    }
                });

            // --- FOOTER FIXE ---
            egui::Area::new(egui::Id::new("footer_fixe"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
                .show(ctx, |ui| {
                    let version = env!("CARGO_PKG_VERSION");
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!("© Application créée par Bilbaukai - v{}", version))
                                .small()
                                .italics()
                                .color(egui::Color32::LIGHT_GRAY),
                        )
                        .wrap(false),
                    );
                });
        }); // Fin du CentralPanel
    } // Fin de update
} // Fin de impl MyApp