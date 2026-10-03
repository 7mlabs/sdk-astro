//! Small, validated selections of shared geometry and natal facts.
use super::*;
use std::collections::HashMap;

const POINT_IDS: [&str; 26] = [
    "sun",
    "moon",
    "mercury",
    "venus",
    "mars",
    "jupiter",
    "saturn",
    "uranus",
    "neptune",
    "pluto",
    "ascendant",
    "midheaven",
    "descendant",
    "imumCoeli",
    "H1",
    "H2",
    "H3",
    "H4",
    "H5",
    "H6",
    "H7",
    "H8",
    "H9",
    "H10",
    "H11",
    "H12",
];
const MIDPOINT_TOLERANCE: f64 = 1e-10;

#[derive(Deserialize)]
struct Header {
    operation: String,
    group: String,
    action: String,
}

// Deserialize each final request directly from JSON: tagged/flattened enum buffers
// would lose the raw numeric literals required by the exact-integer parser.
macro_rules! request {
    ($name:ident { $($fields:tt)* }) => {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct $name {
            #[serde(rename = "operation")]
            _operation: String,
            #[serde(rename = "group")]
            _group: String,
            #[serde(rename = "action")]
            _action: String,
            $($fields)*
        }
    };
}
request!(NormalizeRequest { longitude: f64 });
request!(SeparationRequest {
    longitude1: f64,
    longitude2: f64,
});
request!(MidpointRequest {
    longitude1: f64,
    longitude2: f64,
    #[serde(default, deserialize_with = "natal::present")]
    antipodal_policy: Option<String>,
});
request!(BetweenRequest {
    positions: [Position; 2],
    rule: AspectRule,
    #[serde(default, deserialize_with = "natal::present")]
    motion_mode: Option<String>,
});
request!(LocateRequest { longitude: f64, house_cusps: Vec<f64>, });

#[derive(Deserialize)]
struct ExactInteger(#[serde(deserialize_with = "numbers::i32")] i32);
request!(HouseInspectRequest {
    birth: natal::BirthInput,
    #[serde(default, deserialize_with = "natal::present")]
    house_numbers: Option<Vec<ExactInteger>>,
    #[serde(default, deserialize_with = "natal::present")]
    rulership: Option<String>,
    #[serde(default, deserialize_with = "natal::present")]
    aspect_preset: Option<String>,
    #[serde(default)]
    aspect_rules: natal::AspectSettings,
});
request!(PointInspectRequest {
    birth: natal::BirthInput,
    #[serde(default, deserialize_with = "natal::present")]
    point_ids: Option<Vec<String>>,
    #[serde(default, deserialize_with = "natal::present")]
    rulership: Option<String>,
    #[serde(default, deserialize_with = "natal::present")]
    aspect_preset: Option<String>,
    #[serde(default)]
    aspect_rules: natal::AspectSettings,
});

struct Failure {
    code: &'static str,
    message: String,
}
impl Failure {
    fn input(message: impl Into<String>) -> Self {
        Self {
            code: "INVALID_INPUT",
            message: message.into(),
        }
    }
    fn calculation(message: impl Into<String>) -> Self {
        Self {
            code: "CALCULATION_FAILED",
            message: message.into(),
        }
    }
}
fn parse<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, Failure> {
    serde_json::from_str(raw).map_err(|e| Failure::input(e.to_string()))
}
fn finite_longitudes(values: &[f64]) -> Result<(), Failure> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(Failure::input("Longitudes must be finite degrees"));
    }
    Ok(())
}
fn geometry_metadata(group: &str, action: &str) -> Value {
    json!({"scope":"on-demand-query","computationalScope":"pureGeometry","group":group,"action":action,"provider":"supplied-positions","angleUnit":"degrees"})
}
fn response(data: Value, calculation: Value) -> Value {
    json!({"schemaVersion":"1.0","engineVersion":VERSION,"data":data,"calculation":calculation,"warnings":[],"errors":[]})
}

