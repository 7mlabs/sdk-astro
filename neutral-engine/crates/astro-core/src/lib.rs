//! Neutral local natal and geometry engine. No network, scoring or interpretation.
mod advanced;
mod composite;
mod compression;
mod couple;
mod couple_profiles;
mod domains;
mod events;
mod forecast;
mod natal;
mod numbers;
#[cfg(test)]
mod profile_integration_tests;
mod profiles;
mod query;
mod reports;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAX_REQUEST_BYTES: usize = 4 * 1024 * 1024;
/// Context preparation accepts computed reports, which can exceed request size.
pub const MAX_CONTEXT_BYTES: usize = 256 * 1024 * 1024;
pub use compression::{compress_json, expand_context_json};
const SIGNS: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Position {
    pub id: String,
    pub longitude: f64,
    #[serde(default)]
    pub speed: Option<f64>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AspectRule {
    pub angle: f64,
    pub max_orb: f64,
}

fn default_rules() -> Vec<AspectRule> {
    [(0., 8.), (60., 6.), (90., 8.), (120., 8.), (180., 8.)]
        .into_iter()
        .map(|(angle, max_orb)| AspectRule { angle, max_orb })
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub operation: String,
    pub positions: Vec<Position>,
    #[serde(default)]
    pub other_positions: Option<Vec<Position>>,
    #[serde(default)]
    pub houses: Option<Vec<f64>>,
    #[serde(default)]
    pub other_houses: Option<Vec<f64>>,
    #[serde(default, deserialize_with = "numbers::optional_u32")]
    pub harmonic: Option<u32>,
    #[serde(default = "default_rules")]
    pub aspect_rules: Vec<AspectRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Placement {
    id: String,
    longitude: f64,
    sign_index: usize,
    sign: &'static str,
    degree_in_sign: f64,
    speed: Option<f64>,
    is_retrograde: Option<bool>,
    house: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Aspect {
    body1: String,
    body2: String,
    angle: f64,
    separation: f64,
    orb: f64,
    max_orb: f64,
    applying: Option<bool>,
}

pub fn normalize(value: f64) -> f64 {
    let result = value.rem_euclid(360.);
    if result == 0. || result >= 360. {
        0.
    } else {
        result
    }
}

pub fn signed_difference(a: f64, b: f64) -> f64 {
    (normalize(a) - normalize(b) + 180.).rem_euclid(360.) - 180.
}

fn validate_positions(positions: &[Position]) -> Result<(), String> {
    if positions.is_empty() || positions.len() > 64 {
        return Err("Provide 1 to 64 positions".into());
    }
    let mut ids = HashSet::new();
    for p in positions {
        if p.id.trim().is_empty() || p.id.len() > 128 || !ids.insert(&p.id) {
            return Err("Body IDs must be nonempty, unique and at most 128 UTF-8 bytes".into());
        }
        if !p.longitude.is_finite() || p.speed.is_some_and(|s| !s.is_finite()) {
            return Err("Longitude and speed must be finite".into());
        }
    }
    Ok(())
}

fn validate_houses(houses: &Option<Vec<f64>>) -> Result<(), String> {
    if let Some(cusps) = houses {
        if cusps.len() != 12 || cusps.iter().any(|v| !v.is_finite()) {
            return Err("Houses require exactly 12 finite cusps in house order".into());
        }
        let arcs: Vec<f64> = (0..12)
            .map(|i| normalize(normalize(cusps[(i + 1) % 12]) - normalize(cusps[i])))
            .collect();
        if arcs.iter().any(|arc| *arc <= 1e-9) || (arcs.iter().sum::<f64>() - 360.).abs() > 1e-7 {
            return Err(
                "House cusps must traverse the zodiac exactly once, without duplicates".into(),
            );
        }
    }
    Ok(())
}

fn house_for(longitude: f64, houses: Option<&[f64]>) -> Option<usize> {
    houses.and_then(|cusps| {
        (0..12)
            .find(|i| {
                let width = normalize(normalize(cusps[(i + 1) % 12]) - normalize(cusps[*i]));
                normalize(longitude - normalize(cusps[*i])) < width
            })
            .map(|i| i + 1)
    })
}

fn placements(positions: &[Position], houses: Option<&[f64]>) -> Vec<Placement> {
    positions
        .iter()
        .map(|p| {
            let longitude = normalize(p.longitude);
            let sign_index = (longitude / 30.).floor() as usize;
            Placement {
                id: p.id.clone(),
                longitude,
                sign_index,
                sign: SIGNS[sign_index],
                degree_in_sign: longitude % 30.,
                speed: p.speed,
                is_retrograde: p.speed.map(|s| s < 0.),
                house: house_for(longitude, houses),
            }
        })
        .collect()
}

fn aspects(a: &[Position], b: Option<&[Position]>, rules: &[AspectRule]) -> Vec<Aspect> {
    let mut output = Vec::new();
    for (i, p1) in a.iter().enumerate() {
        let partners = b.unwrap_or(&a[i + 1..]);
        for p2 in partners {
            let delta = signed_difference(p1.longitude, p2.longitude);
            let separation = delta.abs();
            for rule in rules {
                let orb = (separation - rule.angle).abs();
                if orb <= rule.max_orb {
                    // Cross-chart natal velocities do not describe evolution of this comparison.
                    let applying = if b.is_some() || orb <= 1e-12 {
                        None
                    } else {
                        p1.speed.zip(p2.speed).map(|(s1, s2)| {
                            let distance_rate = (s1 - s2) * delta.signum();
                            (separation - rule.angle) * distance_rate < 0.
                        })
                    };
                    output.push(Aspect {
                        body1: p1.id.clone(),
                        body2: p2.id.clone(),
                        angle: rule.angle,
                        separation,
                        orb,
                        max_orb: rule.max_orb,
                        applying,
                    });
                }
            }
        }
    }
    output
}

fn midpoints(positions: &[Position]) -> Vec<Value> {
    let mut output = Vec::new();
    for (i, a) in positions.iter().enumerate() {
        for b in &positions[i + 1..] {
            let difference = signed_difference(b.longitude, a.longitude);
            let ambiguous = (difference.abs() - 180.).abs() <= 1e-10;
            output.push(json!({ "body1": a.id, "body2": b.id, "ambiguous": ambiguous,
                "longitude": if ambiguous { None } else { Some(normalize(normalize(a.longitude) + difference / 2.)) } }));
        }
    }
    output
}

fn validate_aspect_rules(rules: &[AspectRule]) -> Result<(), String> {
    if rules.len() > 16 {
        return Err("At most 16 aspect rules are allowed".into());
    }
    for (i, rule) in rules.iter().enumerate() {
        if !rule.angle.is_finite()
            || !(0. ..=180.).contains(&rule.angle)
            || !rule.max_orb.is_finite()
            || !(0. ..=15.).contains(&rule.max_orb)
        {
            return Err("Aspect angle must be 0..180 and maxOrb 0..15 degrees".into());
        }
        if rules[..i].iter().any(|r| r.angle == rule.angle) {
            return Err("Aspect rule angles must be unique".into());
        }
    }
    Ok(())
}

fn validate(request: &Request) -> Result<(), String> {
    validate_positions(&request.positions)?;
    validate_houses(&request.houses)?;
    validate_houses(&request.other_houses)?;
    validate_aspect_rules(&request.aspect_rules)?;
    match request.operation.as_str() {
        "chart" => {
            if request.other_positions.is_some()
                || request.other_houses.is_some()
                || request.harmonic.is_some()
            {
                return Err("chart does not accept otherPositions, otherHouses or harmonic".into());
            }
        }
        "harmonic" => {
            if !request.harmonic.is_some_and(|h| (1..=360).contains(&h)) {
                return Err("harmonic requires an integer factor 1..360".into());
            }
            if request.houses.is_some()
                || request.other_houses.is_some()
                || request.other_positions.is_some()
            {
                return Err("harmonic does not accept houses or another chart".into());
            }
        }
        "synastry" => {
            validate_positions(
                request
                    .other_positions
                    .as_deref()
                    .ok_or("synastry requires otherPositions")?,
            )?;
            if request.harmonic.is_some() {
                return Err("synastry does not accept harmonic".into());
            }
        }
        _ => return Err("Supported operations: chart, harmonic, synastry".into()),
    }
    Ok(())
}

pub fn calculate(request: Request) -> Result<Value, String> {
    validate(&request)?;
    let mut warnings: Vec<&str> = vec![];
    let data = match request.operation.as_str() {
        "chart" => json!({
            "placements": placements(&request.positions, request.houses.as_deref()),
            "aspects": aspects(&request.positions, None, &request.aspect_rules),
            "midpoints": midpoints(&request.positions)
        }),
        "harmonic" => {
            let factor = request.harmonic.unwrap() as f64;
            let positions: Vec<Position> = request
                .positions
                .iter()
                .map(|p| Position {
                    id: p.id.clone(),
                    longitude: normalize(normalize(p.longitude) * factor),
                    speed: p.speed.map(|s| s * factor),
                })
                .collect();
            if positions
                .iter()
                .any(|p| p.speed.is_some_and(|s| !s.is_finite()))
            {
                return Err("Harmonic speed overflows finite degrees/day".into());
            }
            warnings.push("Harmonic coordinates are transformed longitudes, not a recalculated physical sky or house system");
            json!({ "harmonic": request.harmonic, "placements": placements(&positions, None),
                "aspects": aspects(&positions, None, &request.aspect_rules), "midpoints": midpoints(&positions) })
        }
        "synastry" => {
            let other = request.other_positions.as_ref().unwrap();
            json!({ "personA": placements(&request.positions, request.houses.as_deref()),
                "personB": placements(other, request.other_houses.as_deref()),
                "crossAspects": aspects(&request.positions, Some(other), &request.aspect_rules),
                "overlaysAtoB": placements(&request.positions, request.other_houses.as_deref()),
                "overlaysBtoA": placements(other, request.houses.as_deref()) })
        }
        _ => unreachable!(),
    };
    Ok(json!({ "schemaVersion": "1.0", "engineVersion": VERSION,
        "calculation": { "scope": "geometry-from-positions", "provider": "supplied-positions",
            "angleUnit": "degrees", "speedUnit": "degrees/day", "aspectRules": request.aspect_rules.iter()
                .map(|r| json!({"angle":r.angle,"maxOrb":r.max_orb})).collect::<Vec<_>>() },
        "data": data, "warnings": warnings, "errors": [] }))
}

pub fn error_json(code: &str, message: &str) -> String {
    json!({ "schemaVersion": "1.0", "engineVersion": VERSION, "data": null,
        "warnings": [], "errors": [{ "code": code, "message": message }] })
    .to_string()
}

pub fn calculate_json(input: &str) -> String {
    if input.len() > MAX_REQUEST_BYTES {
        return error_json("INPUT_TOO_LARGE", "Input exceeds 4 MiB");
    }
    #[derive(Deserialize)]
    struct Operation {
        operation: String,
    }
    // Read only the operation; preserve numeric literals for exact normalization.
    let operation = match serde_json::from_str::<Operation>(input) {
        Ok(header) => header.operation,
        Err(error) => return error_json("INVALID_INPUT", &error.to_string()),
    };
    if operation == "events" {
        return events::calculate_json(input);
    }
    if operation == "query" {
        return query::calculate_json(input);
    }
    if operation == "forecast" {
        return forecast::calculate_json(input);
    }
    if operation == "composite" {
        return composite::calculate_json(input);
    }
    if operation == "couple" {
        return couple::calculate_json(input);
    }
    if operation == "natal" || operation == "natalDomains" {
        return natal::calculate_json(input);
    }
    let request = match serde_json::from_str::<Request>(input) {
        Ok(request) => request,
        Err(error) => return error_json("INVALID_INPUT", &error.to_string()),
    };
    match calculate(request) {
        Ok(result) => result.to_string(),
        Err(message) => error_json("INVALID_INPUT", &message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(input: Value) -> Value {
        serde_json::from_str(&calculate_json(&input.to_string())).unwrap()
    }
    fn chart(p: Value) -> Value {
        run(json!({"operation":"chart","positions":p}))
    }
    #[test]
    fn wrap_and_sign() {
        let result = chart(json!([{"id":"sun","longitude":-1},{"id":"moon","longitude":361}]));
        assert_eq!(result["data"]["placements"][0]["longitude"], 359.);
        assert_eq!(result["data"]["placements"][0]["sign"], "Pisces");
        assert_eq!(result["data"]["aspects"][0]["orb"], 2.);
        assert_eq!(result["data"]["midpoints"][0]["longitude"], 0.);
    }
    #[test]
    fn applying_and_unknown_speed() {
        let result = chart(
            json!([{"id":"moon","longitude":359,"speed":13},{"id":"sun","longitude":1,"speed":1}]),
        );
        assert_eq!(result["data"]["aspects"][0]["applying"], true);
        let result = chart(json!([{"id":"moon","longitude":359},{"id":"sun","longitude":1}]));
        assert!(result["data"]["aspects"][0]["applying"].is_null());
    }
    #[test]
    fn antipodal_midpoint_is_ambiguous() {
        let result = chart(json!([{"id":"a","longitude":0},{"id":"b","longitude":180}]));
        assert_eq!(result["data"]["midpoints"][0]["ambiguous"], true);
        assert!(result["data"]["midpoints"][0]["longitude"].is_null());
    }
    #[test]
    fn house_boundary_and_zero_wrap() {
        let result = run(
            json!({"operation":"chart","positions":[{"id":"a","longitude":0},{"id":"b","longitude":350}],
            "houses":[350,20,50,80,110,140,170,200,230,260,290,320]}),
        );
        assert_eq!(result["data"]["placements"][0]["house"], 1);
        assert_eq!(result["data"]["placements"][1]["house"], 1);
    }
    #[test]
    fn harmonic_recomputes_sign_and_aspects() {
        let result = run(
            json!({"operation":"harmonic","harmonic":5,"positions":[{"id":"a","longitude":0},{"id":"b","longitude":72}]}),
        );
        assert_eq!(result["data"]["placements"][1]["longitude"], 0.);
        assert_eq!(result["data"]["aspects"][0]["angle"], 0.);
    }
    #[test]
    fn synastry_preserves_direction() {
        let result = run(
            json!({"operation":"synastry","positions":[{"id":"a","longitude":10}],
            "otherPositions":[{"id":"b","longitude":130}]}),
        );
        assert_eq!(result["data"]["crossAspects"][0]["body1"], "a");
        assert_eq!(result["data"]["crossAspects"][0]["angle"], 120.);
    }
    #[test]
    fn rejects_duplicate_ids_and_unknown_operation() {
        assert_eq!(
            chart(json!([{"id":"a","longitude":0},{"id":"a","longitude":1}]))["errors"][0]["code"],
            "INVALID_INPUT"
        );
        assert_eq!(
            run(json!({"operation":"natal","positions":[{"id":"a","longitude":0}]}))["errors"][0]
                ["code"],
            "INVALID_INPUT"
        );
    }
    #[test]
    fn rejects_invalid_houses_and_rules() {
        let result = run(
            json!({"operation":"chart","positions":[{"id":"a","longitude":0}],"houses":vec![0.;12]}),
        );
        assert!(!result["errors"].as_array().unwrap().is_empty());
        let result = run(
            json!({"operation":"chart","positions":[{"id":"a","longitude":0}],"aspectRules":[{"angle":400,"maxOrb":1}]}),
        );
        assert!(!result["errors"].as_array().unwrap().is_empty());
    }
    #[test]
    fn rejects_malformed_and_oversized_json() {
        assert!(calculate_json("{").contains("INVALID_INPUT"));
        assert!(calculate_json(&" ".repeat(MAX_REQUEST_BYTES + 1)).contains("INPUT_TOO_LARGE"));
    }

    #[test]
    fn huge_harmonic_speed_is_an_error_not_unknown() {
        let result = run(json!({"operation":"harmonic","harmonic":360,
            "positions":[{"id":"a","longitude":1e300,"speed":1e308}]}));
        assert_eq!(result["errors"][0]["code"], "INVALID_INPUT");
    }
}
