use regex::Regex;

// ==================================================================================
// PARSING & LOG PROCESSING
// ==================================================================================

pub fn normalize_combat_only(line: &str) -> String {
    let re_ts = Regex::new(r"^\s*\[?\d{2}:\d{2}:\d{2},\d{3}\]?\s*").unwrap_or_else(|_| Regex::new("").unwrap());
    let clean = re_ts.replace_all(line, "").to_string();
    let clean = clean.replace('\u{202F}', " ").replace("  ", " ");
    if let Some(pos) = clean.find("(combat)]") { clean[pos + 9..].trim().to_string() } else { clean.trim().to_string() }
}

pub fn create_regex_patterns() -> (Regex, Regex, Regex, Regex, Regex, Regex, Regex, Regex) {
    let re_ko = Regex::new(r"(?i)^(.+?)\s+est\s+(K\.?O\.?|KO)\s*!").unwrap();
    let re_turn = Regex::new(r"(?i)\b\d{1,3}\s+seconde(s)?\s+reportée(s)?\s+pour\s+le\s+tour\s+suivant\.").unwrap();
    let re_cast = Regex::new(r"^(.+?) lance le sort (.+)$").unwrap();
    let re_dmg = Regex::new(r":\s*(-?[\d\s]+)").unwrap();
    let re_state_apply = Regex::new(r"^(.+?):\s*(.+?)\s*\((?:Niv\.|\+)\s*\d+.*?\)$").unwrap();
    let re_dmg_with_state = Regex::new(r"^(.+?):\s*-?([\d\s]+)\s*PV.*?\((.+?)\)$").unwrap();
    let re_summon = Regex::new(r"^(.+?):\s*Invoque").unwrap(); 
    let re_revive = Regex::new(r"(?i)^(.+?)\s+est\s+ressuscité\s*!").unwrap();

    (re_ko, re_turn, re_cast, re_dmg, re_state_apply, re_dmg_with_state, re_summon, re_revive)
}