fn normalize_query(r: NormalizeRequest) -> Result<Value, Failure> {
    finite_longitudes(&[r.longitude])?;
    let longitude = normalize(r.longitude);
    let sign_index = (longitude / 30.).floor() as usize;
    Ok(response(
        json!({"group":"geometry","action":"normalize","inputLongitude":r.longitude,"longitude":longitude,"signIndex":sign_index,"sign":SIGNS[sign_index],"degreeInSign":longitude%30.}),
        geometry_metadata("geometry", "normalize"),
    ))
}
fn separation_query(r: SeparationRequest) -> Result<Value, Failure> {
    finite_longitudes(&[r.longitude1, r.longitude2])?;
    let (first, second) = (normalize(r.longitude1), normalize(r.longitude2));
    let delta = signed_difference(second, first);
    Ok(response(
        json!({"group":"geometry","action":"separation","longitude1":first,"longitude2":second,"signedDelta":delta,"separation":delta.abs()}),
        geometry_metadata("geometry", "separation"),
    ))
}
fn midpoint_query(r: MidpointRequest) -> Result<Value, Failure> {
    finite_longitudes(&[r.longitude1, r.longitude2])?;
    let policy = r.antipodal_policy.as_deref().unwrap_or("error");
    if !["error", "lowerLongitude"].contains(&policy) {
        return Err(Failure::input(
            "antipodalPolicy must be error or lowerLongitude",
        ));
    }
    let (first, second) = (normalize(r.longitude1), normalize(r.longitude2));
    let (lower, upper) = (first.min(second), first.max(second));
    let difference = upper - lower;
    let antipodal = (difference - 180.).abs() <= MIDPOINT_TOLERANCE;
    if antipodal && policy == "error" {
        return Err(Failure::calculation("Antipodal longitudes have two midpoint candidates; choose antipodalPolicy lowerLongitude explicitly"));
    }
    let longitude = if antipodal {
        normalize(lower + difference / 2.).min(normalize(lower + difference / 2. + 180.))
    } else {
        normalize(lower + difference / 2. + if difference > 180. { 180. } else { 0. })
    };
    let mut metadata = geometry_metadata("geometry", "midpoint");
    metadata["antipodalPolicy"] = json!(policy);
    metadata["toleranceDegrees"] = json!(MIDPOINT_TOLERANCE);
    Ok(response(
        json!({"group":"geometry","action":"midpoint","longitude1":first,"longitude2":second,"longitude":longitude,"antipodal":antipodal,"antipodalPolicy":policy,"resolution":if antipodal{"lowerLongitude"}else{"unambiguous"}}),
        metadata,
    ))
}
fn between_query(r: BetweenRequest) -> Result<Value, Failure> {
    validate_positions(&r.positions).map_err(Failure::input)?;
    validate_aspect_rules(std::slice::from_ref(&r.rule)).map_err(Failure::input)?;
    let mode = r.motion_mode.as_deref().unwrap_or("none");
    if !["none", "relative", "fixedSecond"].contains(&mode) {
        return Err(Failure::input(
            "motionMode must be none, relative or fixedSecond",
        ));
    }
    let (first, second) = (&r.positions[0], &r.positions[1]);
    let delta = signed_difference(second.longitude, first.longitude);
    let separation = delta.abs();
    let offset = separation - r.rule.angle;
    let orb = offset.abs();
    let rate = match mode {
        "relative" => first.speed.zip(second.speed).map(|(a, b)| b - a),
        "fixedSecond" => first.speed.map(|a| -a),
        _ => None,
    };
    if rate.is_some_and(|r| !r.is_finite()) {
        return Err(Failure::input(
            "Relative speed overflows finite degrees/day",
        ));
    }
    let distance_rate = if separation <= 1e-12 || (separation - 180.).abs() <= 1e-12 {
        None
    } else {
        rate.map(|rate| delta.signum() * rate)
    };
    let applying = if orb <= 1e-12 {
        None
    } else {
        distance_rate.map(|rate| rate != 0. && offset.signum() * rate.signum() < 0.)
    };
    let mut metadata = geometry_metadata("aspects", "between");
    metadata["speedUnit"] = json!("degrees/day");
    metadata["motionMode"] = json!(mode);
    Ok(response(
        json!({"group":"aspects","action":"between","positions":placements(&r.positions,None),"rule":{"angle":r.rule.angle,"maxOrb":r.rule.max_orb},"relation":{"id":format!("{}:{}",first.id,second.id),"point1":first.id,"point2":second.id,"signedDelta":delta,"separation":separation},"aspect":{"point1":first.id,"point2":second.id,"angle":r.rule.angle,"orb":orb,"maxOrb":r.rule.max_orb,"matched":orb<=r.rule.max_orb,"applying":applying},"motionMode":mode,"distanceRate":distance_rate}),
        metadata,
    ))
}
fn locate_query(r: LocateRequest) -> Result<Value, Failure> {
    finite_longitudes(&[r.longitude])?;
    validate_houses(&Some(r.house_cusps.clone())).map_err(Failure::input)?;
    let cusps: Vec<f64> = r.house_cusps.into_iter().map(normalize).collect();
    let longitude = normalize(r.longitude);
    let number = house_for(longitude, Some(&cusps))
        .ok_or_else(|| Failure::calculation("Validated cusps did not contain the longitude"))?;
    let cusp = cusps[number - 1];
    let sign_index = (cusp / 30.).floor() as usize;
    let width = normalize(cusps[number % 12] - cusp);
    Ok(response(
        json!({"group":"houses","action":"locate","longitude":longitude,"houseCusps":cusps,"house":{"id":format!("H{number}"),"number":number,"longitude":cusp,"signIndex":sign_index,"sign":SIGNS[sign_index],"degreeInSign":cusp%30.,"widthDegrees":width},"boundaryRule":"cuspInclusiveNextCuspExclusive"}),
        geometry_metadata("houses", "locate"),
    ))
}

