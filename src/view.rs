use eframe::egui;
use crate::model::{AppState, Tab, SpellEntry};
use std::collections::HashMap;
use chrono::Local;

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
    
    // Couleur de fond (Rouge si Off, Vert si On)
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
// MAIN APPLICATION (MyApp)
// ==================================================================================

pub struct MyApp {
    pub state: std::sync::Arc<std::sync::Mutex<AppState>>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx); 
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let mut s = match self.state.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        // --- 1. APPLICATION DU ZOOM ET DU RESET (PRIORITAIRE) ---
        let applying_change = (s.config.zoom_factor - s.last_applied_zoom).abs() > 0.01;
        
        if applying_change {
            // Applique le zoom
            ctx.set_zoom_factor(s.config.zoom_factor);
            
            // Applique la taille de fenêtre (C'est ici que le 500x650 est envoyé au système)
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(s.config.window_width, s.config.window_height)));
            
            s.last_applied_zoom = s.config.zoom_factor;
        }

        // --- 2. SAUVEGARDE DE LA TAILLE MANUELLE ---
        // On ne sauvegarde que si on n'est pas en train d'appliquer un Reset/Zoom
        if !s.is_compact && !applying_change {
            let current_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
            
            if let Some(size) = current_size {
                if size.x > 250.0 && size.y > 350.0 {
                    // On vérifie si l'utilisateur a étiré la fenêtre à la souris
                    if (size.x - s.config.window_width).abs() > 10.0 || (size.y - s.config.window_height).abs() > 10.0 {
                        s.config.window_width = size.x;
                        s.config.window_height = size.y;
                        crate::persistence::save_config(&s.config);
                    }
                }
            }
        }

        if (s.config.zoom_factor - s.last_applied_zoom).abs() > 0.01 {
            // Applique le zoom interne
            ctx.set_zoom_factor(s.config.zoom_factor);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(s.config.window_width, s.config.window_height)));
            s.last_applied_zoom = s.config.zoom_factor;
        }

        let window_level = if s.config.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(window_level));

        let img_logo = egui::include_image!("../logo.png");
        let img_reduce = egui::include_image!("../reduction_window.png");

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::TRANSPARENT; 
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let rect = ui.max_rect();
            let mut mesh = egui::Mesh::default();
            let color_top = egui::Color32::BLACK;
            let color_bottom = egui::Color32::from_rgb(79, 0, 111);
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.left_top(), uv: egui::Pos2::ZERO, color: color_top });
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.right_top(), uv: egui::Pos2::ZERO, color: color_top });
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.right_bottom(), uv: egui::Pos2::ZERO, color: color_bottom });
            mesh.vertices.push(egui::epaint::Vertex { pos: rect.left_bottom(), uv: egui::Pos2::ZERO, color: color_bottom });
            mesh.indices.extend([0, 1, 2, 0, 2, 3]);
            ui.painter().add(egui::Shape::mesh(mesh));

            ui.add_space(5.0);
            if s.is_compact {
                ui.centered_and_justified(|ui| {
                    if ui.add(egui::ImageButton::new(egui::Image::new(img_logo).max_width(45.0))).clicked() {
    s.is_compact = false;
    // On utilise les dimensions sauvegardées dans la config !
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(s.config.window_width, s.config.window_height)));
}
                });
                return;
            }

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
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                if ui.selectable_label(s.current_tab == Tab::Stats, "📊 Stats").clicked() { s.current_tab = Tab::Stats; }
                if ui.selectable_label(s.current_tab == Tab::Combats, "🕒 Combats").clicked() { s.current_tab = Tab::Combats; }
                if ui.selectable_label(s.current_tab == Tab::Archivage, "📜 Archivage").clicked() {s.current_tab = Tab::Archivage;}
                if ui.selectable_label(s.current_tab == Tab::Settings, "⚙ Réglages").clicked() { s.current_tab = Tab::Settings; }
            });

            ui.separator();

            match s.current_tab {
                Tab::Stats => {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        // --- GAUCHE : Sélection du mode ---
                        if ui.selectable_label(s.mode == "mono", "Mono").clicked() { 
                            s.mode = "mono".to_string(); 
                            if s.players_to_include > 6 { s.players_to_include = 6; }
                            s.full_reset(); 
                            crate::persistence::save_config(&s.config); // On sauvegarde
                        }
                        if ui.selectable_label(s.mode == "multi", "Multi").clicked() { 
                            s.mode = "multi".to_string(); 
                            if s.players_to_include < 2 { s.players_to_include = 2; }
                            s.full_reset(); 
                            crate::persistence::save_config(&s.config); // On sauvegarde
                        }

                        // --- DROITE : Boutons d'action ---
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(10.0);
                            if ui.button("🗑 Reset").clicked() { s.full_reset(); }
                            if ui.button("💾 Sauvegarder & Reset").clicked() { s.save_current_combat(); }
                        });
                    });

                ui.horizontal(|ui| {
    ui.add_space(10.0); // <--- L'espace AVANT le rectangle (à l'extérieur)
    
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("👥 Joueurs :");
            let range = if s.mode == "multi" { 2..=6 } else { 1..=6 };
            for i in range {
                if ui.selectable_label(s.players_to_include == i, i.to_string()).clicked() {
                    s.players_to_include = i; s.full_reset(); s.has_started = true;
                    crate::persistence::save_config(&s.config); // On sauvegarde
                }
            }
        });
    });
});

                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        toggle_ui(ui, &mut s.showing_history);
                        ui.label("Afficher l'historique");
                    });

                    if s.showing_history {
    egui::ScrollArea::vertical().id_source("history_scroll").show(ui, |ui| {
        let mut sorted_players: Vec<_> = s.history.keys().cloned().collect();
        sorted_players.sort_by(|a, b| s.total_damage.get(b).unwrap_or(&0).cmp(s.total_damage.get(a).unwrap_or(&0)));

        for player in sorted_players {
            egui::CollapsingHeader::new(format!("{} (Total: {} PV)", player, s.total_damage.get(&player).unwrap_or(&0)))
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(entries) = s.history.get(&player) {
                        let mut spell_totals: HashMap<String, (i32, bool)> = HashMap::new();
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
                        
                        // Dernier total de tour pour le joueur
                        ui.label(egui::RichText::new(format!("   Total Tour : {} PV", turn_total)).strong().color(egui::Color32::LIGHT_BLUE));
                        
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("📊 Récapitulatif par sort :").italics().color(egui::Color32::GOLD));
                        
                        // Tri des sorts pour éviter le clignotement dans l'historique aussi
                        let mut sorted_spells: Vec<_> = spell_totals.into_iter().collect();
                        sorted_spells.sort_by(|a, b| b.1.0.cmp(&a.1.0));
                        
                        for (spell_name, (total, is_ind)) in sorted_spells {
                            let color = if is_ind { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                            ui.label(egui::RichText::new(format!("   Total {} : {} PV", spell_name, total)).color(color));
                        }
                        ui.add_space(10.0);
                    }
                });
        }
    });
} else {
    // =================== VUE LIVE (SANS HISTORIQUE) ===================
    let expected = (s.players_to_include - s.ko_players.len() as i32).max(1);
    
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(format!("⏳ Tour: {}", s.current_turn));
        ui.separator();
        ui.label(format!("💀 KO: {}", s.ko_players.len()));
        ui.label(format!("(Vus: {}/{})", s.tour_reports_seen, expected));
    });
    
    ui.separator();

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.label("📊 Statistiques :");

            let mut sorted_players: Vec<_> = s.visible_players.iter().collect();
            sorted_players.sort_by(|a, b| s.total_damage.get(*b).unwrap_or(&0).cmp(s.total_damage.get(*a).unwrap_or(&0)));

            egui::ScrollArea::vertical().id_source("stats_scroll").show(ui, |ui| {
                for player in sorted_players {
                    let dmg = s.total_damage.get(player).unwrap_or(&0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("• {}:", player)).strong());
                        ui.label(egui::RichText::new(format!("{} PV", dmg)).color(egui::Color32::LIGHT_GREEN));
                    });
                }
            });
