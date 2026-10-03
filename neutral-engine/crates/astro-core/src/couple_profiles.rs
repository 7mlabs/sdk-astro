//! Declared symmetric selection policies for two independent natal charts.
use super::{profiles, HashSet};

pub(super) const NAMES: [&str; 6] = [
    "attraction",
    "communication",
    "emotionalConnection",
    "longTerm",
    "sharedResources",
    "homeFamily",
];

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn section(
    id: &str,
    houses: &[usize],
    bodies: &[&str],
    angles: &[&str],
) -> profiles::ProfileSection {
    profiles::ProfileSection {
        id: id.into(),
        houses: houses.into(),
        bodies: strings(bodies),
        angles: strings(angles),
    }
}

pub(super) fn builtin(id: &str) -> profiles::Profile {
    let (houses, bodies, angles, sections): (&[usize], &[&str], &[&str], _) = match id {
        "attraction" => (
            &[1, 5, 7, 8],
            &["sun", "moon", "venus", "mars", "pluto"],
            &["ascendant", "descendant"],
            vec![
                section("chemistry", &[1, 5], &["venus", "mars"], &["ascendant"]),
                section("romance", &[5, 7], &["moon", "venus"], &["descendant"]),
                section("intimacy", &[8], &["venus", "mars", "pluto"], &[]),
            ],
        ),
        "communication" => (
            &[3, 7, 9],
            &["moon", "mercury", "jupiter", "saturn"],
            &["descendant"],
            vec![
                section("dialogue", &[3], &["moon", "mercury"], &[]),
                section("understanding", &[3, 9], &["mercury", "jupiter"], &[]),
                section(
                    "conflict",
                    &[3, 7],
                    &["mercury", "mars", "saturn"],
                    &["descendant"],
                ),
            ],
        ),
        "emotionalConnection" => (
            &[4, 7, 8, 12],
            &["moon", "venus", "neptune", "pluto"],
            &["imumCoeli", "descendant"],
            vec![
                section("emotionalNeeds", &[4], &["moon", "venus"], &["imumCoeli"]),
                section(
                    "emotionalSafety",
                    &[4, 8],
                    &["moon", "saturn"],
                    &["imumCoeli"],
                ),
                section("empathy", &[12], &["moon", "neptune"], &[]),
            ],
        ),
        "longTerm" => (
            &[4, 7, 10, 11],
            &["sun", "venus", "jupiter", "saturn"],
            &["descendant", "midheaven"],
            vec![
                section("commitment", &[7], &["venus", "saturn"], &["descendant"]),
                section(
                    "sharedDirection",
                    &[9, 10],
                    &["sun", "jupiter", "saturn"],
                    &["midheaven"],
                ),
                section(
                    "resilience",
                    &[7, 8],
                    &["mars", "saturn", "pluto"],
                    &["descendant"],
                ),
            ],
        ),
        "sharedResources" => (
            &[2, 6, 8, 10],
            &["venus", "jupiter", "saturn", "pluto"],
            &["midheaven"],
            vec![
                section("values", &[2], &["venus", "jupiter"], &[]),
                section("jointResources", &[8], &["venus", "saturn", "pluto"], &[]),
                section(
                    "practicalCooperation",
                    &[6, 10],
                    &["mercury", "jupiter", "saturn"],
                    &["midheaven"],
                ),
            ],
        ),
        "homeFamily" => (
            &[4, 5, 7],
            &["sun", "moon", "venus", "saturn"],
            &["imumCoeli", "descendant"],
            vec![
                section("domesticLife", &[4], &["moon", "venus"], &["imumCoeli"]),
                section("familyBonds", &[4], &["moon", "saturn"], &["imumCoeli"]),
                section("parenting", &[5], &["sun", "moon", "saturn"], &[]),
            ],
        ),
        _ => unreachable!("validated couple profile ID"),
    };
    profiles::Profile {
        id: id.into(),
        version: "1.0".into(),
        houses: houses.into(),
        bodies: strings(bodies),
        angles: strings(angles),
        sections,
    }
}

pub(super) fn validate_selection(
    names: &[String],
    custom: &[profiles::Profile],
    rulership: &str,
) -> Result<(), String> {
    if names.len() > NAMES.len()
        || names.iter().any(|name| !NAMES.contains(&name.as_str()))
        || names.iter().collect::<HashSet<_>>().len() != names.len()
        || (names.is_empty() && custom.is_empty())
    {
        return Err("domains must contain up to 6 unique couple IDs: attraction, communication, emotionalConnection, longTerm, sharedResources, homeFamily; select at least one built-in or custom profile".into());
    }
    if custom.len() > 8 {
        return Err("customProfiles may contain up to 8 profiles".into());
    }
    let mut ids = HashSet::new();
    for profile in custom {
        profiles::validate_profile(profile)?;
        if NAMES.contains(&profile.id.as_str()) || !ids.insert(&profile.id) {
            return Err(
                "Custom profile IDs must be unique and may not use built-in couple IDs".into(),
            );
        }
    }
    if !["traditional", "modern"].contains(&rulership) {
        return Err("rulership must be traditional or modern".into());
    }
    Ok(())
}