enum Selection {
    Houses(Vec<i32>),
    Points(Vec<String>),
}
fn inspect_query(
    group: &str,
    birth: natal::BirthInput,
    selection: Selection,
    rulership: Option<String>,
    aspect_preset: Option<String>,
    aspect_rules: natal::AspectSettings,
) -> Result<Value, Failure> {
    let rulership = rulership.as_deref().unwrap_or("traditional");
    if !["traditional", "modern"].contains(&rulership) {
        return Err(Failure::input("rulership must be traditional or modern"));
    }
    let preset = aspect_preset.as_deref().unwrap_or("major");
    if !["major", "extended"].contains(&preset)
        || (aspect_preset.is_some() && aspect_rules.provided)
    {
        return Err(Failure::input(
            "aspectPreset must be major or extended and cannot be combined with aspectRules",
        ));
    }
    let rules = if aspect_rules.provided {
        aspect_rules.rules
    } else if preset == "extended" {
        domains::extended_rules()
    } else {
        default_rules()
    };
    validate_aspect_rules(&rules).map_err(Failure::input)?;
    let mut unique = HashSet::new();
    match &selection {
        Selection::Houses(numbers) => {
            if numbers.is_empty()
                || numbers.len() > 12
                || numbers
                    .iter()
                    .any(|n| !(1..=12).contains(n) || !unique.insert(n.to_string()))
            {
                return Err(Failure::input(
                    "houseNumbers requires 1 to 12 unique integers in 1..12",
                ));
            }
        }
        Selection::Points(ids) => {
            let minimum = if group == "aspects" { 2 } else { 1 };
            if ids.len() < minimum
                || ids.len() > 26
                || ids
                    .iter()
                    .any(|id| !POINT_IDS.contains(&id.as_str()) || !unique.insert(id.clone()))
            {
                return Err(Failure::input(format!(
                    "pointIds requires {minimum} to 26 unique natal body, angle or H1..H12 IDs"
                )));
            }
        }
    }
    let system = natal::validate_birth(&birth).map_err(Failure::input)?;
    let (natal, mut calculation) =
        natal::compute_birth(&birth, system, &rules).map_err(Failure::calculation)?;
    let context = domains::build(natal, &rules, &[], rulership, &[])["context"].take();
    let all_points = context["points"].as_array().unwrap();
    let all_houses = context["houses"].as_array().unwrap();
    let mut primary = HashSet::new();
    let (point_ids, house_numbers) = match selection {
        Selection::Houses(numbers) => {
            for house in all_houses
                .iter()
                .filter(|h| numbers.contains(&(h["number"].as_i64().unwrap() as i32)))
            {
                primary.insert(house["id"].as_str().unwrap().to_string());
                primary.insert(house["rulerBodyId"].as_str().unwrap().to_string());
                for occupant in house["occupants"].as_array().unwrap() {
                    primary.insert(occupant["id"].as_str().unwrap().to_string());
                }
            }
            (vec![], numbers)
        }
        Selection::Points(ids) => {
            primary.extend(ids.iter().cloned());
            (ids, vec![])
        }
    };
    let house_ids: HashSet<String> = if group == "houses" {
        house_numbers.iter().map(|n| format!("H{n}")).collect()
    } else {
        primary
            .iter()
            .filter(|id| id.starts_with('H'))
            .cloned()
            .collect()
    };
    let houses: Vec<Value> = all_houses
        .iter()
        .filter(|h| house_ids.contains(h["id"].as_str().unwrap()))
        .cloned()
        .collect();
    let selected = |first: &str, second: &str| {
        if group == "aspects" {
            primary.contains(first) && primary.contains(second)
        } else {
            primary.contains(first) || primary.contains(second)
        }
    };
    let mut index_map = HashMap::new();
    let aspects: Vec<Value> = context["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| selected(a["point1"].as_str().unwrap(), a["point2"].as_str().unwrap()))
        .enumerate()
        .map(|(index, a)| {
            index_map.insert(a["index"].as_u64().unwrap(), index);
            let mut a = a.clone();
            a["index"] = json!(index);
            a
        })
        .collect();
    let relations: Vec<Value> = context["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| selected(r["point1"].as_str().unwrap(), r["point2"].as_str().unwrap()))
        .map(|r| {
            let mut r = r.clone();
            r["aspectIndexes"] = json!(r["aspectIndexes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| index_map[&i.as_u64().unwrap()])
                .collect::<Vec<_>>());
            r
        })
        .collect();
    let mut included = primary.clone();
    for relation in &relations {
        included.insert(relation["point1"].as_str().unwrap().to_string());
        included.insert(relation["point2"].as_str().unwrap().to_string());
    }
    for house in &houses {
        included.insert(house["rulerBodyId"].as_str().unwrap().to_string());
        for occupant in house["occupants"].as_array().unwrap() {
            included.insert(occupant["id"].as_str().unwrap().to_string());
        }
    }
    let points: Vec<Value> = all_points
        .iter()
        .filter(|p| included.contains(p["id"].as_str().unwrap()))
        .cloned()
        .collect();
    let primary_point_ids: Vec<&Value> = all_points
        .iter()
        .filter(|p| primary.contains(p["id"].as_str().unwrap()))
        .map(|p| &p["id"])
        .collect();
    let primary_house_ids: Vec<&Value> = houses.iter().map(|h| &h["id"]).collect();
    let policy = if group == "aspects" {
        "bothSelectedEndpoints"
    } else {
        "atLeastOneSelectedEndpoint"
    };
    calculation["sourceScope"] = calculation["scope"].clone();
    calculation["scope"] = json!("on-demand-query");
    calculation["computationalScope"] = json!("fullNatalThenSelection");
    calculation["group"] = json!(group);
    calculation["action"] = json!("inspect");
    calculation["rulership"] = json!(rulership);
    calculation["aspectPreset"] = json!(if aspect_rules.provided {
        "custom"
    } else {
        preset
    });
    calculation["selectionPolicy"] = json!(policy);
    let coverage = json!({"primaryPoints":primary.len(),"supportingPoints":points.len()-primary.len(),"selectedHouses":houses.len(),"pointPairs":relations.len(),"matchedAspects":aspects.len()});
    Ok(response(
        json!({"group":group,"action":"inspect","chartKind":"individualNatal","selection":{"pointIds":point_ids,"houseNumbers":house_numbers,"aspectSelection":policy},"primaryPointIds":primary_point_ids,"primaryHouseIds":primary_house_ids,"points":points,"houses":houses,"relations":relations,"aspects":aspects,"coverage":coverage}),
        calculation,
    ))
}

fn dispatch(raw: &str, header: Header) -> Result<Value, Failure> {
    if header.operation != "query" {
        return Err(Failure::input("Expected query operation"));
    }
    match (header.group.as_str(), header.action.as_str()) {
        ("geometry", "normalize") => normalize_query(parse(raw)?),
        ("geometry", "separation") => separation_query(parse(raw)?),
        ("geometry", "midpoint") => midpoint_query(parse(raw)?),
        ("aspects", "between") => between_query(parse(raw)?),
        ("houses", "locate") => locate_query(parse(raw)?),
        ("houses", "inspect") => {
            let r: HouseInspectRequest = parse(raw)?;
            inspect_query("houses", r.birth, Selection::Houses(r.house_numbers.map(|v| v.into_iter().map(|n| n.0).collect()).unwrap_or_else(|| (1..=12).collect())),r.rulership,r.aspect_preset,r.aspect_rules)
        }
        ("points" | "aspects", "inspect") => {
            let r: PointInspectRequest = parse(raw)?;
            inspect_query(&header.group,r.birth,Selection::Points(r.point_ids.unwrap_or_else(|| POINT_IDS.into_iter().map(String::from).collect())),r.rulership,r.aspect_preset,r.aspect_rules)
        }
        _ => Err(Failure::input("Query supports geometry.normalize/separation/midpoint, aspects.between/inspect, houses.locate/inspect and points.inspect")),
    }
}
pub(super) fn calculate_json(raw: &str) -> String {
    let result = parse(raw).and_then(|header| dispatch(raw, header));
    match result {
        Ok(result) => result.to_string(),
        Err(failure) => error_json(failure.code, &failure.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(request: Value) -> Value {
        serde_json::from_str(&crate::calculate_json(&request.to_string())).unwrap()
    }
    fn valid(request: Value) -> Value {
        let result = run(request);
        assert_eq!(result["errors"], json!([]), "{result}");
        result
    }
    fn birth() -> Value {
        json!({"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"houseSystem":"placidus"})
    }
    fn inspect(group: &str) -> Value {
        json!({"operation":"query","group":group,"action":"inspect","birth":birth()})
    }
    fn context(rulership: &str, rules: Value) -> Value {
        let mut natal = birth();
        natal["operation"] = json!("natalDomains");
        natal["rulership"] = json!(rulership);
        natal["domains"] = json!(["identity"]);
        natal["aspectRules"] = rules;
        valid(natal)["data"]["context"].clone()
    }
    fn closure(data: &Value) {
        let points: HashMap<&str, &Value> = data["points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p["id"].as_str().unwrap(), p))
            .collect();
        assert_eq!(points.len(), data["points"].as_array().unwrap().len());
        for id in data["primaryPointIds"].as_array().unwrap() {
            assert!(points.contains_key(id.as_str().unwrap()));
        }
        let aspects = data["aspects"].as_array().unwrap();
        for (i, a) in aspects.iter().enumerate() {
            assert_eq!(a["index"], json!(i));
            for key in ["point1", "point2"] {
                assert!(points.contains_key(a[key].as_str().unwrap()));
            }
        }
        for r in data["relations"].as_array().unwrap() {
            let first = points[r["point1"].as_str().unwrap()];
            let second = points[r["point2"].as_str().unwrap()];
            assert!(
                (r["signedDelta"].as_f64().unwrap()
                    - signed_difference(
                        second["longitude"].as_f64().unwrap(),
                        first["longitude"].as_f64().unwrap()
                    ))
                .abs()
                    < 1e-10
            );
            for i in r["aspectIndexes"].as_array().unwrap() {
                let a = &aspects[i.as_u64().unwrap() as usize];
                assert_eq!(a["point1"], r["point1"]);
                assert_eq!(a["point2"], r["point2"]);
            }
        }
        for h in data["houses"].as_array().unwrap() {
            assert!(points.contains_key(h["id"].as_str().unwrap()));
            assert_eq!(
                points[h["rulerBodyId"].as_str().unwrap()]["longitude"],
                h["rulerPlacement"]["longitude"]
            );
            for occupant in h["occupants"].as_array().unwrap() {
                assert_eq!(
                    points[occupant["id"].as_str().unwrap()]["longitude"],
                    occupant["longitude"]
                );
            }
        }
    }
    #[test]
    fn geometry_helpers_handle_zero_wrap_and_short_arc_direction() {
        for (raw, expected) in [(-1., 359_f64), (360., 0.), (-0., 0.), (1080., 0.)] {
            let result = valid(
                json!({"operation":"query","group":"geometry","action":"normalize","longitude":raw}),
            );
            assert_eq!(result["data"]["longitude"], expected);
            assert_eq!(
                result["data"]["signIndex"],
                json!((expected / 30.).floor() as usize)
            );
            assert_eq!(result["calculation"]["computationalScope"], "pureGeometry");
        }
        let result = valid(
            json!({"operation":"query","group":"geometry","action":"normalize","longitude":1e308}),
        );
        assert!((0. ..360.).contains(&result["data"]["longitude"].as_f64().unwrap()));
        for (a, b, delta) in [(350., 10., 20_f64), (10., 350., -20.), (0., 180., -180.)] {
            let result = valid(
                json!({"operation":"query","group":"geometry","action":"separation","longitude1":a,"longitude2":b}),
            );
            assert_eq!(result["data"]["signedDelta"], delta);
            assert_eq!(result["data"]["separation"], delta.abs());
        }
    }
    #[test]
    fn midpoint_ambiguity_is_explicit_and_swap_symmetric() {
        for (a, b, expected) in [(350., 10., 0.), (20., 100., 60.), (-1., 359., 359.)] {
            for (first, second) in [(a, b), (b, a)] {
                let result = valid(
                    json!({"operation":"query","group":"geometry","action":"midpoint","longitude1":first,"longitude2":second}),
                );
                assert_eq!(result["data"]["longitude"], expected);
                assert_eq!(result["data"]["antipodal"], false);
            }
        }
        let request = json!({"operation":"query","group":"geometry","action":"midpoint","longitude1":10.,"longitude2":190.});
        let failed = run(request.clone());
        assert_eq!(failed["errors"][0]["code"], "CALCULATION_FAILED");
        assert!(failed["data"].is_null());
        let mut resolved = request;
        resolved["antipodalPolicy"] = json!("lowerLongitude");
        assert_eq!(valid(resolved)["data"]["longitude"], 100.);
    }
    #[test]
    fn arbitrary_aspect_angle_preserves_explicit_motion_models() {
        let mut request = json!({"operation":"query","group":"aspects","action":"between","positions":[{"id":"moving","longitude":0.,"speed":2.},{"id":"second","longitude":37.125,"speed":4.}],"rule":{"angle":37.125,"maxOrb":0.}});
        let exact = valid(request.clone());
        assert_eq!(exact["data"]["aspect"]["matched"], true);
        assert_eq!(exact["data"]["aspect"]["angle"], 37.125);
        assert!(exact["data"]["aspect"]["applying"].is_null());
        request["positions"][1]["longitude"] = json!(38.125);
        request["motionMode"] = json!("relative");
        let relative = valid(request.clone());
        assert_eq!(relative["data"]["aspect"]["matched"], false);
        assert_eq!(relative["data"]["aspect"]["applying"], false);
        assert_eq!(relative["data"]["distanceRate"], 2.);
        request["motionMode"] = json!("fixedSecond");
        let fixed = valid(request.clone());
        assert_eq!(fixed["data"]["aspect"]["applying"], true);
        assert_eq!(fixed["data"]["distanceRate"], -2.);
        request["motionMode"] = json!("relative");
        request["positions"][1]["speed"] = Value::Null;
        assert!(valid(request.clone())["data"]["distanceRate"].is_null());
        request["positions"][1]["longitude"] = json!(180.);
        request["motionMode"] = json!("fixedSecond");
        assert!(valid(request.clone())["data"]["aspect"]["applying"].is_null());
        request["positions"][0]["speed"] = json!(0.);
        request["positions"][1]["longitude"] = json!(36.125);
        let stationary = valid(request);
        assert_eq!(stationary["data"]["distanceRate"], 0.);
        assert_eq!(stationary["data"]["aspect"]["applying"], false);
    }
    #[test]
    fn house_location_uses_cusp_inclusive_intervals_through_zero() {
        let cusps = vec![
            350., 20., 50., 80., 110., 140., 170., 200., 230., 260., 290., 320.,
        ];
        for (i, cusp) in cusps.iter().enumerate() {
            let result = valid(
                json!({"operation":"query","group":"houses","action":"locate","longitude":cusp,"houseCusps":cusps}),
            );
            assert_eq!(result["data"]["house"]["number"], json!(i + 1));
            assert_eq!(result["data"]["house"]["widthDegrees"], 30.);
        }
        let result = valid(
            json!({"operation":"query","group":"houses","action":"locate","longitude":0.,"houseCusps":cusps}),
        );
        assert_eq!(result["data"]["house"]["number"], 1);
        let failed = run(
            json!({"operation":"query","group":"houses","action":"locate","longitude":0.,"houseCusps":vec![0.;12]}),
        );
        assert_eq!(failed["errors"][0]["code"], "INVALID_INPUT");
    }
    #[test]
    fn point_and_aspect_inspection_preserve_facts_with_local_reference_closure() {
        let rules = json!([{"angle":37.125,"maxOrb":15.},{"angle":90.,"maxOrb":8.}]);
        let source = context("modern", rules.clone());
        for group in ["points", "aspects"] {
            let mut request = inspect(group);
            request["pointIds"] = json!(["sun", "moon", "H4"]);
            request["rulership"] = json!("modern");
            request["aspectRules"] = rules.clone();
            let result = valid(request);
            let data = &result["data"];
            closure(data);
            assert_eq!(
                result["calculation"]["computationalScope"],
                "fullNatalThenSelection"
            );
            assert_eq!(result["calculation"]["aspectPreset"], "custom");
            assert_eq!(data["primaryPointIds"], json!(["sun", "moon", "H4"]));
            assert_eq!(data["primaryHouseIds"], json!(["H4"]));
            for point in data["points"].as_array().unwrap() {
                assert_eq!(
                    point,
                    source["points"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|p| p["id"] == point["id"])
                        .unwrap()
                );
            }
            assert_eq!(
                data["relations"].as_array().unwrap().len(),
                if group == "aspects" { 3 } else { 72 }
            );
            for relation in data["relations"].as_array().unwrap() {
                let first = ["sun", "moon", "H4"].contains(&relation["point1"].as_str().unwrap());
                let second = ["sun", "moon", "H4"].contains(&relation["point2"].as_str().unwrap());
                assert!(if group == "aspects" {
                    first && second
                } else {
                    first || second
                });
            }
        }
    }
    #[test]
    fn house_inspection_focuses_cusps_occupants_and_rulers_without_reports() {
        let source = context("traditional", json!([]));
        let mut request = inspect("houses");
        request["houseNumbers"] = json!([10, 4]);
        request["aspectRules"] = json!([]);
        let result = valid(request);
        let data = &result["data"];
        closure(data);
        assert_eq!(data["primaryHouseIds"], json!(["H4", "H10"]));
        assert_eq!(data["selection"]["houseNumbers"], json!([10, 4]));
        assert_eq!(data["aspects"], json!([]));
        let primary: HashSet<_> = data["primaryPointIds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect();
        for house in data["houses"].as_array().unwrap() {
            assert_eq!(
                house,
                source["houses"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|h| h["id"] == house["id"])
                    .unwrap()
            );
            assert!(primary.contains(house["id"].as_str().unwrap()));
            assert!(primary.contains(house["rulerBodyId"].as_str().unwrap()));
            for body in house["occupants"].as_array().unwrap() {
                assert!(primary.contains(body["id"].as_str().unwrap()));
            }
        }
        assert!(data.get("advanced").is_none());
        assert!(data.get("domains").is_none());
    }
    #[test]
    fn selectors_and_birth_integers_use_exact_decimal_normalization() {
        let raw = r#"{"operation":"query","group":"houses","action":"inspect","birth":{"utc":{"year":2e3,"month":1.0,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297}},"houseNumbers":[1.0,2e0],"aspectRules":[]}"#;
        let actual: Value = serde_json::from_str(&crate::calculate_json(raw)).unwrap();
        assert_eq!(actual["errors"], json!([]));
        let canonical = raw
            .replace("2e3", "2000")
            .replace("1.0", "1")
            .replace("2e0", "2");
        assert_eq!(
            actual,
            serde_json::from_str::<Value>(&crate::calculate_json(&canonical)).unwrap()
        );
        for invalid in [
            raw.replace("1.0,2e0", "1.0000000000000001,2"),
            raw.replace("2e3", "2000.0000000000000001"),
        ] {
            let failed: Value = serde_json::from_str(&crate::calculate_json(&invalid)).unwrap();
            assert_eq!(failed["errors"][0]["code"], "INVALID_INPUT");
            assert!(failed["data"].is_null());
        }
        for group in ["points", "aspects", "houses"] {
            let mut request = inspect(group);
            request["aspectRules"] = json!([]);
            let result = valid(request);
            assert_eq!(result["data"]["points"].as_array().unwrap().len(), 26);
            // All houses focus their 12 cusps and 10 occupants; four angles are
            // supporting endpoints, so their six mutual pairs are not selected.
            assert_eq!(
                result["data"]["relations"].as_array().unwrap().len(),
                if group == "houses" { 319 } else { 325 }
            );
        }
    }
    #[test]
    fn invalid_queries_reject_null_unknown_fields_and_validate_before_provider() {
        let mut point = inspect("points");
        point["birth"]["location"]["latitude"] = json!(80.);
        point["pointIds"] = json!(["unknown"]);
        let failed = run(point.clone());
        assert_eq!(failed["errors"][0]["code"], "INVALID_INPUT");
        assert!(failed["data"].is_null());
        point["pointIds"] = json!(["sun"]);
        assert_eq!(run(point)["errors"][0]["code"], "CALCULATION_FAILED");
        for (group, key, value) in [
            ("points", "pointIds", Value::Null),
            ("points", "pointIds", json!([])),
            ("aspects", "pointIds", json!(["sun"])),
            ("houses", "houseNumbers", json!([1, 1])),
            ("houses", "houseNumbers", Value::Null),
            ("points", "rulership", Value::Null),
            ("points", "aspectPreset", Value::Null),
            ("points", "aspectRules", Value::Null),
            ("points", "unexpected", json!(true)),
        ] {
            let mut request = inspect(group);
            request[key] = value;
            let failed = run(request);
            assert_eq!(failed["errors"][0]["code"], "INVALID_INPUT", "{failed}");
            assert!(failed["data"].is_null());
        }
        let mut conflicting = inspect("aspects");
        conflicting["aspectPreset"] = json!("major");
        conflicting["aspectRules"] = json!([]);
        assert_eq!(run(conflicting)["errors"][0]["code"], "INVALID_INPUT");
        for (key, value) in [
            ("motionMode", Value::Null),
            ("motionMode", json!("unknown")),
            ("rule", json!({"angle":181.,"maxOrb":1.})),
        ] {
            let mut request = json!({"operation":"query","group":"aspects","action":"between","positions":[{"id":"a","longitude":0.},{"id":"b","longitude":1.}],"rule":{"angle":1.,"maxOrb":0.}});
            request[key] = value;
            assert_eq!(run(request)["errors"][0]["code"], "INVALID_INPUT");
        }
        let overflow = json!({"operation":"query","group":"aspects","action":"between","positions":[{"id":"a","longitude":0.,"speed":-f64::MAX},{"id":"b","longitude":1.,"speed":f64::MAX}],"rule":{"angle":1.,"maxOrb":0.},"motionMode":"relative"});
        assert_eq!(run(overflow)["errors"][0]["code"], "INVALID_INPUT");
    }
}
