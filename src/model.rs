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
    pub opacity: f32,
    pub always_on_top: bool,
    pub tracked_players: Vec<String>,
    pub window_width: f32,
    pub window_height: f32,
    pub mode: String,
    pub players_to_include: i32,
    pub permanent_archives: Vec<CombatArchive>,
    pub combat_counter: i32,
    pub text_color: [u8; 3], 
    #[serde(skip)]
    pub statut_maj: String,
    #[serde(skip)]
    pub maj_disponible: bool,
    pub maj_prete_pour_redemarrage: bool,
    pub history_horizontal: bool,
    
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_path: "".to_string(),
            zoom_factor: 1.2,
            always_on_top: true,
            tracked_players: Vec::new(),
            window_width: 500.0,
            window_height: 650.0,
            mode: "mono".to_string(),
            players_to_include: 1,
            permanent_archives: Vec::new(),
            combat_counter: 1,
            opacity: 0.8,
            text_color: [200, 200, 200],
            statut_maj: "Recherche de mise à jour...".to_string(),
            maj_disponible: false,
            maj_prete_pour_redemarrage: false,
            history_horizontal: true,
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
    #[serde(skip)]
    pub multi_buffer: HashMap<String, BufferEntry>,
    pub summon_to_owner: HashMap<String, String>,
    pub pending_summon_owner: Option<String>,
    
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
            permanent_archives: config.permanent_archives.clone(),
            combat_counter: config.combat_counter,
            multi_buffer: HashMap::new(),
            summon_to_owner: HashMap::new(),
            pending_summon_owner: None,
        }
    }

    pub fn reset_to_defaults(&mut self) {
        let saved_log_path = self.config.log_path.clone();
        let saved_tracked_players = self.config.tracked_players.clone();
        let saved_archives = self.config.permanent_archives.clone();
        self.config = AppConfig::default();
        self.config.log_path = saved_log_path;
        self.config.tracked_players = saved_tracked_players;
        self.config.permanent_archives = saved_archives;
        self.mode = self.config.mode.clone();
        self.players_to_include = self.config.players_to_include;
        self.last_applied_zoom = 0.0; 
        self.full_reset();
        crate::persistence::save_config(&self.config);
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
        self.multi_buffer.clear();
        self.summon_to_owner.clear();
        self.pending_summon_owner = None;
    }

    // --- LOGIQUE DE TAMPON ---

    pub fn check_multi_buffer_timeouts(&mut self) {
        if self.mode != "multi" { return; }
        let now = Instant::now();
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
            self.apply_final_stats(source, value, action_type, spell_name);
            return;
        }

        let event_key = format!("{}_{}_{}_{:?}", action_type, source, value, spell_name);
        let now = Instant::now();

        if self.multi_buffer.contains_key(&event_key) {
            self.multi_buffer.remove(&event_key);
        } else {
            let entry = BufferEntry {
                time: Some(now),
                source: source.clone(),
                value,
                action_type: action_type.clone(),
                spell_name: spell_name.clone(),
            };
            self.multi_buffer.insert(event_key, entry);
            self.apply_final_stats(source, value, action_type, spell_name);
        }
    }

    fn apply_final_stats(&mut self, source: String, value: i32, action_type: String, spell_name: Option<String>) {
        if action_type == "KO" {
            self.ko_players.insert(source);
            return;
        }
        if action_type == "REVIVE" {
            self.ko_players.remove(&source);
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

        // Redirection vers le système d'enregistrement des dégâts (Directs et Indirects)
        self.enregistrer_degat(&action_type, source, spell_name, value);
    }

    // ===============================================================================
    // ENREGISTREMENT DES DÉGÂTS (Directs, Indirects)
    // ===============================================================================

    pub fn enregistrer_degat(&mut self, action_type: &str, source: String, spell_name: Option<String>, value: i32) {
        if action_type == "DMG" {
            let s_name = spell_name.unwrap_or_else(|| "Sort Direct".to_string());
            crate::degats_directs::appliquer_degat_direct(self, source, s_name, value);
        } else if action_type == "INDIRECT" {
            let s_name = spell_name.unwrap_or_else(|| "Sort Indirect".to_string());
            crate::degats_indirects::appliquer_degat_indirect(self, source, s_name, value);
    
        } else if action_type == "SUMMON" {
            // ROUTAGE VERS LES DÉGÂTS D'INVOCATION
            let s_name = spell_name.unwrap_or_else(|| "Attaque d'invocation".to_string());
            crate::degats_invocations::appliquer_degat_invocation(self, source, s_name, value);
        }
    }

    pub fn push_current_spell_to_history(&mut self, tracked_players: &[String]) {
        if let Some(caster) = self.current_caster.clone() {
            if !self.current_hits.is_empty() {
                let is_tracked = tracked_players.is_empty() || tracked_players.contains(&caster);
                if is_tracked {
                    let total: i32 = self.current_hits.iter().sum();
                    let spell_name = self.current_spell.clone().unwrap_or_else(|| "Sort Direct".to_string());

                    let entry = SpellEntry {
                        spell: spell_name,
                        hits: self.current_hits.clone(),
                        total,
                        turn: self.current_turn,
                        is_indirect: false,
                    };

                    self.history.entry(caster).or_insert_with(Vec::new).push(entry);
                }
            }
        }
        self.current_hits.clear();
        self.current_spell = None;
    }

    pub fn save_current_combat(&mut self) {
        if self.total_damage.is_empty() {
            return;
        }

        let mut damages: Vec<(String, i32)> = self.total_damage
            .iter()
            .map(|(player, &dmg)| (player.clone(), dmg))
            .collect();

        damages.sort_by(|a, b| b.1.cmp(&a.1));

        let players_list: Vec<String> = self.visible_players.iter().cloned().collect();
        let players_involved = if players_list.is_empty() {
            "Inconnu".to_string()
        } else {
            players_list.join(", ")
        };

        let archive = CombatArchive {
            name: format!("Combat #{}", self.combat_counter),
            players_involved,
            total_turns: self.current_turn,
            damages,
            details_history: self.history.clone(),
        };

        self.archives.push(archive.clone());
        self.permanent_archives.push(archive);
        self.config.permanent_archives = self.permanent_archives.clone();

        self.combat_counter += 1;
        self.config.combat_counter = self.combat_counter;
        crate::persistence::save_config(&self.config);

        self.full_reset();
    }
}