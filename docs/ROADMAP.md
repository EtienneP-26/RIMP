# Roadmap RIMP

## Comment lire un numéro de version

Une version s'écrit `vX.Y.Z`.

| Chiffre | Signification | Exemple |
|---|---|---|
| `X` (le premier) | Version stable et fonctionnelle, utilisable et publiable | `v1.0.0` |
| `Y` (le deuxième) | Modification majeure : une fonctionnalité importante | `v0.4.0` |
| `Z` (le troisième) | Modification mineure : petit ajout, réglage ou correction | `v0.4.2` |

Règles :
- Il n'y a pas de date : une version sort quand tout ce qu'elle décrit est terminé.
- Il peut y avoir autant de versions mineures que nécessaire. Les choses les plus simples sont rangées dans les versions mineures, la fonctionnalité principale est dans la version majeure (`v0.Y.0`).
- Quand `Y` augmente, `Z` repasse à 0.
- Tant que `X` vaut 0, l'application est en construction et tout peut changer.
- Chaque version est un tag git `vX.Y.Z`. Avant la stable, des candidates `v1.0.0-rc.N` peuvent être publiées.
- Linux uniquement.

## Avant la v1.0.0

### Mise en place

| Version | Nom | Description |
|---|---|---|
| v0.1.0 | Mise en place du dépôt | Le dépôt est prêt à accueillir le code : `Cargo.toml` et `Cargo.lock` commités, dossiers `src/core`, `src/brush`, `src/io` et `src/app` avec un `src/lib.rs`, README et ROADMAP écrits, fichier `LICENSE` temporaire, `.gitignore` nettoyé, ancien fichier de test vide supprimé. Le programme affiche toujours seulement `Hello, world!`. |

### Fondations du moteur

| Version | Nom | Description |
|---|---|---|
| v0.2.0 | Mélange de pixels | Première vraie fonction du moteur : `blend_normal` pose un pixel (la source) par-dessus un autre (la destination) avec la formule de transparence prémultipliée `out = src + dst × (1 − src.a)`. Trois tests unitaires la vérifient avec des valeurs connues. Ce sont les premiers tests du projet. |
| v0.2.1 | Tuile | `Tile` : un carré de 64×64 pixels (rouge, vert, bleu et alpha en nombres décimaux) avec `get` et `set` pour lire et écrire un pixel. Tests inclus. |
| v0.2.2 | Export PNG d'une tuile | Écrit une tuile dans un fichier PNG (crate `png`), pour pouvoir enfin la regarder. |
| v0.2.3 | Modes de fusion | `BlendMode` : Normal, Produit et Addition. Chaque mode est testé avec des valeurs connues. |
| v0.2.4 | Erreurs propres | Un type d'erreur défini avec `thiserror`, et l'export utilise `?` au lieu de `unwrap`. |
| v0.2.5 | Code propre | `cargo fmt --check` et `cargo clippy -- -D warnings` ne signalent plus rien. |

### Fenêtre et GPU

| Version | Nom | Description |
|---|---|---|
| v0.3.0 | Fenêtre | Une fenêtre (winit 0.30) dont le GPU (wgpu) remplit tout l'intérieur d'une couleur unie. La fenêtre se ferme proprement. |
| v0.3.1 | Vérification de la tablette | Contrôle avec `libinput debug-events` que la tablette envoie bien la pression au système (événements `TABLET_TOOL_AXIS`). Si rien ne s'affiche, le problème vient du pilote et pas du code. Le résultat est noté dans le README. |
| v0.3.2 | Bibliothèques système | La liste des paquets Linux nécessaires (Wayland, Vulkan) est ajoutée à l'installation développeur du README. |
| v0.3.3 | Redimensionnement | Redimensionner la fenêtre ne fait ni planter ni déformer l'affichage. |
| v0.3.4 | Texture plein écran | Une image (texture) est affichée sur toute la fenêtre avec un petit shader WGSL. C'est la future zone de dessin. |

### Tablette et premier trait

