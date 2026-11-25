# merge_hosts

Outil Rust en ligne de commande pour fusionner deux fichiers hosts sans perdre leurs commentaires. Aucune dépendance externe.

## Utilisation

```sh
./merge_hosts fichier1 fichier2 fichier_sortie
./merge_hosts --help
```

Le fichier de sortie doit être inexistant. Le programme refuse de l'écraser, y compris lorsqu'il s'agit d'un fichier d'entrée. Il ne modifie jamais automatiquement `/etc/hosts`. Lire les deux sources doit être autorisé et le répertoire de sortie doit être accessible en écriture.

## Règles de fusion

- Même entrée = même IP et mêmes noms d'hôtes dans le même ordre ; les différences d'espacement sont ignorées pour la comparaison.
- Les adresses IP sont comparées comme du texte : deux écritures différentes d'une IPv6 ne sont pas normalisées.
- Une entrée reste à sa première position ; les suivantes enrichissent ses commentaires.
- Les commentaires de fin de ligne sont séparés par ` | `.
- Avant chaque ajout, chaque commentaire est comparé à la liste existante après suppression des espaces aux extrémités. La comparaison respecte la casse.
- Le caractère `|` est réservé comme séparateur de commentaires ; il n'existe pas de mécanisme d'échappement.
- Les doublons sont fusionnés entre fichiers et à l'intérieur de chaque fichier.
- Les commentaires seuls, titres de sections et lignes vides restent présents, dans l'ordre des deux sources. Les sections ne sont pas regroupées ; une section peut devenir vide. Réimporter un fichier peut répéter ses titres et commentaires seuls, mais pas les commentaires des entrées fusionnées.
- Des IP différentes restent distinctes : aucun conflit d'adresse n'est résolu automatiquement.
- Les fichiers doivent être en UTF-8. Les fins de ligne sont uniformisées en CRLF si le premier fichier contient du CRLF, sinon en LF. Une sortie non vide se termine par un saut de ligne.
- Les entrées ne sont pas validées syntaxiquement : toute ligne non vide avec du contenu avant `#` est traitée comme une entrée.

### Exemple

Premier fichier :

```text
# Serveurs
192.168.1.10 nas.local # NAS principal | Stockage
```

Second fichier :

```text
# Compléments
192.168.1.10 nas.local # Stockage | Sauvegarde
```

Résultat :

```text
# Serveurs
192.168.1.10 nas.local # NAS principal | Stockage | Sauvegarde
# Compléments
```

## Compiler et tester

Installer Rust avec rustup, puis :

```sh
cargo test --locked
cargo build --release --locked
./target/release/merge_hosts hosts1 hosts2 hosts_fusionne
```

Le programme renvoie 0 en cas de succès et un code non nul en cas d'erreur. Une erreur d'écriture peut laisser un fichier de sortie incomplet : le supprimer avant de recommencer.

## Binaires des releases

Chaque tag `v*` déclenche les tests puis la compilation et publie une release GitHub avec :

- `merge_hosts-<tag>-x86_64-unknown-linux-gnu.tar.gz` : Linux x86_64 (compilé sur Ubuntu 22.04 ; une glibc compatible est nécessaire).
- `merge_hosts-<tag>-x86_64-apple-darwin.tar.gz` : macOS Intel, cible minimale macOS 12.
- `merge_hosts-<tag>-aarch64-apple-darwin.tar.gz` : macOS Apple Silicon, cible minimale macOS 12.
- `SHA256SUMS` : sommes de contrôle des archives.

Les binaires macOS ne sont ni signés ni notarisés. Les sommes SHA-256 vérifient l'intégrité, pas l'identité de l'éditeur.

Après téléchargement, extraire l'archive puis exécuter le binaire :

```sh
tar -xzf merge_hosts-v0.1.0-aarch64-apple-darwin.tar.gz
./merge_hosts hosts1 hosts2 hosts_fusionne
```

Adapter le nom d'archive à son système et à la version téléchargée.

## Publier une version

Mettre à jour la version de Cargo.toml et Cargo.lock, committer puis :

```sh
git push origin main
git tag v0.1.0
git push origin v0.1.0
```

Le tag doit pointer sur le commit à publier. Aucune release n'est créée par un simple push sur main. Les pushes et pull requests exécutent les tests sur les trois plateformes. Les archives passent par GitHub Actions avant leur publication. Le workflow utilise le GITHUB_TOKEN du dépôt avec `contents: write` uniquement pour le job de publication ; aucun secret personnel à ajouter. Les politiques du dépôt doivent autoriser GitHub Actions et cette permission.