// =================== ENCART : ACTION EN DIRECT ===================
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.set_width(ui.available_width() - 5.0);
            
            // 1. On vérifie d'abord s'il y a des dégâts en train de tomber (current_hits)
            if !s.current_hits.is_empty() {
                ui.label(egui::RichText::new("⚡ ACTION EN DIRECT").strong().color(egui::Color32::LIGHT_RED));
                ui.separator();

                let caster = s.current_caster.as_deref().unwrap_or("Inconnu");
                let spell = s.current_spell.as_deref().unwrap_or("Sort...");
                let total_live: i32 = s.current_hits.iter().sum();

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(format!("👤 {}", caster)).strong());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("✨ {}", spell)).color(egui::Color32::WHITE));
                        ui.label(egui::RichText::new(format!("(x{})", s.current_hits.len())).small().italics());
                    });
                    
                    ui.add_space(5.0);
                    // Affichage du gros chiffre qui monte en temps réel
                    ui.label(egui::RichText::new(format!("💥 {} PV", total_live))
                        .size(26.0)
                        .strong()
                        .color(egui::Color32::WHITE));

                    ui.label(egui::RichText::new(format!("{:?}", s.current_hits))
                        .small()
                        .color(egui::Color32::GRAY));
                });
            } else {
                // 2. Si aucune action n'est en cours, on affiche la dernière action finie
                ui.label(egui::RichText::new("⚡Sort en cours:").strong().color(egui::Color32::DARK_GRAY));
                ui.separator();
                
                // --- CORRECTION DU TYPE ICI ---
                let mut last_entry: Option<&SpellEntry> = None; 
                // ------------------------------

                for entries in s.history.values() {
                    if let Some(last) = entries.last() {
                        if last_entry.is_none() || last.turn >= last_entry.unwrap().turn {
                            last_entry = Some(last);
                        }
                    }
                }

                if let Some(entry) = last_entry {
                    ui.label(egui::RichText::new(format!("Dernière action : {} ({} PV)", entry.spell, entry.total))
                        .small()
                        .italics()
                        .color(egui::Color32::GRAY));
                } else {
                    ui.label(egui::RichText::new("Aucun dégât détecté").small().italics().color(egui::Color32::GRAY));
                }
            }
        });
    });
});

                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                            ui.add_space(10.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("✨ États actifs :").color(egui::Color32::from_rgb(170, 100, 250)).strong());
                                if s.state_to_caster.is_empty() {
                                    ui.label(egui::RichText::new("Aucun état").small().italics());
                                } else {
                                    for (orig_name, caster, _) in s.state_to_caster.values() {
                                        ui.label(egui::RichText::new(format!("• {} ({})", orig_name, caster)).small().color(egui::Color32::LIGHT_GRAY));
                                    }
                                }
                            });
                        });
                    }
                }
                Tab::Combats => {
    ui.horizontal(|ui| {
        ui.heading("🕒 Historique de Session");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Tout effacer").clicked() { 
                s.archives.clear(); 
                s.combat_counter = 1; 
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical().id_source("archives_scroll").show(ui, |ui| {
        if s.archives.is_empty() { 
            ui.label("Aucun combat enregistré."); 
        }

        let mut to_delete = None;
        let mut to_archive_item = None; // 1. On prépare une variable temporaire

        for (idx, archive) in s.archives.iter().enumerate().rev() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(&archive.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Bouton Supprimer
                        if ui.button("❌").on_hover_text("Supprimer définitivement").clicked() { 
                            to_delete = Some(idx); 
                        }
                        // Bouton Archiver
                        if ui.button("📦").on_hover_text("Archiver ce combat").clicked() {
                            // 2. On fait juste une copie ici, on ne touche pas encore à 's'
                            to_archive_item = Some(archive.clone());
                        }
                    });
                });
                                
                                ui.label(format!("🕒 {} tours", archive.total_turns));
                                ui.label(egui::RichText::new(format!("👥 Participants: {}", archive.players_involved)).small().italics());
                                
                                ui.collapsing("Détails des dégâts", |ui| {
                                    for (player, total) in &archive.damages {
                                        ui.collapsing(format!("{}: {} PV", player, total), |ui| {
                                            if let Some(entries) = archive.details_history.get(player) {
                                                let mut spell_totals: HashMap<String, (i32, bool)> = HashMap::new();
                                                let mut current_t = -1;
                                                let mut turn_total = 0;
                                                for entry in entries {
                                                    let entry_key = if entry.is_indirect { format!("{} (indirect)", entry.spell) } else { entry.spell.clone() };
                                                    let stat = spell_totals.entry(entry_key).or_insert((0, entry.is_indirect));
                                                    stat.0 += entry.total;

                                                    if entry.turn != current_t {
                                                        if current_t != -1 { 
                                                            ui.label(egui::RichText::new(format!("   Total Tour : {} PV", turn_total)).small().color(egui::Color32::LIGHT_BLUE)); 
                                                        }
                                                        current_t = entry.turn; turn_total = 0;
                                                        ui.label(egui::RichText::new(format!("⏳ Tour {}", entry.turn)).underline());
                                                    }
                                                    let color = if entry.is_indirect { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                                    ui.add(egui::Label::new(egui::RichText::new(format!("   - {} : +{} PV", entry.spell, entry.total)).color(color)).wrap(false));
                                                    turn_total += entry.total;
                                                }
                                                ui.label(egui::RichText::new(format!("   Total Tour : {} PV", turn_total)).small().color(egui::Color32::LIGHT_BLUE));
                                                ui.add_space(5.0);
                                                ui.label(egui::RichText::new("📊 Récapitulatif :").italics().color(egui::Color32::GOLD));
                                                let mut sorted_spells: Vec<_> = spell_totals.into_iter().collect();
                                                sorted_spells.sort_by(|a, b| b.1.0.cmp(&a.1.0));
                                                for (name, (sum, is_ind)) in sorted_spells {
                                                    let color = if is_ind { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                                    ui.label(egui::RichText::new(format!("   Total {} : {} PV", name, sum)).color(color));
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                            ui.add_space(5.0);
                        }

                        // --- TRAITEMENT DES ACTIONS ---
                        if let Some(mut combat_to_copy) = to_archive_item {
            let date_str = Local::now().format("%d/%m/%Y - %H:%M").to_string();
            combat_to_copy.name = format!("Combat ({})", date_str);
            s.permanent_archives.push(combat_to_copy);
            }
                            if let Some(idx) = to_delete {
                            s.archives.remove(idx);
                        }
                    });
                }

                Tab::Archivage => {
                            let mut changed = false; // 1. On crée un marqueur pour savoir s'il faut sauvegarder

                                ui.horizontal(|ui| {
                                ui.heading("📜 Archives Permanentes");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                             if ui.button("🗑 Tout effacer").clicked() { 
                                s.permanent_archives.clear(); 
                                changed = true; // 2. Marquer le changement
                             }
                        });
                     });
                                ui.separator();

    egui::ScrollArea::vertical().id_source("permanent_scroll").show(ui, |ui| {
                        if s.permanent_archives.is_empty() {
                            ui.vertical_centered(|ui| { ui.label("Aucun combat archivé."); });
        }

        let mut to_delete = None;
                        for (idx, archive) in s.permanent_archives.iter_mut().enumerate().rev() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let res = ui.add(egui::TextEdit::singleline(&mut archive.name)
                                        .hint_text("Nom du combat...")
                                        .desired_width(300.0));
                                    if res.changed() { changed = true; }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("❌").clicked() { 
                                            to_delete = Some(idx); 
                                            changed = true;
                        }
                    });
                });

                ui.label(format!("🕒 {} tours", archive.total_turns));
                                ui.label(egui::RichText::new(format!("👥 Participants: {}", archive.players_involved)).small().italics());
                
                ui.collapsing("Détails des dégâts", |ui| {
                                    // Utilisation correcte de player et total pour enlever les warnings
                                    for (player, total) in &archive.damages {
                                        ui.label(format!("{}: {} PV", player, total));
                    }
                });
            });
            ui.add_space(5.0);
        }

        if let Some(i) = to_delete { 
            s.permanent_archives.remove(i); 
        }
    });
        // 5. SI QUELQUE CHOSE A CHANGÉ : On synchronise et on enregistre
        if changed {
                        s.config.permanent_archives = s.permanent_archives.clone();
                        crate::persistence::save_config(&s.config);
        }
    }

                Tab::Settings => {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0); // Le décalage de 10 vers la droite pour tout l'onglet

                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.heading("📁 Fichier de Log");

                                // Ligne 1 : Barre de texte + bouton parcourir
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    ui.text_edit_singleline(&mut s.config.log_path);

                                    if ui.button("📂").on_hover_text("Parcourir les fichiers...").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_title("Sélectionner le fichier de log Wakfu")
                                            .add_filter("Fichiers Log", &["log", "txt"])
                                            .pick_file() 
                                        {
                                            s.config.log_path = path.display().to_string();
                                            crate::persistence::save_config(&s.config);
                                        }
                                    }
                                });

                                // Ligne 2 : Bouton enregistrer en dessous
                                ui.add_space(2.0);
                                if ui.button("💾 Enregistrer le chemin").clicked() { 
                                    crate::persistence::save_config(&s.config); 
                                }

                                ui.add_space(15.0);
                                ui.heading("👥 Personnages suivis");
                                let mut to_remove = None;
                let mut to_swap = None; // Déclaré ICI pour être accessible
                let players_count = s.config.tracked_players.len();

                ui.vertical(|ui| {
                    for i in 0..players_count {
                        ui.horizontal(|ui| {
                            // Champ de texte pour le nom
                            ui.text_edit_singleline(&mut s.config.tracked_players[i]);

                            // Bouton Monter
                            let btn_up = egui::Button::image(egui::include_image!("../arrow_up.png"))
                                .rounding(3.0);
                            if ui.add_enabled(i > 0, btn_up).on_hover_text("Monter").clicked() {
                                to_swap = Some((i, i - 1));
                            }

                            // Bouton Descendre
                            let btn_down = egui::Button::image(egui::include_image!("../arrow_down.png"))
                                .rounding(3.0);
                            if ui.add_enabled(i < players_count - 1, btn_down).on_hover_text("Descendre").clicked() {
                                to_swap = Some((i, i + 1));
                            }

                            // Bouton Supprimer
                            if ui.button("❌").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }

                    // Application des changements après la boucle
                    if let Some((i, j)) = to_swap {
                        s.config.tracked_players.swap(i, j);
                        crate::persistence::save_config(&s.config);
                    }
                    if let Some(i) = to_remove {
                        s.config.tracked_players.remove(i);
                        crate::persistence::save_config(&s.config);
                    }

                    ui.add_space(5.0);
                    if ui.button("➕ Ajouter un personnage").clicked() {
                        s.config.tracked_players.push("Nouveau".to_string());
                        crate::persistence::save_config(&s.config);
                    }
                });

                                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label("🔍 Zoom UI :");

                    // Bouton Moins
                    if ui.button("➖").clicked() {
                        s.config.zoom_factor = (s.config.zoom_factor - 0.1).max(0.5);
                        crate::persistence::save_config(&s.config);
                    }

                    // Champ numérique (cliquable pour taper au clavier)
                    let res = ui.add(
                        egui::DragValue::new(&mut s.config.zoom_factor)
                            .clamp_range(0.5..=2.0) // Correction ici : clamp_range au lieu de range
                            .speed(0.01)
                            .fixed_decimals(2)
                            .suffix("x")
                    );
                    
                    if res.changed() {
                        crate::persistence::save_config(&s.config);
                    }

                    // Bouton Plus
                    if ui.button("➕").clicked() {
                        s.config.zoom_factor = (s.config.zoom_factor + 0.1).min(2.0);
                        crate::persistence::save_config(&s.config);
                    }
                });
