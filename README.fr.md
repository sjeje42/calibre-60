# Calibre 60

**Un chronomètre et compte à rebours natif en Rust, avec un cadran de chronographe classique.**

[English](README.md) · Français

Calibre 60 associe un cadran ivoire, une aiguille fluide des secondes, un petit
compteur de 30 minutes et un affichage numérique heures/minutes/secondes/millisecondes.
L'application est conçue pour Linux, notamment Debian 13, et Windows, avec
egui/eframe et rodio.

L'interface peut être basculée instantanément entre **français et anglais**.
La langue choisie est mémorisée entre les sessions.

## Fonctionnalités

- Chronomètre et compte à rebours indépendants : changer d'onglet ne les arrête pas.
- Démarrage, pause, reprise, remise à zéro et affichage des millisecondes.
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

La [compilation automatique](../../actions/workflows/ci.yml) compile le programme
et exécute les tests sous Debian 13 et Windows. Consulte le dernier lancement pour
connaître son résultat réel.

Après une exécution réussie, les fichiers sont disponibles dans la section
**Artifacts** : `calibre-60-debian13-x64` et `calibre-60-windows-x64`.
Le téléchargement nécessite l'accès à ce dépôt privé.
Aucun installateur ni paquet de distribution signé n'est fourni pour l'instant.

La CI valide la compilation et les tests automatisés du moteur temporel et des
statistiques. Elle ne peut pas valider complètement le matériel audio réel,
l'affichage des notifications, l'intégration de la zone de notification de GNOME
ou le comportement pendant la veille.

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

Avec ton authentification GitHub déjà configurée :

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
| `src/main.rs` | État de l'application, commandes, raccourcis, préréglages et interface |
| `src/clock.rs` | Moteur temporel et tests unitaires déterministes |
| `src/dial.rs` | Dessin vectoriel du chronographe |
| `src/alarm.rs` | Fil d'exécution indépendant pour l'alarme sonore |
| `src/notifications.rs` | Notification native de fin de compte à rebours |
| `src/tray.rs` | Intégration de la zone de notification Linux/Windows |
| `src/laps.rs` | Calcul des statistiques de tours et export CSV |
| `src/i18n.rs` | Traductions français/anglais |
| `src/settings.rs` | Préférences persistantes |

```bash
cargo test
cargo build --release
cargo fmt
```

Les versions des dépendances directes sont fixées dans `Cargo.toml`.

## Licence

Consulte [LICENSE](LICENSE) pour la licence GNU GPL version 3.
