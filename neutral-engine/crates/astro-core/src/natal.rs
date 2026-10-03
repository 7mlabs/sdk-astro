use super::*;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Utc {
    #[serde(deserialize_with = "numbers::i32")]
    year: i32,
    #[serde(deserialize_with = "numbers::i32")]
    month: i32,
    #[serde(deserialize_with = "numbers::i32")]
    day: i32,
    #[serde(deserialize_with = "numbers::i32")]
    hour: i32,
    #[serde(deserialize_with = "numbers::i32")]
    minute: i32,
    #[serde(default)]
    second: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Location {
    latitude: f64,
    longitude: f64,
}

pub(super) fn default_house_system() -> String {
    "placidus".into()
}

#[derive(Default)]
pub(super) struct AspectSettings {
    pub provided: bool,
    pub rules: Vec<AspectRule>,
}
impl<'de> Deserialize<'de> for AspectSettings {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self {
            provided: true,
            rules: Vec::<AspectRule>::deserialize(d)?,
        })
    }
}
pub(super) fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BirthInput {
    utc: Utc,
    location: Location,
    #[serde(default = "default_house_system")]
    house_system: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NatalRequest {
    operation: String,
    utc: Utc,
    location: Location,
    #[serde(default = "default_house_system")]
    house_system: String,
    #[serde(default)]
    aspect_rules: AspectSettings,
    #[serde(default, deserialize_with = "present")]
    domains: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present")]
    rulership: Option<String>,
    #[serde(default, deserialize_with = "present")]
    aspect_preset: Option<String>,
    #[serde(default, deserialize_with = "present")]
    custom_profiles: Option<Vec<profiles::Profile>>,
}

fn validate_natal(r: &NatalRequest, rules: &[AspectRule]) -> Result<u8, String> {
    if r.operation != "natal" && r.operation != "natalDomains" {
        return Err("Expected natal or natalDomains operation".into());
    }
    if r.operation == "natal"
        && (r.domains.is_some()
            || r.rulership.is_some()
            || r.aspect_preset.is_some()
            || r.custom_profiles.is_some())
    {
        return Err(
            "domains, rulership, aspectPreset and customProfiles require operation natalDomains"
                .into(),
        );
    }
    let system = validate_birth_parts(&r.utc, &r.location, &r.house_system)?;
    validate_aspect_rules(rules)?;
    Ok(system)
}

pub(super) fn validate_birth(birth: &BirthInput) -> Result<u8, String> {
    validate_birth_parts(&birth.utc, &birth.location, &birth.house_system)
}

fn validate_birth_parts(utc: &Utc, location: &Location, house_system: &str) -> Result<u8, String> {
    let u = utc;
    if !(1800..=2399).contains(&u.year) || !(1..=12).contains(&u.month) {
        return Err("UTC requires a Gregorian year 1800..2399 and month 1..12".into());
    }
    let leap = u.year % 4 == 0 && (u.year % 100 != 0 || u.year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=days[(u.month - 1) as usize]).contains(&u.day)
        || !(0..=23).contains(&u.hour)
        || !(0..=59).contains(&u.minute)
        || !u.second.is_finite()
        || !(0.0..60.0).contains(&u.second)
    {
        return Err("UTC date/time is invalid; leap-second input is not supported".into());
    }
    if !location.latitude.is_finite()
        || !location.longitude.is_finite()
        || !(-90.0..=90.0).contains(&location.latitude)
        || !(-180.0..=180.0).contains(&location.longitude)
    {
        return Err("Latitude must be -90..90 and longitude -180..180, in degrees".into());
    }
    if location.latitude.abs() == 90.0 {
        return Err("House angles are undefined at the geographic poles".into());
    }
    let system = match house_system {
        "placidus" => b'P',
        "wholeSign" => b'W',
        _ => return Err("houseSystem must be placidus or wholeSign".into()),
    };
    Ok(system)
}

pub(super) fn compute_birth(
    birth: &BirthInput,
    system: u8,
    rules: &[AspectRule],
) -> Result<(Value, Value), String> {
    let u = &birth.utc;
    let sky = match astro_provider_swiss::natal(
        [u.year, u.month, u.day, u.hour, u.minute],
        u.second,
        birth.location.latitude,
        birth.location.longitude,
        system,
    ) {
        Ok(sky) => sky,
        Err(e) => return Err(e),
    };
    validate_houses(&Some(sky.houses.clone()))?;
    let ids = [
        "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
        "pluto",
    ];
    let positions: Vec<Position> = sky
        .bodies
        .iter()
        .zip(ids)
        .map(|(p, id)| Position {
            id: id.into(),
            longitude: p[0],
            speed: Some(p[3]),
        })
        .collect();
    let mut bodies = serde_json::to_value(placements(&positions, Some(&sky.houses))).unwrap();
    for (body, coords) in bodies.as_array_mut().unwrap().iter_mut().zip(&sky.bodies) {
        body["latitude"] = json!(coords[1]);
        body["distanceAu"] = json!(coords[2]);
    }
    let angles: Vec<Position> = [
        ("ascendant", sky.ascendant),
        ("midheaven", sky.midheaven),
        ("descendant", normalize(sky.ascendant + 180.)),
        ("imumCoeli", normalize(sky.midheaven + 180.)),
    ]
    .into_iter()
    .map(|(id, longitude)| Position {
        id: id.into(),
        longitude,
        speed: None,
    })
    .collect();
    let houses: Vec<Value> = sky.houses.iter().enumerate().map(|(i, cusp)| {
        let longitude = normalize(*cusp);
        json!({"number": i+1, "longitude": longitude, "sign": SIGNS[(longitude / 30.).floor() as usize], "degreeInSign": longitude % 30.})
    }).collect();
    let natal = json!({"utc":birth.utc, "location":birth.location, "placements":bodies,
        "angles":placements(&angles, None), "houses":houses, "houseCusps":sky.houses,
        "aspects":aspects(&positions, None, rules)});
    let calculation = json!({"scope":"basic-natal", "provider":"swiss-ephemeris", "providerVersion":"2.10.03",
            "ephemeris":"moshier", "zodiac":"tropical", "coordinates":"geocentric",
            "positionType":"apparent", "referenceFrame":"ecliptic-of-date", "calendar":"gregorian",
            "inputTimeScale":"UTC", "houseTimeScale":"UT1", "planetTimeScale":"TT",
            "timeModel":"Swiss built-in leap seconds and Delta T; pre-1972 civil input treated as UT1",
            "julianDayTt":sky.jd_tt, "julianDayUt1":sky.jd_ut1, "houseSystem":birth.house_system,
            "angleUnit":"degrees", "speedUnit":"degrees/day", "distanceUnit":"AU",
            "aspectRules":rules.iter().map(|v| json!({"angle":v.angle,"maxOrb":v.max_orb})).collect::<Vec<_>>()});
    Ok((natal, calculation))
}

pub(super) fn calculate_json(raw: &str) -> String {
    let r: NatalRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &e.to_string()),
    };
    let is_domains = r.operation == "natalDomains";
    let preset =
        r.aspect_preset
            .as_deref()
            .unwrap_or(if is_domains { "extended" } else { "major" });
    if !["major", "extended"].contains(&preset)
        || (r.aspect_preset.is_some() && r.aspect_rules.provided)
    {
        return error_json(
            "INVALID_INPUT",
            "aspectPreset must be major or extended and cannot be combined with aspectRules",
        );
    }
    let rules = if r.aspect_rules.provided {
        r.aspect_rules.rules.clone()
    } else if preset == "extended" {
        domains::extended_rules()
    } else {
        default_rules()
    };
    let names = r
        .domains
        .clone()
        .unwrap_or_else(|| profiles::NAMES.into_iter().map(String::from).collect());
    let rulership = r.rulership.as_deref().unwrap_or("traditional");
    let custom = r.custom_profiles.as_deref().unwrap_or(&[]);
    if is_domains {
        if r.custom_profiles.is_some() && custom.is_empty() {
            return error_json(
                "INVALID_INPUT",
                "customProfiles must contain 1 to 8 profiles when provided",
            );
        }
        if let Err(e) = profiles::validate_selection(&names, custom, rulership) {
            return error_json("INVALID_INPUT", &e);
        }
    }
    let system = match validate_natal(&r, &rules) {
        Ok(s) => s,
        Err(e) => return error_json("INVALID_INPUT", &e),
    };
    let birth = BirthInput {
        utc: r.utc,
        location: r.location,
        house_system: r.house_system,
    };
    let (natal, mut calculation) = match compute_birth(&birth, system, &rules) {
        Ok(values) => values,
        Err(e) => return error_json("CALCULATION_FAILED", &e),
    };
    calculation["scope"] = json!(if is_domains {
        "natal-domain-data"
    } else {
        "basic-natal"
    });
    let data = if is_domains {
        domains::build(natal, &rules, &names, rulership, custom)
    } else {
        natal
    };
    let mut result = json!({"schemaVersion":"1.0", "engineVersion":VERSION,
        "calculation":calculation,
        "data":data,
        "warnings":["Moshier analytical ephemeris; no JPL/Swiss data files. Future UTC uses the provider's built-in time model, not live Earth-orientation data."],
        "errors":[]});
    if is_domains {
        result["calculation"]["aspectPreset"] = json!(if r.aspect_rules.provided {
            "custom"
        } else {
            preset
        });
        result["calculation"]["rulership"] = json!(rulership);
        result["calculation"]["domainProfile"] =
            json!({"id":"sevenmlabs-domain-selection","version":"2.0"});
        result["calculation"]["chartKind"] = json!("individualNatal");
        result["calculation"]["reportProfile"] =
            json!({"id":"sevenmlabs-individual-report","version":"1.1"});
        result["calculation"]["customProfileCount"] = json!(custom.len());
        result["warnings"].as_array_mut().unwrap().push(json!("Domain groups follow the declared selection profile; no scores or predictions are produced."));
    }
    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Value {
        json!({"operation":"natal", "utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},
        "location":{"latitude":10.8231,"longitude":106.6297}})
    }
    fn run(r: Value) -> Value {
        serde_json::from_str(&super::super::calculate_json(&r.to_string())).unwrap()
    }
    #[test]
    fn natal_has_real_sky_and_houses() {
        let v = run(request());
        assert_eq!(v["errors"], json!([]));
        assert_eq!(v["data"]["placements"].as_array().unwrap().len(), 10);
        assert_eq!(v["data"]["houses"].as_array().unwrap().len(), 12);
        let sun = v["data"]["placements"][0]["longitude"].as_f64().unwrap();
        assert!((sun - 280.369).abs() < 0.002);
        assert_eq!(v["data"]["placements"][0]["sign"], "Capricorn");
        assert!(v["data"]["placements"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["house"].is_number()));
    }
    #[test]
    fn validates_calendar_and_options() {
        for (key, value) in [
            ("month", 13),
            ("day", 32),
            ("hour", 24),
            ("minute", 60),
            ("second", 60),
            ("year", 2400),
        ] {
            let mut r = request();
            r["utc"][key] = json!(value);
            assert!(!run(r)["errors"].as_array().unwrap().is_empty());
        }
        let mut r = request();
        r["utc"]["year"] = json!(1900);
        r["utc"]["month"] = json!(2);
        r["utc"]["day"] = json!(29);
        assert!(!run(r)["errors"].as_array().unwrap().is_empty());
        let mut r = request();
        r["utc"]["month"] = json!(2);
        r["utc"]["day"] = json!(29);
        assert_eq!(run(r)["errors"], json!([]));
    }
    #[test]
    fn integer_utc_notations_return_the_same_chart() {
        let expected = run(request());
        let float_request = r#"{"operation":"natal","utc":{"year":2000.0,"month":1.0,"day":1e0,"hour":1.2e1,"minute":-0.0},"location":{"latitude":10.8231,"longitude":106.6297}}"#;
        let actual: Value =
            serde_json::from_str(&super::super::calculate_json(float_request)).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            actual["data"]["utc"].to_string(),
            r#"{"day":1,"hour":12,"minute":0,"month":1,"second":0.0,"year":2000}"#
        );
        for key in ["year", "month", "day", "hour", "minute"] {
            for invalid in [json!(1.5), json!("1"), json!(true), Value::Null] {
                let mut r = request();
                r["utc"][key] = invalid;
                assert_eq!(run(r)["errors"][0]["code"], "INVALID_INPUT");
            }
        }
    }
    #[test]
    fn zero_coordinates_time_sensitivity_and_polar_error() {
        let mut r = request();
        r["location"] = json!({"latitude":0,"longitude":0});
        let a = run(r.clone());
        assert_eq!(a["errors"], json!([]));
        r["utc"]["hour"] = json!(13);
        let b = run(r.clone());
        assert_ne!(
            a["data"]["placements"][1]["longitude"],
            b["data"]["placements"][1]["longitude"]
        );
        assert_ne!(
            a["data"]["angles"][0]["longitude"],
            b["data"]["angles"][0]["longitude"]
        );
        r["location"]["latitude"] = json!(80);
        assert_eq!(run(r.clone())["errors"][0]["code"], "CALCULATION_FAILED");
        r["houseSystem"] = json!("wholeSign");
        assert_eq!(run(r)["errors"], json!([]));
    }
    #[test]
    fn concurrent_options_are_isolated() {
        let a = request();
        let mut b = request();
        b["houseSystem"] = json!("wholeSign");
        b["location"]["latitude"] = json!(-33.86);
        let expected_a = run(a.clone());
        let expected_b = run(b.clone());
        std::thread::scope(|s| {
            for i in 0..8 {
                let (r, e) = if i % 2 == 0 {
                    (&a, &expected_a)
                } else {
                    (&b, &expected_b)
                };
                s.spawn(move || {
                    for _ in 0..25 {
                        assert_eq!(&run(r.clone()), e);
                    }
                });
            }
        });
    }
}
