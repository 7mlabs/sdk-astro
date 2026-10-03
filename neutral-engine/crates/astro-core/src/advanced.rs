//! Versioned, reproducible facts for an individual natal report; no interpretation.
use super::*;
use std::collections::BTreeMap;

const TRADITIONAL: [&str; 12] = [
    "mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars", "jupiter", "saturn",
    "saturn", "jupiter",
];
const ELEMENTS: [&str; 4] = ["fire", "earth", "air", "water"];
const MODALITIES: [&str; 3] = ["cardinal", "fixed", "mutable"];
const POLARITIES: [&str; 2] = ["positive", "negative"];
const HOUSE_TYPES: [&str; 3] = ["angular", "succedent", "cadent"];
const BOUNDARY_TOLERANCE: f64 = 1e-12;

fn dignity(id: &str, sign: usize) -> Value {
    let (domiciles, exaltation): (&[usize], Option<usize>) = match id {
        "sun" => (&[4], Some(0)),
        "moon" => (&[3], Some(1)),
        "mercury" => (&[2, 5], Some(5)),
        "venus" => (&[1, 6], Some(11)),
        "mars" => (&[0, 7], Some(9)),
        "jupiter" => (&[8, 11], Some(3)),
        "saturn" => (&[9, 10], Some(6)),
        _ => (&[], None),
    };
    let supported = exaltation.is_some();
    json!({"system":"traditional","supported":supported,
        "domicile":supported.then(|| domiciles.contains(&sign)),
        "detriment":supported.then(|| domiciles.iter().any(|s| (s + 6) % 12 == sign)),
        "exaltation":exaltation.map(|s| s == sign),
        "fall":exaltation.map(|s| (s + 6) % 12 == sign)})
}

fn solar_condition(id: &str, longitude: f64, sun: f64) -> Value {
    if id == "sun" {
        return json!({"separation":null,"condition":"notApplicable"});
    }
    let separation = signed_difference(longitude, sun).abs();
    let condition = if separation <= 17. / 60. + BOUNDARY_TOLERANCE {
        "cazimi"
    } else if separation <= 8.5 + BOUNDARY_TOLERANCE {
        "combust"
    } else if separation <= 17. + BOUNDARY_TOLERANCE {
        "underSunbeams"
    } else {
        "free"
    };
    json!({"separation":separation,"condition":condition})
}

fn distribution(states: &[Value], field: &str, groups: &[&str]) -> Vec<Value> {
    groups
        .iter()
        .map(|group| {
            let ids: Vec<&Value> = states
                .iter()
                .filter(|s| s[field] == *group)
                .map(|s| &s["bodyId"])
                .collect();
            json!({"id":group,"count":ids.len(),"bodyIds":ids})
        })
        .collect()
}

fn chains(bodies: &[Value], rulers: &[&str; 12]) -> Vec<Value> {
    bodies
        .iter()
        .map(|body| {
            let body_id = body["id"].as_str().unwrap();
            let mut path = vec![body_id];
            loop {
                let current = *path.last().unwrap();
                let placement = bodies.iter().find(|p| p["id"] == current).unwrap();
                let next = rulers[placement["signIndex"].as_u64().unwrap() as usize];
                if next == current {
                    return json!({"id":format!("dispositor:{body_id}"),"bodyId":body_id,
                        "path":path,"termination":"selfRuler","terminalBodyId":current,"cycleBodyIds":[]});
                }
                if let Some(start) = path.iter().position(|id| *id == next) {
                    return json!({"id":format!("dispositor:{body_id}"),"bodyId":body_id,
                        "path":path,"termination":"cycle","terminalBodyId":null,"cycleBodyIds":path[start..]});
                }
                path.push(next);
            }
        })
        .collect()
}

