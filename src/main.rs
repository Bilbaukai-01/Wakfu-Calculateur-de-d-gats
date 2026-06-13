#![windows_subsystem = "windows"]

mod model;
mod persistence;
mod parser;
mod view;
mod invocations;
mod degats_directs;
mod degats_indirects;
mod degats_invocations;




use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


use model::AppState;
use persistence::load_config;

fn main() {
    // 1. On charge la configuration de manière modifiable (mut)
    let mut config = load_config();
    
    // 2. Si le booléen de redémarrage est à true, cela signifie qu'on vient de redémarrer 
    // sur la NOUVELLE VERSION. On nettoie tout et on sauvegarde.
    if config.maj_prete_pour_redemarrage {
        config.maj_prete_pour_redemarrage = false;
        config.maj_disponible = false;
        config.statut_maj = "À jour".to_string();
        persistence::save_config(&config); // Sauvegarde immédiate dans le JSON
    }

    // 3. On crée l'état partagé avec notre configuration nettoyée
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
                            
                                                                // ==================================================================================
                                                                // ==================================================================================
                                // 2. DÉTECTION DE L'ACTION "INVOQUE" (INVOCATIONS)
                                // ==================================================================================
                                else if let Some(cap) = re_summon.captures(&clean) {
                                    let invocateur = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

                                    if tracked_players.contains(&invocateur) {
                                        // Cas A : Détection spécifique de l'Osamodas
                                        if clean.contains("Invoque une créature du Gobgob") {
                                            s.pending_summon_owner = Some(invocateur);
                                        } 
                                        // Cas B : Invocation traditionnelle
                                        else {
                                            // La ligne ressemble à : "Soldier-Blood: Invoque un(e) La Goulue"
                                            let invocation_raw = clean.split("Invoque").nth(1).unwrap_or("").trim();

                                            // Nettoyage complet des articles pour récupérer le nom propre exact
                                            let invocation = invocation_raw
                                                .replace("un(e)", "")
                                                .replace("un", "")
                                                .replace("une", "")
                                                .trim()
                                                .to_string();

                                            // Sécurité : évite d'enregistrer "une créature du Gobgob" si la chaîne s'est faufilée ici
                                            if !invocation.is_empty() && !invocation.to_lowercase().contains("créature du gobgob") {
                                                s.summon_to_owner.insert(invocation, invocateur);
                                            }
                                        }
                                    }
                                }

                                // ==================================================================================
                                // 3. CHANGEMENT DE TOUR
                                // ==================================================================================
                                else if re_turn.is_match(&clean) {
                                    s.process_action("System".to_string(), 0, "TURN".to_string(), None);
                                    // ICI : Dès que le log du tour est détecté, on nettoie le temps réel !
                                                s.current_hits.clear();
                                                s.current_caster = None;
                                                s.current_spell = None;

                                }

                                // ==================================================================================
                                // 4. LANCEMENT DE SORT (JOUEURS ET INVOCATIONS)
                                // ==================================================================================
                                else if let Some(cap) = re_cast.captures(&clean) {
                                    s.push_current_spell_to_history(&tracked_players);
                                    let c = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                    let sp = cap.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

                                    // --- OSAMODAS : On associe le nom de la créature de la liste au joueur en attente ---
                                    if let Some(owner) = s.pending_summon_owner.clone() {
                                        if crate::invocations::OSAMODAS_SUMMONS.contains(&c.as_str()) {
                                            s.summon_to_owner.insert(c.clone(), owner);
                                            s.pending_summon_owner = None; // Fin de l'attente
                                        }
                                    }

                                    // On accepte le lanceur si c'est un joueur suivi OU une invocation enregistrée (normale ou Osamodas identifiée)
                                    if tracked_players.contains(&c) || s.summon_to_owner.contains_key(&c) {
                                        s.current_caster = Some(c);
                                        s.current_spell = Some(sp);
                                    } else {
                                        s.current_caster = None;
                                        s.current_spell = None;
                                    }
                                }


                                // ==================================================================================
                                // 5. APPLICATION D'ÉTAT (DÉGÂTS INDIRECTS FUTURS)
                                // ==================================================================================
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

                                // ==================================================================================
                                // 6. GESTION DU CALCUL DES DÉGÂTS (PV PERDUS)
                                // ==================================================================================
                                else if clean.contains("PV") {
                                    let target = clean.split(':').next().unwrap_or("").trim();
                                    if !tracked_players.contains(&target.to_string()) {
                                        let mut found_damage_processed = false;
                                        let cur_t = s.current_turn;

                                        // --- PRIORITÉ A : DÉGÂTS DIRECTS D'INVOCATION (Nouveau !) ---
                                        // On vérifie en tout premier si les dégâts proviennent d'un sort direct d'invocation.
                                        // Cela évite que des indicateurs comme "(Hydratée)" soient confondus avec un poison indirect.
                                        if let Some(caster) = s.current_caster.clone() {
                                            if let Some(owner) = s.summon_to_owner.get(&caster).cloned() {
                                                if let Some(cap) = re_dmg.captures(&clean) {
                                                    let val = cap.get(1).map(|m| m.as_str().replace(' ', "").parse::<i32>().unwrap_or(0).abs()).unwrap_or(0);
                                                    let spell_temp = s.current_spell.clone();
                                                    
                                                    // On attribue les dégâts directs au propriétaire de l'invocation sous la forme "SUMMON"
                                                    s.process_action(owner, val, "SUMMON".to_string(), spell_temp);
                                                    found_damage_processed = true;
                                                }
                                            }
                                        }

                                        // --- PRIORITÉ B : DÉGÂTS INDIRECTS (EFFETS ET POISONS) ---
                                        if !found_damage_processed {
                                            if let Some(cap) = re_dmg_with_state.captures(&clean) {
                                                let val = cap.get(2).map(|m| m.as_str().replace(' ', "").parse::<i32>().unwrap_or(0).abs()).unwrap_or(0);
                                                let state_ref = cap.get(3).map(|m| m.as_str().trim().to_lowercase()).unwrap_or_default();
                                                if let Some((orig_name, caster_name, _)) = s.state_to_caster.get(&state_ref).cloned() {
                                                    if let Some(entry) = s.state_to_caster.get_mut(&state_ref) { entry.2 = cur_t; }

                                                    s.process_action(caster_name, val, "INDIRECT".to_string(), Some(orig_name));
                                                    found_damage_processed = true;
                                                }
                                            }
                                        }

                                        if !found_damage_processed {
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
                                                found_damage_processed = true;
                                            }
                                        }

                                        // --- PRIORITÉ C : DÉGÂTS DIRECTS STANDARD DU JOUEUR ---
                                        if !found_damage_processed {
                                            if let Some(cap) = re_dmg.captures(&clean) {
                                                let val = cap.get(1).map(|m| m.as_str().replace(' ', "").parse::<i32>().unwrap_or(0).abs()).unwrap_or(0);

                                                if let Some(caster) = s.current_caster.clone() {
                                                    if tracked_players.contains(&caster) {
                                                        let spell_temp = s.current_spell.clone();
                                                        s.process_action(caster, val, "DMG".to_string(), spell_temp);
                                                    }
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


{
        let state_clone = Arc::clone(&state);
        thread::spawn(move || {
            // On attend 1 seconde pour laisser l'application s'ouvrir tranquillement
            thread::sleep(Duration::from_secs(1));
            
            match self_update::backends::github::Update::configure()
                .repo_owner("Bilbaukai-01")
                .repo_name("Wakfu-Calculateur-de-d-gats")
                .bin_name("wakfu_calculateur")
                .current_version(env!("CARGO_PKG_VERSION"))
                .build()
            {
                Ok(updater) => {
                    if let Ok(latest) = updater.get_latest_release() {
                        let current = env!("CARGO_PKG_VERSION");
                        if latest.version != current {
                            if let Ok(mut s) = state_clone.lock() {
                                s.config.maj_disponible = true;
                                s.config.statut_maj = format!("Version {} disponible !", latest.version);
                            }
                        } else {
                            if let Ok(mut s) = state_clone.lock() {
                                s.config.statut_maj = "À jour".to_string();
                            }
                        }
                    }
                }
                Err(_) => {
                    if let Ok(mut s) = state_clone.lock() {
                        s.config.statut_maj = "Impossible de vérifier les mises à jour".to_string();
                    }
                }
            }
        });
    }

    // --- LANCEMENT DE L'INTERFACE ---
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.window_width, config.window_height])
            .with_min_inner_size([60.0, 60.0])
            .with_title("Wakfu calculateur")
            .with_icon(persistence::get_embedded_window_icon())
            .with_transparent(true)
            .with_resizable(true),
        ..Default::default()
    };

    let _ = eframe::run_native("Wakfu calculateur", options, Box::new(move |cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_zoom_factor(config.zoom_factor); 
        Box::new(view::MyApp { state })
    }));
}
fn verifier_et_mettre_a_jour() -> Result<self_update::Status, Box<dyn std::error::Error>> {
    // Cette fonction recherche la dernière release GitHub et remplace l'exécutable local
    let status = self_update::backends::github::Update::configure()
        .repo_owner("Bilbaukai-01")             // Ton nom GitHub
        .repo_name("Wakfu-Calculateur-de-d-gats") // Le nom de ton dépôt
        .bin_name("Wakfu_calculateur")          // Le nom de ton binaire
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION")) // Lit la version de ton Cargo.toml
        .build()?
        .update()?;
    
    Ok(status)
}
