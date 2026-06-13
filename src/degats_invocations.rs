use crate::model::{AppState, SpellEntry};

pub fn appliquer_degat_invocation(state: &mut AppState, owner: String, spell_name: String, value: i32) {
    // 1. On applique le dégât au joueur qui possède l'invocation (l'invocateur)
    let total_dmg = state.total_damage.entry(owner.clone()).or_insert(0);
    *total_dmg += value;

    // 2. On rend l'invocateur visible s'il ne l'était pas
    state.visible_players.insert(owner.clone());

    // 3. On enregistre le hit dans l'historique du joueur sous le format "NomSort [Invoc]"
    let formatted_spell = format!("{} [Invoc]", spell_name);
    
    let player_history = state.history.entry(owner).or_insert_with(Vec::new);
    
    // On cherche si une entrée identique existe déjà pour ce tour afin de regrouper les hits du même sort
    if let Some(entry) = player_history.iter_mut().find(|e| e.spell == formatted_spell && e.turn == state.current_turn) {
        entry.hits.push(value);
        entry.total += value;
    } else {
        // Sinon, on insère un nouvel enregistrement
        player_history.push(SpellEntry {
            spell: formatted_spell,
            hits: vec![value],
            total: value,
            turn: state.current_turn,
            is_indirect: false,
        });
    }
}