| Version | Nom | Description |
|---|---|---|
| v0.4.0 | Premier trait à pression | Le stylet dessine. octotablet est branché (avec winit 0.29 ou un fork si la version récente ne compile pas) et chaque position reçue dessine un disque dans un tableau de pixels en mémoire. La taille du disque suit la pression et la texture est renvoyée au GPU à chaque image. C'est lent, mais suffisant pour tester. |
| v0.4.1 | Souris | Quand il n'y a pas de pression (souris, ou valeur NaN), on utilise une pression de 1 : la souris peut dessiner aussi. |
| v0.4.2 | Écrans HiDPI | Les positions sont multipliées par `scale_factor`, pour que le trait reste sous le stylet sur un écran à haute densité de pixels. |
| v0.4.3 | Interface egui | egui est affiché par-dessus le canevas (`egui-winit` et `egui-wgpu`, avec le wgpu réexporté par egui pour éviter deux versions). |
| v0.4.4 | Fenêtre de debug | Une petite fenêtre egui affiche la pression courante, le nombre d'événements par image et le temps d'une image. |

### Canevas et vue

| Version | Nom | Description |
|---|---|---|
| v0.5.0 | Canevas en tuiles | Le canevas est découpé en tuiles de 64×64 pixels. Une tuile vide n'occupe pas de mémoire. Les tuiles modifiées sont repérées et seules celles-là sont renvoyées au GPU (`write_texture` sur la zone concernée). |
| v0.5.1 | Vue | `Viewport` (crate `glam`) : décalage, zoom, angle et miroir, avec la conversion écran → canevas et son inverse. Un test vérifie qu'un aller-retour redonne la position de départ, pour plusieurs zooms et angles. |
| v0.5.2 | Stylet dans la vue | La matrice de la vue est envoyée au shader et la position du stylet passe par `screen_to_canvas` : le trait apparaît sous la pointe du stylet quelle que soit la vue. |
| v0.5.3 | Déplacer | Espace + glisser (ou bouton du milieu) déplace le canevas. |
| v0.5.4 | Zoomer | Ctrl + molette zoome en gardant le point sous le curseur fixe. |
| v0.5.5 | Pivoter et miroir | R + glisser fait pivoter le canevas, M le retourne horizontalement. |
| v0.5.6 | Damier | Un damier gris s'affiche sous les zones transparentes. |
| v0.5.7 | Filtrage du zoom | Pixels nets (`Nearest`) au-delà de 200 % de zoom, mipmaps quand on dézoome. |
| v0.5.8 | Grand canevas | Un canevas de 4000×3000 se zoome de 5 % à 3200 % et pivote sans que le trait ralentisse. |

### Moteur de trait

| Version | Nom | Description |
|---|---|---|
| v0.6.0 | Pinceau à tampons | Le trait n'est plus un disque par événement. `StrokeEngine` pose des tampons (les empreintes rondes du pinceau) à intervalles réguliers le long du chemin, avec position et pression interpolées (espacement d'environ 10 % du diamètre, 1 pixel minimum). Tous les événements reçus entre deux images sont traités (`Sample`), pas seulement le dernier. |
| v0.6.1 | Forme du tampon | `dab_alpha` : le bord du tampon est adouci selon la dureté, avec au moins 1 pixel d'anti-aliasing. |
| v0.6.2 | Aucun trou | Tests automatiques : un trait rapide ne laisse aucun trou, même quand la pression varie. |
| v0.6.3 | Filtre « 1 € » | Lissage de la position : peu de tremblement quand le stylet va lentement, peu de retard quand il va vite. |
| v0.6.4 | Courbe de pression | La pression est transformée par `p.powf(gamma)` pour régler la sensibilité du pinceau. |
| v0.6.5 | Stabilisateur | Stabilisateur « fil tendu » optionnel, utile pour le lineart : le trait suit le stylet avec un fil de longueur réglable. |

### Trait en cours et rejeu