fn receptions(bodies: &[Value], rulers: &[&str; 12], rulership: &str) -> Vec<Value> {
    let mut output = Vec::new();
    for (i, first) in bodies.iter().enumerate() {
        for second in &bodies[i + 1..] {
            let first_id = first["id"].as_str().unwrap();
            let second_id = second["id"].as_str().unwrap();
            if rulers[first["signIndex"].as_u64().unwrap() as usize] == second_id
                && rulers[second["signIndex"].as_u64().unwrap() as usize] == first_id
            {
                let mut ids = [first_id, second_id];
                ids.sort();
                output.push(json!({"id":format!("reception:{}:{}",ids[0],ids[1]),
                    "bodyIds":ids,"type":"mutualDomicile","rulership":rulership}));
            }
        }
    }
    output
}

fn patterns(bodies: &[Value], aspects: &[Value]) -> Vec<Value> {
    // One canonical edge for each pair and configured angle, retaining its global index.
    let ids: Vec<&str> = bodies.iter().map(|p| p["id"].as_str().unwrap()).collect();
    let mut edges = BTreeMap::new();
    for (fallback, aspect) in aspects.iter().enumerate() {
        let Some(first) = ids.iter().position(|id| aspect["point1"] == *id) else {
            continue;
        };
        let Some(second) = ids.iter().position(|id| aspect["point2"] == *id) else {
            continue;
        };
        let angle = aspect["angle"].as_f64().unwrap();
        if first == second || ![60., 90., 120., 150., 180.].contains(&angle) {
            continue;
        }
        let key = (first.min(second), first.max(second), angle as u16);
        let index = aspect["index"].as_u64().unwrap_or(fallback as u64) as usize;
        edges
            .entry(key)
            .and_modify(|stored: &mut usize| *stored = (*stored).min(index))
            .or_insert(index);
    }
    let edge = |first: usize, second: usize, angle: u16| {
        edges
            .get(&(first.min(second), first.max(second), angle))
            .copied()
    };
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    let mut emit = |kind: &str, members: &[usize], indexes: &[usize], apex: Option<usize>| {
        let mut body_ids: Vec<&str> = members.iter().map(|i| ids[*i]).collect();
        body_ids.sort();
        let apex_id = apex.map(|i| ids[i]);
        let id = format!(
            "pattern:{kind}:{}:{}",
            body_ids.join(":"),
            apex_id.unwrap_or("none")
        );
        if seen.insert(id.clone()) {
            let mut aspect_indexes = indexes.to_vec();
            aspect_indexes.sort_unstable();
            aspect_indexes.dedup();
            output.push(json!({"id":id,"type":kind,"bodyIds":body_ids,
                "aspectIndexes":aspect_indexes,"apexBodyId":apex_id}));
        }
    };
    for a in 0..ids.len() {
        for b in a + 1..ids.len() {
            for c in b + 1..ids.len() {
                let members = [a, b, c];
                if let (Some(ab), Some(ac), Some(bc)) =
                    (edge(a, b, 120), edge(a, c, 120), edge(b, c, 120))
                {
                    emit("grandTrine", &members, &[ab, ac, bc], None);
                    for additional in 0..ids.len() {
                        if members.contains(&additional) {
                            continue;
                        }
                        for (opposed, other1, other2) in [(a, b, c), (b, a, c), (c, a, b)] {
                            if let (Some(opposition), Some(sextile1), Some(sextile2)) = (
                                edge(additional, opposed, 180),
                                edge(additional, other1, 60),
                                edge(additional, other2, 60),
                            ) {
                                emit(
                                    "kite",
                                    &[a, b, c, additional],
                                    &[ab, ac, bc, opposition, sextile1, sextile2],
                                    Some(additional),
                                );
                            }
                        }
                    }
                }
                for (apex, other1, other2) in [(a, b, c), (b, a, c), (c, a, b)] {
                    if let (Some(first), Some(second), Some(base)) = (
                        edge(apex, other1, 90),
                        edge(apex, other2, 90),
                        edge(other1, other2, 180),
                    ) {
                        emit("tSquare", &members, &[first, second, base], Some(apex));
                    }
                    if let (Some(first), Some(second), Some(base)) = (
                        edge(apex, other1, 150),
                        edge(apex, other2, 150),
                        edge(other1, other2, 60),
                    ) {
                        emit("yod", &members, &[first, second, base], Some(apex));
                    }
                }
                for d in c + 1..ids.len() {
                    // Three possible disjoint opposition pairings of four bodies.
                    for (first, second, third, fourth) in [(a, b, c, d), (a, c, b, d), (a, d, b, c)]
                    {
                        if let (
                            Some(opposition1),
                            Some(opposition2),
                            Some(s1),
                            Some(s2),
                            Some(s3),
                            Some(s4),
                        ) = (
                            edge(first, second, 180),
                            edge(third, fourth, 180),
                            edge(first, third, 90),
                            edge(first, fourth, 90),
                            edge(second, third, 90),
                            edge(second, fourth, 90),
                        ) {
                            emit(
                                "grandCross",
                                &[a, b, c, d],
                                &[opposition1, opposition2, s1, s2, s3, s4],
                                None,
                            );
                        }
                    }
                }
            }
        }
    }
    output
}

