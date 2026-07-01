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
    let mut changed = false;
    let mut to_delete = None;

    egui::Frame::none()
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Archives Permanentes");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Tout effacer").clicked() {
                            s.permanent_archives.clear();
                            changed = true;
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().id_source("permanent_scroll").show(ui, |ui| {
                    if s.permanent_archives.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.label("Aucun combat archivé sur le disque.");
                        });
                    } else {
                        // On doit pouvoir modifier les noms, donc on passe par des indices mutables
                        let mut permanent_archives = s.permanent_archives.clone();
                        
                        for (idx, archive) in permanent_archives.iter_mut().enumerate() {
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        // Champ éditable pour renommer directement le titre du combat
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Nom :").small().color(egui::Color32::GRAY));
                                            let response = ui.add(
                                                egui::TextEdit::singleline(&mut archive.name)
                                                    .font(egui::TextStyle::Body) // <-- CORRIGÉ : .font() au lieu de .text_style()
                                                    .desired_width(200.0)
                                            );
                                            if response.changed() {
                                                changed = true;
                                            }
                                        });
                                        
                                        ui.label(format!("Membres : {}", archive.players_involved));
                                        ui.label(format!("Nombre de tours : {}", archive.total_turns));
                                    });
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Supprimer").clicked() {
                                            to_delete = Some(idx);
                                        }
                                    });
                                });

                                ui.separator();

                                // Accordéon pour voir le détail exact
                                ui.collapsing("Afficher le détail complet du combat", |ui| {
                                    for (player, entries) in &archive.details_history {
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
                            });
                            ui.add_space(10.0);
                        }
                        
                        // Réinjecte les archives potentiellement renommées
                        s.permanent_archives = permanent_archives;
                    }
                });
            });
        });

    if let Some(idx) = to_delete {
        s.permanent_archives.remove(idx);
        changed = true;
    }

    if changed {
        s.config.permanent_archives = s.permanent_archives.clone();
        crate::persistence::save_config(&s.config);
    }
}
