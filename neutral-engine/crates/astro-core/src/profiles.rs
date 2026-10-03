//! Versioned software selection profiles for one person's natal report data.
//! These mappings declare project policy rather than universal astrology rules.
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

pub(super) const NAMES: [&str; 10] = [
    "career",
    "love",
    "relationships",
    "family",
    "finance",
    "identity",
    "learning",
    "creativity",
    "innerLife",
    "dailyLife",
];
const BODIES: [&str; 10] = [
    "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune", "pluto",
];
const ANGLES: [&str; 4] = ["ascendant", "midheaven", "descendant", "imumCoeli"];
const RESERVED: [&str; 3] = ["constructor", "prototype", "__proto__"];

fn default_version() -> String {
    "1.0".into()
}

fn houses<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<usize>, D::Error> {
    #[derive(Deserialize)]
    struct House(#[serde(deserialize_with = "super::numbers::i32")] i32);
    Vec::<House>::deserialize(deserializer)?
        .into_iter()
        .map(|house| {
            usize::try_from(house.0)
                .map_err(|_| D::Error::custom("House numbers must be nonnegative integers"))
        })
        .collect()
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Profile {
    pub id: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default, deserialize_with = "houses")]
    pub houses: Vec<usize>,
    #[serde(default)]
    pub bodies: Vec<String>,
    #[serde(default)]
    pub angles: Vec<String>,
    #[serde(default)]
    pub sections: Vec<ProfileSection>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ProfileSection {
    pub id: String,
    #[serde(default, deserialize_with = "houses")]
    pub houses: Vec<usize>,
    #[serde(default)]
    pub bodies: Vec<String>,
    #[serde(default)]
    pub angles: Vec<String>,
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn section(id: &str, houses: &[usize], bodies: &[&str], angles: &[&str]) -> ProfileSection {
    ProfileSection {
        id: id.into(),
        houses: houses.into(),
        bodies: strings(bodies),
        angles: strings(angles),
    }
}

pub(super) fn builtin(id: &str) -> Profile {
    let (houses, bodies, angles, sections): (&[usize], &[&str], &[&str], _) = match id {
        "career" => (
            &[2, 6, 10],
            &["sun", "mercury", "jupiter", "saturn"],
            &["midheaven"],
            vec![
                section(
                    "profession",
                    &[10],
                    &["sun", "jupiter", "saturn"],
                    &["midheaven"],
                ),
                section("workHabits", &[6], &["mercury", "saturn"], &[]),
                section("resources", &[2], &["venus", "jupiter"], &[]),
            ],
        ),
        "love" => (
            &[5, 7, 8],
            &["moon", "venus", "mars"],
            &["descendant"],
            vec![
                section("romance", &[5], &["moon", "venus", "mars"], &[]),
                section(
                    "marriage",
                    &[7],
                    &["moon", "venus", "saturn"],
                    &["descendant"],
                ),
                section("intimacy", &[8], &["venus", "mars", "pluto"], &[]),
            ],
        ),
        "relationships" => (
            &[3, 7, 11],
            &["moon", "mercury", "venus"],
            &["ascendant", "descendant"],
            vec![
                section("communication", &[3], &["moon", "mercury"], &[]),
                section("partnerships", &[7], &["mercury", "venus"], &["descendant"]),
                section("friendships", &[11], &["mercury", "venus", "jupiter"], &[]),
            ],
        ),
        "family" => (
            &[3, 4, 5],
            &["sun", "moon", "saturn"],
            &["imumCoeli"],
            vec![
                section("roots", &[4], &["moon", "saturn"], &["imumCoeli"]),
                section("home", &[4], &["moon", "venus"], &["imumCoeli"]),
                section("parenting", &[5], &["sun", "moon", "saturn"], &[]),
            ],
        ),
        "finance" => (
            &[2, 8, 10, 11],
            &["venus", "jupiter", "saturn"],
            &["midheaven"],
            vec![
                section("personalResources", &[2], &["venus", "jupiter"], &[]),
                section("sharedResources", &[8], &["venus", "saturn", "pluto"], &[]),
                section(
                    "income",
                    &[10, 11],
                    &["venus", "jupiter", "saturn"],
                    &["midheaven"],
                ),
            ],
        ),
        "identity" => (
            &[1],
            &["sun", "moon"],
            &["ascendant"],
            vec![
                section("coreIdentity", &[], &["sun"], &["ascendant"]),
                section("emotionalStyle", &[], &["moon"], &[]),
                section(
                    "personalPresentation",
                    &[1],
                    &["sun", "moon"],
                    &["ascendant"],
                ),
            ],
        ),
        "learning" => (
            &[3, 9],
            &["mercury", "jupiter"],
            &[],
            vec![
                section("thinking", &[3], &["mercury"], &[]),
                section("education", &[9], &["mercury", "jupiter"], &[]),
                section("exploration", &[9], &["jupiter"], &[]),
            ],
        ),
        "creativity" => (
            &[5],
            &["sun", "venus", "mercury"],
            &[],
            vec![
                section("selfExpression", &[5], &["sun", "venus"], &[]),
                section("creativeThinking", &[3, 5], &["mercury", "venus"], &[]),
                section("play", &[5], &["sun", "moon", "venus"], &[]),
            ],
        ),
        "innerLife" => (
            &[4, 8, 12],
            &["moon", "saturn", "neptune"],
            &["imumCoeli"],
            vec![
                section("emotionalRoots", &[4], &["moon", "saturn"], &["imumCoeli"]),
                section("transformation", &[8], &["moon", "saturn", "pluto"], &[]),
                section("reflection", &[12], &["moon", "saturn", "neptune"], &[]),
            ],
        ),
        "dailyLife" => (
            &[1, 6, 12],
            &["sun", "moon", "mars", "saturn"],
            &["ascendant"],
            vec![
                section("routines", &[6], &["sun", "moon", "saturn"], &[]),
                section(
                    "selfCare",
                    &[1, 6],
                    &["sun", "moon", "mars"],
                    &["ascendant"],
                ),
                section("rest", &[12], &["moon", "saturn", "neptune"], &[]),
            ],
        ),
        _ => unreachable!("validated built-in profile ID"),
    };
    Profile {
        id: id.into(),
        version: "2.0".into(),
        houses: houses.into(),
        bodies: strings(bodies),
        angles: strings(angles),
        sections,
    }
}

fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-')
        && !RESERVED.contains(&id)
}

fn valid_version(version: &str) -> bool {
    let bytes = version.as_bytes();
    (1..=32).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes[1..].iter().all(|byte| {
            byte.is_ascii_alphanumeric() || *byte == b'.' || *byte == b'_' || *byte == b'-'
        })
}

fn validate_selectors(
    houses: &[usize],
    bodies: &[String],
    angles: &[String],
) -> Result<(), String> {
    if houses.is_empty() && bodies.is_empty() && angles.is_empty() {
        return Err(
            "A profile or report section requires at least one house, body or angle".into(),
        );
    }
    if houses.len() > 12
        || houses.iter().any(|house| !(1..=12).contains(house))
        || houses.iter().collect::<HashSet<_>>().len() != houses.len()
    {
        return Err("houses must contain up to 12 unique integer numbers from 1 to 12".into());
    }
    if bodies.len() > BODIES.len()
        || bodies.iter().any(|body| !BODIES.contains(&body.as_str()))
        || bodies.iter().collect::<HashSet<_>>().len() != bodies.len()
    {
        return Err("bodies must contain up to 10 unique IDs: sun, moon, mercury, venus, mars, jupiter, saturn, uranus, neptune, pluto".into());
    }
    if angles.len() > ANGLES.len()
        || angles.iter().any(|angle| !ANGLES.contains(&angle.as_str()))
        || angles.iter().collect::<HashSet<_>>().len() != angles.len()
    {
        return Err(
            "angles must contain up to 4 unique IDs: ascendant, midheaven, descendant, imumCoeli"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn validate_profile(profile: &Profile) -> Result<(), String> {
    if !valid_id(&profile.id) || NAMES.contains(&profile.id.as_str()) {
        return Err("Custom profile IDs must use 1 to 64 ASCII characters, start with a lowercase letter, and contain letters, numbers, '_' or '-'; built-in and reserved IDs are not allowed".into());
    }
    if !valid_version(&profile.version) {
        return Err("Profile version must use 1 to 32 ASCII characters, start with a letter or number, and contain letters, numbers, '.', '_' or '-'".into());
    }
    validate_selectors(&profile.houses, &profile.bodies, &profile.angles)?;
    if profile.sections.len() > 8 {
        return Err("A custom profile may contain up to 8 report sections".into());
    }
    let mut section_ids = HashSet::new();
    for section in &profile.sections {
        if !valid_id(&section.id) || !section_ids.insert(&section.id) {
            return Err("Report section IDs must be unique within a profile and use 1 to 64 ASCII characters, start with a lowercase letter, and contain letters, numbers, '_' or '-'; reserved IDs are not allowed".into());
        }
        // A section may introduce selectors outside its parent profile.
        validate_selectors(&section.houses, &section.bodies, &section.angles)?;
    }
    Ok(())
}

pub(super) fn validate_selection(
    names: &[String],
    custom: &[Profile],
    rulership: &str,
) -> Result<(), String> {
    if names.len() > NAMES.len()
        || names.iter().any(|name| !NAMES.contains(&name.as_str()))
        || names.iter().collect::<HashSet<_>>().len() != names.len()
        || (names.is_empty() && custom.is_empty())
    {
        return Err("domains must contain up to 10 unique built-in IDs: career, love, relationships, family, finance, identity, learning, creativity, innerLife, dailyLife; select at least one built-in or custom profile".into());
    }
    if custom.len() > 8 {
        return Err("customProfiles may contain up to 8 profiles".into());
    }
    let mut custom_ids = HashSet::new();
    for profile in custom {
        validate_profile(profile)?;
        if !custom_ids.insert(&profile.id) {
            return Err("Custom profile IDs must be unique".into());
        }
    }
    if !["traditional", "modern"].contains(&rulership) {
        return Err("rulership must be traditional or modern".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn custom(raw: &str) -> Profile {
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn catalog_has_ten_unique_individual_profiles_and_valid_sections() {
        assert_eq!(NAMES.iter().collect::<HashSet<_>>().len(), 10);
        for name in NAMES {
            let profile = builtin(name);
            assert_eq!(profile.id, name);
            assert_eq!(profile.version, "2.0");
            validate_selectors(&profile.houses, &profile.bodies, &profile.angles).unwrap();
            assert_eq!(profile.sections.len(), 3);
            let mut ids = HashSet::new();
            for section in profile.sections {
                assert!(valid_id(&section.id));
                assert!(ids.insert(section.id));
                validate_selectors(&section.houses, &section.bodies, &section.angles).unwrap();
            }
        }
        assert_eq!(builtin("career").houses, [2, 6, 10]);
        assert_eq!(builtin("love").houses, [5, 7, 8]);
        assert_eq!(builtin("relationships").houses, [3, 7, 11]);
        assert_eq!(builtin("family").houses, [3, 4, 5]);
        assert_eq!(builtin("finance").houses, [2, 8, 10, 11]);
        assert_eq!(builtin("identity").houses, [1]);
        assert_eq!(builtin("learning").houses, [3, 9]);
        assert_eq!(builtin("creativity").houses, [5]);
        assert_eq!(builtin("innerLife").houses, [4, 8, 12]);
        assert_eq!(builtin("dailyLife").houses, [1, 6, 12]);
    }

    #[test]
    fn defaults_are_normalized_and_sections_can_extend_parent_selectors() {
        let profile = custom(
            r#"{"id":"customA","bodies":["sun"],"sections":[{"id":"finance","houses":[8]}]}"#,
        );
        validate_profile(&profile).unwrap();
        assert_eq!(profile.version, "1.0");
        assert_eq!(
            serde_json::to_value(profile).unwrap(),
            json!({"id":"customA","version":"1.0","houses":[],"bodies":["sun"],"angles":[],
                "sections":[{"id":"finance","houses":[8],"bodies":[],"angles":[]}]})
        );
        let profile = custom(r#"{"id":"customA","angles":["ascendant"]}"#);
        assert_eq!(
            serde_json::to_value(profile).unwrap()["sections"],
            json!([])
        );
    }

    #[test]
    fn house_numbers_use_exact_decimal_normalization() {
        let profile =
            custom(r#"{"id":"exactHouses","houses":[1.0,2e0,30e-1,12.000000000000000000]}"#);
        validate_profile(&profile).unwrap();
        assert_eq!(profile.houses, [1, 2, 3, 12]);
        assert_eq!(
            serde_json::to_value(profile).unwrap()["houses"],
            json!([1, 2, 3, 12])
        );
        for raw in [
            "1.000000000000000001",
            "0.999999999999999999",
            "1e-100",
            "-1",
            "2147483648",
            "\"1\"",
            "true",
            "null",
            "{}",
            "[]",
        ] {
            let request = format!(r#"{{"id":"test","houses":[{raw}]}}"#);
            assert!(serde_json::from_str::<Profile>(&request).is_err(), "{raw}");
        }
    }

    #[test]
    fn profile_validation_rejects_invalid_ids_versions_and_selectors() {
        let base = json!({"id":"customA","houses":[1]});
        for id in [
            "",
            "Career",
            "1abc",
            "a.b",
            "a b",
            "café",
            "career",
            "love",
            "constructor",
            "prototype",
            "__proto__",
        ] {
            let mut input = base.clone();
            input["id"] = json!(id);
            assert!(
                validate_profile(&serde_json::from_value(input).unwrap()).is_err(),
                "{id}"
            );
        }
        for version in ["", ".1", "v 1", "v/1", "α1"] {
            let mut input = base.clone();
            input["version"] = json!(version);
            assert!(
                validate_profile(&serde_json::from_value(input).unwrap()).is_err(),
                "{version}"
            );
        }
        let mut input = base.clone();
        input["id"] = json!("a".repeat(65));
        assert!(validate_profile(&serde_json::from_value(input).unwrap()).is_err());
        let mut input = base.clone();
        input["version"] = json!("1".repeat(33));
        assert!(validate_profile(&serde_json::from_value(input).unwrap()).is_err());
        for input in [
            json!({"id":"x"}),
            json!({"id":"x","houses":[0]}),
            json!({"id":"x","houses":[13]}),
            json!({"id":"x","houses":[1,1]}),
            json!({"id":"x","bodies":["chiron"]}),
            json!({"id":"x","bodies":["sun","sun"]}),
            json!({"id":"x","angles":["ASC"]}),
            json!({"id":"x","angles":["ascendant","ascendant"]}),
            json!({"id":"x","houses":[1],"sections":[{"id":"empty"}]}),
            json!({"id":"x","houses":[1],"sections":[{"id":"constructor","houses":[1]}]}),
            json!({"id":"x","houses":[1],"sections":[{"id":"a","houses":[1]},{"id":"a","houses":[2]}]}),
        ] {
            assert!(
                validate_profile(&serde_json::from_value(input.clone()).unwrap()).is_err(),
                "{input}"
            );
        }
        let valid = custom(&format!(
            r#"{{"id":"{}","version":"{}","houses":[1]}}"#,
            "a".repeat(64),
            "1".repeat(32)
        ));
        validate_profile(&valid).unwrap();
    }

    #[test]
    fn null_unknown_fields_and_excessive_sections_are_rejected() {
        for key in ["version", "houses", "bodies", "angles", "sections"] {
            let mut input = json!({"id":"x","houses":[1]});
            input[key] = Value::Null;
            assert!(serde_json::from_value::<Profile>(input).is_err(), "{key}");
        }
        for key in ["houses", "bodies", "angles"] {
            let mut input = json!({"id":"x","bodies":["sun"]});
            input[key] = Value::Null;
            assert!(
                serde_json::from_value::<ProfileSection>(input).is_err(),
                "{key}"
            );
        }
        assert!(
            serde_json::from_str::<Profile>(r#"{"id":"x","houses":[1],"unknown":true}"#).is_err()
        );
        assert!(serde_json::from_str::<ProfileSection>(
            r#"{"id":"x","houses":[1],"version":"1.0"}"#
        )
        .is_err());
        let mut profile = custom(r#"{"id":"x","houses":[1]}"#);
        profile.sections = (0..8)
            .map(|i| section(&format!("s{i}"), &[1], &[], &[]))
            .collect();
        validate_profile(&profile).unwrap();
        profile.sections.push(section("tooMany", &[1], &[], &[]));
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn selection_checks_unique_known_profiles_and_limits() {
        let names = strings(&NAMES);
        validate_selection(&names, &[], "traditional").unwrap();
        let custom: Vec<Profile> = (0..8)
            .map(|i| custom(&format!(r#"{{"id":"custom{i}","houses":[1]}}"#)))
            .collect();
        validate_selection(&names, &custom, "modern").unwrap();
        validate_selection(&[], &custom, "traditional").unwrap();
        assert!(validate_selection(&[], &[], "traditional").is_err());
        assert!(validate_selection(&strings(&["career", "career"]), &[], "traditional").is_err());
        assert!(validate_selection(&strings(&["unknown"]), &[], "traditional").is_err());
        assert!(validate_selection(&names, &[], "unknown").is_err());
        let mut many = custom.clone();
        many.push(custom[0].clone());
        assert!(validate_selection(&names, &many, "traditional").is_err());
        assert!(validate_selection(
            &names,
            &[custom[0].clone(), custom[0].clone()],
            "traditional"
        )
        .is_err());
    }
}
