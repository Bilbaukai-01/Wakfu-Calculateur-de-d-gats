use crate::model::{AppState, SpellEntry};

/// Applique immédiatement un dégât direct : mise à jour des totaux globaux 
/// et enregistrement instantané dans l'historique.
pub fn appliquer_degat_direct(state: &mut AppState, source: String, spell_name: String, value: i32) {
    // 1. Ajout immédiat aux dégâts globaux du joueur
    *state.total_damage.entry(source.clone()).or_insert(0) += value;
    state.visible_players.insert(source.clone());

    // 2. Enregistrement instantané dans l'historique du joueur
    let history_entry = state.history.entry(source.clone()).or_insert_with(Vec::new);

    // Si le joueur a déjà une entrée pour ce sort au tour actuel (et non indirect), on ajoute les dégâts.
    // Sinon, on crée une nouvelle entrée de sort.
    if let Some(existing_spell) = history_entry.iter_mut().last().filter(|s| s.spell == spell_name && s.turn == state.current_turn && !s.is_indirect) {
        existing_spell.hits.push(value);
        existing_spell.total += value;
    } else {
        let new_entry = SpellEntry {
            spell: spell_name,
            hits: vec![value],
            total: value,
            turn: state.current_turn,
            is_indirect: false,
        };
        history_entry.push(new_entry);
    }
}
