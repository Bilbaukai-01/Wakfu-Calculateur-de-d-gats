use eframe::egui;
use crate::model::AppState;

/// Widget personnalisé de bouton poussoir (Toggle Switch) pour l'Overlay
fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        
        // Couleur de fond (vert si actif, gris sombre si inactif)
        let bg_fill = if *on {
            egui::Color32::from_rgb(46, 125, 50) // Vert
        } else {
            ui.style().visuals.widgets.inactive.bg_fill
        };
        
        ui.painter().rect(rect, rect.height() / 2.0, bg_fill, visuals.bg_stroke);
        
        // Position du curseur rond
        let circle_x = egui::lerp((rect.left() + rect.height() / 2.0)..=(rect.right() - rect.height() / 2.0), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        let radius = rect.height() / 2.0 - 2.0;
        ui.painter().circle(center, radius, visuals.fg_stroke.color, visuals.fg_stroke);
    }
    response
}

pub fn render(s: &mut AppState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().id_source("settings_scroll").show(ui, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    
                    // ==================================================================
                    // BLOC CONFIGURATION (Fond transparent, bordures conservées)
                    // ==================================================================
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.heading("🛠 Configuration");
                                ui.separator();
                                ui.add_space(5.0);

                                // Fichier de Log (Menu déroulant réductible)
                                egui::CollapsingHeader::new(egui::RichText::new("📂 Fichier de Log").heading())
                                    .id_source("log_file_header")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.add_space(5.0);
                                        ui.horizontal(|ui| {
                                            ui.text_edit_singleline(&mut s.config.log_path);
                                            if ui.button("📂").on_hover_text("Parcourir...").clicked() {
                                                if let Some(path) = rfd::FileDialog::new()
                                                    .set_title("Sélectionner le fichier de log Wakfu")
                                                    .add_filter("Fichiers Log", &["log", "txt"])
                                                    .pick_file() 
                                                {
                                                    s.config.log_path = path.display().to_string();
                                                }
                                            }
                                        });
                                    });

                                ui.add_space(10.0);

                                // Personnages suivis (Menu déroulant réductible)
                                let count = s.config.tracked_players.len();
                                let tracked_title = format!("👥 Personnages suivis ({})", count);
                                
                                egui::CollapsingHeader::new(egui::RichText::new(tracked_title).heading())
                                    .id_source("tracked_players_header")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.add_space(5.0);
                                        let mut to_remove = None;
                                        let mut to_swap = None;
                                        let players_count = s.config.tracked_players.len();

                                        for (idx, player) in s.config.tracked_players.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.label(player);
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button("❌").clicked() {
                                                        to_remove = Some(idx);
                                                    }
                                                    if idx + 1 < players_count && ui.button("⬇").clicked() {
                                                        to_swap = Some((idx, idx + 1));
                                                    }
                                                    if idx > 0 && ui.button("⬆").clicked() {
                                                        to_swap = Some((idx, idx - 1));
                                                    }
                                                });
                                            });
                                        }

                                        if let Some(idx) = to_remove {
                                            s.config.tracked_players.remove(idx);
                                        }
                                        if let Some((i1, i2)) = to_swap {
                                            s.config.tracked_players.swap(i1, i2);
                                        }

                                        ui.horizontal(|ui| {
                                            static mut NEW_PLAYER_BUF: String = String::new();
                                            ui.text_edit_singleline(unsafe { &mut *std::ptr::addr_of_mut!(NEW_PLAYER_BUF) });
                                            if ui.button("➕ Ajouter").clicked() {
                                                let name = unsafe { NEW_PLAYER_BUF.trim().to_string() };
                                                if !name.is_empty() && !s.config.tracked_players.contains(&name) {
                                                    s.config.tracked_players.push(name);
                                                    unsafe { NEW_PLAYER_BUF.clear(); }
                                                }
                                            }
                                        });
                                    });
                            });
                        });

                    ui.add_space(15.0);

                    // ==================================================================
                    // BLOC INTERFACE (Fond transparent, bordures conservées)
                    // ==================================================================
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.heading("📺 Interface");
                                ui.separator();
                                ui.add_space(5.0);

                                // Zoom UI
                                ui.horizontal(|ui| {
                                    ui.label("🔍 Zoom UI : ");
                                    if ui.button("➖").clicked() {
                                        s.config.zoom_factor = (s.config.zoom_factor - 0.1).max(0.5);
                                    }

                                    ui.add(
                                        egui::DragValue::new(&mut s.config.zoom_factor)
                                            .clamp_range(0.5..=2.0)
                                            .speed(0.01)
                                            .fixed_decimals(2)
                                            .suffix("x")
                                    );

                                    if ui.button("➕").clicked() {
                                        s.config.zoom_factor = (s.config.zoom_factor + 0.1).min(2.0);
                                    }
                                });

                                ui.add_space(10.0);

                                // Opacité
                                ui.horizontal(|ui| {
                                    ui.label("Opacité : ");
                                    let mut opacity_pct = (s.config.opacity * 100.0).round() as i32;
                                    let res = ui.add(
                                        egui::Slider::new(&mut opacity_pct, 20..=100).suffix("%")
                                    );
                                    if res.changed() {
                                        s.config.opacity = (opacity_pct as f32) / 100.0;
                                    }
                                });

                                ui.add_space(10.0);

                                // Overlay avec bouton poussoir (Toggle Switch) personnalisé
                                ui.horizontal(|ui| {
                                    ui.label("Overlay :");
                                    toggle_ui(ui, &mut s.config.always_on_top);
                                    ui.label(if s.config.always_on_top { "Activé (Toujours au-dessus)" } else { "Désactivé" });
                                });
                            });
                        });

                    ui.add_space(15.0);

                    // ==================================================================
                    // BOUTONS DE COMMANDE (Élargis et hors groupe)
                    // ==================================================================
                    ui.horizontal(|ui| {
                        // Largeur confortable pour les boutons
                        let btn_size = egui::vec2(190.0, 30.0);

                        if ui.add_sized(btn_size, egui::Button::new("⛭ Paramètres par défaut")).clicked() {
                            s.reset_to_defaults();
                        }

                        if ui.add_sized(btn_size, egui::Button::new("💾 Enregistrer les paramètres")).clicked() {
                            crate::persistence::save_config(&s.config);
                        }
                    });

                    // ==================================================================
                    // SECTION MISE À JOUR
                    // ==================================================================
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(5.0);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Mise à jour de l'application :").strong());
                            ui.label(
                                egui::RichText::new(format!("({})", s.config.statut_maj))
                                    .small()
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        });

                        ui.add_space(5.0);

                        if s.config.maj_prete_pour_redemarrage {
                            let redemarrer_btn = egui::Button::new("✨ Redémarrer pour appliquer la mise à jour")
                                .fill(egui::Color32::from_rgb(34, 139, 34));
                            if ui.add(redemarrer_btn).clicked() {
                                use std::process::Command;
                                use std::env;
                                if let Ok(chemin_exe) = env::current_exe() {
                                    let _ = Command::new(chemin_exe).spawn();
                                }
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        } 
                        else if s.config.maj_disponible {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::from_rgb(250, 200, 50), "⚠️ Nouvelle version disponible !");

                                let maj_btn = egui::Button::new(
                                    egui::RichText::new("📥 Installer la mise à jour")
                                        .color(egui::Color32::BLACK)
                                ).fill(egui::Color32::from_rgb(50, 200, 50));

                                if ui.add(maj_btn).clicked() {
                                    s.config.statut_maj = "Téléchargement en cours...".to_string();
                                    match crate::verifier_et_mettre_a_jour() {
                                        Ok(status) => {
                                            if status.updated() {
                                                s.config.statut_maj = format!("Succès ! v{} installée. Redémarrez.", status.version());
                                                s.config.maj_disponible = false;
                                                s.config.maj_prete_pour_redemarrage = true;
                                                crate::persistence::save_config(&s.config);
                                                ui.ctx().request_repaint();
                                            } else {
                                                s.config.statut_maj = "Déjà à jour".to_string();
                                            }
                                        }
                                        Err(e) => {
                                            s.config.statut_maj = format!("Erreur : {}", e);
                                        }
                                    }
                                }
                            });
                        } 
                        else {
                            if ui.button("🔄 Vérifier manuellement").clicked() {
                                s.config.statut_maj = "Vérification en cours...".to_string();
                                if let Ok(updater) = self_update::backends::github::Update::configure()
                                    .repo_owner("Bilbaukai-01")
                                    .repo_name("Wakfu-Calculateur-de-d-gats")
                                    .bin_name("wakfu_calculateur")
                                    .current_version(env!("CARGO_PKG_VERSION"))
                                    .build()
                                {
                                    if let Ok(latest) = updater.get_latest_release() {
                                        if latest.version != env!("CARGO_PKG_VERSION") {
                                            s.config.maj_disponible = true;
                                            s.config.statut_maj = format!("Version {} disponible !", latest.version);
                                        } else {
                                            s.config.statut_maj = "Déjà à jour !".to_string();
                                        }
                                    } else {
                                        s.config.statut_maj = "Erreur de connexion".to_string();
                                    }
                                }
                            }
                        }
                    });
                    
                });
            });
    });
}
