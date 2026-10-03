//! One natal, moving skies, root-solved events and reference-based forecast views.
//! Message contexts contain evidence only; this module does not write predictions.
use super::*;
use serde_json::value::RawValue;
use std::collections::{BTreeMap, BTreeSet};

fn yes() -> bool {
    true
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ForecastRequest {
    operation: String,
    birth: Box<RawValue>,
    period: events::Period,
    #[serde(default, deserialize_with = "natal::present")]
    bodies: Option<Vec<String>>,
    #[serde(default, deserialize_with = "natal::present")]
    event_types: Option<Vec<String>>,
    #[serde(default = "yes")]
    include_natal_transits: bool,
    #[serde(default)]
    aspect_rules: natal::AspectSettings,
    #[serde(default, deserialize_with = "natal::present")]
    aspect_preset: Option<String>,
    #[serde(default, deserialize_with = "natal::present")]
    rulership: Option<String>,
    #[serde(default, deserialize_with = "natal::present")]
    domains: Option<Vec<String>>,
    #[serde(default, deserialize_with = "natal::present")]
    custom_profiles: Option<Vec<profiles::Profile>>,
}
fn array(v: &Value) -> &[Value] {
    v.as_array().unwrap()
}
fn qualified(v: &Value) -> BTreeSet<String> {
    array(v)
        .iter()
        .map(|s| format!("N:{}", s.as_str().unwrap()))
        .collect()
}
fn point<'a>(context: &'a Value, id: &str) -> &'a Value {
    let id = id.strip_prefix("N:").unwrap_or(id);
    array(&context["points"])
        .iter()
        .find(|p| p["id"] == id)
        .unwrap()
}
fn overlays(positions: &[Value], natal: &Value, context: &Value) -> Vec<Value> {
    let cusps: Vec<f64> = array(&natal["houseCusps"])
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    positions.iter().map(|p| {
        let n = house_for(p["longitude"].as_f64().unwrap(), Some(&cusps)).unwrap();
        let h = &context["houses"][n-1];
        json!({"transitPointId":format!("T:{}",p["id"].as_str().unwrap()),"natalHouseId":format!("N:H{n}"),"natalHouseNumber":n,"natalRulerPointId":format!("N:{}",h["rulerBodyId"].as_str().unwrap()),"natalOccupantPointIds":array(&h["occupants"]).iter().map(|id|format!("N:{}",id["id"].as_str().unwrap())).collect::<Vec<_>>()})
    }).collect()
}
/// Birth velocities never enter this comparison: every natal longitude is fixed.
fn contacts(
    positions: &[Value],
    context: &Value,
    rules: &[AspectRule],
    all_pairs: bool,
) -> (Vec<Value>, Vec<Value>) {
    let mut relations = Vec::new();
    let mut matched = Vec::new();
    for p in positions {
        let p_id = format!("T:{}", p["id"].as_str().unwrap());
        for target in array(&context["points"]) {
            let n_id = format!("N:{}", target["id"].as_str().unwrap());
            let id = format!("{p_id}|{n_id}");
            let delta = signed_difference(
                p["longitude"].as_f64().unwrap(),
                target["longitude"].as_f64().unwrap(),
            );
            let separation = delta.abs();
            let speed = p["speed"].as_f64().unwrap();
            let mut indexes = Vec::new();
            for rule in rules {
                let orb = (separation - rule.angle).abs();
                if orb <= rule.max_orb {
                    let index = matched.len();
                    indexes.push(index);
                    let applying = if orb <= 1e-6 || speed.abs() <= 1e-12 {
                        None
                    } else {
                        Some((separation - rule.angle) * speed * delta.signum() < 0.)
                    };
                    matched.push(json!({"index":index,"relationId":id,"transitPointId":p_id,"natalPointId":n_id,"angle":rule.angle,"separation":separation,"orb":orb,"maxOrb":rule.max_orb,"applying":applying}));
                }
            }
            if all_pairs {
                relations.push(json!({"id":id,"transitPointId":p_id,"natalPointId":n_id,"signedDelta":delta,"separation":separation,"aspectIndexes":indexes}));
            }
        }
    }
    (relations, matched)
}
fn personal_impact(event: &Value, natal: &Value, context: &Value, rules: &[AspectRule]) -> Value {
    let positions = array(&event["positions"]);
    let house_overlays = overlays(positions, natal, context);
    let (_, matched) = contacts(positions, context, rules, false);
    let primary = if event["type"] == "natalTransit" {
        event["details"]["targetPointId"].as_str()
    } else {
        None
    };
    let mut points: BTreeSet<String> = matched
        .iter()
        .map(|c| c["natalPointId"].as_str().unwrap().into())
        .collect();
    if let Some(p) = primary {
        points.insert(p.into());
    }
    let mut houses: BTreeSet<String> = house_overlays
        .iter()
        .map(|o| o["natalHouseId"].as_str().unwrap().into())
        .collect();
    let mut ruler_links = Vec::new();
    let cusps: Vec<f64> = array(&natal["houseCusps"])
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    for id in &points {
        let p = point(context, id);
        let n = if p["kind"] == "houseCusp" {
            p["id"].as_str().unwrap()[1..].parse::<usize>().unwrap()
        } else {
            house_for(p["longitude"].as_f64().unwrap(), Some(&cusps)).unwrap()
        };
        houses.insert(format!("N:H{n}"));
        if p["kind"] == "body" {
            let ruled: Vec<String> = array(&context["houses"])
                .iter()
                .filter(|h| h["rulerBodyId"] == p["id"])
                .map(|h| format!("N:{}", h["id"].as_str().unwrap()))
                .collect();
            houses.extend(ruled.iter().cloned());
            if !ruled.is_empty() {
                ruler_links.push(json!({"natalPointId":id,"natalHouseIds":ruled}));
            }
        }
    }
    json!({"primaryNatalPointId":primary,"primaryAngle":if primary.is_some(){event["details"]["angle"].clone()}else{Value::Null},"houseOverlays":house_overlays,"contacts":matched,"affectedNatalPointIds":points,"affectedNatalHouseIds":houses,"rulerLinks":ruler_links})
}
const SLOW: [&str; 6] = ["mars", "jupiter", "saturn", "uranus", "neptune", "pluto"];
fn highlight_reasons(event: &Value) -> Vec<&'static str> {
    let ids: Vec<&str> = array(&event["bodyIds"])
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    match event["type"].as_str().unwrap() {
        "station" => vec!["station"],
        "solarEclipse" | "lunarEclipse" => vec!["eclipse"],
        "lunarPhase"
            if event["details"]["phase"] == "newMoon"
                || event["details"]["phase"] == "fullMoon" =>
        {
            vec!["newOrFullMoon"]
        }
        "ingress" if ids.iter().any(|id| SLOW.contains(id)) => vec!["slowBodyIngress"],
        "planetaryAspect" if !ids.contains(&"moon") && ids.iter().any(|id| SLOW.contains(id)) => {
            vec!["slowBodyPlanetaryAspect"]
        }
        "natalTransit" if !ids.contains(&"moon") => vec!["nonLunarNatalTransit"],
        _ => Vec::new(),
    }
}
struct Selection {
    points: BTreeSet<String>,
    houses: BTreeSet<String>,
}
fn selection(natal: &Value, context: &Value, profile: &profiles::Profile) -> Selection {
    let selected = domains::select_context(natal, context, profile);
    Selection {
        points: qualified(&selected["pointIds"]),
        houses: profile.houses.iter().map(|n| format!("N:H{n}")).collect(),
    }
}
fn reason_set(event: &Value, selected: &Selection) -> Vec<Value> {
    let impact = &event["personalImpact"];
    let matching: Vec<&Value> = array(&impact["contacts"])
        .iter()
        .filter(|c| {
            selected
                .points
                .contains(c["natalPointId"].as_str().unwrap())
        })
        .collect();
    let mut reasons = Vec::new();
    if !matching.is_empty() {
        let ids: BTreeSet<&str> = matching
            .iter()
            .map(|c| c["natalPointId"].as_str().unwrap())
            .collect();
        reasons.push(json!({"code":"natalContact","natalPointIds":ids,"natalHouseIds":[],"aspectIndexes":matching.iter().map(|c|c["index"].as_u64().unwrap()).collect::<Vec<_>>()}));
    }
    if let Some(id) = impact["primaryNatalPointId"].as_str() {
        if selected.points.contains(id) {
            reasons.push(json!({"code":"exactNatalTarget","natalPointIds":[id],"natalHouseIds":[],"aspectIndexes":[]}));
        }
    }
    let houses: BTreeSet<&str> = array(&impact["houseOverlays"])
        .iter()
        .filter_map(|o| o["natalHouseId"].as_str())
        .filter(|h| selected.houses.contains(*h))
        .collect();
    if !houses.is_empty() {
        reasons.push(json!({"code":"transitThroughFocusHouse","natalPointIds":[],"natalHouseIds":houses,"aspectIndexes":[]}));
    }
    reasons
}
fn snapshot_refs(snapshot: &Value, selected: &Selection) -> (Vec<u64>, Vec<String>) {
    let aspects = array(&snapshot["aspects"])
        .iter()
        .filter(|a| {
            selected
                .points
                .contains(a["natalPointId"].as_str().unwrap())
        })
        .map(|a| a["index"].as_u64().unwrap())
        .collect();
    let overlays = array(&snapshot["houseOverlays"])
        .iter()
        .filter(|o| {
            selected
                .houses
                .contains(o["natalHouseId"].as_str().unwrap())
        })
        .map(|o| o["transitPointId"].as_str().unwrap().into())
        .collect();
    (aspects, overlays)
}
fn build_view(
    natal: &Value,
    context: &Value,
    snapshot: &Value,
    events: &[Value],
    profile: &profiles::Profile,
    origin: &str,
    kind: &str,
) -> Value {
    let primary = selection(natal, context, profile);
    let mut union = Selection {
        points: primary.points.clone(),
        houses: primary.houses.clone(),
    };
    let section_selections: Vec<_> = profile
        .sections
        .iter()
        .map(|s| {
            let p = profiles::Profile {
                id: s.id.clone(),
                version: profile.version.clone(),
                houses: s.houses.clone(),
                bodies: s.bodies.clone(),
                angles: s.angles.clone(),
                sections: vec![],
            };
            (s.id.clone(), selection(natal, context, &p))
        })
        .collect();
    for (_, s) in &section_selections {
        union.points.extend(s.points.iter().cloned());
        union.houses.extend(s.houses.iter().cloned());
    }
    let mut sections = Vec::new();
    for (id, selected) in &section_selections {
        let refs: Vec<&Value> = events
            .iter()
            .filter(|e| !reason_set(e, selected).is_empty())
            .collect();
        let (aspects, overlays) = snapshot_refs(snapshot, selected);
        sections.push(json!({"id":id,"natalPointIds":selected.points,"natalHouseIds":selected.houses,"snapshotAspectIndexes":aspects,"snapshotOverlayPointIds":overlays,"eventIds":refs.iter().map(|e|e["id"].clone()).collect::<Vec<_>>(),"highlightEventIds":refs.iter().filter(|e|!array(&e["highlightReasons"]).is_empty()).map(|e|e["id"].clone()).collect::<Vec<_>>()}));
    }
    let mut refs = Vec::new();
    for e in events {
        let reasons = reason_set(e, &union);
        if reasons.is_empty() {
            continue;
        }
        refs.push(json!({"eventId":e["id"],"primaryContact":!reason_set(e,&primary).is_empty(),"sectionIds":section_selections.iter().filter(|(_,s)|!reason_set(e,s).is_empty()).map(|(id,_)|id).collect::<Vec<_>>(),"reasons":reasons}));
    }
    let event_ids: Vec<Value> = refs.iter().map(|r| r["eventId"].clone()).collect();
    let ids: BTreeSet<&str> = event_ids.iter().map(|v| v.as_str().unwrap()).collect();
    let highlights: Vec<Value> = events
        .iter()
        .filter(|e| {
            ids.contains(e["id"].as_str().unwrap()) && !array(&e["highlightReasons"]).is_empty()
        })
        .map(|e| e["id"].clone())
        .collect();
    let (aspects, overlays) = snapshot_refs(snapshot, &union);
    let message_kind = match kind {
        "day" => "dailyMessage",
        "month" => "monthlyOverview",
        _ => "yearlyOverview",
    };
    json!({"id":profile.id,"origin":origin,"profileVersion":profile.version,"definition":profile,"primaryNatalPointIds":primary.points,"primaryNatalHouseIds":primary.houses,"natalPointIds":union.points,"natalHouseIds":union.houses,"snapshotAspectIndexes":aspects,"snapshotOverlayPointIds":overlays,"eventReferences":refs,"highlightEventIds":highlights,"sections":sections,"messageContext":{"kind":message_kind,"snapshotAspectIndexes":aspects,"eventIds":event_ids,"highlightEventIds":highlights,"natalHouseIds":union.houses,"narrative":null}})
}
fn overview(period: &Value, events: &[Value], views: &Value, custom: &Value) -> Value {
    let mut types: BTreeMap<&str, usize> = [
        "ingress",
        "station",
        "lunarPhase",
        "planetaryAspect",
        "natalTransit",
        "solarEclipse",
        "lunarEclipse",
    ]
    .map(|s| (s, 0))
    .into_iter()
    .collect();
    let mut houses: BTreeMap<String, usize> = (1..=12).map(|n| (format!("N:H{n}"), 0)).collect();
    let mut days: BTreeMap<String, (Vec<Value>, Vec<Value>)> = BTreeMap::new();
    let year = period["year"].as_i64().unwrap();
    let first = if period["kind"] == "year" {
        1
    } else {
        period["month"].as_i64().unwrap()
    };
    let last = if period["kind"] == "year" { 12 } else { first };
    let mut months: BTreeMap<i64, (Vec<Value>, Vec<Value>)> =
        (first..=last).map(|m| (m, (vec![], vec![]))).collect();
    let mut highlights = Vec::new();
    for e in events {
        *types.get_mut(e["type"].as_str().unwrap()).unwrap() += 1;
        for h in array(&e["personalImpact"]["affectedNatalHouseIds"]) {
            *houses.get_mut(h.as_str().unwrap()).unwrap() += 1;
        }
        let local = &e["local"];
        let key = format!(
            "{:04}-{:02}-{:02}",
            local["year"].as_i64().unwrap(),
            local["month"].as_i64().unwrap(),
            local["day"].as_i64().unwrap()
        );
        let day = days.entry(key).or_default();
        day.0.push(e["id"].clone());
        let month = months.get_mut(&local["month"].as_i64().unwrap()).unwrap();
        month.0.push(e["id"].clone());
        if !array(&e["highlightReasons"]).is_empty() {
            highlights.push(e["id"].clone());
            day.1.push(e["id"].clone());
            month.1.push(e["id"].clone());
        }
    }
    let counts: BTreeMap<&str, usize> = views
        .as_object()
        .unwrap()
        .iter()
        .chain(custom.as_object().unwrap())
        .map(|(id, v)| (id.as_str(), array(&v["eventReferences"]).len()))
        .collect();
    json!({"eventCount":events.len(),"byType":types,"natalHouseEventCounts":houses,"domainEventCounts":counts,"highlightEventIds":highlights,"days":days.into_iter().map(|(date,(ids,highlights))|json!({"date":date,"eventIds":ids,"highlightEventIds":highlights})).collect::<Vec<_>>(),"months":months.into_iter().map(|(month,(ids,highlights))|json!({"year":year,"month":month,"eventIds":ids,"highlightEventIds":highlights})).collect::<Vec<_>>()})
}
pub(super) fn calculate_json(raw: &str) -> String {
    let r: ForecastRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &e.to_string()),
    };
    if r.operation != "forecast" {
        return error_json("INVALID_INPUT", "Expected forecast operation");
    }
    let bounds = match r.period.validate() {
        Ok(v) => v,
        Err(e) => return error_json("INVALID_INPUT", &e),
    };
    let preset = r.aspect_preset.as_deref().unwrap_or("major");
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
    let custom = r.custom_profiles.as_deref().unwrap_or(&[]);
    let rulership = r.rulership.as_deref().unwrap_or("traditional");
    if r.custom_profiles.is_some() && custom.is_empty() {
        return error_json(
            "INVALID_INPUT",
            "customProfiles must contain 1 to 8 profiles when provided",
        );
    }
    if let Err(e) = profiles::validate_selection(&names, custom, rulership) {
        return error_json("INVALID_INPUT", &e);
    }
    let options = events::ScanOptions {
        bodies: r
            .bodies
            .clone()
            .unwrap_or_else(|| events::BODY_IDS.into_iter().map(String::from).collect()),
        event_types: r.event_types.clone().unwrap_or_else(|| {
            [
                "ingress",
                "station",
                "lunarPhase",
                "planetaryAspect",
                "solarEclipse",
                "lunarEclipse",
            ]
            .into_iter()
            .map(String::from)
            .collect()
        }),
        rules: rules.clone(),
    };
    if let Err(e) = options.validate() {
        return error_json("INVALID_INPUT", &e);
    }
    let birth: natal::BirthInput = match serde_json::from_str(r.birth.get()) {
        Ok(v) => v,
        Err(e) => return error_json("INVALID_INPUT", &e.to_string()),
    };
    let system = match natal::validate_birth(&birth) {
        Ok(v) => v,
        Err(e) => return error_json("INVALID_INPUT", &e),
    };
    let (natal, calculation) = match natal::compute_birth(&birth, system, &rules) {
        Ok(v) => v,
        Err(e) => return error_json("CALCULATION_FAILED", &e),
    };
    let shared = domains::build(natal.clone(), &rules, &[], rulership, &[]);
    let context = &shared["context"];
    let targets: Vec<events::Target> = if r.include_natal_transits {
        array(&context["points"])
            .iter()
            .map(|p| events::Target {
                id: format!("N:{}", p["id"].as_str().unwrap()),
                longitude: p["longitude"].as_f64().unwrap(),
            })
            .collect()
    } else {
        vec![]
    };
    let mut scanned = match events::scan(&bounds, &options, &targets) {
        Ok(v) => v,
        Err(e) => return error_json("CALCULATION_FAILED", &e),
    };
    let (relations, aspects) =
        contacts(array(&scanned.snapshot["positions"]), context, &rules, true);
    let house_overlays = overlays(array(&scanned.snapshot["positions"]), &natal, context);
    for p in scanned.snapshot["positions"].as_array_mut().unwrap() {
        p["pointId"] = json!(format!("T:{}", p["id"].as_str().unwrap()));
    }
    scanned.snapshot["relations"] = json!(relations);
    scanned.snapshot["aspects"] = json!(aspects);
    scanned.snapshot["houseOverlays"] = json!(house_overlays);
    for e in &mut scanned.events {
        e["personalImpact"] = personal_impact(e, &natal, context, &rules);
        e["highlightReasons"] = json!(highlight_reasons(e));
    }
    let mut views = serde_json::Map::new();
    let mut customs = serde_json::Map::new();
    for name in &names {
        views.insert(
            name.clone(),
            build_view(
                &natal,
                context,
                &scanned.snapshot,
                &scanned.events,
                &profiles::builtin(name),
                "builtin",
                scanned.period["kind"].as_str().unwrap(),
            ),
        );
    }
    for profile in custom {
        customs.insert(
            profile.id.clone(),
            build_view(
                &natal,
                context,
                &scanned.snapshot,
                &scanned.events,
                profile,
                "custom",
                scanned.period["kind"].as_str().unwrap(),
            ),
        );
    }
    let views = Value::Object(views);
    let customs = Value::Object(customs);
    let summary = overview(&scanned.period, &scanned.events, &views, &customs);
    json!({"schemaVersion":"1.0","engineVersion":VERSION,"calculation":{"scope":"individual-forecast-data","chartKind":"individualForecast","provider":"swiss-ephemeris","providerVersion":"2.10.03","ephemeris":"moshier","zodiac":"tropical","coordinates":"geocentric","positionType":"apparent","referenceFrame":"ecliptic-of-date","calendar":"gregorian","inputTimeScale":"UTC","planetTimeScale":"TT","searchTimeScale":"UT1","timeModel":"Swiss built-in leap seconds and Delta T; pre-1972 civil input treated as UT1","angleUnit":"degrees","speedUnit":"degrees/day","distanceUnit":"AU","bodyIds":options.bodies,"eventTypes":options.event_types,"includeNatalTransits":r.include_natal_transits,"aspectPreset":if r.aspect_rules.provided{"custom"}else{preset},"aspectRules":rules.iter().map(|v|json!({"angle":v.angle,"maxOrb":v.max_orb})).collect::<Vec<_>>(),"rulership":rulership,"domainProfile":{"id":"sevenmlabs-domain-selection","version":"2.0"},"forecastProfile":{"id":"sevenmlabs-individual-forecast","version":"1.0"},"customProfileCount":custom.len(),"transitTargetMotion":"fixedNatalLongitudes","snapshotPolicy":"periodMidpoint","highlightPolicy":{"id":"sevenmlabs-calendar-highlights","version":"1.0","slowBodyIds":SLOW},"search":scanned.metadata},"data":{"chartKind":"individualForecast","subjectCount":1,"subject":{"id":"N","natal":natal,"context":context,"calculation":calculation},"period":scanned.period,"snapshot":scanned.snapshot,"events":scanned.events,"domains":views,"customDomains":customs,"profileCatalog":shared["profileCatalog"],"overview":summary},"warnings":["Moshier analytical ephemeris with the provider's built-in time model; no live Earth-orientation data.","Forecast/message payloads contain geometry and evidence, not narrative, scores or guaranteed outcomes. Highlights follow the declared calendar selection policy.","Civil periods use a fixed UTC offset supplied by the caller, not historical timezone or daylight-saving conversion. Transit targets are fixed tropical natal longitudes."],"errors":[]}).to_string()
}