| Version | Nom | Description |
|---|---|---|
| v0.7.0 | Stroke buffer | Les tampons ne s'écrivent plus directement dans le calque. Ils s'accumulent dans un masque temporaire propre au trait, plafonné à l'opacité, qui est fusionné dans le calque quand on lève le stylet. Deux modes : lavis (le maximum) et accumulation. Fini les taches aux endroits où le trait se recoupe. |
| v0.7.1 | Flux et opacité | Le flux (intensité de chaque tampon) et l'opacité (plafond du trait entier) se règlent séparément. |
| v0.7.2 | Gomme | Même moteur que le pinceau, avec la fusion `DestinationOut` qui retire de la matière au lieu d'en ajouter. |
| v0.7.3 | Enregistrement d'un trait | Les `Sample` bruts d'un trait sont sauvegardés dans un fichier (serde + bincode). |
| v0.7.4 | Rejeu d'un trait | Un fichier de trait peut être rejoué et donne toujours exactement le même résultat. Test de non-régression avec des fichiers dans `tests/fixtures/`. |
| v0.7.5 | Benchmark | `criterion` mesure le temps de calcul d'un trait rejoué. |
| v0.7.6 | Comparaison avec Krita | 20 minutes de croquis avec le même pinceau dans RIMP et dans Krita, comparées côte à côte, puis réglages. |

### Calques

| Version | Nom | Description |
|---|---|---|
| v0.8.0 | Document et calques | `Document` (taille, liste de calques, calque actif) et `Layer` (nom, visibilité, opacité, mode de fusion, tuiles). L'image affichée est la composition des calques du bas vers le haut, tuile par tuile. |
| v0.8.1 | Cinq modes de fusion | Ajout de Superposition (screen) et Incrustation (overlay) aux trois modes existants, avec des tests pour les cinq. |
| v0.8.2 | Cache de composition | L'image aplatie est gardée en mémoire et seules les tuiles modifiées sont recomposées. |

### Annuler et refaire

| Version | Nom | Description |
|---|---|---|
| v0.9.0 | Annuler | Chaque action peut être annulée (Ctrl+Z). Les tuiles sont partagées (`Arc`) et copiées seulement quand on les modifie (`Arc::make_mut`), donc l'historique coûte peu de mémoire. Les actions sont des commandes d'un `enum` : trait, ajout de calque, suppression, déplacement, changement de propriété. |
| v0.9.1 | Refaire | Ctrl+Maj+Z refait ce qui vient d'être annulé. |
| v0.9.2 | Limite mémoire | L'historique oublie les actions les plus anciennes au-delà de 1 Go. |
| v0.9.3 | Tests d'historique | Tests annuler puis refaire pour chaque type de commande. |

### Fichiers et images

| Version | Nom | Description |
|---|---|---|
| v0.10.0 | Fichiers ORA | Enregistrer et ouvrir un document au format ORA : un zip contenant `mimetype` (en premier et non compressé), `stack.xml` et un PNG par calque. |
| v0.10.1 | Aller-retour | Test : sauver puis recharger redonne exactement les mêmes pixels. Un fichier enregistré par RIMP s'ouvre dans Krita. |
| v0.10.2 | Export PNG | Exporte l'image aplatie en PNG (dé-prémultiplication puis conversion en 8 bits par canal). |
| v0.10.3 | Import d'images | Importer un PNG ou un JPEG comme nouveau calque. |
| v0.10.4 | Sauvegarde automatique | Une copie du document est écrite régulièrement dans un thread séparé (copier les `Arc` est presque gratuit), pour ne rien perdre en cas de plantage. |

### Interface

| Version | Nom | Description |
|---|---|---|
| v0.11.0 | Disposition | Le canevas au centre, une barre d'outils fine à gauche, les panneaux couleur et calques à droite. La touche Tab masque les panneaux (mode focus). |
| v0.11.1 | Routage des entrées | Le stylet sur un panneau pilote l'interface, sur le canevas il peint. Si le système n'envoie plus d'événements souris pour le stylet (cas de Wayland), les événements tablette sont traduits en événements egui. |
| v0.11.2 | Raccourcis de base | B (pinceau), E (gomme), `[` et `]` (taille), Ctrl+Z et Ctrl+Maj+Z. |
| v0.11.3 | Panneau de calques | Liste des calques avec un œil de visibilité, l'opacité, le mode de fusion, et le renommage par double-clic. |
| v0.11.4 | Réordonner les calques | Changer l'ordre des calques par glisser-déposer (`egui_dnd`). |
| v0.11.5 | Menu Fichier | Ouvrir, Enregistrer, Importer et Exporter, reliés aux fonctions de la v0.10. |

