use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

// ==================================================================================
// DATA STRUCTURES
// ==================================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct SpellEntry {
    pub spell: String,
    pub hits: Vec<i32>,
    pub total: i32,
    pub turn: i32,
    pub is_indirect: bool, 
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CombatArchive {
    pub name: String,
    pub players_involved: String, 
    pub total_turns: i32,
    pub damages: Vec<(String, i32)>,
    pub details_history: HashMap<String, Vec<SpellEntry>>, 
}

#[derive(Serialize, Deserialize)]
pub struct BufferEntry {
    #[serde(skip)]
    pub time: Option<Instant>,
    pub source: String,
    pub value: i32,
    pub action_type: String,
    pub spell_name: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Tab { Stats, Combats, Archivage, Settings }

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub log_path: String,
    pub zoom_factor: f32,
    pub always_on_top: bool,
    pub tracked_players: Vec<String>,
    pub window_width: f32,
    pub window_height: f32,
    pub mode: String,
    pub players_to_include: i32,
    pub permanent_archives: Vec<CombatArchive>,
    pub combat_counter: i32,
    #[serde(skip)]
    pub statut_maj: String,
    #[serde(skip)]
    pub maj_disponible: bool,
    pub maj_prete_pour_redemarrage: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_path: "".to_string(), //le fichier les dlogs
            zoom_factor: 1.2,
            always_on_top: true,
            tracked_players: Vec::new(), //les noms des joueurs suivis
            window_width: 500.0,
            window_height: 650.0,
            mode: "mono".to_string(),
            players_to_include: 1,
            permanent_archives: Vec::new(),
            combat_counter: 1,

            statut_maj: "Recherche de mise à jour...".to_string(),
            maj_disponible: false,
            maj_prete_pour_redemarrage: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub total_damage: HashMap<String, i32>,
    pub history: HashMap<String, Vec<SpellEntry>>,
    pub state_to_caster: HashMap<String, (String, String, i32)>, 
    pub visible_players: HashSet<String>,
    pub ko_players: HashSet<String>,
    pub current_turn: i32,
    pub tour_reports_seen: i32,
    pub mode: String,
    pub players_to_include: i32,
    pub config: AppConfig,
    pub current_caster: Option<String>,
    pub current_spell: Option<String>,
    pub current_hits: Vec<i32>,
    pub has_started: bool,
    pub detected_main_player: Option<String>,
    pub main_player_first_turn_ignored: bool,
    pub showing_history: bool,
    pub current_tab: Tab,
    pub is_compact: bool,
    pub last_applied_zoom: f32,
    pub archives: Vec<CombatArchive>,
    pub permanent_archives: Vec<CombatArchive>,
    pub combat_counter: i32,
    pub pending_summoner: Option<String>,
    pub summon_window_owner: Option<String>,
    pub active_summoners: HashSet<String>,
    #[serde(skip)]
    pub multi_buffer: HashMap<String, BufferEntry>,

}

