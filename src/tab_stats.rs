use eframe::egui;
use crate::model::AppState;

/// Fonction utilitaire pour dessiner le bouton commutateur (iOS-style toggle) de l'historique
fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    // Correction de l'erreur E0061 : 3 arguments au lieu de 4
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        // Correction de l'erreur E0599 : utilisation de animate_bool à la place de animate_bool_responsive
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter().rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter().circle(center, radius - 2.0, visuals.fg_stroke.color, visuals.fg_stroke);
    }

    response
}

pub fn render(s: &mut AppState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().id_source("stats_main_scroll").show(ui, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {

                    // =================== 1. SÉLECTION DU MODE & BOUTONS D'ACTION ===================
                    ui.horizontal(|ui| {
                        // --- GAUCHE : Sélection du mode ---
                        if ui.selectable_label(s.mode == "mono", "Mono").clicked() { 
                            s.mode = "mono".to_string(); 
                            if s.players_to_include > 6 { s.players_to_include = 6; }
                            s.full_reset(); 
                            crate::persistence::save_config(&s.config);
                        }
                        if ui.selectable_label(s.mode == "multi", "Multi").clicked() { 
                            s.mode = "multi".to_string(); 
                            if s.players_to_include < 2 { s.players_to_include = 2; }
                            s.full_reset(); 
                            crate::persistence::save_config(&s.config);
                        }

                        // --- DROITE : Boutons d'action ---
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    if ui.button("[x] Reset").clicked() { 
        s.full_reset(); 
    }
    if ui.button("[+] Sauvegarder & Reset").clicked() { 
        // Sauvegarde dans l'historique de session (onglet Combats) et réinitialise
        s.save_to_session_combats_and_reset(); 
    }
});
                    });

                    ui.add_space(5.0);

                    // =================== 2. SÉLECTION DU NOMBRE DE JOUEURS ===================
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label("👥 Joueurs :");
                            let range = if s.mode == "multi" { 2..=6 } else { 1..=6 };
                            for i in range {
                                if ui.selectable_label(s.players_to_include == i, i.to_string()).clicked() {
                                    s.players_to_include = i; 
                                    s.full_reset(); 
                                    s.has_started = true;
                                    crate::persistence::save_config(&s.config);
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // =================== 3. COMMUTATEUR D'AFFICHAGE DE L'HISTORIQUE ===================
                    ui.horizontal(|ui| {
                        toggle_ui(ui, &mut s.showing_history);
                        ui.label("Afficher l'historique");

                        if s.showing_history {
                            ui.separator();
                            ui.label("Disposition :");
                            if ui.selectable_label(!s.config.history_horizontal, "⬇️ Vertical").clicked() {
                                s.config.history_horizontal = false;
                                crate::persistence::save_config(&s.config);
                            }
                            if ui.selectable_label(s.config.history_horizontal, "➡️ Horizontal").clicked() {
                                s.config.history_horizontal = true;
                                crate::persistence::save_config(&s.config);
                            }
                        }
                    });

                    ui.add_space(5.0);

                    // =================== 4. SECTION HISTORIQUE DE COMBAT ===================
                    if s.showing_history {
                        let scroll_area = if s.config.history_horizontal {
                            egui::ScrollArea::both()
                        } else {
                            egui::ScrollArea::vertical()
                        };

                        scroll_area.id_source("history_scroll").show(ui, |ui| {
                            let mut sorted_players: Vec<_> = s.history.keys().cloned().collect();
                            sorted_players.sort_by(|a, b| s.total_damage.get(b).unwrap_or(&0).cmp(s.total_damage.get(a).unwrap_or(&0)));

                            let history = &s.history;
                            let total_damage = &s.total_damage;

                            let render_player = |ui: &mut egui::Ui, player: &String| {
                                ui.vertical(|ui| {
                                    ui.group(|ui| {
                                        egui::CollapsingHeader::new(format!("{} (Total: {} PV)", player, total_damage.get(player).unwrap_or(&0)))
                                            .default_open(true)
                                            .show(ui, |ui| {
                                                if let Some(entries) = history.get(player) {
                                                    let mut spell_totals: std::collections::HashMap<String, (i32, bool)> = std::collections::HashMap::new();
                                                    let mut current_t = -1;
                                                    let mut turn_total = 0;

                                                    for entry in entries {
                                                        let entry_key = if entry.is_indirect { format!("{} (indirect)", entry.spell) } else { entry.spell.clone() };
                                                        let stat = spell_totals.entry(entry_key).or_insert((0, entry.is_indirect));
                                                        stat.0 += entry.total;

                                                        if entry.turn != current_t {
                                                            if current_t != -1 { 
                                                                ui.label(egui::RichText::new(format!("   Total Tour : {} PV", turn_total)).strong().color(egui::Color32::LIGHT_BLUE)); 
                                                            }
                                                            current_t = entry.turn; 
                                                            turn_total = 0;
                                                            ui.label(egui::RichText::new(format!("⏳ Tour {}", entry.turn)).underline());
                                                        }

                                                        let color = if entry.is_indirect { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                                        let tag = if entry.is_indirect { " (indirect)" } else { "" };
                                                        ui.label(egui::RichText::new(format!("   - {}{} : +{} PV {:?}", entry.spell, tag, entry.total, entry.hits)).color(color));
                                                        turn_total += entry.total;
                                                    }

                                                    ui.label(egui::RichText::new(format!("   Total Tour : {} PV", turn_total)).strong().color(egui::Color32::LIGHT_BLUE));
                                                    ui.add_space(5.0);

                                                    egui::CollapsingHeader::new(
                                                        egui::RichText::new("📊 Récapitulatif par sort")
                                                            .italics()
                                                            .color(egui::Color32::GOLD)
                                                    )
                                                    .default_open(false)
                                                    .show(ui, |ui| {
                                                        let mut sorted_spells: Vec<_> = spell_totals.into_iter().collect();
                                                        sorted_spells.sort_by(|a, b| b.1.0.cmp(&a.1.0));

                                                        for (spell_name, (total, is_ind)) in sorted_spells {
                                                            let color = if is_ind { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                                            ui.label(egui::RichText::new(format!("   Total {} : {} PV", spell_name, total)).color(color));
                                                        }
                                                    });
                                                    ui.add_space(10.0);
                                                }
                                            });
                                    });
                                });
                            };

                            if s.config.history_horizontal {
                                ui.horizontal(|ui| {
                                    for player in sorted_players {
                                        render_player(ui, &player);
                                        ui.add_space(15.0);
                                    }
                                });
                            } else {
                                for player in sorted_players {
                                    render_player(ui, &player);
                                    ui.add_space(15.0);
                                }
                            }
                        });
                        ui.add_space(10.0);
                    }

                    // =================== 5. BANDEAU INFO ET STATS DE TOUR ===================
                    let expected = (s.players_to_include - s.ko_players.len() as i32).max(1);

                    ui.horizontal(|ui| {
                        ui.label(format!("⏳ Tour: {}", s.current_turn));
                        ui.separator();
                        ui.label(format!("💀 KO: {}", s.ko_players.len()));
                        ui.label(format!("(Vus: {}/{})", s.tour_reports_seen, expected));
                    });

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);

                    // =================== 6. STATISTIQUES GLOBAL / DÉGÂTS GLOBALS ===================
                    ui.label(egui::RichText::new("📊 Statistiques :").strong());
                    
                    let mut sorted_players: Vec<_> = s.visible_players.iter().collect();
                    sorted_players.sort_by(|a, b| s.total_damage.get(*b).unwrap_or(&0).cmp(s.total_damage.get(*a).unwrap_or(&0)));

                    if sorted_players.is_empty() {
                        ui.label("Aucune donnée de dégâts enregistrée pour le moment.");
                    } else {
                        // Un conteneur défilant de hauteur maximale fixe pour éviter de pousser l'action directe hors de l'écran
                        egui::ScrollArea::vertical()
                            .id_source("damage_list_scroll")
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for player in sorted_players {
                                    let dmg = s.total_damage.get(player).unwrap_or(&0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("• {}:", player)).strong());
                                        ui.label(egui::RichText::new(format!("{} PV", dmg)).color(egui::Color32::LIGHT_GREEN));
                                    });
                                }
                            });
                    }

                    ui.add_space(10.0);

                    // =================== 7. ENCART : ACTION EN DIRECT ===================
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());

                        // --- 1. RÉCUPÉRATION ET TRI CHRONOLOGIQUE DES ACTIONS ---
                        let mut all_actions = Vec::new();
                        for (player, entries) in s.history.iter() {
                            for entry in entries {
                                all_actions.push((player.clone(), entry.clone()));
                            }
                        }

                        // On trie les actions par numéro de tour (ou ordre d'apparition)
                        all_actions.sort_by(|a, b| a.1.turn.cmp(&b.1.turn));

                        // On extrait les deux dernières actions
                        let latest_action = all_actions.last().cloned();
                        let previous_action = if all_actions.len() >= 2 {
                            all_actions.get(all_actions.len() - 2).cloned()
                        } else {
                            None
                        };

                        // --- 2. AFFICHAGE DE L'ACTION EN COURS (La plus récente) ---
                        ui.label(egui::RichText::new("⚡ ACTION EN DIRECT").strong().color(egui::Color32::LIGHT_RED));
                        ui.separator();

                        if let Some((caster, entry)) = latest_action {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&caster).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new("lance").color(egui::Color32::GRAY));
                                    // Nom du sort
                                    ui.label(egui::RichText::new(&entry.spell)
                                        .strong()
                                        .color(egui::Color32::from_rgb(255, 221, 107)));
                                });

                                ui.add_space(5.0);

                                // Affichage des dégâts principaux
                                ui.label(egui::RichText::new(format!("💥 {} PV", entry.total))
                                    .size(26.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(255, 221, 107)));

                                ui.label(egui::RichText::new(format!("{:?}", entry.hits))
                                    .small()
                                    .color(egui::Color32::GRAY));
                            });
                        } else {
                            ui.label(egui::RichText::new("En attente d'une action...").small().italics().color(egui::Color32::GRAY));
                        }

                        ui.add_space(10.0);
                        ui.separator();

                        // --- 3. AFFICHAGE DE L'AVANT-DERNIÈRE ACTION ---
                        if let Some((prev_caster, prev_entry)) = previous_action {
                            ui.label(egui::RichText::new(format!("Dernière action : {} - {} ({} PV)", prev_caster, prev_entry.spell, prev_entry.total))
                                .small()
                                .italics()
                                .color(egui::Color32::GRAY));
                        } else {
                            ui.label(egui::RichText::new("Aucune action précédente").small().italics().color(egui::Color32::GRAY));
                        }
                    });

                });
            });
    });
}