### Couleur et réglages

| Version | Nom | Description |
|---|---|---|
| v0.12.0 | Sélecteur de couleur | Un anneau de teinte et un carré saturation/luminosité dessinés avec `egui::Painter`. Les conversions de couleur passent par la crate `palette`. |
| v0.12.1 | Couleurs enregistrées | Champ hexadécimal, liste des couleurs récentes et palette sauvegardable. |
| v0.12.2 | Pipette | Alt + clic prélève la couleur de l'image composée. |
| v0.12.3 | Réglages du pinceau | Curseurs pour la taille, l'opacité, le flux, la dureté et l'espacement. |
| v0.12.4 | Éditeur de courbe de pression | Une courbe modifiable à la souris remplace `powf(gamma)`. |
| v0.12.5 | Raccourcis configurables | Les raccourcis sont lus dans un fichier de configuration TOML. |
| v0.12.6 | Préréglages 1 à 9 | Les touches 1 à 9 changent de préréglage de pinceau. |
| v0.12.7 | Alpha utilisable | On peut faire un dessin complet, du croquis aux aplats de couleur, sans ouvrir Krita. |

### Sélection et outils essentiels

| Version | Nom | Description |
|---|---|---|
| v0.13.0 | Sélection au lasso | Tracer un contour à main levée pour sélectionner une zone. |
| v0.13.1 | Déplacer et redimensionner | Déplacer et redimensionner une sélection ou un calque entier (y compris une image importée). |
| v0.13.2 | Rotation | Faire pivoter une sélection ou un calque, avec annulation de toutes les transformations. |
| v0.13.3 | Verrou alpha | Option par calque : peindre ne modifie que les pixels déjà opaques, ce qui permet de colorer sans déborder. |
| v0.13.4 | Masque d'écrêtage | Un calque n'est visible que là où le calque du dessous est opaque. |
| v0.13.5 | Pot de peinture | Remplit une zone de couleur proche du point cliqué, avec une tolérance réglable. |
| v0.13.6 | Remplissage inter-calques | Le pot de peinture peut lire un autre calque (celui du lineart) pour trouver les contours. |
| v0.13.7 | Remplissage au lasso | Remplir de la couleur courante une zone tracée au lasso. |

### Calibration de la tablette

| Version | Nom | Description |
|---|---|---|
| v0.14.0 | Calibration | Un écran affiche 9 cibles (grille 3×3) ; on touche chacune avec le stylet. Le décalage entre cibles et positions mesurées donne une transformation affine calculée par moindres carrés (`nalgebra`), testée puis appliquée avant `screen_to_canvas`. |
| v0.14.1 | Une calibration par tablette | La calibration est sauvegardée par appareil, grâce à l'identifiant USB fourni par octotablet. |
| v0.14.2 | Tablette sans écran | Pour une tablette sans écran, on règle la zone active et on conserve le rapport largeur/hauteur. |
| v0.14.3 | Compatibilité KDE | Vérification que la calibration de RIMP ne se cumule pas avec celle de KDE Plasma 6. |

### Publication

| Version | Nom | Description |
|---|---|---|
| v0.15.0 | AppImage | Un script construit un fichier AppImage : après `chmod +x`, il se lance sur une autre machine Linux sans rien installer d'autre. |
| v0.15.1 | Release GitHub | Publication manuelle de l'AppImage et de sa somme de contrôle sur la page Releases du dépôt. |
| v0.15.2 | Vitrine | Un GIF de dessin dans le README, et les sections Installation et Utilisation (utilisateur) vérifiées sur une machine propre. |
| v0.15.3 | Candidate | Tag `v1.0.0-rc.1` et essai complet à la main. D'autres `rc.N` si des bugs sont trouvés. |