// ==================================================================================
// APPSTATE IMPLEMENTATION
// ==================================================================================

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            total_damage: HashMap::new(),
            history: HashMap::new(),
            state_to_caster: HashMap::new(),
            visible_players: HashSet::new(),
            ko_players: HashSet::new(),
            current_turn: 1,
            tour_reports_seen: 0,
            mode: config.mode.clone(),
            players_to_include: config.players_to_include,
            config: config.clone(),
            current_caster: None,
            current_spell: None,
            current_hits: Vec::new(),
            has_started: false,
            detected_main_player: None,
            main_player_first_turn_ignored: false,
            showing_history: false,
            current_tab: Tab::Stats,
            is_compact: false,
            last_applied_zoom: 1.0,
            archives: Vec::new(),
            pending_summoner: None,
            summon_window_owner: None,
            active_summoners: HashSet::new(), // <--- AJOUTE CETTE LIGNE ICI !
            permanent_archives: config.permanent_archives.clone(),
            combat_counter: config.combat_counter,
            multi_buffer: HashMap::new(),
        }
    }

    pub fn reset_to_defaults(&mut self) {
// 1. sauvegarde temporairement les données utilisateur qu'on veut ABSOLUMENT garder
        let saved_log_path = self.config.log_path.clone();
        let saved_tracked_players = self.config.tracked_players.clone();
        let saved_archives = self.config.permanent_archives.clone(); // Optionnel : évite aussi de perdre tes combats archivés !

        // 2.charge la configuration par défaut de l'application
        self.config = AppConfig::default(); // Charge les réglages par défaut

        // 3. réinjecte les données sauvegardées juste avant
        self.config.log_path = saved_log_path;
        self.config.tracked_players = saved_tracked_players;
        self.config.permanent_archives = saved_archives;

        // 4. Suite logique 
        self.mode = self.config.mode.clone();
        self.players_to_include = self.config.players_to_include;
        self.last_applied_zoom = 0.0; 
        self.full_reset(); // Vide les listes de dégâts, KO, etc.
        crate::persistence::save_config(&self.config); // Enregistre immédiatement
    }


    pub fn full_reset(&mut self) {
        self.total_damage.clear();
        self.history.clear();
        self.state_to_caster.clear();
        self.visible_players.clear();
        self.ko_players.clear();
        self.current_turn = 1;
        self.tour_reports_seen = 0;
        self.current_caster = None;
        self.current_spell = None;
        self.current_hits.clear();
        self.has_started = false;
        self.detected_main_player = None;
        self.main_player_first_turn_ignored = false;
        self.multi_buffer.clear(); // RESET TAMPON
        self.pending_summoner = None;
        self.summon_window_owner = None;
        self.active_summoners.clear();
    }

    // --- LOGIQUE DE TAMPON ---

    pub fn check_multi_buffer_timeouts(&mut self) {
        if self.mode != "multi" { return; }
        let now = Instant::now();
        // On nettoie les événements expirés (plus vieux de 1.5 seconde) sans les appliquer, 
        // car s'ils sont encore là, c'est qu'ils ont déjà été appliqués lors de leur premier passage.
        self.multi_buffer.retain(|_, entry| {
            if let Some(time) = entry.time {
                now.duration_since(time).as_secs_f32() < 1.5
            } else {
                true
            }
        });
    }

    pub fn process_action(&mut self, source: String, value: i32, action_type: String, spell_name: Option<String>) {
        if self.mode != "multi" {
            // Mode Mono-compte : On applique l'action directement
            self.apply_final_stats(source, value, action_type, spell_name);
            return;
        }

        // Mode Multi-compte : Gestion du tampon anti-doublon
        let event_key = format!("{}_{}_{}_{:?}", action_type, source, value, spell_name);
        let now = Instant::now();

        if self.multi_buffer.contains_key(&event_key) {
            // C'est le doublon (le deuxième log identique qui arrive) !
            // On l'a déjà appliqué lors du premier passage, donc on se contente de le supprimer silencieusement.
            self.multi_buffer.remove(&event_key);
        } else {
            // C'est la toute première fois qu'on voit cet événement (Compte A).
            // 1. On le garde en mémoire pour bloquer le doublon qui arrivera du Compte B.
            let entry = BufferEntry {
                time: Some(now),
                source: source.clone(),
                value,
                action_type: action_type.clone(),
                spell_name: spell_name.clone(),
            };
            self.multi_buffer.insert(event_key, entry);

            // 2. On l'applique DIRECTEMENT à l'écran pour éviter toute latence !
            self.apply_final_stats(source, value, action_type, spell_name);
        }
    }


     fn apply_final_stats(&mut self, source: String, value: i32, action_type: String, spell_name: Option<String>) {
        if action_type == "KO" {
            // .insert() est utilisé ici à la place de .push() car ko_players est un HashSet
            self.ko_players.insert(source);
            return;
        }
            // --- NOUVEAU : GESTION DE LA RÉSURRECTION ---
        if action_type == "REVIVE" {
            self.ko_players.remove(&source); // Retire le joueur des KO pour qu'il re-compte dans le calcul des tours
            return;
        }

        if action_type == "TURN" {
            self.tour_reports_seen += 1;
            let expected = (self.players_to_include - self.ko_players.len() as i32).max(1);
            if self.tour_reports_seen >= expected {
                self.current_turn += 1;
                self.tour_reports_seen = 0;
                self.current_caster = None;
                self.current_spell = None;
            }
            return;
        }

        if action_type == "DMG" {
            // Redirection vers l'invocateur si la source n'est pas un joueur suivi
            let final_source = if !self.config.tracked_players.contains(&source) {
                if let Some(owner) = &self.summon_window_owner {
                    owner.clone()
                } else {
                    source.clone()
                }
            } else {
                source.clone()
            };

            // 1. Ajout au total de dégâts
            *self.total_damage.entry(final_source.clone()).or_insert(0) += value;
            self.visible_players.insert(final_source.clone());

            // 2. Historique des dégâts
            if self.current_caster.is_none() {
                // Dégâts d'une invocation (ou hors tour direct du joueur)
                let entry = SpellEntry {
                    spell: "sort de l'invocation".to_string(),
                    hits: vec![value],
                    total: value,
                    turn: self.current_turn,
                    is_indirect: false,
                };
                self.history.entry(final_source).or_insert(vec![]).push(entry);
            } else {
                // Dégâts d'un sort direct du joueur
                self.current_hits.push(value);
            }
        } else if action_type == "INDIRECT" {
            if let Some(s_name) = spell_name {
                self.add_indirect_damage(source, s_name, value);
            }
        }
    }


    pub fn save_current_combat(&mut self) {
        if self.total_damage.is_empty() { return; }
        let mut damages_list: Vec<(String, i32)> = self.total_damage.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        damages_list.sort_by(|a, b| b.1.cmp(&a.1));
        let players_names = damages_list.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", ");
        let archive = CombatArchive {
            name: format!("Combat {}", self.combat_counter),
            players_involved: players_names,
            total_turns: self.current_turn,
            damages: damages_list,
            details_history: self.history.clone(),
        };
        self.archives.push(archive.clone());
        self.combat_counter += 1;
        self.config.combat_counter = self.combat_counter; // Synchronise avec la config
        crate::persistence::save_config(&self.config); // Sauvegarde
        self.full_reset();
    }

    pub fn push_current_spell_to_history(&mut self, tracked: &[String]) {
        if let (Some(caster), Some(spell)) = (&self.current_caster, &self.current_spell) {
            if !self.current_hits.is_empty() && tracked.contains(caster) {
                let total = self.current_hits.iter().sum();
                let entry = SpellEntry {
                    spell: spell.clone(),
                    hits: self.current_hits.clone(),
                    total,
                    turn: self.current_turn,
                    is_indirect: false,
                };
                self.history.entry(caster.clone()).or_insert(vec![]).push(entry);
            }
        }
        self.current_hits.clear();
    }

    pub fn add_indirect_damage(&mut self, caster: String, state_name: String, dmg: i32) {
        *self.total_damage.entry(caster.clone()).or_insert(0) += dmg;
        self.visible_players.insert(caster.clone());
        let entry = SpellEntry {
            spell: state_name,
            hits: vec![dmg],
            total: dmg,
            turn: self.current_turn,
            is_indirect: true,
        };
        self.history.entry(caster).or_insert(vec![]).push(entry);
    }
}
