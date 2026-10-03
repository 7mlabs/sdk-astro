//! Symbolic third chart constructed from corresponding natal longitudes.
//! It has no birth instant, geographic location or astronomical velocity.
use super::*;
use serde_json::value::RawValue;

const TOLERANCE: f64 = 1e-10;
fn default_house_method() -> String {
    "midpoint".into()
}
fn default_antipodal_policy() -> String {
    "error".into()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    #[serde(default = "default_house_method")]
    pub house_method: String,
    #[serde(default = "default_antipodal_policy")]
    pub antipodal_policy: String,
    #[serde(default, deserialize_with = "natal::present")]
    pub domains: Option<Vec<String>>,
    #[serde(default, deserialize_with = "natal::present")]
    pub custom_profiles: Option<Vec<profiles::Profile>>,
}
impl Options {
    fn names(&self) -> Vec<String> {
        self.domains
            .clone()
            .unwrap_or_else(|| profiles::NAMES.into_iter().map(String::from).collect())
    }
    fn custom(&self) -> &[profiles::Profile] {
        self.custom_profiles.as_deref().unwrap_or(&[])
    }
    pub(super) fn validate(&self, rulership: &str) -> Result<(), String> {
        if !["midpoint", "wholeSignFromMidpointAscendant"].contains(&self.house_method.as_str()) {
            return Err("houseMethod must be midpoint or wholeSignFromMidpointAscendant".into());
        }
        if !["error", "lowerLongitude"].contains(&self.antipodal_policy.as_str()) {
            return Err("antipodalPolicy must be error or lowerLongitude".into());
        }
        if self.custom_profiles.is_some() && self.custom().is_empty() {
            return Err("customProfiles must contain 1 to 8 profiles when provided".into());
        }
        profiles::validate_selection(&self.names(), self.custom(), rulership)
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CompositeRequest {
    operation: String,
    person_a: Box<RawValue>,
    person_b: Box<RawValue>,
    #[serde(default = "default_house_method")]
    house_method: String,
    #[serde(default = "default_antipodal_policy")]
    antipodal_policy: String,
    #[serde(default, deserialize_with = "natal::present")]
    domains: Option<Vec<String>>,
    #[serde(default, deserialize_with = "natal::present")]
    custom_profiles: Option<Vec<profiles::Profile>>,
    #[serde(default)]
    aspect_rules: natal::AspectSettings,
    #[serde(default, deserialize_with = "natal::present")]
    aspect_preset: Option<String>,
    #[serde(default, deserialize_with = "natal::present")]
    rulership: Option<String>,
}
fn antipodal(a: f64, b: f64) -> bool {
    ((normalize(a) - normalize(b)).abs() - 180.).abs() <= TOLERANCE
}
/// Sorting first keeps A/B swaps bit-identical, including rounding near 0/360.
fn midpoint(a: f64, b: f64, policy: &str, id: &str) -> Result<f64, String> {
    let a = normalize(a);
    let b = normalize(b);
    let lower = a.min(b);
    let upper = a.max(b);
    let difference = upper - lower;
    if (difference - 180.).abs() <= TOLERANCE {
        if policy == "error" {
            return Err(format!("Composite point {id} has antipodal source longitudes; choose antipodalPolicy lowerLongitude explicitly to resolve its two midpoints"));
        }
        return Ok(
            normalize(lower + difference / 2.).min(normalize(lower + difference / 2. + 180.))
        );
    }
    Ok(normalize(
        lower + difference / 2. + if difference > 180. { 180. } else { 0. },
    ))
}
fn source_longitude(natal: &Value, kind: &str, id: &str) -> f64 {
    if kind == "houseCusp" {
        let number: usize = id[1..].parse().unwrap();
        natal["houseCusps"][number - 1].as_f64().unwrap()
    } else {
        let key = if kind == "body" {
            "placements"
        } else {
            "angles"
        };
        natal[key]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap()["longitude"]
            .as_f64()
            .unwrap()
    }
}
fn provenance_point(
    id: &str,
    kind: &str,
    a: f64,
    b: f64,
    longitude: f64,
    construction: &str,
) -> Value {
    let ambiguous = antipodal(a, b);
    let derived = construction != "shortestArcMidpoint";
    json!({"id":id,"kind":kind,"sourcePointIds":[format!("A:{id}"),format!("B:{id}")],"sourceLongitudes":[a,b],"longitude":longitude,"construction":construction,"antipodal":ambiguous,"resolution":if derived {"notRequiredForDerivedPoint"} else if ambiguous {"lowerLongitude"} else {"unambiguous"}})
}
fn source_calculations(a: &Value, b: &Value) -> Value {
    json!({"A":{"julianDayTt":a["julianDayTt"],"julianDayUt1":a["julianDayUt1"],"houseSystem":a["houseSystem"]},"B":{"julianDayTt":b["julianDayTt"],"julianDayUt1":b["julianDayUt1"],"houseSystem":b["houseSystem"]}})
}
pub(super) fn metadata(
    options: &Options,
    rules: &[AspectRule],
    rulership: &str,
    preset: &str,
    a: &Value,
    b: &Value,
) -> Value {
    json!({"scope":"midpoint-composite-data","chartKind":"midpointComposite","construction":"symbolic","method":"shortestArcMidpoint","houseMethod":options.house_method,"antipodalPolicy":options.antipodal_policy,"toleranceDegrees":TOLERANCE,"sourceProvider":a["provider"],"sourceProviderVersion":a["providerVersion"],"sourceEphemeris":a["ephemeris"],"zodiac":"tropical","angleUnit":"degrees","speedUnit":null,"motion":"notApplicable","aspectPreset":preset,"aspectRules":rules.iter().map(|v|json!({"angle":v.angle,"maxOrb":v.max_orb})).collect::<Vec<_>>(),"rulership":rulership,"domainProfile":{"id":"sevenmlabs-domain-selection","version":"2.0"},"reportProfile":{"id":"sevenmlabs-individual-report","version":"1.1"},"customProfileCount":options.custom().len(),"sourceCalculations":source_calculations(a,b)})
}
pub(super) const WARNING:&str="Composite C is a symbolic shortest-arc midpoint construction, not an astronomical sky at a birth instant. Velocity, retrograde and applying are not applicable; dignity and solar conditions are geometric sign/longitude facts.";
/// Both birth skies are already computed; this function never calls a provider.
pub(super) fn build_from_natals(
    a: &Value,
    calculation_a: &Value,
    b: &Value,
    calculation_b: &Value,
    options: &Options,
    rules: &[AspectRule],
    rulership: &str,
) -> Result<Value, String> {
    let mut provenance = Vec::new();
    let positions: Vec<Position> = a["placements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|body| {
            let id = body["id"].as_str().unwrap();
            let first = source_longitude(a, "body", id);
            let second = source_longitude(b, "body", id);
            let longitude = midpoint(first, second, &options.antipodal_policy, id)?;
            provenance.push(provenance_point(
                id,
                "body",
                first,
                second,
                longitude,
                "shortestArcMidpoint",
            ));
            Ok(Position {
                id: id.into(),
                longitude,
                speed: None,
            })
        })
        .collect::<Result<_, String>>()?;
    let mut angle_positions: Vec<Position> = Vec::new();
    for id in ["ascendant", "midheaven", "descendant", "imumCoeli"] {
        let first = source_longitude(a, "angle", id);
        let second = source_longitude(b, "angle", id);
        let (longitude, construction) = match id {
            "descendant" => (
                normalize(angle_positions[0].longitude + 180.),
                "oppositeCompositeAscendant",
            ),
            "imumCoeli" => (
                normalize(angle_positions[1].longitude + 180.),
                "oppositeCompositeMidheaven",
            ),
            _ => (
                midpoint(first, second, &options.antipodal_policy, id)?,
                "shortestArcMidpoint",
            ),
        };
        provenance.push(provenance_point(
            id,
            "angle",
            first,
            second,
            longitude,
            construction,
        ));
        angle_positions.push(Position {
            id: id.into(),
            longitude,
            speed: None,
        });
    }
    let whole_sign = options.house_method == "wholeSignFromMidpointAscendant";
    let first_cusp = (angle_positions[0].longitude / 30.).floor() * 30.;
    let cusps: Vec<f64> = (0..12)
        .map(|index| {
            let id = format!("H{}", index + 1);
            let first = source_longitude(a, "houseCusp", &id);
            let second = source_longitude(b, "houseCusp", &id);
            let longitude = if whole_sign {
                normalize(first_cusp + index as f64 * 30.)
            } else {
                midpoint(first, second, &options.antipodal_policy, &id)?
            };
            provenance.push(provenance_point(
                &id,
                "houseCusp",
                first,
                second,
                longitude,
                if whole_sign {
                    "wholeSignFromMidpointAscendant"
                } else {
                    "shortestArcMidpoint"
                },
            ));
            Ok(longitude)
        })
        .collect::<Result<_, String>>()?;
    validate_houses(&Some(cusps.clone())).map_err(|e|format!("Composite house cusps are invalid: {e}; choose houseMethod wholeSignFromMidpointAscendant explicitly instead"))?;
    let houses:Vec<Value>=cusps.iter().enumerate().map(|(i,longitude)|json!({"number":i+1,"longitude":longitude,"sign":SIGNS[(longitude/30.).floor() as usize],"degreeInSign":longitude%30.})).collect();
    let chart = json!({"placements":placements(&positions,Some(&cusps)),"angles":placements(&angle_positions,None),"houses":houses,"houseCusps":cusps,"aspects":aspects(&positions,None,rules)});
    let mut data = domains::build_with_kind(
        chart,
        rules,
        &options.names(),
        rulership,
        options.custom(),
        "midpointComposite",
    );
    let chart = data["natal"].take();
    let object = data.as_object_mut().unwrap();
    object.remove("natal");
    object.remove("subjectCount");
    data["chart"] = chart;
    data["id"] = json!("C");
    data["sourceSubjectIds"] = json!(["A", "B"]);
    data["provenance"] = json!({"method":"shortestArcMidpoint","houseMethod":options.house_method,"antipodalPolicy":options.antipodal_policy,"toleranceDegrees":TOLERANCE,"sourceHouseSystems":{"A":calculation_a["houseSystem"],"B":calculation_b["houseSystem"]},"points":provenance});
    Ok(data)
}
pub(super) fn calculate_json(raw: &str) -> String {
    let r: CompositeRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &e.to_string()),
    };
    if r.operation != "composite" {
        return error_json("INVALID_INPUT", "Expected composite operation");
    }
    let a: natal::BirthInput = match serde_json::from_str(r.person_a.get()) {
        Ok(v) => v,
        Err(e) => return error_json("INVALID_INPUT", &format!("personA: {e}")),
    };
    let b: natal::BirthInput = match serde_json::from_str(r.person_b.get()) {
        Ok(v) => v,
        Err(e) => return error_json("INVALID_INPUT", &format!("personB: {e}")),
    };
    let preset = r.aspect_preset.as_deref().unwrap_or("extended");
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
    if let Err(e) = validate_aspect_rules(&rules) {
        return error_json("INVALID_INPUT", &e);
    }
    let rulership = r.rulership.as_deref().unwrap_or("traditional");
    let options = Options {
        house_method: r.house_method,
        antipodal_policy: r.antipodal_policy,
        domains: r.domains,
        custom_profiles: r.custom_profiles,
    };
    if let Err(e) = options.validate(rulership) {
        return error_json("INVALID_INPUT", &e);
    }
    let system_a = match natal::validate_birth(&a) {
        Ok(s) => s,
        Err(e) => return error_json("INVALID_INPUT", &format!("personA: {e}")),
    };
    let system_b = match natal::validate_birth(&b) {
        Ok(s) => s,
        Err(e) => return error_json("INVALID_INPUT", &format!("personB: {e}")),
    };
    let (natal_a, calculation_a) = match natal::compute_birth(&a, system_a, &rules) {
        Ok(v) => v,
        Err(e) => return error_json("CALCULATION_FAILED", &format!("personA: {e}")),
    };
    let (natal_b, calculation_b) = match natal::compute_birth(&b, system_b, &rules) {
        Ok(v) => v,
        Err(e) => return error_json("CALCULATION_FAILED", &format!("personB: {e}")),
    };
    let composite = match build_from_natals(
        &natal_a,
        &calculation_a,
        &natal_b,
        &calculation_b,
        &options,
        &rules,
        rulership,
    ) {
        Ok(v) => v,
        Err(e) => return error_json("CALCULATION_FAILED", &e),
    };
    let calculation = metadata(
        &options,
        &rules,
        rulership,
        if r.aspect_rules.provided {
            "custom"
        } else {
            preset
        },
        &calculation_a,
        &calculation_b,
    );
    json!({"schemaVersion":"1.0","engineVersion":VERSION,"calculation":calculation,"data":{"chartKind":"compositeRelationship","subjectCount":2,"chartCount":3,"subjects":{"A":{"id":"A","natal":natal_a,"calculation":calculation_a},"B":{"id":"B","natal":natal_b,"calculation":calculation_b}},"composite":composite},"warnings":["Moshier analytical ephemeris; no JPL/Swiss data files. Future UTC uses the provider's built-in time model, not live Earth-orientation data.",WARNING],"errors":[]}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    const IDS: [&str; 10] = [
        "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
        "pluto",
    ];
    fn options(value: Value) -> Options {
        serde_json::from_value(value).unwrap()
    }
    fn sky(cusps: Vec<f64>, offset: f64) -> Value {
        let bodies: Vec<Position> = IDS
            .into_iter()
            .enumerate()
            .map(|(i, id)| Position {
                id: id.into(),
                longitude: normalize(i as f64 * 31. + offset),
                speed: Some(1.),
            })
            .collect();
        let angles: Vec<Position> = [
            ("ascendant", cusps[0]),
            ("midheaven", cusps[9]),
            ("descendant", normalize(cusps[0] + 180.)),
            ("imumCoeli", normalize(cusps[9] + 180.)),
        ]
        .into_iter()
        .map(|(id, longitude)| Position {
            id: id.into(),
            longitude,
            speed: None,
        })
        .collect();
        json!({"placements":placements(&bodies,Some(&cusps)),"angles":placements(&angles,None),"houseCusps":cusps})
    }
    fn equal_houses(offset: f64) -> Vec<f64> {
        (0..12)
            .map(|i| normalize(offset + i as f64 * 30.))
            .collect()
    }
    fn meta() -> Value {
        json!({"houseSystem":"placidus"})
    }
    #[test]
    fn circular_midpoint_wrap_identity_and_swap() {
        assert_eq!(midpoint(350., 10., "error", "sun").unwrap(), 0.);
        for (a, b) in [
            (10., 350.),
            (42.25, 42.25),
            (0., 359.999999999),
            (181.125, 15.25),
            (360., 0.),
        ] {
            let c = midpoint(a, b, "error", "sun").unwrap();
            assert_eq!(c, midpoint(b, a, "error", "sun").unwrap());
            assert!((0. ..360.).contains(&c));
            if normalize(a) == normalize(b) {
                assert_eq!(c, normalize(a));
            }
        }
    }
    #[test]
    fn antipodal_ambiguity_and_explicit_lower_candidate() {
        for (a, b, expected) in [(0., 180., 90.), (100., 280., 10.), (10., 190., 100.)] {
            assert!(midpoint(a, b, "error", "venus")
                .unwrap_err()
                .contains("venus"));
            assert_eq!(midpoint(a, b, "lowerLongitude", "venus").unwrap(), expected);
            assert_eq!(
                midpoint(a, b, "lowerLongitude", "venus").unwrap(),
                midpoint(b, a, "lowerLongitude", "venus").unwrap()
            );
        }
        assert!(midpoint(0., 180. + TOLERANCE / 2., "error", "sun").is_err());
        assert!(midpoint(0., 180. + TOLERANCE * 2., "error", "sun").is_ok());
    }
    #[test]
    fn invalid_cusp_midpoints_fail_and_whole_sign_is_explicit() {
        let a = sky(equal_houses(0.), 0.);
        let b = sky(
            vec![
                170., 225., 230., 240., 250., 260., 270., 280., 290., 300., 310., 320.,
            ],
            1.,
        );
        validate_houses(&Some(
            b["houseCusps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect(),
        ))
        .unwrap();
        let r = build_from_natals(
            &a,
            &meta(),
            &b,
            &meta(),
            &options(json!({})),
            &default_rules(),
            "traditional",
        );
        assert!(r.unwrap_err().contains("wholeSignFromMidpointAscendant"));
        let c = build_from_natals(
            &a,
            &meta(),
            &b,
            &meta(),
            &options(json!({"houseMethod":"wholeSignFromMidpointAscendant"})),
            &default_rules(),
            "traditional",
        )
        .unwrap();
        assert_eq!(c["chart"]["houseCusps"], json!(equal_houses(60.)));
        for p in &c["provenance"]["points"].as_array().unwrap()[14..] {
            assert_eq!(p["resolution"], "notRequiredForDerivedPoint");
        }
    }
    #[test]
    fn opposite_axes_are_preserved_when_antipodal_policy_selects_other_branch() {
        let a = sky(equal_houses(100.), 0.);
        let b = sky(equal_houses(280.), 1.);
        let c=build_from_natals(&a,&meta(),&b,&meta(),&options(json!({"houseMethod":"wholeSignFromMidpointAscendant","antipodalPolicy":"lowerLongitude"})),&default_rules(),"traditional").unwrap();
        let angles = &c["chart"]["angles"];
        assert_eq!(angles[0]["longitude"], 10.);
        assert_eq!(angles[2]["longitude"], 190.);
        assert_eq!(
            angles[3]["longitude"].as_f64().unwrap(),
            normalize(angles[1]["longitude"].as_f64().unwrap() + 180.)
        );
        assert_eq!(
            c["provenance"]["points"][12]["construction"],
            "oppositeCompositeAscendant"
        );
        assert_eq!(
            c["provenance"]["points"][12]["resolution"],
            "notRequiredForDerivedPoint"
        );
    }
    #[test]
    fn symbolic_facts_recompute_without_velocity_or_fabricated_coordinates() {
        let a = sky(equal_houses(0.), 350.);
        let b = sky(equal_houses(10.), 10.);
        let c = build_from_natals(
            &a,
            &meta(),
            &b,
            &meta(),
            &options(json!({})),
            &domains::extended_rules(),
            "modern",
        )
        .unwrap();
        assert_eq!(c["chart"]["placements"][0]["longitude"], 0.);
        assert_eq!(c["context"]["advanced"]["bodyStates"][0]["element"], "fire");
        assert_eq!(
            c["context"]["advanced"]["rules"]["motion"]["source"],
            "symbolicMidpointConstruction"
        );
        assert_eq!(
            c["context"]["advanced"]["rules"]["distributions"]["population"],
            "allCompositeBodies"
        );
        for p in c["chart"]["placements"].as_array().unwrap() {
            assert!(p["speed"].is_null() && p["isRetrograde"].is_null());
            assert!(!p.as_object().unwrap().contains_key("latitude"));
            assert!(!p.as_object().unwrap().contains_key("distanceAu"));
        }
        for state in c["context"]["advanced"]["bodyStates"].as_array().unwrap() {
            assert_eq!(state["motion"], "notApplicable");
        }
        for aspect in c["context"]["aspects"].as_array().unwrap() {
            assert!(aspect["applying"].is_null());
        }
        assert_eq!(c["context"]["points"].as_array().unwrap().len(), 26);
        assert_eq!(c["context"]["relations"].as_array().unwrap().len(), 325);
        assert_eq!(
            c["context"]["houseRulerRelations"]
                .as_array()
                .unwrap()
                .len(),
            66
        );
        assert_eq!(c["domains"].as_object().unwrap().len(), 10);
        assert_eq!(c["provenance"]["points"].as_array().unwrap().len(), 26);
        for view in c["domains"].as_object().unwrap().values() {
            assert_eq!(view["report"]["chartKind"], "midpointComposite");
            assert_eq!(view["report"]["sections"].as_array().unwrap().len(), 3);
        }
        for key in ["utc", "location", "julianDayTt", "julianDayUt1"] {
            assert!(!c["chart"].as_object().unwrap().contains_key(key));
        }
    }
}