// =================== OVERLAY ===================
                ui.add_space(10.0); // Petit espace avant l'overlay
                ui.horizontal(|ui| {
                    toggle_ui(ui, &mut s.config.always_on_top);
                    ui.label("Overlay");
                });
// =================== parametres par default ===================
                ui.add_space(20.0);

                let reset_btn = egui::Button::new(
                    egui::RichText::new("⚙ Paramètres par défaut")
                ).min_size(egui::vec2(120.0, 30.0));

                if ui.add(reset_btn).on_hover_text("Réinitialise l'application aux réglages d'origine").clicked() {
                    s.reset_to_defaults();
                }
                            });
                                // --- SECTION MISE À JOUR ---
                                ui.add_space(15.0);
                                ui.separator();
                                ui.add_space(5.0);

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Mise à jour de l'application :").strong());
                                        
                                        // Indicateur visuel discret du statut actuel
                                        ui.label(
                                            egui::RichText::new(format!("({})", s.config.statut_maj))
                                                .small()
                                                .color(egui::Color32::LIGHT_GRAY),
                                        );
                                    });

                                    ui.add_space(5.0);

                                    if s.config.maj_disponible {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(egui::Color32::from_rgb(250, 200, 50), "⚠️ Nouvelle version disponible !");
                                            
                                            let maj_btn = egui::Button::new(
                                                egui::RichText::new("📥 Installer la mise à jour")
                                                    .color(egui::Color32::BLACK)
                                            ).fill(egui::Color32::from_rgb(50, 200, 50));

                                            if ui.add(maj_btn).clicked() {
                                                s.config.statut_maj = "Téléchargement en cours...".to_string();
                                                
                                                // Appel à la fonction définie dans main.rs
                                                match crate::verifier_et_mettre_a_jour() {
                                                    Ok(status) => {
                                                        if status.updated() {
                                                            s.config.statut_maj = format!("Succès ! v{} installée. Redémarrez.", status.version());
                                                            s.config.maj_disponible = false;
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
                                    } else {
                                        // Bouton pour forcer manuellement une vérification si l'utilisateur le souhaite
                                        if ui.button("🔄 Vérifier manuellement").clicked() {
                                            s.config.statut_maj = "Vérification en cours...".to_string();
                                            
                                            // On réutilise la configuration de self_update pour vérifier
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
                }
            }
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
        }); // Fin du CentralPanel ou de la zone parente
    } // Fin de la fonction update
} // Fin de l'impl MyApp
