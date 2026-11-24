//! Full journal names for the abbreviations that bibliography providers write.
//!
//! JOSS, among others, asks for venue names in full. `InspireHEP` writes its own
//! short titles (`Phys. Rev. D`) and NASA ADS writes AAS macros (`\prd`), so
//! both are mapped here. The table is maintained by hand on purpose: INSPIRE's
//! journal database spells titles inconsistently (`Astrophysical Journal,
//! Supplement Series`, `The Journal of Cosmology and Astroparticle Physics
//! (JCAP)`), and an abbreviation with no entry is reported rather than guessed.

/// Abbreviations as `InspireHEP` or NASA ADS write them, and the full name.
///
/// Lookup ignores case, whitespace, periods and braces, so `Phys. Rev. D`,
/// `Phys.Rev.D` and `{Phys. Rev. D}` all match the same row.
const JOURNALS: &[(&str, &str)] = &[
    // InspireHEP short titles.
    ("Annalen Phys.", "Annalen der Physik"),
    (
        "Ann. Rev. Astron. Astrophys.",
        "Annual Review of Astronomy and Astrophysics",
    ),
    (
        "Ann. Rev. Nucl. Part. Sci.",
        "Annual Review of Nuclear and Particle Science",
    ),
    ("Appl. Opt.", "Applied Optics"),
    ("Astron. Astrophys.", "Astronomy \\& Astrophysics"),
    ("Astron. Comput.", "Astronomy and Computing"),
    ("Astron. J.", "The Astronomical Journal"),
    ("Astron. Nachr.", "Astronomische Nachrichten"),
    ("Astropart. Phys.", "Astroparticle Physics"),
    ("Astrophys. J.", "The Astrophysical Journal"),
    ("Astrophys. J. Lett.", "The Astrophysical Journal Letters"),
    (
        "Astrophys. J. Suppl.",
        "The Astrophysical Journal Supplement Series",
    ),
    ("Astrophys. Space Sci.", "Astrophysics and Space Science"),
    ("Class. Quant. Grav.", "Classical and Quantum Gravity"),
    ("Comput. Phys. Commun.", "Computer Physics Communications"),
    ("Eur. Phys. J. C", "The European Physical Journal C"),
    (
        "Front. Astron. Space Sci.",
        "Frontiers in Astronomy and Space Sciences",
    ),
    ("Gen. Rel. Grav.", "General Relativity and Gravitation"),
    (
        "Int. J. Mod. Phys. D",
        "International Journal of Modern Physics D",
    ),
    ("J. Open Source Softw.", "Journal of Open Source Software"),
    (
        "J. Phys. Conf. Ser.",
        "Journal of Physics: Conference Series",
    ),
    ("JCAP", "Journal of Cosmology and Astroparticle Physics"),
    ("JHEP", "Journal of High Energy Physics"),
    ("Living Rev. Rel.", "Living Reviews in Relativity"),
    (
        "Mach. Learn. Sci. Tech.",
        "Machine Learning: Science and Technology",
    ),
    (
        "Mon. Not. Roy. Astron. Soc.",
        "Monthly Notices of the Royal Astronomical Society",
    ),
    ("Nature Astron.", "Nature Astronomy"),
    ("Nature Commun.", "Nature Communications"),
    ("Nature Phys.", "Nature Physics"),
    ("New Astron. Rev.", "New Astronomy Reviews"),
    (
        "Nucl. Instrum. Meth. A",
        "Nuclear Instruments and Methods in Physics Research Section A: Accelerators, Spectrometers, Detectors and Associated Equipment",
    ),
    ("Nucl. Phys. B", "Nuclear Physics B"),
    ("Open J. Astrophys.", "The Open Journal of Astrophysics"),
    ("Opt. Express", "Optics Express"),
    (
        "Phil. Trans. Roy. Soc. Lond. A",
        "Philosophical Transactions of the Royal Society A: Mathematical, Physical and Engineering Sciences",
    ),
    ("Phys. Dark Univ.", "Physics of the Dark Universe"),
    ("Phys. Lett. B", "Physics Letters B"),
    ("Phys. Rept.", "Physics Reports"),
    ("Phys. Rev.", "Physical Review"),
    ("Phys. Rev. A", "Physical Review A"),
    ("Phys. Rev. Applied", "Physical Review Applied"),
    ("Phys. Rev. B", "Physical Review B"),
    ("Phys. Rev. C", "Physical Review C"),
    ("Phys. Rev. D", "Physical Review D"),
    ("Phys. Rev. E", "Physical Review E"),
    ("Phys. Rev. Lett.", "Physical Review Letters"),
    ("Phys. Rev. Res.", "Physical Review Research"),
    ("Phys. Rev. X", "Physical Review X"),
    ("Proc. SPIE Int. Soc. Opt. Eng.", "Proceedings of SPIE"),
    ("PTEP", "Progress of Theoretical and Experimental Physics"),
    (
        "Publ. Astron. Soc. Austral.",
        "Publications of the Astronomical Society of Australia",
    ),
    (
        "Publ. Astron. Soc. Jap.",
        "Publications of the Astronomical Society of Japan",
    ),
    (
        "Publ. Astron. Soc. Pac.",
        "Publications of the Astronomical Society of the Pacific",
    ),
    ("Rept. Prog. Phys.", "Reports on Progress in Physics"),
    ("Res. Notes AAS", "Research Notes of the AAS"),
    ("Rev. Mod. Phys.", "Reviews of Modern Physics"),
    ("Rev. Sci. Instrum.", "Review of Scientific Instruments"),
    ("SciPost Phys.", "SciPost Physics"),
    ("Space Sci. Rev.", "Space Science Reviews"),
    // Journals whose InspireHEP short title is already the full name.
    ("Galaxies", "Galaxies"),
    ("Nature", "Nature"),
    ("Science", "Science"),
    ("SoftwareX", "SoftwareX"),
    ("Universe", "Universe"),
    // NASA ADS (AAS journal macros).
    ("\\aap", "Astronomy \\& Astrophysics"),
    ("\\aj", "The Astronomical Journal"),
    ("\\apj", "The Astrophysical Journal"),
    ("\\apjl", "The Astrophysical Journal Letters"),
    ("\\apjs", "The Astrophysical Journal Supplement Series"),
    ("\\apss", "Astrophysics and Space Science"),
    ("\\araa", "Annual Review of Astronomy and Astrophysics"),
    ("\\grl", "Geophysical Research Letters"),
    ("\\icarus", "Icarus"),
    ("\\jcap", "Journal of Cosmology and Astroparticle Physics"),
    (
        "\\mnras",
        "Monthly Notices of the Royal Astronomical Society",
    ),
    ("\\na", "New Astronomy"),
    ("\\nar", "New Astronomy Reviews"),
    ("\\nat", "Nature"),
    (
        "\\pasa",
        "Publications of the Astronomical Society of Australia",
    ),
    (
        "\\pasj",
        "Publications of the Astronomical Society of Japan",
    ),
    (
        "\\pasp",
        "Publications of the Astronomical Society of the Pacific",
    ),
    ("\\physrep", "Physics Reports"),
    ("\\pra", "Physical Review A"),
    ("\\prb", "Physical Review B"),
    ("\\prc", "Physical Review C"),
    ("\\prd", "Physical Review D"),
    ("\\pre", "Physical Review E"),
    ("\\prl", "Physical Review Letters"),
    ("\\procspie", "Proceedings of SPIE"),
    ("\\ssr", "Space Science Reviews"),
];

