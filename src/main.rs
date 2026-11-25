use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

fn analyser(ligne: &str) -> Option<(String, &str, &str)> {
    let (entree, commentaire) = ligne.split_once('#').unwrap_or((ligne, ""));
    let cle = entree.split_whitespace().collect::<Vec<_>>().join(" ");
    if cle.is_empty() {
        None
    } else {
        Some((cle, entree.trim_end(), commentaire.trim()))
    }
}

fn commentaires_uniques(commentaires: &str, liste: &mut Vec<String>) {
    for morceau in commentaires.split('|').map(str::trim) {
        if !morceau.is_empty() && !liste.iter().any(|existant| existant == morceau) {
            liste.push(morceau.to_owned());
        }
    }
}

fn fusionner(premier: &str, second: &str) -> (String, usize) {
    let mut lignes: Vec<String> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();
    let mut compteur = 0;
    let separateur = if premier.contains("\r\n") { "\r\n" } else { "\n" };

    for texte in [premier, second] {
        for ligne in texte.lines() {
            let Some((cle, _, commentaire)) = analyser(ligne) else {
                lignes.push(ligne.to_owned());
                continue;
            };
            if let Some(&position) = positions.get(&cle) {
                let (_, entree, ancien) = analyser(&lignes[position]).unwrap();
                let mut commentaires = Vec::new();
                commentaires_uniques(ancien, &mut commentaires);
                commentaires_uniques(commentaire, &mut commentaires);
                lignes[position] = if commentaires.is_empty() {
                    entree.to_owned()
                } else {
                    format!("{entree} # {}", commentaires.join(" | "))
                };
                compteur += 1;
            } else {
                positions.insert(cle, lignes.len());
                lignes.push(ligne.to_owned());
            }
        }
    }
    let mut resultat = lignes.join(separateur);
    if !lignes.is_empty() {
        resultat.push_str(separateur);
    }
    (resultat, compteur)
}

fn lire(chemin: &Path) -> io::Result<String> {
    fs::read_to_string(chemin).map_err(|e| {
        io::Error::new(e.kind(), format!("lecture de '{}': {e}", chemin.display()))
    })
}

fn executer() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!("Usage : merge_hosts <fichier1> <fichier2> <sortie>");
        return Ok(());
    }
    if args.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Usage : merge_hosts <fichier1> <fichier2> <sortie>",
        )
        .into());
    }
    let premier = lire(Path::new(&args[0]))?;
    let second = lire(Path::new(&args[1]))?;
    let sortie = Path::new(&args[2]);
    let (resultat, compteur) = fusionner(&premier, &second);
    let mut fichier = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(sortie)
        .map_err(|e| {
            io::Error::new(e.kind(), format!("création de '{}': {e}", sortie.display()))
        })?;
    fichier.write_all(resultat.as_bytes())?;
    fichier.sync_all()?;
    println!("Fichier créé : {}", sortie.display());
    println!("Lignes d'entrée fusionnées : {compteur}");
    Ok(())
}

fn main() -> ExitCode {
    match executer() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Erreur : {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fusion_commentaires_sans_doublons() {
        let a = "192.168.1.10 nas.local # NAS | Stockage\n";
        let b = "192.168.1.10 nas.local # Stockage | Sauvegarde\n";
        assert_eq!(
            fusionner(a, b),
            ("192.168.1.10 nas.local # NAS | Stockage | Sauvegarde\n".into(), 1)
        );
    }

    #[test]
    fn prochains_merges_ne_repetent_pas_les_commentaires() {
        let a = "127.0.0.1 localhost # Local\n";
        let b = "127.0.0.1 localhost # Local | Test\n";
        let (une_fois, _) = fusionner(a, b);
        assert_eq!(fusionner(&une_fois, b).0, une_fois);
    }

    #[test]
    fn conserve_sections_et_lignes_vides() {
        assert_eq!(
            fusionner("# Local\n\n127.0.0.1 localhost\n", "# Serveurs\n10.0.0.1 nas # NAS\n").0,
            "# Local\n\n127.0.0.1 localhost\n# Serveurs\n10.0.0.1 nas # NAS\n"
        );
    }

    #[test]
    fn ignore_espacement_pour_identifier_entree() {
        assert_eq!(
            fusionner("127.0.0.1\tlocalhost # A\n", "127.0.0.1 localhost # B\n").0,
            "127.0.0.1\tlocalhost # A | B\n"
        );
    }

    #[test]
    fn conserve_ips_differentes() {
        assert_eq!(fusionner("127.0.0.1 local\n", "::1 local\n").1, 0);
    }

    #[test]
    fn fusionne_aussi_dans_le_premier_fichier() {
        assert_eq!(
            fusionner("127.0.0.1 local # A\n127.0.0.1 local # B\n", "").0,
            "127.0.0.1 local # A | B\n"
        );
    }

    #[test]
    fn gere_absence_de_commentaire_et_de_saut_final() {
        assert_eq!(
            fusionner("127.0.0.1 local", "127.0.0.1 local # A\n").0,
            "127.0.0.1 local # A\n"
        );
    }

    #[test]
    fn gere_fichiers_vides_et_crlf() {
        assert_eq!(fusionner("", ""), (String::new(), 0));
        assert_eq!(
            fusionner("# Local\r\n127.0.0.1 local # A\r\n", "127.0.0.1 local # B\n").0,
            "# Local\r\n127.0.0.1 local # A | B\r\n"
        );
    }
}
