#![windows_subsystem = "windows"]

mod model;
mod persistence;
mod parser;
mod view;

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use model::AppState;
use persistence::load_config;

fn main() {
    let config = load_config();
    let state = Arc::new(Mutex::new(AppState::new(config.clone())));
    let state_thread = Arc::clone(&state);

    let (re_ko, re_turn, re_cast, re_dmg, re_state_apply, re_dmg_with_state, re_summon, re_revive) = parser::create_regex_patterns();




    // --- THREAD DE LECTURE DU LOG ---
    thread::spawn(move || {
        let mut last_size = 0;
        let mut current_path = String::new();
        
        loop {
            let (path_to_open, tracked_players) = if let Ok(s) = state_thread.lock() {
                let filtered: Vec<String> = s.config.tracked_players.iter()
                    .filter(|n| !n.trim().is_empty())
                    .map(|n| n.trim().to_string())
                    .collect();
                (s.config.log_path.clone(), filtered)
            } else { (String::new(), vec![]) };

            if path_to_open.is_empty() { 
                thread::sleep(Duration::from_millis(500)); 
                continue; 
            }

            if path_to_open != current_path {
                current_path = path_to_open.clone();
                if let Ok(m) = std::fs::metadata(&current_path) { last_size = m.len(); }
            }

            if let Ok(mut file) = File::open(&current_path) {
                if let Ok(meta) = file.metadata() {
                    let current_len = meta.len();
                    if current_len > last_size {
                        let _ = file.seek(SeekFrom::Start(last_size));
                        let reader = BufReader::new(file);

                        if let Ok(mut s) = state_thread.lock() {
                            s.check_multi_buffer_timeouts();

                            for line in reader.lines().flatten() {
                                let clean = parser::normalize_combat_only(&line);
                                if clean.is_empty() { continue; }

                                // 1. DÉTECTION KO
                                if let Some(cap) = re_ko.captures(&clean) {
                                    if let Some(p) = cap.get(1).map(|m| m.as_str().trim()) {
                                        if tracked_players.iter().any(|name| name == p) {
                                            s.push_current_spell_to_history(&tracked_players);

                                            // On passe par process_action pour bénéficier du tampon anti-doublon en multi !
                                            s.process_action(p.to_string(), 0, "KO".to_string(), None);
                                        }
                                    }
                                }

                                // --- NOUVEAU : DÉTECTION DE LA RÉSURRECTION ---
                                else if let Some(cap) = re_revive.captures(&clean) {
                                    if let Some(p) = cap.get(1).map(|m| m.as_str().trim()) {
                                        if tracked_players.iter().any(|name| name == p) {
                                            // On passe par process_action pour gérer le multi-compte également
                                            s.process_action(p.to_string(), 0, "REVIVE".to_string(), None);
                                        }
                                    }
                                }
                            

                                // 2. DÉTECTION ACTION "INVOQUE"
                                else if let Some(cap) = re_summon.captures(&clean) {
                                    let summoner = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                    if tracked_players.contains(&summoner) {
                                        s.pending_summoner = Some(summoner.clone()); // <-- Ajoute .clone() ici
                                        s.active_summoners.insert(summoner); // <-- Plus d'erreur de move !
                                    }
                                }
                                // 3. CHANGEMENT DE TOUR
                                else if re_turn.is_match(&clean) {
                                    // On envoie le signal "TURN" à process_action pour le filtrer si on est en mode multi
                                    s.process_action("System".to_string(), 0, "TURN".to_string(), None);

                                    // Ta logique d'attribution des fenêtres d'invocation reste 100% INCHANGÉE :
                                    if let Some(ref caster) = s.current_caster {
                                        if s.active_summoners.contains(caster) {
                                            s.summon_window_owner = Some(caster.clone());
                                        } else {
                                            s.summon_window_owner = None; // Un non-invocateur finit son tour -> on ferme la fenêtre
                                        }
                                    } else {
                                        // Si pas de caster identifié mais qu'on avait un pending_summoner à ce tour
                                        if let Some(pending) = s.pending_summoner.take() {
                                            s.summon_window_owner = Some(pending);
                                        }
                                    }
                                }



                                // 4. LANCEMENT DE SORT
                                else if let Some(cap) = re_cast.captures(&clean) {
                                    s.push_current_spell_to_history(&tracked_players);
                                    let c = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                    let sp = cap.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                    
                                    if tracked_players.contains(&c) {
                                        s.current_caster = Some(c);
                                        s.current_spell = Some(sp);
                                        s.summon_window_owner = None; // Un joueur joue, on ferme la fenêtre d'invoc
                                    } else {
                                        // Si c'est un monstre ou une invocation qui lance un sort
                                        s.current_caster = None;
                                        s.current_spell = None;
                                    }
                                } 
                                // 5. APPLICATION D'ÉTAT (INDIRECT)
                                else if let Some(cap) = re_state_apply.captures(&clean) {
                                    let target = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                                    let state_name = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                                    if let Some(caster) = s.current_caster.clone() {
                                        if tracked_players.contains(&caster) && !tracked_players.contains(&target.to_string()) {
                                            let cur_t = s.current_turn;
                                            s.state_to_caster.insert(state_name.to_lowercase(), (state_name.to_string(), caster, cur_t));
                                        }
                                    }
                                } 
                                // 6. GESTION DES DÉGÂTS (PV)
                                else if clean.contains("PV") {
                                    let target = clean.split(':').next().unwrap_or("").trim();
                                    if !tracked_players.contains(&target.to_string()) {
                                        let mut found_indirect = false;
                                        let cur_t = s.current_turn;

                                        // --- A. DÉGÂTS INDIRECTS (PRIORITÉ 1) ---
                                        if let Some(cap) = re_dmg_with_state.captures(&clean) {
                                            let val = cap.get(2).map(|m| m.as_str().replace(' ', "").parse::<i32>().unwrap_or(0).abs()).unwrap_or(0);
                                            let state_ref = cap.get(3).map(|m| m.as_str().trim().to_lowercase()).unwrap_or_default();
                                            if let Some((orig_name, caster_name, _)) = s.state_to_caster.get(&state_ref).cloned() {
                                                if let Some(entry) = s.state_to_caster.get_mut(&state_ref) { entry.2 = cur_t; }
                                                s.process_action(caster_name, val, "INDIRECT".to_string(), Some(orig_name));
                                                found_indirect = true;
                                            }
                                        }
                                        if !found_indirect {
                                            let mut matched_state = None;
                                            for (key, (orig, caster, _)) in &s.state_to_caster {
                                                if clean.to_lowercase().contains(key) {
                                                    if let Some(cap) = re_dmg.captures(&clean) {
                                                        let val = cap.get(1).map(|m| m.as_str().replace(' ', "").parse::<i32>().unwrap_or(0).abs()).unwrap_or(0);
                                                        matched_state = Some((key.clone(), orig.clone(), caster.clone(), val));
                                                        break;
                                                    }
                                                }
                                            }
                                            if let Some((key, orig, caster, val)) = matched_state {
                                                s.process_action(caster, val, "INDIRECT".to_string(), Some(orig));
                                                if let Some(entry) = s.state_to_caster.get_mut(&key) { entry.2 = cur_t; }
                                                found_indirect = true;
                                            }
                                        }

                                        // --- B. DÉGÂTS DIRECTS OU INVOCATION (PRIORITÉ 2) ---
                                        if !found_indirect {
                                            if let Some(cap) = re_dmg.captures(&clean) {
                                                let val = cap.get(1).map(|m| m.as_str().replace(' ', "").parse::<i32>().unwrap_or(0).abs()).unwrap_or(0);
                                                
                                                if let Some(caster) = s.current_caster.clone() {
                                                    if tracked_players.contains(&caster) {
                                                        let spell_temp = s.current_spell.clone();
                                                        s.process_action(caster, val, "DMG".to_string(), spell_temp);
                                                    }
                                                } else if let Some(owner) = s.summon_window_owner.clone() {
                                                    // Si aucun lanceur suivi, mais qu'une fenêtre d'invocation est ouverte
                                                    s.process_action(owner, val, "DMG".to_string(), Some("sort de l'invocation".to_string()));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        last_size = current_len;
                    }
                }
            }
            thread::sleep(Duration::from_millis(200));
        }
    });

    // --- LANCEMENT DE L'INTERFACE ---
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.window_width, config.window_height])
            .with_min_inner_size([60.0, 60.0])
            .with_title("Wakfu calculateur")
            .with_icon(persistence::get_embedded_window_icon())
            .with_resizable(true),
        ..Default::default()
    };

    let _ = eframe::run_native("Wakfu calculateur", options, Box::new(move |cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_zoom_factor(config.zoom_factor); 
        Box::new(view::MyApp { state })
    }));
}