### Première version stable

| Version | Nom | Description |
|---|---|---|
| v1.0.0 | Première version stable | Les fonctionnalités sont figées et les bugs trouvés pendant les candidates sont corrigés. L'AppImage est publiée et le tag est posé sur `main`, après fusion de `dev`. |
| v1.0.1 | Licence libre | La licence temporaire est remplacée par une licence libre (à choisir ensemble : MIT ou Apache-2.0, ou GPL-3.0). |
| v1.0.2 | Ouverture aux contributions | Protection des branches `main` et `dev` (pull request obligatoire, pas de force push) et `CONTRIBUTING.md` adapté aux contributeurs extérieurs (fork puis pull request). |
| v1.0.3 | Correctifs | Correction des bugs signalés, autant de versions que nécessaire (`v1.0.4`, `v1.0.5`…). |

## Après la v1.0.0

### Animation

| Version | Nom | Description |
|---|---|---|
| v1.1.0 | Images d'animation | Un document peut contenir plusieurs images. Chaque image a ses propres calques, on choisit l'image courante et on dessine dessus. Un panneau « timeline » montre une case par image. Dupliquer une image coûte presque rien en mémoire, car les tuiles sont partagées. |
| v1.1.1 | Gérer les images | Boutons pour ajouter, supprimer et dupliquer une image, avec annuler/refaire. |
| v1.1.2 | Navigation | Flèche gauche et flèche droite passent à l'image précédente et suivante. |
| v1.1.3 | Cadence | Le nombre d'images par seconde se règle par document (12 par défaut). |
| v1.1.4 | Sauvegarde de l'animation | Les images sont enregistrées dans le fichier ORA avec un fichier supplémentaire propre à RIMP. Le but est que le fichier reste ouvrable ailleurs comme une image fixe (à vérifier avec Krita). |
| v1.2.0 | Lecture | Boutons lecture et pause. L'animation se joue à la cadence réglée sans ralentir le dessin. |
| v1.2.1 | Boucle | Lecture en boucle, et en aller-retour. |
| v1.2.2 | Plage de lecture | Choisir la première et la dernière image à jouer. |
| v1.3.0 | Papier calque | Les images voisines s'affichent en transparence sous l'image courante (onion skin), pour dessiner un mouvement en voyant le précédent. |
| v1.3.1 | Nombre d'images | Régler combien d'images avant et après sont visibles. |
| v1.3.2 | Couleurs | Les images précédentes sont teintées en rouge et les suivantes en vert. |
| v1.3.3 | Raccourci | Un raccourci active et désactive le papier calque. |
| v1.4.0 | Export en GIF | Exporte toute l'animation en GIF animé (crate `gif`). |
| v1.4.1 | Export en images | Exporte une suite de PNG numérotés (`frame_0001.png`, `frame_0002.png`…). |
| v1.4.2 | Export en APNG | Exporte en PNG animé. |
| v1.4.3 | Export en vidéo | Exporte en MP4 ou WebM avec ffmpeg, si ffmpeg est installé. |
| v1.4.4 | Export d'une plage | Exporter seulement une partie des images. |
| v1.5.0 | Durée des images | Chaque image a sa propre durée (elle peut être tenue plusieurs pas), visible dans la timeline et respectée à la lecture et à l'export. |
| v1.5.1 | Copier-coller d'images | Copier des images et les coller ailleurs dans la timeline. |
| v1.5.2 | Déplacer des images | Changer l'ordre des images par glisser-déposer. |
| v1.5.3 | Sélection multiple | Sélectionner plusieurs images pour les déplacer, copier ou supprimer d'un coup. |

### Retouche d'images

