use crate::model::{AppState, SpellEntry};

/// Enregistre l'application d'un état ou d'un effet indirect si celui-ci n'est pas déjà présent.
pub fn enregistrer_etat_indirect(state: &mut AppState, state_name: String, spell_orig_name: String, caster_name: String, current_turn: i32) {
    let state_key = state_name.to_lowercase().trim().to_string();
    
    // Si l'état est déjà enregistré, on ignore (on ne ré-écrit pas l'auteur d'origine)
    state.state_to_caster.entry(state_key).or_insert_with(|| {
        (spell_orig_name, caster_name, current_turn)
    });
}

/// Applique immédiatement les dégâts indirects : mise à jour des scores globaux
/// et ajout propre à l'historique de sorts avec le flag is_indirect à true.
pub fn appliquer_degat_indirect(state: &mut AppState, caster: String, state_name: String, dmg: i32) {
    // 1. Ajout immédiat aux dégâts globaux du joueur
    *state.total_damage.entry(caster.clone()).or_insert(0) += dmg;
    state.visible_players.insert(caster.clone());

    // 2. Enregistrement instantané dans l'historique du joueur (avec is_indirect: true)
    let history_entry = state.history.entry(caster.clone()).or_insert_with(Vec::new);

    // Si on a déjà une entrée pour ce même effet au tour actuel ET déjà flaggée en indirect, on cumule
    if let Some(existing_spell) = history_entry.iter_mut().last().filter(|s| s.spell == state_name && s.turn == state.current_turn && s.is_indirect) {
        existing_spell.hits.push(dmg);
        existing_spell.total += dmg;
    } else {
        // Sinon on crée une nouvelle entrée spécifique pour ce poison/effet indirect
        let new_entry = SpellEntry {
            spell: state_name,
            hits: vec![dmg],
            total: dmg,
            turn: state.current_turn,
            is_indirect: true,
        };
        history_entry.push(new_entry);
    }
}