#[cfg(test)]
pub(super) fn build(
    natal: &Value,
    houses: &[Value],
    aspects: &[Value],
    rulers: &[&str; 12],
) -> Value {
    build_with_kind(natal, houses, aspects, rulers, "individualNatal")
}

pub(super) fn build_with_kind(
    natal: &Value,
    houses: &[Value],
    aspects: &[Value],
    rulers: &[&str; 12],
    chart_kind: &str,
) -> Value {
    let composite = chart_kind == "midpointComposite";
    debug_assert_eq!(houses.len(), 12);
    let bodies = natal["placements"].as_array().unwrap();
    let sun = bodies.iter().find(|p| p["id"] == "sun").unwrap()["longitude"]
        .as_f64()
        .unwrap();
    let rulership = if rulers == &TRADITIONAL {
        "traditional"
    } else {
        "modern"
    };
    let body_states: Vec<Value> = bodies
        .iter()
        .map(|p| {
            let id = p["id"].as_str().unwrap();
            let longitude = normalize(p["longitude"].as_f64().unwrap());
            let sign = (longitude / 30.).floor() as usize;
            let house = p["house"].as_u64().unwrap() as usize;
            let motion = if composite {
                "notApplicable"
            } else {
                let speed = p["speed"].as_f64().unwrap();
                if speed < 0. { "retrograde" } else if speed > 0. { "direct" } else { "stationary" }
            };
            json!({"bodyId":id,"element":ELEMENTS[sign % 4],"modality":MODALITIES[sign % 3],
                "polarity":POLARITIES[sign % 2],"houseType":HOUSE_TYPES[(house - 1) % 3],"motion":motion,
                "dignity":dignity(id,sign),"solarCondition":solar_condition(id,longitude,sun)})
        })
        .collect();
    json!({"rules":{"id":if composite { "sevenmlabs-composite-facts" } else { "sevenmlabs-natal-facts" },"version":"1.0",
        "dignity":{"system":"traditional","scope":"signOnly","supportedBodyIds":["sun","moon","mercury","venus","mars","jupiter","saturn"],"excluded":["triplicity","terms","faces","degreeOfExaltation"]},
        "motion":{"source":if composite { "symbolicMidpointConstruction" } else { "instantaneousLongitudeSpeed" },"stationary":if composite { "notApplicable" } else { "exactZero" },"predictsStation":false},
        "solarCondition":{"distance":"shortestEclipticLongitude","unit":"degrees","inclusiveThresholds":{"cazimi":17./60.,"combust":8.5,"underSunbeams":17.},"boundaryToleranceDegrees":BOUNDARY_TOLERANCE,"classification":"geometricOnly","exemptBodyIds":["sun"]},
        "dispositors":{"rulership":rulership,"path":"uniqueBodyIds","cycle":"repeatWithoutAppendingRepeatedBody"},
        "receptions":{"type":"mutualDomicile","rulership":rulership},
        "aspectPatterns":{"points":"bodiesOnly","edges":"configuredMatchedAspects","angles":"exactConfiguredAngle","orb":"inheritedFromConfiguredAspectRules","duplicateEdges":"lowestGlobalAspectIndex","kiteApex":"additionalBodyOppositeOneGrandTrineBody","additionalMatchedEdges":"allowed","uniqueBy":"typeBodyIdsAndApex"},
        "distributions":{"population":if composite { "allCompositeBodies" } else { "allNatalBodies" },"weights":"onePerBody","emptyGroups":"retained"}},
        "bodyStates":body_states,
        "distributions":{"elements":distribution(&body_states,"element",&ELEMENTS),"modalities":distribution(&body_states,"modality",&MODALITIES),
            "polarities":distribution(&body_states,"polarity",&POLARITIES),"houseTypes":distribution(&body_states,"houseType",&HOUSE_TYPES)},
        "dispositorChains":chains(bodies,rulers),"receptions":receptions(bodies,rulers,rulership),
        "aspectPatterns":patterns(bodies,aspects)})
}

