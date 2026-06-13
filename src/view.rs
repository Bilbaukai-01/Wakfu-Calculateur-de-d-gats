use eframe::egui;
use crate::model::{AppState, Tab};
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

     // FORCAGE DU FOND TRANSPARENT
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

        // --- 1. APPLICATION DU ZOOM ET DU RESET (UNIQUEMENT QUAND LE ZOOM CHANGE) ---
        let zoom_changed = (s.config.zoom_factor - s.last_applied_zoom).abs() > 0.01;
        
        if zoom_changed {
            // Applique le zoom
            ctx.set_zoom_factor(s.config.zoom_factor);
            
            // Applique la taille de fenêtre (C'est ici que le 500x650 est envoyé au système)
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(s.config.window_width, s.config.window_height)));
            
            s.last_applied_zoom = s.config.zoom_factor;
        }

        // --- 2. SAUVEGARDE DE LA TAILLE MANUELLE (SANS FORCER LA TAILLE) ---
        // On ne sauvegarde que si l'utilisateur a manuellement étiré la fenêtre
        if !s.is_compact && !zoom_changed {
            let current_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
            
            if let Some(size) = current_size {
                // On vérifie si l'utilisateur a étiré la fenêtre à la souris
                // Seuil suffisamment élevé pour ignorer les changements dus au DPI lors du changement d'écran
                if size.x > 250.0 && size.y > 350.0 {
                    if (size.x - s.config.window_width).abs() > 20.0 || (size.y - s.config.window_height).abs() > 20.0 {
                        s.config.window_width = size.x;
                        s.config.window_height = size.y;
                        crate::persistence::save_config(&s.config);
                    }
                }
            }
        }

        let window_level = if s.config.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(window_level));

        let img_logo = egui::include_image!("../logo.png");
        let img_reduce = egui::include_image!("../reduction_window.png");

        let mut visuals = egui::Visuals::dark();
        
        // On récupère la couleur depuis la config de l'utilisateur
        let r = s.config.text_color[0];
        let g = s.config.text_color[1];
        let b = s.config.text_color[2];
        let ma_couleur_globale = egui::Color32::from_rgb(r, g, b); 

        // On applique cette couleur partout sur les textes par défaut
        visuals.override_text_color = Some(ma_couleur_globale);

        // On l'applique aussi aux flèches, cases à cocher et sliders
        visuals.widgets.noninteractive.fg_stroke.color = ma_couleur_globale;
        visuals.widgets.inactive.fg_stroke.color = ma_couleur_globale;
        visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE; // Blanc au survol
        visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        
        visuals.panel_fill = egui::Color32::TRANSPARENT; 
        ctx.set_visuals(visuals);

                egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let rect = ui.max_rect();
            let mut mesh = egui::Mesh::default();
            
            // On applique l'opacité utilisateur (s.config.opacity) aux deux couleurs du dégradé de fond !
            let color_top = egui::Color32::BLACK.linear_multiply(s.config.opacity);
            let color_bottom = egui::Color32::from_rgb(79, 0, 111).linear_multiply(s.config.opacity);
            
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
                if ui.selectable_label(s.current_tab == Tab::Aide, "ℹ Aide").clicked() { s.current_tab = Tab::Aide; }
                if ui.selectable_label(s.current_tab == Tab::Stats, "📊 Stats").clicked() { s.current_tab = Tab::Stats; }
                if ui.selectable_label(s.current_tab == Tab::Combats, "🕒 Combats").clicked() { s.current_tab = Tab::Combats; }
                if ui.selectable_label(s.current_tab == Tab::Archivage, "📜 Archivage").clicked() {s.current_tab = Tab::Archivage;}
                if ui.selectable_label(s.current_tab == Tab::Settings, "⚙ Réglages").clicked() { s.current_tab = Tab::Settings; }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("main_scroll_area")
                .auto_shrink([false; 2]) // Empêche des comportements de redimensionnement bizarres
                .show(ui, |ui| {

            match s.current_tab {

                // ==================================================================
                // ONGLET : AIDE & TUTO (NOUVEAU)
                // ==================================================================
                Tab::Aide => {
                    // On applique une marge de 10.0 pixels à gauche et à droite pour que le texte ne colle pas aux bords
                    egui::Frame::none()
                        .inner_margin(egui::Margin {
                            left: 10.0,
                            right: 10.0,
                            top: 0.0,
                            bottom: 0.0,
                        })
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0);
                                ui.heading("ℹ Comment fonctionne l'application ?");
                                ui.add_space(10.0);

                                // --- SECTION INTRODUCTION (Sans bloc/groupe physique) ---
                                ui.label(egui::RichText::new("Bienvenue sur le Calculateur de Dégâts Wakfu par Bilbaukai !").strong());
                                ui.label(
                                    "Cet outil analyse en temps réel les fichiers de logs générés par le jeu afin de calculer avec précision les dégâts que vous faites. \
                                    Que ce soit pour vérifier votre puissance avec votre nouveau build, tester des combos de sorts pour trouver la meilleure configuration \
                                    ou savoir qui tape le plus fort en combat, les possibilités sont infinies."
                                );
                                ui.label("Voici les étapes rapides pour commencer à l'utiliser :");
                                ui.add_space(15.0);

                                // --- ÉTAPE 1 ---
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(egui::RichText::new("🚀 Étape 1 : Configuration initiale").strong());
                                    ui.add_space(5.0);
                                    ui.label("Allez dans l'onglet ⚙ Réglages pour :");
                                    
                                    // Utilisation de r"..." pour que Rust accepte les \ du chemin Windows sans erreur
                                    ui.label(r"• Spécifier le chemin de votre fichier de log Wakfu. C'est ce lien qui lira vos informations du jeu en direct, absolument indispensable pour fonctionner. Le fichier devrait se trouver à cet endroit : C:\Utilisateurs\Nom de votre ordinateur\AppData\Roaming\zaap\gamesLogs\wakfu\logs");
                                    
                                    ui.label("• Ajouter la liste des personnages que vous souhaitez suivre. Vous pouvez ajouter autant de personnages que vous voulez, pas besoin qu'ils soient dans votre groupe, l'application reconnaîtra automatiquement les personnages à suivre.");
                                    ui.add_space(3.0);
                                    ui.label(egui::RichText::new("💡 N'oubliez pas d'enregistrer vos réglages !").italics().color(egui::Color32::LIGHT_BLUE));
                                });

                                ui.add_space(10.0);

                                // --- ÉTAPE 2 ---
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(egui::RichText::new("📊 Étape 2 : Mode de jeu").strong());
                                    ui.add_space(5.0);
                                    ui.label("Dans l'onglet 📊 Stats, choisissez votre mode de jeu :");
                                    ui.label("• Mono-compte : Analyse les données avec une seule fenêtre du jeu ouverte (de 1 à 6 joueurs).");
                                    ui.label("• Multi-compte : Analyse les données avec deux fenêtres du jeu ouvertes (de 2 à 6 joueurs).");
                                });

                                ui.add_space(10.0);

                                // --- ÉTAPE 3 ---
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(egui::RichText::new("🕒 Étape 3 : Historique & Archivage").strong());
                                    ui.add_space(5.0);
                                    ui.label("• Combats : Visualisez vos récents combats mémorisés temporairement.");
                                    ui.label("• Archivage : Sauvegardez vos combats de manière permanente pour ne jamais les perdre, même quand vous fermez l'application.");
                                });

                                ui.add_space(15.0);
                                ui.centered_and_justified(|ui| {
                                    ui.label(egui::RichText::new("Bon jeu sur Wakfu ! ⚔").strong().size(14.0));
                                });
                            });
                        });
                }

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

                        if s.showing_history {
                            ui.separator();
                            ui.label("Disposition :");
                            // Utilisation de boutons radio sélectionnables
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

                    // On n'affiche la ScrollArea de l'historique QUE si showing_history est actif
                    if s.showing_history {
                        // En mode horizontal, on permet le scroll horizontal. En mode vertical, uniquement vertical.
                        let scroll_area = if s.config.history_horizontal {
                            egui::ScrollArea::both()
                        } else {
                            egui::ScrollArea::vertical()
                        };

                        scroll_area.id_source("history_scroll").show(ui, |ui| {
                            let mut sorted_players: Vec<_> = s.history.keys().cloned().collect();
                            sorted_players.sort_by(|a, b| s.total_damage.get(b).unwrap_or(&0).cmp(s.total_damage.get(a).unwrap_or(&0)));

                            // Extraction locale pour éviter l'erreur de borrow-check (double emprunt de `s`)
                            let history = &s.history;
                            let total_damage = &s.total_damage;

                            // Macro/Closure locale pour dessiner un personnage
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

                            // On dessine avec la disposition choisie
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
                    } // Fin de s.showing_history

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

            // --- 2. AFFICHAGE DE L'ACTION EN COURS (La plus récente : "B") ---
            ui.label(egui::RichText::new("⚡ ACTION EN DIRECT").strong().color(egui::Color32::LIGHT_RED));
            ui.separator();

            if let Some((caster, entry)) = latest_action {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&caster).strong().color(egui::Color32::WHITE));
                        ui.label(egui::RichText::new("lance").color(egui::Color32::GRAY));
                        // Nom du sort avec la couleur RGB (255, 198, 255)
                        ui.label(egui::RichText::new(&entry.spell)
                            .strong()
                            .color(egui::Color32::from_rgb(255, 221, 107)));
                    });

                    ui.add_space(5.0);
                    
                    // Affichage des dégâts avec la couleur RGB (255, 198, 255)
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

            // --- 3. AFFICHAGE DE LA DERNIÈRE ACTION (L'avant-dernière : "A") ---
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


            } // Fin de Tab::Stats

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
                                                // --- ÉTAPE 1 : CALCUL ET CUMUL DE TOUTES LES DONNÉES ---
                                                let mut spell_totals: HashMap<String, (i32, bool)> = HashMap::new();
                                                let mut turns_data: Vec<(i32, Vec<(String, i32, bool)>, i32)> = Vec::new();
                                                
                                                let mut current_t = -1;
                                                let mut turn_spells = Vec::new();
                                                let mut turn_total = 0;

                                                for entry in entries {
                                                    let entry_key = if entry.is_indirect { format!("{} (indirect)", entry.spell) } else { entry.spell.clone() };
                                                    
                                                    // Cumul pour le récapitulatif global du joueur
                                                    let stat = spell_totals.entry(entry_key.clone()).or_insert((0, entry.is_indirect));
                                                    stat.0 += entry.total;

                                                    // Structuration par Tour pour le fil chronologique
                                                    if entry.turn != current_t {
                                                        if current_t != -1 {
                                                            turns_data.push((current_t, turn_spells, turn_total));
                                                        }
                                                        current_t = entry.turn;
                                                        turn_spells = Vec::new();
                                                        turn_total = 0;
                                                    }
                                                    turn_spells.push((entry.spell.clone(), entry.total, entry.is_indirect));
                                                    turn_total += entry.total;
                                                }
                                                // Ne pas oublier le dernier tour traité
                                                if current_t != -1 {
                                                    turns_data.push((current_t, turn_spells, turn_total));
                                                }

                                                // --- ÉTAPE 2 : AFFICHAGE CHRONOLOGIQUE DES TOURS ---
                                                for (turn_num, spells, total_p_tour) in turns_data {
                                                    ui.label(egui::RichText::new(format!("⏳ Tour {}", turn_num)).underline());
                                                    for (spell_name, val, is_indirect) in spells {
                                                        let color = if is_indirect { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::WHITE };
                                                        ui.add(egui::Label::new(egui::RichText::new(format!("   - {} : +{} PV", spell_name, val)).color(color)).wrap(false));
                                                    }
                                                    ui.label(egui::RichText::new(format!("   Total Tour : {} PV", total_p_tour)).small().color(egui::Color32::LIGHT_BLUE));
                                                }

                                                ui.add_space(5.0);
                                                ui.label(egui::RichText::new("📊 Récapitulatif :").italics().color(egui::Color32::GOLD));

                                                // --- ÉTAPE 3 : TRI ULTRA-STABLE UNIVERSEL (ZÉRO FLICKERING GARANTI) ---
                                                        let mut sorted_spells: Vec<_> = spell_totals.into_iter().collect();
                                                        sorted_spells.sort_by(|a, b| {
                                                            // 1. Critère majeur : Les dégâts cumulés (décroissant)
                                                            let cmp_damage = b.1.0.cmp(&a.1.0);
                                                            if cmp_damage != std::cmp::Ordering::Equal {
                                                                return cmp_damage;
                                                            }

                                                            // 2. Critère secondaire : Est-ce indirect ? (On met les directs en premier)
                                                            let cmp_indirect = a.1.1.cmp(&b.1.1);
                                                            if cmp_indirect != std::cmp::Ordering::Equal {
                                                                return cmp_indirect;
                                                            }

                                                            // 3. Critère tertiaire : Normalisation Unicode & Suppression complète des accents
                                                            let clean_string = |s: &str| -> String {
                                                                s.to_lowercase()
                                                                    .chars()
                                                                    .map(|c| {
                                                                        // Remplacement manuel des ligatures courantes et cas spécifiques
                                                                        match c {
                                                                            'œ' => "oe".to_string(),
                                                                            'æ' => "ae".to_string(),
                                                                            _ => {
                                                                                // Décomposition de tous les caractères accentués (e.g. 'ï' -> 'i' + '¨')
                                                                                // On ne conserve que le caractère de base (celui qui est ASCII / alphabétique)
                                                                                let mut base_char = String::new();
                                                                                for decomposed in c.to_string().chars() {
                                                                                    // On ignore les marques de diacritiques (accents) combinatoires
                                                                                    if !('\u{0300}'..='\u{036F}').contains(&decomposed) {
                                                                                        base_char.push(decomposed);
                                                                                    }
                                                                                }
                                                                                base_char
                                                                            }
                                                                        }
                                                                    })
                                                                    .collect::<Vec<String>>()
                                                                    .join("")
                                                            };

                                                            let a_clean = clean_string(&a.0);
                                                            let b_clean = clean_string(&b.0);

                                                            a_clean.cmp(&b_clean)
                                                        });

                                                // --- ÉTAPE 4 : AFFICHAGE DU RÉCAPITULATIF TRIÉ ---
                                                for (spell_name, (total, is_indirect)) in sorted_spells {
                                                    let color = if is_indirect { egui::Color32::from_rgb(170, 100, 250) } else { egui::Color32::LIGHT_GREEN };
                                                    ui.label(egui::RichText::new(format!("   ⭐ {} : {} PV", spell_name, total)).color(color));
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
                                ui.heading("🔧 Configuration");
                                ui.add_space(10.0);

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

                                ui.add_space(15.0);
                                ui.heading("👥 Personnages suivis");
                                
                                let mut to_remove = None;
                                let mut to_swap = None;
                                let players_count = s.config.tracked_players.len();

                                ui.vertical(|ui| {
                                    // Utilisation d'un menu déroulant (CollapsingHeader) pour masquer/afficher la gestion des personnages
                                    egui::CollapsingHeader::new(format!("Gérer les personnages suivis ({})", players_count))
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            ui.add_space(5.0);
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

                                            ui.add_space(5.0);
                                            if ui.button("➕ Ajouter un personnage").clicked() {
                                                s.config.tracked_players.push("Nouveau".to_string());
                                                crate::persistence::save_config(&s.config);
                                            }
                                        });

                                    // Application des changements après la boucle
                                    if let Some((i, j)) = to_swap {
                                        s.config.tracked_players.swap(i, j);
                                        crate::persistence::save_config(&s.config);
                                    }
                                    if let Some(i) = to_remove {
                                        s.config.tracked_players.remove(i);
                                        crate::persistence::save_config(&s.config);
                                    }
                                });
                            }); // <--- FERMETURE DU GROUP "Configuration"

                            ui.add_space(15.0); // Espace physique entre les deux blocs
                            
                            // On commence le grand bloc "Interface"
                            ui.group(|ui| {
                                ui.heading("🖥 Interface");
                                ui.add_space(10.0);

                                // --- ZOOM UI (Placé au tout début du bloc Interface) ---
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
                                            .clamp_range(0.5..=2.0)
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

                                // =================== OPACITÉ ===================
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.label("Opacité : ");
                                    let mut opacity_percentage = (s.config.opacity * 100.0) as i32;
                                    let res = ui.add(
                                        egui::Slider::new(&mut opacity_percentage, 0..=100)
                                            .suffix("%")
                                    );
                                    if res.changed() {
                                        s.config.opacity = (opacity_percentage as f32) / 100.0;
                                        crate::persistence::save_config(&s.config);
                                        ui.ctx().request_repaint();
                                    }
                                });

                                // ==================== CHOIX COULEUR =====================
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.label("Couleur des textes :");

                                    // On crée un Color32 temporaire à partir de notre config RGB [u8; 3]
                                    let mut color_tmp = egui::Color32::from_rgb(
                                        s.config.text_color[0],
                                        s.config.text_color[1],
                                        s.config.text_color[2],
                                    );

                                    // On affiche le bouton d'édition de couleur.
                                    // S'il change, on met à jour la configuration et on l'enregistre en JSON.
                                    if ui.color_edit_button_srgba(&mut color_tmp).changed() {
                                        s.config.text_color = [color_tmp.r(), color_tmp.g(), color_tmp.b()];
                                        crate::persistence::save_config(&s.config);
                                    }
                                });

                                // =================== OVERLAY ===================
                                ui.add_space(10.0); // Petit espace avant l'overlay
                                ui.horizontal(|ui| {
                                    toggle_ui(ui, &mut s.config.always_on_top);
                                    ui.label("Overlay");
                                });
                            }); // <--- FERMETURE DU GROUP "Interface" (Exclut les boutons ci-dessous)

                            // =================== BOUTONS DE BAS DE PAGE (HORS GROUPES) ===================
                            ui.add_space(15.0);
                            ui.horizontal(|ui| {
                                // Bouton Paramètres par défaut
                                let reset_btn = egui::Button::new(
                                    egui::RichText::new("⚙ Paramètres par défaut")
                                ).min_size(egui::vec2(120.0, 30.0));

                                if ui.add(reset_btn).on_hover_text("Réinitialise l'application aux réglages d'origine").clicked() {
                                    s.reset_to_defaults();
                                }

                                ui.add_space(10.0);

                                // Bouton Enregistrer les paramètres
                                let save_btn = egui::Button::new(
                                    egui::RichText::new("💾 Enregistrer les paramètres")
                                ).min_size(egui::vec2(120.0, 30.0));

                                if ui.add(save_btn).on_hover_text("Sauvegarde immédiatement toute la configuration").clicked() {
                                    crate::persistence::save_config(&s.config);
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

                                    // 1. PRIORITÉ : Si la mise à jour est prête, on affiche TOUJOURS le bouton vert de Redémarrage !
                                    if s.config.maj_prete_pour_redemarrage {
                                        let redemarrer_btn = egui::Button::new("✨ Redémarrer pour appliquer la mise à jour")
                                            .fill(egui::Color32::from_rgb(34, 139, 34)); // Joli bouton vert forêt

                                        if ui.add(redemarrer_btn).clicked() {
                                            use std::process::Command;
                                            use std::env;
                                            // Récupère le chemin du .exe actuel (qui a été remplacé sur le disque)
                                            if let Ok(chemin_exe) = env::current_exe() {
                                                // Relance l'application
                                                let _ = Command::new(chemin_exe).spawn();
                                            }
                                            // Ferme la version actuelle
                                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                        }
                                    } 
                                    // 2. Si une mise à jour est disponible mais pas encore téléchargée
                                    else if s.config.maj_disponible {
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
                                                            s.config.maj_prete_pour_redemarrage = true; // On l'active ici !
                                                            crate::persistence::save_config(&s.config); // On sauvegarde l'état
                                                            ctx.request_repaint(); // Force le rafraîchissement immédiat de l'écran
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
                                    // 3. Sinon, on affiche le bouton standard pour vérifier manuellement
                                    else {
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
            ui.add_space(20.0); 
        });
// ================================= FOOTER FIXE ===================================

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
