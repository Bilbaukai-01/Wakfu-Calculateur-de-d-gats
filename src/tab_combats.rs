use eframe::egui;
use crate::model::AppState;
use std::collections::HashMap;

// Helper interne pour nettoyer les accents et stabiliser le tri alphabétique
fn clean_accents(s: &str) -> String {
    let mut base_char = String::new();
    for c in s.chars() {
        for decomposed in c.to_string().chars() {
            if !('\u{0300}'..='\u{036F}').contains(&decomposed) {
                base_char.push(decomposed);
            }
        }
    }
    base_char.to_lowercase()
}

pub fn render(s: &mut AppState, ui: &mut egui::Ui) {
    let mut to_archive_idx = None;
    let mut to_delete_idx = None;

    egui::ScrollArea::vertical().id_source("combats_scroll").show(ui, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Session : Combats Sauvegardés");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !s.session_combats.is_empty() && ui.button("Tout effacer").clicked() {
                                s.session_combats.clear();
                            }
                        });
                    });
                    ui.separator();

                    if s.session_combats.is_empty() {
                        ui.label("Aucun combat sauvegardé dans cette session.");
                    } else {
                        for (idx, combat) in s.session_combats.iter().enumerate() {
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&combat.name).strong().size(15.0));
                                        ui.label(format!("Joueurs : {}", combat.players_involved));
                                        ui.label(format!("Tours totaux : {}", combat.total_turns));
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Archiver").clicked() {
                                            to_archive_idx = Some(idx);
                                        }
                                        if ui.button("Supprimer").clicked() {
                                            to_delete_idx = Some(idx);
                                        }
                                    });
                                });

                                ui.separator();

                                // Détails du combat par joueur (sans préfixes laids)
                                for (player, entries) in &combat.details_history {
                                    ui.collapsing(egui::RichText::new(player).strong(), |ui| {
                                        let mut turns_data: HashMap<i32, (Vec<(String, i32, bool)>, i32)> = HashMap::new();
                                        let mut spell_totals: HashMap<String, (i32, bool)> = HashMap::new();

                                        for entry in entries {
                                            let turn_bucket = turns_data.entry(entry.turn).or_insert_with(|| (Vec::new(), 0));
                                            turn_bucket.0.push((entry.spell.clone(), entry.total, entry.is_indirect));
                                            turn_bucket.1 += entry.total;

                                            let spell_bucket = spell_totals.entry(entry.spell.clone()).or_insert((0, entry.is_indirect));
                                            spell_bucket.0 += entry.total;
                                        }

                                        let mut sorted_turns: Vec<_> = turns_data.into_iter().collect();
                                        sorted_turns.sort_by_key(|k| k.0);
                                        for (turn_num, (spells, total_p_tour)) in sorted_turns {
                                            ui.label(egui::RichText::new(format!("Tour {}", turn_num)).underline());
                                            for (spell_name, val, is_indirect) in spells {
                                                let color = if is_indirect { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                                ui.add(egui::Label::new(egui::RichText::new(format!("   - {} : +{} PV", spell_name, val)).color(color)).wrap(false));
                                            }
                                            ui.label(egui::RichText::new(format!("   Total Tour : {} PV", total_p_tour)).small().color(egui::Color32::LIGHT_BLUE));
                                        }

                                        ui.add_space(5.0);
                                        ui.label(egui::RichText::new("Récapitulatif :").italics().color(egui::Color32::GOLD));
                                        
                                        let mut sorted_spells: Vec<_> = spell_totals.into_iter().collect();
                                        sorted_spells.sort_by(|a, b| {
                                            let cmp_damage = b.1.0.cmp(&a.1.0);
                                            if cmp_damage != std::cmp::Ordering::Equal {
                                                return cmp_damage;
                                            }
                                            let cmp_indirect = a.1.1.cmp(&b.1.1);
                                            if cmp_indirect != std::cmp::Ordering::Equal {
                                                return cmp_indirect;
                                            }
                                            clean_accents(&a.0).cmp(&clean_accents(&b.0))
                                        });

                                        for (spell_name, (total, is_indirect)) in sorted_spells {
                                            let color = if is_indirect { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                            ui.label(egui::RichText::new(format!("   * {} : {} PV", spell_name, total)).color(color));
                                        }
                                    });
                                }
                            });
                            ui.add_space(10.0);
                        }
                    }
                });
            });
    });

    if let Some(idx) = to_archive_idx {
        let combat_to_archive = s.session_combats[idx].clone();
        
        s.permanent_archives.push(combat_to_archive);
        s.config.permanent_archives = s.permanent_archives.clone();
        crate::persistence::save_config(&s.config);
        
        s.session_combats.remove(idx);
    }

    if let Some(idx) = to_delete_idx {
        s.session_combats.remove(idx);
    }
}