/// The full name for a journal abbreviation, if the table knows it.
pub(crate) fn full_name(journal: &str) -> Option<&'static str> {
    let key = lookup_key(journal);
    JOURNALS
        .iter()
        .find(|(abbreviation, _)| lookup_key(abbreviation) == key)
        .map(|(_, full)| *full)
}

/// Words that end in a period inside full journal names.
const FULL_NAME_ABBREVIATIONS: &[&str] = &["St.", "Ste.", "Mt.", "Ft."];

/// How the venue check judges a venue name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VenueVerdict {
    /// One of the table's full names, or a name that does not look abbreviated.
    Full,
    /// An abbreviation in the table, with its full name.
    KnownAbbreviation(&'static str),
    /// Not in the table, but looks abbreviated by [`looks_abbreviated`].
    LooksAbbreviated,
}

/// Judge a venue name for the venue check.
///
/// The table decides when it can: one of its abbreviations is abbreviated,
/// and one of its full names is not. Any other name is judged by
/// [`looks_abbreviated`].
pub(crate) fn judge_venue(venue: &str) -> VenueVerdict {
    if let Some(full) = full_name(venue).filter(|full| lookup_key(full) != lookup_key(venue)) {
        return VenueVerdict::KnownAbbreviation(full);
    }
    if is_known_full_name(venue) || !looks_abbreviated(venue) {
        return VenueVerdict::Full;
    }
    VenueVerdict::LooksAbbreviated
}