| Version | Nom | Description |
|---|---|---|
| v1.6.0 | Recadrage | Recadrer ou agrandir le canevas (toutes les images de l'animation suivent). |
| v1.6.1 | Redimensionnement | Redimensionner l'image entière, avec rééchantillonnage pour garder une bonne qualité. |
| v1.6.2 | Annuler la retouche | Annuler et refaire pour le recadrage et le redimensionnement. |
| v1.6.3 | Rotation et retournement | Pivoter l'image de 90° et la retourner, horizontalement ou verticalement. |
| v1.7.0 | Luminosité et contraste | Régler la luminosité et le contraste d'un calque ou d'une sélection. |
| v1.7.1 | Teinte et saturation | Régler la teinte et la saturation. |
| v1.7.2 | Niveaux | Régler les niveaux (points noir, blanc et gris). |
| v1.7.3 | Aperçu en direct | Voir le résultat d'un réglage en direct avant de le valider. |
| v1.8.0 | Flou | Flou gaussien avec rayon réglable. |
| v1.8.1 | Netteté | Accentuer la netteté (masque flou). |
| v1.8.2 | Sur sélection | Appliquer flou et netteté seulement à la zone sélectionnée. |

### Pinceaux avancés

| Version | Nom | Description |
|---|---|---|
| v1.9.0 | Couleur portée | Mélangeur : à chaque tampon, le pinceau prélève la couleur sous lui, et la couleur portée évolue (`lerp`). La couleur déposée mélange celle du pinceau et celle qu'il porte. |
| v1.9.1 | Paramètres du mélange | Réglages de charge, de dilution et de persistance. |
| v1.9.2 | Tests du mélange | Vérification avec le rejeu de traits. |
| v1.10.0 | Mélange spectral | Port de l'algorithme de spectral.js (licence MIT) : jaune + bleu donne du vert, comme avec de la vraie peinture. |
| v1.10.1 | Force du mélange | Choisir entre mélange simple et mélange spectral, avec un curseur. |
| v1.10.2 | Test jaune + bleu | Test automatique et comparaison avec le mélange simple. |
| v1.11.0 | Grain de papier | Une texture de papier appliquée dans l'espace du canevas, donc immobile pendant qu'on dessine. |
| v1.11.1 | Pointe en image | Une pointe de pinceau définie par une image PNG. |
| v1.11.2 | Pointe orientée | La pointe tourne pour suivre la direction du trait. |
| v1.12.0 | Variations aléatoires | Petites variations aléatoires de taille, d'angle et de position, avec un générateur à graine fixe (`rand_pcg`) pour que le rejeu donne toujours le même résultat. |
| v1.12.1 | Préréglages en fichiers | Les préréglages de pinceau sont enregistrés en TOML ou RON. |
| v1.12.2 | Partage de préréglages | Importer et exporter des préréglages. |

### Performance et distribution

| Version | Nom | Description |
|---|---|---|
| v1.13.0 | Calcul en parallèle | Les tampons d'un trait sont calculés en parallèle sur plusieurs cœurs avec `rayon`. |
| v1.13.1 | Mesures | Benchmark avec le rejeu de traits, avant et après. |
| v1.13.2 | Réglage fin | Optimisations guidées par les mesures. |
| v1.14.0 | Pinceaux sur GPU | Les tampons sont calculés par un compute shader, seulement si un pinceau doux de 500 pixels reste sous 60 images par seconde après la v1.13. |
| v1.14.1 | Comparaison CPU et GPU | Le rejeu vérifie que le rendu est identique sur CPU et sur GPU. |
| v1.14.2 | Choix du moteur | Une option pour choisir le calcul sur CPU ou sur GPU. |
| v1.15.0 | Flatpak | Un paquet Flatpak, avec la permission d'accéder au socket Wayland. |
| v1.15.1 | Test Flatpak | Essai sur une autre distribution Linux. |
| v1.15.2 | Guide d'installation | La section Installation du README est mise à jour avec Flatpak. |

## Jalons à retenir

- **v0.4.0** : premier trait à pression
- **v0.7.4** : trait rejouable à l'identique
- **v0.12.7** : alpha utilisable pour dessiner
- **v1.0.0** : première version stable
- **v1.4.0** : première animation exportée