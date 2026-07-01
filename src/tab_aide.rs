use eframe::egui;
use crate::model::AppState;

pub fn render(_s: &mut AppState, ui: &mut egui::Ui) {
    egui::Frame::none()
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_source("help_scroll").show(ui, |ui| {
                ui.vertical(|ui| {
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
                        ui.label(r"• Spécifiez le chemin de votre fichier de log Wakfu. C'est ce lien qui lira vos informations du jeu en direct, absolument indispensable pour fonctionner. Le fichier devrait se trouver à cet endroit : C:\Utilisateurs\Nom de votre ordinateur\AppData\Roaming\zaap\gamesLogs\wakfu\logs");

                        ui.label("• Ajoutez la liste des personnages que vous souhaitez suivre. Vous pouvez ajouter autant de personnages que vous voulez, pas besoin qu'ils soient dans votre groupe, l'application reconnaîtra automatiquement les personnages à suivre.");
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

                    // Grand espace supplémentaire en bas pour un défilement confortable
                    ui.add_space(150.0);
                });
            });
        });
}
