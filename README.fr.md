# Calibre 60

**Un chronomètre et compte à rebours natif en Rust, avec un cadran de chronographe classique.**

[English](README.md) · Français

Calibre 60 associe un cadran ivoire, une aiguille fluide des secondes, un petit
compteur de 30 minutes et un affichage numérique heures/minutes/secondes/millisecondes.
L'application est conçue pour Linux, notamment Debian 13, et Windows, avec
egui/eframe et rodio.

L'interface de cette version est **en français**. Le README principal est en anglais,
mais une interface anglaise n'est pas encore implémentée.

## Fonctionnalités

- Chronomètre et compte à rebours indépendants : changer d'onglet ne les arrête pas.
- Démarrage, pause, reprise et remise à zéro.
- Temps de chaque tour et temps cumulé, avec les tours les plus récents en premier.
- Copie des tours dans le presse-papiers au format CSV, séparateur point-virgule.
- Compte à rebours jusqu'à 99 h 59 min 59 s ; préréglages de 1, 3, 5, 10 et 25 minutes.
- Alarme répétée, activation du son et message de fin visible dans les deux onglets.
- Alarme gérée dans un fil d'exécution indépendant du rafraîchissement de la fenêtre.
- Interface vectorielle redimensionnable, adaptée aux écrans haute définition.
- Raccourcis clavier et fonctionnement sans compte ni service en ligne.

## État du projet et téléchargements

Il s'agit d'une première version du code source. La
[compilation automatique](../../actions/workflows/ci.yml) compile le programme
et exécute les tests du moteur temporel sous Debian 13 et Windows.
Consulte le dernier lancement pour connaître son résultat réel : la présence
du workflow ne signifie pas que la compilation a réussi.

Après une exécution réussie, les fichiers sont disponibles dans sa section
**Artifacts** : `calibre-60-debian13-x64` et `calibre-60-windows-x64`.
Le téléchargement nécessite l'accès à ce dépôt privé.
Aucun installateur ni paquet de distribution signé n'est fourni pour l'instant.

Ces vérifications portent sur la compilation et les calculs de durée. Elles ne
testent pas la fenêtre, la mise à l'échelle, le presse-papiers, le son ou la veille
sur un véritable poste de travail.

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
fonctionnels et les bibliothèques partagées Linux. Ce n'est pas un binaire
portable entièrement statique.

Pour le copier dans le dossier des exécutables de ton utilisateur :

```bash
install -Dm755 target/release/calibre-60 "$HOME/.local/bin/calibre-60"
```

Si `~/.local/bin` figure dans ton PATH, tu peux ensuite lancer `calibre-60`
depuis un terminal.

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
La compilation automatique utilise MSVC sur Windows Server 2022 ; le comportement
sur ton bureau Windows reste à vérifier.

## Utilisation

### Chronomètre

1. Sélectionne **Chronomètre** et clique sur **Démarrer / Reprendre**.
2. Clique sur **Tour** pour enregistrer un temps intermédiaire.
3. Utilise **Pause**, puis le même bouton pour reprendre.
4. Une fois arrêté, **Réinitialiser** remet le chronomètre à zéro et efface les tours.

Le bouton **Copier CSV** copie les numéros de tour, les durées de tour et les temps
cumulés. Colle le résultat dans un tableur ou un éditeur de texte.

### Compte à rebours

1. Sélectionne **Compte à rebours**.
2. À l'arrêt, règle les heures, minutes et secondes, puis clique sur **Appliquer**.
   Tu peux aussi sélectionner directement un préréglage.
3. Clique sur **Démarrer / Reprendre**.
4. À la fin, clique sur **Arrêter l'alarme** ou appuie sur Échap.

Modifier les champs sans cliquer sur **Appliquer** ne change pas la durée utilisée.
Appliquer une durée ou choisir un préréglage efface la progression d'un compte à
rebours en pause. Une durée nulle ne peut pas démarrer.
**Relancer** redémarre un compte à rebours terminé avec sa durée appliquée.
La case **Son** permet de couper ou réactiver le son, y compris pendant l'alarme.

La grande aiguille fait un tour en 60 secondes ; la petite, en 30 minutes.
En mode compte à rebours, elles représentent la durée restante et reculent.
L'affichage numérique indique la durée complète, heures comprises.

## Raccourcis clavier

Ils fonctionnent quand la fenêtre a le focus et qu'aucun champ de durée
n'est en cours de saisie.

| Touche | Action |
| --- | --- |
| Espace | Démarrer ou mettre en pause le mode sélectionné |
| L | Enregistrer un tour dans l'onglet Chronomètre |
| R | Réinitialiser le mode sélectionné, uniquement à l'arrêt |
| Échap | Arrêter l'alarme d'un compte à rebours terminé |

## Précision et durée de vie des données

Le moteur utilise [Instant, l'horloge monotone de Rust](https://doc.rust-lang.org/std/time/struct.Instant.html)
et des durées entières. Le temps est calculé à partir d'horodatages ; il n'est pas
obtenu en additionnant un incrément à chaque image. Un retard de rafraîchissement
ne crée donc pas de dérive cumulative. L'affichage actif demande une mise à jour
environ toutes les 16 ms : chaque milliseconde n'est pas dessinée.

Afficher les millisecondes ne garantit **pas une précision physique de ±1 ms**.
Le traitement des clics, l'ordonnanceur du système, l'horloge matérielle et la
mise en mémoire tampon audio influencent les délais observés. L'alarme utilise
une échéance absolue, mais n'offre pas de garantie audio en temps réel strict.

Garde l'ordinateur éveillé pour une mesure continue. Le comportement de
`Instant` pendant la veille dépend de la plateforme. L'application ne bloque pas
la veille, ne réveille pas l'ordinateur et ne garantit pas d'alarme pendant sa suspension.

Fermer l'application arrête l'alarme et efface les temps, les tours et les réglages.
Il n'y a ni service d'arrière-plan, ni icône de zone de notification, ni sauvegarde
de session. Si la sortie audio ne peut pas être initialisée, un message s'affiche
et l'alerte visuelle reste disponible. Relance l'application après avoir corrigé
la sortie audio.

## Développement

| Fichier | Rôle |
| --- | --- |
| `src/main.rs` | État de l'application, commandes, raccourcis et liste des tours |
| `src/clock.rs` | Moteur temporel et tests unitaires déterministes |
| `src/dial.rs` | Dessin vectoriel du chronographe |
| `src/alarm.rs` | Fil d'exécution indépendant pour l'alarme sonore |

```bash
cargo test
cargo build --release
cargo fmt
```

Les versions des dépendances directes sont fixées dans `Cargo.toml`.
Cargo génère `Cargo.lock` lors de la première compilation ; ce fichier n'est pas ignoré.
Cette première publication du code source ne contient pas de fichier de verrouillage généré.
Le workflow Debian joint son `Cargo.lock` à l'exécutable.
Il faudra conserver dans Git un fichier généré et validé pour figer aussi les
dépendances transitives ; actuellement, chaque plateforme les résout indépendamment.

## Licence

Consulte [LICENSE](LICENSE) pour la licence GNU GPL version 3 déjà présente dans le dépôt.