#[cfg(test)]
mod tests {
    use super::*;
    const IDS: [&str; 10] = [
        "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
        "pluto",
    ];
    const MODERN: [&str; 12] = [
        "mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "pluto", "jupiter",
        "saturn", "uranus", "neptune",
    ];

    fn natal(longitudes: [f64; 10]) -> Value {
        let placements: Vec<Value> = IDS
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let longitude = normalize(longitudes[i]);
                json!({"id":id,"longitude":longitude,"signIndex":(longitude/30.).floor() as usize,
                "house":i+1,"speed":if i==2 {-1.} else if i==3 {0.} else {1.}})
            })
            .collect();
        json!({"placements":placements})
    }

    fn run(natal: &Value, aspects: &[Value], rulers: &[&str; 12]) -> Value {
        let houses: Vec<Value> = (1..=12).map(|number| json!({"number":number})).collect();
        build(natal, &houses, aspects, rulers)
    }

    fn state<'a>(advanced: &'a Value, id: &str) -> &'a Value {
        advanced["bodyStates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["bodyId"] == id)
            .unwrap()
    }

    fn chain<'a>(advanced: &'a Value, id: &str) -> &'a Value {
        advanced["dispositorChains"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["bodyId"] == id)
            .unwrap()
    }

    fn aspect(first: &str, second: &str, angle: f64, index: usize) -> Value {
        json!({"point1":first,"point2":second,"angle":angle,"index":index})
    }

    #[test]
    fn traditional_dignity_flags_and_outer_exclusion() {
        let result = run(
            &natal([0., 30., 150., 330., 270., 90., 180., 0., 0., 0.]),
            &[],
            &TRADITIONAL,
        );
        for id in &IDS[..7] {
            assert_eq!(state(&result, id)["dignity"]["exaltation"], true, "{id}");
            assert_eq!(state(&result, id)["dignity"]["fall"], false);
        }
        assert_eq!(state(&result, "mercury")["dignity"]["domicile"], true);
        for id in &IDS[7..] {
            let dignity = &state(&result, id)["dignity"];
            assert_eq!(dignity["supported"], false);
            for flag in ["domicile", "detriment", "exaltation", "fall"] {
                assert!(dignity[flag].is_null());
            }
        }
        let opposite = run(
            &natal([180., 210., 330., 150., 90., 270., 0., 0., 0., 0.]),
            &[],
            &TRADITIONAL,
        );
        for id in &IDS[..7] {
            assert_eq!(state(&opposite, id)["dignity"]["fall"], true, "{id}");
        }
        let detriments = run(
            &natal([300., 270., 240., 210., 180., 60., 90., 0., 0., 0.]),
            &[],
            &TRADITIONAL,
        );
        for id in &IDS[..7] {
            assert_eq!(state(&detriments, id)["dignity"]["detriment"], true, "{id}");
        }
        let domiciles = run(
            &natal([120., 90., 60., 30., 0., 240., 270., 0., 0., 0.]),
            &[],
            &TRADITIONAL,
        );
        for id in &IDS[..7] {
            assert_eq!(state(&domiciles, id)["dignity"]["domicile"], true, "{id}");
        }
    }

    #[test]
    fn solar_boundaries_motion_and_distributions() {
        let result = run(
            &natal([
                0.,
                17. / 60.,
                8.5,
                17.,
                17.000001,
                359.9,
                0.283334,
                8.500001,
                180.,
                90.,
            ]),
            &[],
            &TRADITIONAL,
        );
        for (id, condition) in [
            ("sun", "notApplicable"),
            ("moon", "cazimi"),
            ("mercury", "combust"),
            ("venus", "underSunbeams"),
            ("mars", "free"),
            ("jupiter", "cazimi"),
            ("saturn", "combust"),
            ("uranus", "underSunbeams"),
        ] {
            assert_eq!(
                state(&result, id)["solarCondition"]["condition"],
                condition,
                "{id}"
            );
        }
        assert!(state(&result, "sun")["solarCondition"]["separation"].is_null());
        assert_eq!(state(&result, "mercury")["motion"], "retrograde");
        assert_eq!(state(&result, "venus")["motion"], "stationary");
        assert_eq!(state(&result, "sun")["motion"], "direct");
        for name in ["elements", "modalities", "polarities", "houseTypes"] {
            let groups = result["distributions"][name].as_array().unwrap();
            assert_eq!(
                groups
                    .iter()
                    .map(|g| g["count"].as_u64().unwrap())
                    .sum::<u64>(),
                10
            );
            let ids: Vec<&str> = groups
                .iter()
                .flat_map(|g| {
                    g["bodyIds"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|id| id.as_str().unwrap())
                })
                .collect();
            assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 10);
        }
    }

    #[test]
    fn unique_dispositor_paths_self_cycles_and_receptions() {
        let result = run(
            &natal([0., 90., 60., 240., 210., 180., 240., 330., 0., 210.]),
            &[],
            &TRADITIONAL,
        );
        assert_eq!(chain(&result, "sun")["path"], json!(["sun", "mars"]));
        assert_eq!(chain(&result, "sun")["terminalBodyId"], "mars");
        assert_eq!(chain(&result, "moon")["path"], json!(["moon"]));
        assert_eq!(chain(&result, "moon")["termination"], "selfRuler");
        assert_eq!(chain(&result, "venus")["path"], json!(["venus", "jupiter"]));
        assert_eq!(chain(&result, "venus")["termination"], "cycle");
        assert_eq!(
            chain(&result, "venus")["cycleBodyIds"],
            json!(["venus", "jupiter"])
        );
        assert!(chain(&result, "venus")["terminalBodyId"].is_null());
        assert_eq!(
            chain(&result, "saturn")["path"],
            json!(["saturn", "jupiter", "venus"])
        );
        assert_eq!(
            chain(&result, "saturn")["cycleBodyIds"],
            json!(["jupiter", "venus"])
        );
        let receptions = result["receptions"].as_array().unwrap();
        assert_eq!(receptions.len(), 1);
        assert_eq!(receptions[0]["bodyIds"], json!(["jupiter", "venus"]));
        assert_eq!(receptions[0]["rulership"], "traditional");
        for value in result["dispositorChains"].as_array().unwrap() {
            let path = value["path"].as_array().unwrap();
            assert_eq!(
                path.iter()
                    .map(|p| p.as_str().unwrap())
                    .collect::<HashSet<_>>()
                    .len(),
                path.len()
            );
        }
    }

    #[test]
    fn modern_dispositor_rules_do_not_change_traditional_dignity() {
        let chart = natal([0., 90., 60., 30., 270., 240., 270., 300., 330., 210.]);
        let traditional = run(&chart, &[], &TRADITIONAL);
        let modern = run(&chart, &[], &MODERN);
        assert_eq!(chain(&modern, "pluto")["path"], json!(["pluto"]));
        assert_eq!(
            chain(&traditional, "pluto")["path"],
            json!(["pluto", "mars", "saturn"])
        );
        for id in IDS {
            assert_eq!(
                state(&traditional, id)["dignity"],
                state(&modern, id)["dignity"]
            );
        }
        assert_eq!(state(&modern, "mars")["dignity"]["exaltation"], true);
        assert_eq!(state(&modern, "pluto")["dignity"]["supported"], false);
    }

    #[test]
    fn exact_pattern_topologies_and_negative_edges() {
        let chart = natal([0.; 10]);
        let cases = [
            (
                "grandTrine",
                vec![
                    aspect("sun", "moon", 120., 0),
                    aspect("sun", "mercury", 120., 1),
                    aspect("moon", "mercury", 120., 2),
                ],
                None,
            ),
            (
                "tSquare",
                vec![
                    aspect("sun", "moon", 90., 0),
                    aspect("sun", "mercury", 90., 1),
                    aspect("moon", "mercury", 180., 2),
                ],
                Some("sun"),
            ),
            (
                "yod",
                vec![
                    aspect("sun", "moon", 150., 0),
                    aspect("sun", "mercury", 150., 1),
                    aspect("moon", "mercury", 60., 2),
                ],
                Some("sun"),
            ),
            (
                "grandCross",
                vec![
                    aspect("sun", "moon", 180., 0),
                    aspect("mercury", "venus", 180., 1),
                    aspect("sun", "mercury", 90., 2),
                    aspect("sun", "venus", 90., 3),
                    aspect("moon", "mercury", 90., 4),
                    aspect("moon", "venus", 90., 5),
                ],
                None,
            ),
            (
                "kite",
                vec![
                    aspect("sun", "moon", 120., 0),
                    aspect("sun", "mercury", 120., 1),
                    aspect("moon", "mercury", 120., 2),
                    aspect("venus", "sun", 180., 3),
                    aspect("venus", "moon", 60., 4),
                    aspect("venus", "mercury", 60., 5),
                ],
                Some("venus"),
            ),
        ];
        for (kind, edges, apex) in cases {
            let result = run(&chart, &edges, &TRADITIONAL);
            let patterns = result["aspectPatterns"].as_array().unwrap();
            let pattern = patterns.iter().find(|p| p["type"] == kind).unwrap();
            assert_eq!(pattern["apexBodyId"], json!(apex), "{kind}");
            assert_eq!(
                pattern["aspectIndexes"].as_array().unwrap().len(),
                edges.len(),
                "{kind}"
            );
            let incomplete = run(&chart, &edges[..edges.len() - 1], &TRADITIONAL);
            assert!(
                !incomplete["aspectPatterns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["type"] == kind),
                "{kind}"
            );
        }
    }

    #[test]
    fn patterns_deduplicate_edges_ignore_aliases_and_obey_configured_rules() {
        let chart = natal([0.; 10]);
        let edges = [
            aspect("sun", "moon", 120., 9),
            aspect("moon", "sun", 120., 3),
            aspect("sun", "mercury", 120., 5),
            aspect("moon", "mercury", 120., 7),
            aspect("H1", "moon", 120., 8),
            aspect("sun", "ascendant", 120., 10),
            aspect("sun", "sun", 120., 11),
        ];
        let result = run(&chart, &edges, &TRADITIONAL);
        let patterns = result["aspectPatterns"].as_array().unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0]["aspectIndexes"], json!([3, 5, 7]));
        assert_eq!(patterns[0]["bodyIds"], json!(["mercury", "moon", "sun"]));
        assert_eq!(run(&chart, &[], &TRADITIONAL)["aspectPatterns"], json!([]));
        let almost = [
            aspect("sun", "moon", 120.0001, 0),
            aspect("sun", "mercury", 120., 1),
            aspect("moon", "mercury", 120., 2),
        ];
        assert_eq!(
            run(&chart, &almost, &TRADITIONAL)["aspectPatterns"],
            json!([])
        );
        assert_eq!(result, run(&chart, &edges, &TRADITIONAL));
    }
}
