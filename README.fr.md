# Calibre 60

**Un chronomètre et compte à rebours natif en Rust, avec un cadran de chronographe classique.**

[English](README.md) · Français

[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/egui-0.31.1-6A5ACD)](https://github.com/emilk/egui)
[![Debian 13](https://img.shields.io/badge/Linux-Debian%2013-A81D33?logo=debian&logoColor=white)](https://www.debian.org/)
[![Windows x64](https://img.shields.io/badge/Windows-x64-0078D4?logo=windows11&logoColor=white)](https://www.microsoft.com/windows/)
[![CI](https://github.com/sjeje42/calibre-60/actions/workflows/ci.yml/badge.svg)](https://github.com/sjeje42/calibre-60/actions/workflows/ci.yml)
[![Licence : GPL-3.0](https://img.shields.io/badge/Licence-GPL--3.0-blue.svg)](LICENSE)

Calibre 60 propose des cadrans ivoire classique et bleu marine entièrement dessinés en code,
une aiguille fluide des secondes, un petit compteur de 30 minutes et un affichage numérique
heures/minutes/secondes/millisecondes.
L'application est conçue pour Linux, notamment Debian 13, et Windows, avec
egui/eframe et rodio.

L'interface peut être basculée instantanément entre **français et anglais**.
La langue choisie est mémorisée entre les sessions.

## Fonctionnalités

- Chronomètre et compte à rebours indépendants : changer d'onglet ne les arrête pas.
- Démarrage, pause, reprise, remise à zéro et affichage des millisecondes.
- Mode compact pour garder le temps et les commandes essentielles dans un coin de l'écran.
- Thèmes d'interface clair/sombre et cadrans ivoire classique/bleu marine, tous persistants.
- Cadran bleu marine avec bordure du compteur 30 minutes en bâtonnets jaunes et rouges, dessinée entièrement en code sans image matricielle.
- Temps de chaque tour et temps cumulé, avec les tours les plus récents en premier.
- Statistiques des tours : meilleur tour, plus lent et moyenne.
- Copie des tours dans le presse-papiers au format CSV, séparateur point-virgule.
- Export direct des tours dans un fichier CSV UTF-8 dans le dossier Téléchargements.
- Compte à rebours jusqu'à 99 h 59 min 59 s.
- Cinq préréglages personnalisables et persistants ; valeurs par défaut : 1, 3, 5, 10 et 25 minutes.
- Quatre sonneries, réglage du volume, intervalle de répétition, test du son et mode muet.
- Notification système native à la fin d'un compte à rebours.
- Fonctionnement en arrière-plan dans la zone de notification sous Linux et Windows.
- Menu de zone de notification : ouvrir, marche/pause et quitter.
- Alarme gérée dans un fil d'exécution indépendant du rafraîchissement de la fenêtre.
- Préférences persistantes : langue, préréglages, alarme et dernière durée de compte à rebours.
- Interface vectorielle redimensionnable, adaptée aux écrans haute définition.
- Raccourcis clavier et fonctionnement sans compte ni service en ligne.

## État du projet et téléchargements

La [compilation automatique](../../actions/workflows/ci.yml) contrôle le formatage avec
`rustfmt`, exécute **Clippy avec les avertissements traités comme des erreurs**, audite
les dépendances avec `cargo-audit`, puis lance les tests et compile le programme sous
Debian 13 et Windows. Consulte le dernier lancement pour connaître son résultat réel.

Après une exécution réussie, les builds de développement sont disponibles dans la
section **Artifacts** : `calibre-60-debian13-x64` et `calibre-60-windows-x64`.
Pour une installation normale, privilégie les fichiers joints à une GitHub Release taguée.

Lorsqu'un tag `v*` est poussé, le workflow de publication construit automatiquement
un paquet **`.deb` Debian 13**, une archive **Linux x64 `.tar.gz`**, une archive
**Windows x64 `.zip`**, génère les sommes **SHA-256**, puis crée la GitHub Release
correspondante. Le paquet Debian installe également l'icône et l'entrée du menu
Applications via le fichier `.desktop`.

La CI valide la qualité statique, l'audit des dépendances, la compilation et les tests
automatisés du moteur temporel et des statistiques. Elle ne peut pas valider complètement
le matériel audio réel, l'affichage des notifications, l'intégration de la zone de
notification de GNOME ou le comportement pendant la veille.

## Compilation sous Debian 13

Installe les dépendances :

```bash
sudo apt update
sudo apt install git curl ca-certificates build-essential pkg-config \
  libasound2-dev libxkbcommon-dev libwayland-dev \
  libegl1-mesa-dev libgl1-mesa-dev
```

Installe ensuite une version stable récente de Rust avec [rustup](https://rustup.rs/).
Rouvre ton terminal si la commande `cargo` n'est pas reconnue.

Clone le dépôt puis lance :

```bash
git clone https://github.com/sjeje42/calibre-60.git
cd calibre-60
cargo test
cargo run --release
```

Le programme compilé se trouve dans `target/release/calibre-60`.
Il nécessite un environnement graphique X11 ou Wayland, des pilotes graphiques
fonctionnels et les bibliothèques partagées Linux.

Pour le copier dans le dossier des exécutables de ton utilisateur :

```bash
install -Dm755 target/release/calibre-60 "$HOME/.local/bin/calibre-60"
```

## Compilation sous Windows

1. Installe [Rust pour Windows](https://rustup.rs/).
2. Installe les outils Microsoft C++ proposés, avec la charge de travail
   **Développement Desktop en C++** et un SDK Windows.
3. Ouvre un nouveau terminal PowerShell dans le dossier du dépôt cloné.

```powershell
cargo test
cargo build --release
.\target\release\calibre-60.exe
```

L'exécutable se trouve dans `target\release\calibre-60.exe`.
Rust sert à compiler ; il n'est pas nécessaire pour lancer ensuite l'application.

## Utilisation

Le sélecteur **FR / EN**, placé sous le titre, change immédiatement la langue de
l'interface. Ce choix est mémorisé et s'applique également aux notifications de
fin de compte à rebours et au menu de la zone de notification.

### Chronomètre

Sélectionne **Chronomètre**, puis **Démarrer / Reprendre**. Clique sur **Tour** pour
enregistrer les temps intermédiaires. Le panneau affiche automatiquement le meilleur
tour, le plus lent et la moyenne.

**Copier CSV** place le tableau dans le presse-papiers. **Exporter CSV** écrit un
fichier CSV UTF-8 dans ton dossier Téléchargements. Les intitulés de colonnes suivent
la langue actuellement sélectionnée.

### Compte à rebours

Sélectionne **Compte à rebours**. À l'arrêt, règle les heures, minutes et secondes,
puis clique sur **Appliquer**. Tu peux aussi choisir directement l'un des cinq
préréglages rapides.

Ouvre **Personnaliser les préréglages** pour modifier les cinq durées. Elles sont
enregistrées automatiquement. **Valeurs par défaut** remet 1, 3, 5, 10 et 25 minutes.

À la fin du compte à rebours, Calibre 60 affiche son alerte visuelle, joue la sonnerie
sélectionnée si le son est actif et envoie une notification système native.

### Mode compact

Le bouton **Mode compact** réduit la fenêtre à une barre discrète affichant le type de
minuteur, le temps courant et les commandes essentielles. En chronomètre, **Tour** reste
disponible pendant la marche. Le bouton **Vue normale** restaure le grand cadran et la
taille précédente de la fenêtre.

### Fonctionnement en arrière-plan

Sous Linux et Windows, **Réduire en arrière-plan** envoie Calibre 60 dans la zone de
notification. Si un minuteur est actif, fermer la fenêtre principale le laisse tourner.
Le menu de l'icône permet d'ouvrir Calibre 60, de mettre en marche/pause ou de quitter.

Sous GNOME, l'affichage des icônes de zone de notification peut nécessiter une
extension AppIndicator/KStatusNotifier.

## Raccourcis clavier

Ils fonctionnent quand la fenêtre a le focus et qu'aucun champ de durée n'est en cours de saisie.

| Touche | Action |
| --- | --- |
| Espace | Démarrer ou mettre en pause le mode sélectionné |
| L | Enregistrer un tour dans le chronomètre |
| R | Réinitialiser le mode sélectionné, uniquement à l'arrêt |
| Échap | Arrêter l'alarme d'un compte à rebours terminé |

## Précision et durée de vie des données

Le moteur utilise `Instant`, l'horloge monotone de Rust. Le temps est calculé à partir
d'horodatages et non en additionnant un incrément à chaque image ; un retard de
rafraîchissement ne crée donc pas de dérive cumulative. L'affichage actif demande une
mise à jour environ toutes les 16 ms.

Afficher les millisecondes ne garantit **pas une précision physique de ±1 ms**.
L'ordonnanceur du système, l'horloge matérielle, le traitement des entrées et la mise
en mémoire tampon audio influencent les délais observés. L'alarme utilise une échéance
absolue, mais Calibre 60 n'est pas un système temps réel dur.

L'application ne bloque pas la mise en veille et ne réveille pas un ordinateur suspendu.
Le comportement de l'horloge monotone pendant une suspension dépend de la plateforme.

Les préférences sont conservées, mais la progression d'un chronomètre ou d'un compte à
rebours actif et la session de tours ne sont pas restaurées après la fermeture complète
du programme. **Quitter** depuis la zone de notification termine réellement l'application.

## Développement

| Fichier | Rôle |
| --- | --- |
| `src/main.rs` | Point d'entrée, configuration de la fenêtre et démarrage d'eframe |
| `src/app.rs` | État de l'application, commandes, raccourcis, préréglages et interface |
| `src/clock.rs` | Moteur temporel et tests unitaires déterministes |
| `src/dial.rs` | Dessin vectoriel du chronographe |
| `src/alarm.rs` | Fil d'exécution indépendant pour l'alarme sonore |
| `src/notifications.rs` | Notification native de fin de compte à rebours |
| `src/tray.rs` | Intégration de la zone de notification Linux/Windows |
| `src/laps.rs` | Calcul des statistiques de tours et export CSV |
| `src/i18n.rs` | Traductions français/anglais |
| `src/settings.rs` | Préférences persistantes |
| `packaging/linux/` | Fichier `.desktop` du paquet Debian |
| `assets/` | Icônes et ressources de distribution |

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

`Cargo.lock` est versionné afin que les builds de l'application utilisent le même jeu de dépendances résolues.

## Licence

Consulte [LICENSE](LICENSE) pour la licence GNU GPL version 3.