/// Whether a name looks abbreviated: an unexpanded macro (`\\prd`), a word cut
/// short with a period (`Phys.`, or `J.Phys.` written without spaces), or a
/// short all-capitals acronym standing alone (`JCAP`).
fn looks_abbreviated(venue: &str) -> bool {
    let name = venue.trim().trim_matches(['{', '}']).trim();
    if name.starts_with('\\') {
        return true;
    }
    let words = name.split_whitespace().collect::<Vec<_>>();
    let cut_word = words.iter().any(|word| is_cut_word(word));
    let acronym = words.len() == 1
        && (2..=6).contains(&name.chars().count())
        && name.chars().filter(char::is_ascii_uppercase).count() >= 2;
    cut_word || acronym
}

/// A word containing a period whose period-separated pieces are short runs of
/// letters (`Phys.`, `J.Phys.`, `Rev.Mod.Phys.`). Longer pieces such as
/// `Astrophysics.` end a phrase rather than cut a word short.
fn is_cut_word(word: &str) -> bool {
    const MAX_CUT_LETTERS: usize = 9;
    word.contains('.')
        && !FULL_NAME_ABBREVIATIONS.contains(&word)
        && word.split('.').all(|piece| {
            piece.chars().all(char::is_alphabetic) && piece.chars().count() <= MAX_CUT_LETTERS
        })
}

/// Whether `journal` is written as one of the table's full names.
pub(crate) fn is_known_full_name(journal: &str) -> bool {
    let key = venue_key(journal);
    JOURNALS.iter().any(|(_, full)| venue_key(full) == key)
}

/// A comparison key for a venue name that ignores case, whitespace, braces,
/// ties and the punctuation between a title and its section, so
/// `Journal of Physics. Conference Series` matches `Journal of Physics:
/// Conference Series`.
pub(crate) fn venue_key(venue: &str) -> String {
    lookup_key(venue).replace(':', "")
}

fn lookup_key(journal: &str) -> String {
    journal
        .chars()
        .filter(|ch| !ch.is_whitespace() && !matches!(ch, '.' | '{' | '}' | '~'))
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{JOURNALS, full_name, lookup_key};
    use std::collections::BTreeMap;

    #[test]
    fn knows_full_names() {
        use super::is_known_full_name;
        assert!(is_known_full_name("Physical Review D"));
        assert!(is_known_full_name("{Nature}"));
        assert!(!is_known_full_name("eConf"));
        assert!(!is_known_full_name("Phys. Rev. D"));
        assert!(!is_known_full_name(
            "Bulletin of the Russian Academy of Sciences. Physics"
        ));
    }

    #[test]
    fn venue_check_uses_the_table_before_the_heuristic() {
        use super::{VenueVerdict, judge_venue};
        assert_eq!(
            judge_venue("Phys. Rev. D"),
            VenueVerdict::KnownAbbreviation("Physical Review D")
        );
        assert_eq!(judge_venue("RNA"), VenueVerdict::LooksAbbreviated);
        for name in [
            "Phys. Rev. D",
            "Phys.Rev.D",
            "{\\mnras}",
            "JCAP",
            "Nature Astron.",
            "New J.Phys.",
            "AIP Conf.Proc.",
            "Proc. SPIE",
            "ApJL",
        ] {
            assert_ne!(judge_venue(name), VenueVerdict::Full, "{name}");
        }
        for name in [
            "Physical Review D",
            "Nature",
            "SoftwareX",
            "Universe",
            "Astronomy \\& Astrophysics",
            "St. Petersburg Mathematical Journal",
            "Astronomy and Astrophysics. Supplement Series",
        ] {
            assert_eq!(judge_venue(name), VenueVerdict::Full, "{name}");
        }
    }

    #[test]
    fn expands_inspire_and_ads_spellings() {
        assert_eq!(full_name("Phys. Rev. D"), Some("Physical Review D"));
        assert_eq!(full_name("Phys.Rev.D"), Some("Physical Review D"));
        assert_eq!(full_name("{\\prd}"), Some("Physical Review D"));
        assert_eq!(
            full_name("\\apjl"),
            Some("The Astrophysical Journal Letters")
        );
        assert_eq!(full_name("Physical Review D"), None);
        assert_eq!(full_name("Phys. Rev. Z"), None);
    }

    #[test]
    fn table_has_no_ambiguous_rows() {
        let mut seen = BTreeMap::new();
        for (abbreviation, full) in JOURNALS {
            if let Some(previous) = seen.insert(lookup_key(abbreviation), *full) {
                panic!("{abbreviation} maps to both {previous} and {full}");
            }
        }
    }
}
