Rapport d'analyse — guitarproparser
Structure du workspace
Crate	Binaire	Rôle	État
lib (scorelib)	—	Bibliothèque de parsing	Production
cli (score_tool)	score_tool	Interface CLI	Fonctionnel
web_server	score_server	Serveur web	Vide (stub)
~7 500 lignes de code dans la bibliothèque, 161 tests, 248 fichiers de test.

Support des formats
Lecture
Format	Extension	État	Méthode
GP3	.gp3	✅ Complet	Parsing binaire
GP4	.gp4	✅ Complet	Parsing binaire
GP5	.gp5	✅ Complet	Parsing binaire
GP6	.gpx	⚠️ Fonctionnel	BCFZ + GPIF XML
GP7+	.gp	⚠️ Fonctionnel	ZIP + GPIF XML
GP6 et GP7 sont fonctionnels mais pas exhaustifs — certaines fonctionnalités avancées du XML GPIF ne sont pas encore mappées.

Écriture
Format	État	Couverture
GP3/4/5	✅ Partiel	~80% (métadonnées, pistes, mesures, notes)
GP6/GP7	❌ Aucune	Non implémenté
Un TODO existe dans song.rs pour l'écriture des canaux MIDI.

Modèle de données
La hiérarchie Song → Track → Measure → Voice → Beat → Note est complète :

Song : métadonnées complètes (titre, artiste, album, auteur, etc.)
Track : nom, cordes, frettes, canal MIDI, percussions, paramètres RSE
Measure/MeasureHeader : tempo, signature, tonalité, répétitions, marqueurs
Beat : durée, effets, tuples, accords
Note : vélocité, frette, 12 types d'effets (bend, slide, harmonique, grace, hammer, etc.)
Effets : BendEffect (avec points), GraceEffect, HarmonicEffect (6 types), TrillEffect, SlideType (6 types), tremolo picking, vibrato, palm mute, let ring, staccato, ghost, dead notes
Partiellement implémenté : RSE (Realistic Sound Engine) — parsé mais pas pleinement exploité.

Tests (161 tests)
Format	Nombre de tests	Couverture
GPX	69	Effets, slides, harmoniques, accords, percussions
GP5	42	Couverture complète des fonctionnalités
GP4	28	Complète sauf fonctionnalités GP5
GP3	10	Fonctionnalités de base
GP7	1	Lecture basique uniquement
Multi-format	11	Accords, comparaisons cross-format
Points forts : slides (12 tests), effets (11 tests), harmoniques (9 tests), accords (7 tests).

CLI (score_tool)
Fonctionnalités :

Affichage des métadonnées (titre, artiste, album, version, etc.)
Génération de tablature ASCII
Supporte GP3, GP4, GP5, GPX, GP
Limite de fichier : 16 Mo
Limitations :

Tablature ASCII pour la première piste uniquement
Pas d'export MIDI
Pas de conversion de format
Pas de traitement par lots
Gestion des erreurs
État actuel : hybride

La couche I/O (gpx.rs, gpif.rs) utilise Result<T, String>
Le CLI et certains chemins de parsing utilisent panic!() / unwrap()
Pas de type d'erreur unifié dans la bibliothèque
Ce qui reste à faire
Priorité	Tâche	Détails
Haute	Compléter le support GP6/GP7	Mapper tous les attributs GPIF (wah, auto-brush, rasgueado, etc.)
Haute	Gestion d'erreurs unifiée	Remplacer panic!() par des Result avec un type d'erreur dédié
Haute	Tests GP7	1 seul test actuellement vs 74 fichiers de test disponibles
Moyenne	Compléter l'écriture GP3/4/5	Finaliser les ~20% manquants (canaux MIDI, etc.)
Moyenne	Écriture GP6/GP7	Non implémenté — nécessite génération XML GPIF
Moyenne	RSE complet	Données parsées mais pas entièrement exploitées
Basse	Web server	Complètement vide — main.rs ne contient que fn main() {}
Basse	CLI amélioré	Multi-pistes, export MIDI, conversion de format
Basse	Support MuseScore (.mscz)	Mentionné nulle part dans le code mais potentiellement intéressant
Basse	Message d'erreur CLI	Le message d'erreur à la ligne 57 du CLI ne mentionne pas GPX dans les formats supportés
Points forts du projet
Architecture trait-based propre (13 traits spécialisés)
Modèle de données complet et fidèle à la structure musicale
Excellente couverture de tests pour GP5 et GPX
Support des formats legacy (GP3) jusqu'aux modernes (GP7)
Code propre, passe clippy sans warnings
Points d'attention
Le format GP7 n'a qu'un seul test malgré 74 fichiers de test disponibles — c'est le plus gros gap de couverture
L'absence de Result unifié rend la bibliothèque difficile à intégrer proprement dans d'autres projets
Le web server est un placeholder vide