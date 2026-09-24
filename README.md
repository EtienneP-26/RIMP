# **RIMP**

> RIMP est un éditeur d'images pour Linux, pensé d'abord pour le dessin à la tablette graphique.

**Dernière version stable** : aucune pour l'instant

**En développement** : v0.1.0

## Sommaire

- [Description](#description)
- [Installation](#installation)
- [Utilisation](#utilisation)
- [Architecture](#architecture)
- [Tests](#tests)
- [Roadmap](#roadmap)
- [Licence](#licence)
- [Équipe](#équipe)

## Description
RIMP sert à dessiner (pinceaux sensibles à la pression, calques, annuler/refaire), mais aussi à retoucher des images existantes : on peut importer une image (PNG, JPEG), la déplacer, la redimensionner, dessiner ou effacer dessus, puis l'exporter.

- **Objectif** : une application de dessin et de retouche simple, utilisable de bout en bout, publiée à partir de la v1.0.0.
- **Plateforme** : Linux uniquement pour l'instant.
- **Technologies** : Rust, wgpu (affichage), egui (interface), octotablet (tablette graphique).
- **Formats** : ORA (lisible par Krita et MyPaint), PNG, JPEG.
- **Prévu après la v1.0.0** : animation image par image, retouche d'images (recadrage, réglages de couleur, flou), pinceaux avancés.

## Installation

### Utilisateur
Aucune version stable n'est publiée pour l'instant. À partir de la v1.0.0 :

1. Télécharger le fichier `.AppImage` depuis la page **Releases** du dépôt.
2. Le rendre exécutable :
   ```bash
   chmod +x RIMP-x86_64.AppImage
   ```

Prérequis : Linux et une carte graphique compatible Vulkan. Une tablette graphique est recommandée, la souris fonctionne aussi.

En attendant, il est possible de compiler soi-même (voir plus bas), ou avec `cargo install` :
```bash
cargo install --git https://github.com/EtienneP-26/RIMP.git
```

### Développeur
Prérequis :
- Linux
- [Rust via rustup](https://rustup.rs)
- Docker (optionnel, pour l'environnement conteneurisé et le linter)

```bash
git clone git@github.com:EtienneP-26/RIMP.git
cd RIMP
./scripts/add-git-hooks   # obligatoire
cargo build --release
```

Avant de coder, lis [CONTRIBUTING.md](CONTRIBUTING.md) : format des commits (`type(scope): message`), nom des branches (`type/description-courte`, créées depuis `dev`) et hooks git.

Les bibliothèques système nécessaires à la fenêtre et au GPU (Wayland, Vulkan) seront listées ici à la v0.3.2.

## Utilisation

### Utilisateur
À partir de la v1.0.0 :
```bash
./RIMP-x86_64.AppImage
```

### Développeur
```bash
cargo run --release
```

Utilise toujours `--release` pour juger la sensation du trait : le mode debug peut être 10 à 50 fois plus lent.

Pour travailler dans le conteneur Docker :
```bash
docker compose run --rm env
```

La première fenêtre arrive à la v0.3.0 et le premier trait à pression à la v0.4.0.

## Architecture
```
.
├── .git/hooks/                 # (Ces fichiers ne sont pas commits/trackés)
│   ├── commit-msg              # Vérification du nom de commit
│   └── pre-commit              # Vérification pre-commits (secrets, .env)
├── src/                        # Code source
│   ├── main.rs                 # Point d'entrée
│   ├── lib.rs                  # Déclare les modules (permet tests d'intégration et benchmarks)
│   ├── core/                   # Tuiles, calques, modes de fusion, annuler/refaire
│   ├── brush/                  # Moteur de trait
│   ├── io/                     # Lecture/écriture de fichiers (ORA, PNG, JPEG)
│   └── app/                    # Fenêtre, interface, tablette
├── docs/                       # Documentation complémentaire
│   ├── index.md
│   └── ROADMAP.md              # Toutes les versions prévues
├── scripts/
│   └── add-git-hooks           # Installe les hooks git locaux
│
├── .gitignore                  # Fichiers à ne pas Commit
├── Cargo.toml                  # Configuration du projet Rust
├── Cargo.lock                  # Versions exactes des dépendances
├── docker-compose.yml          # Environnement Docker (Rust) et linter
├── LICENSE                     # Licence (temporaire)
├── README.md                   # Description rapide du projet
└── CONTRIBUTING.md             # Guide pour les contributeurs
```

Les modules `core`, `brush` et `io` n'ouvrent jamais de fenêtre : seul `app` s'occupe de l'affichage.

## Tests
Il n'y a aucun test pour l'instant : les commandes ci-dessous fonctionnent mais ne vérifient encore rien. Les premiers tests arrivent à la v0.2.0. Ils seront écrits dans les fichiers de `src/` (`#[cfg(test)]`).

```bash
cargo test                       # tests
cargo fmt --check                # formatage
cargo clippy -- -D warnings      # lint Rust
docker compose run --rm lint     # Super-Linter
```

## Roadmap
Une version s'écrit `vX.Y.Z` :

| Chiffre | Signification | Exemple |
|---|---|---|
| `X` (le premier) | Version stable et fonctionnelle | `v1.0.0` |
| `Y` (le deuxième) | Modification majeure : une fonctionnalité importante | `v0.4.0` |
| `Z` (le troisième) | Modification mineure : petit ajout, réglage ou correction | `v0.4.2` |

Tant que `X` vaut 0, l'application est en construction et tout peut changer. Il n'y a pas de date : une version sort quand elle est prête, et il peut y avoir autant de versions mineures que nécessaire. Chaque version est un tag git.

Toutes les versions, avec leur nom et leur description, sont dans [docs/ROADMAP.md](docs/ROADMAP.md).

## Licence
Licence temporaire (voir [LICENSE](LICENSE)) : RIMP peut être utilisé librement, mais pas modifié ni redistribué sans l'accord de l'auteur. Elle sera remplacée par une licence libre après la v1.0.0, et les contributions extérieures ne sont pas acceptées d'ici là.

## Équipe
| Nom | Rôle | GitHub |
|---|---|---|
| Etienne Pouille | Dev | EtienneP-26 |