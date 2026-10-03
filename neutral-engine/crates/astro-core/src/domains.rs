//! Versioned selection views over shared chart facts; no scoring or interpretation.
use super::*;

const TRADITIONAL: [&str; 12] = [
    "mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars", "jupiter", "saturn",
    "saturn", "jupiter",
];
const MODERN: [&str; 12] = [
    "mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "pluto", "jupiter", "saturn",
    "uranus", "neptune",
];

pub(super) fn extended_rules() -> Vec<AspectRule> {
    [
        (0., 8.),
        (30., 2.),
        (45., 2.),
        (60., 6.),
        (72., 2.),
        (90., 8.),
        (120., 8.),
        (135., 2.),
        (144., 2.),
        (150., 3.),
        (180., 8.),
    ]
    .into_iter()
    .map(|(angle, max_orb)| AspectRule { angle, max_orb })
    .collect()
}

struct Facts<'a> {
    chart_kind: &'a str,
    natal: &'a Value,
    bodies: &'a [Value],
    angles: &'a [Value],
    houses: &'a [Value],
    points: &'a [Value],
    aspects: &'a [Value],
    relations: &'a [Value],
    advanced: &'a Value,
}

fn select(
    facts: &Facts<'_>,
    house_numbers: &[usize],
    body_ids: &[String],
    angle_ids: &[String],
) -> Value {
    let domain_houses: Vec<&Value> = facts
        .houses
        .iter()
        .filter(|h| house_numbers.contains(&(h["number"].as_u64().unwrap() as usize)))
        .collect();
    let mut selected = Vec::new();
    for body in facts.bodies {
        let id = body["id"].as_str().unwrap();
        let mut reasons = Vec::new();
        if body_ids.iter().any(|b| b == id) {
            reasons.push(json!({"rule":"profileBody"}));
        }
        if house_numbers.contains(&(body["house"].as_u64().unwrap() as usize)) {
            reasons.push(json!({"rule":"houseOccupant","houseId":format!("H{}",body["house"])}));
        }
        for house in &domain_houses {
            if house["rulerBodyId"] == id {
                reasons.push(json!({"rule":"houseRuler","houseId":house["id"]}));
            }
        }
        if !reasons.is_empty() {
            selected.push(json!({"placement":body,"selectionReasons":reasons}));
        }
    }
    let mut selected_ids: HashSet<&str> = selected
        .iter()
        .map(|b| b["placement"]["id"].as_str().unwrap())
        .collect();
    selected_ids.extend(angle_ids.iter().map(String::as_str));
    selected_ids.extend(domain_houses.iter().map(|h| h["id"].as_str().unwrap()));
    let domain_aspects: Vec<Value> = facts
        .aspects
        .iter()
        .filter(|a| {
            selected_ids.contains(a["point1"].as_str().unwrap())
                || selected_ids.contains(a["point2"].as_str().unwrap())
        })
        .map(|a| {
            let mut a = a.clone();
            a["selectedEndpoints"] =
                json!(
                    [a["point1"].as_str().unwrap(), a["point2"].as_str().unwrap()]
                        .into_iter()
                        .filter(|id| selected_ids.contains(id))
                        .collect::<Vec<_>>()
                );
            a
        })
        .collect();
    let relation_ids: Vec<&Value> = facts
        .relations
        .iter()
        .filter(|r| {
            selected_ids.contains(r["point1"].as_str().unwrap())
                || selected_ids.contains(r["point2"].as_str().unwrap())
        })
        .map(|r| &r["id"])
        .collect();
    let point_ids: Vec<&Value> = facts
        .points
        .iter()
        .filter(|p| selected_ids.contains(p["id"].as_str().unwrap()))
        .map(|p| &p["id"])
        .collect();
    json!({"selectionRules":{"houses":house_numbers,"bodies":body_ids,"angles":angle_ids,"aspectSelection":"atLeastOneSelectedEndpoint"},
        "houses":domain_houses,"bodies":selected,"angles":facts.angles.iter().filter(|a|angle_ids.iter().any(|id|a["id"]==*id)).collect::<Vec<_>>(),
        "pointIds":point_ids,"relationIds":relation_ids,"aspects":domain_aspects})
}

/// Reuse the individual selection policy over a subject's shared natal context.
pub(super) fn select_context(natal: &Value, context: &Value, profile: &profiles::Profile) -> Value {
    select(
        &Facts {
            chart_kind: "individualNatal",
            natal,
            bodies: natal["placements"].as_array().unwrap(),
            angles: natal["angles"].as_array().unwrap(),
            houses: context["houses"].as_array().unwrap(),
            points: context["points"].as_array().unwrap(),
            aspects: context["aspects"].as_array().unwrap(),
            relations: context["relations"].as_array().unwrap(),
            advanced: &context["advanced"],
        },
        &profile.houses,
        &profile.bodies,
        &profile.angles,
    )
}

fn view(facts: &Facts<'_>, profile: &profiles::Profile, origin: &str) -> Value {
    let mut view = select(facts, &profile.houses, &profile.bodies, &profile.angles);
    view["profileId"] = json!(if origin == "builtin" {
        "sevenmlabs-domain-selection"
    } else {
        &profile.id
    });
    view["profileVersion"] = json!(profile.version);
    view["origin"] = json!(origin);
    view["definition"] = serde_json::to_value(profile).unwrap();
    let report = reports::build_with_kind(
        facts.chart_kind,
        facts.natal,
        facts.houses,
        facts.points,
        facts.aspects,
        facts.advanced,
        &view,
    );
    let sections: Vec<(String, Value, Value)> = profile
        .sections
        .iter()
        .map(|section| {
            let selected = select(facts, &section.houses, &section.bodies, &section.angles);
            let report = reports::build_with_kind(
                facts.chart_kind,
                facts.natal,
                facts.houses,
                facts.points,
                facts.aspects,
                facts.advanced,
                &selected,
            );
            (section.id.clone(), selected, report)
        })
        .collect();
    view["report"] = reports::merge_sections(report, &sections, facts.points, facts.advanced);
    view
}

pub(super) fn build(
    natal: Value,
    rules: &[AspectRule],
    names: &[String],
    rulership: &str,
    custom: &[profiles::Profile],
) -> Value {
    build_with_kind(natal, rules, names, rulership, custom, "individualNatal")
}

pub(super) fn build_with_kind(
    natal: Value,
    rules: &[AspectRule],
    names: &[String],
    rulership: &str,
    custom: &[profiles::Profile],
    chart_kind: &str,
) -> Value {
    let rulers = if rulership == "traditional" {
        TRADITIONAL
    } else {
        MODERN
    };
    let bodies = natal["placements"].as_array().unwrap();
    let angles = natal["angles"].as_array().unwrap();
    let mut points: Vec<Value> = bodies
        .iter()
        .map(|p| {
            let mut p = p.clone();
            p["kind"] = json!("body");
            p
        })
        .chain(angles.iter().map(|p| {
            let mut p = p.clone();
            p["kind"] = json!("angle");
            p
        }))
        .collect();
    let houses: Vec<Value> = natal["houses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            let number = h["number"].as_u64().unwrap();
            let longitude = h["longitude"].as_f64().unwrap();
            let sign_index = (longitude / 30.).floor() as usize;
            let ruler_id = rulers[sign_index];
            let ruler = bodies.iter().find(|b| b["id"] == ruler_id).unwrap();
            let occupants: Vec<Value> = bodies
                .iter()
                .filter(|b| b["house"] == number)
                .cloned()
                .collect();
            let mut h = h.clone();
            h["id"] = json!(format!("H{number}"));
            h["signIndex"] = json!(sign_index);
            h["occupants"] = json!(occupants);
            h["rulerBodyId"] = json!(ruler_id);
            h["rulerPlacement"] = ruler.clone();
            points.push(
                json!({"id":h["id"],"kind":"houseCusp","houseNumber":number,"longitude":longitude,
            "signIndex":sign_index,"sign":h["sign"],"degreeInSign":h["degreeInSign"],"speed":null}),
            );
            h
        })
        .collect();
    let positions: Vec<Position> = points
        .iter()
        .map(|p| Position {
            id: p["id"].as_str().unwrap().into(),
            longitude: p["longitude"].as_f64().unwrap(),
            speed: p["speed"].as_f64(),
        })
        .collect();
    let aspect_facts: Vec<Value>=aspects(&positions,None,rules).into_iter().enumerate().map(|(index,a)| {
        json!({"index":index,"point1":a.body1,"point2":a.body2,"angle":a.angle,"separation":a.separation,
            "orb":a.orb,"maxOrb":a.max_orb,"applying":a.applying})
    }).collect();
    let relations: Vec<Value>=positions.iter().enumerate().flat_map(|(i,a)| {
        let aspect_facts=&aspect_facts;
        positions[i+1..].iter().map(move |b| {
            let indexes: Vec<usize>=aspect_facts.iter().enumerate().filter(|(_,v)| v["point1"]==a.id && v["point2"]==b.id).map(|(i,_)|i).collect();
            let delta=signed_difference(b.longitude,a.longitude);
            json!({"id":format!("{}:{}",a.id,b.id),"point1":a.id,"point2":b.id,"signedDelta":delta,
                "separation":delta.abs(),"aspectIndexes":indexes})
        })
    }).collect();
    let dispositors: Vec<Value> = bodies
        .iter()
        .map(|p| {
            let id = rulers[p["signIndex"].as_u64().unwrap() as usize];
            json!({"bodyId":p["id"],"sign":p["sign"],"dispositorBodyId":id,
            "dispositorPlacement":bodies.iter().find(|b| b["id"]==id).unwrap()})
        })
        .collect();
    let advanced = advanced::build_with_kind(&natal, &houses, &aspect_facts, &rulers, chart_kind);
    let ruler_relations: Vec<Value> = houses.iter().enumerate().flat_map(|(i, first)| {
        let relations = &relations;
        houses[i + 1..].iter().map(move |second| {
            let a = &first["rulerBodyId"];
            let b = &second["rulerBodyId"];
            let relation = relations.iter().find(|r| {
                (&r["point1"] == a && &r["point2"] == b) || (&r["point1"] == b && &r["point2"] == a)
            });
            json!({"house1":first["id"],"house2":second["id"],"ruler1":a,"ruler2":b,
                "sameRuler":a==b,"relationId":relation.map(|r| &r["id"]),
                "aspectIndexes":relation.map(|r|r["aspectIndexes"].clone()).unwrap_or_else(||json!([]))})
        })
    }).collect();
    let facts = Facts {
        chart_kind,
        natal: &natal,
        bodies,
        angles,
        houses: &houses,
        points: &points,
        aspects: &aspect_facts,
        relations: &relations,
        advanced: &advanced,
    };
    let mut views = serde_json::Map::new();
    for name in names {
        views.insert(
            name.clone(),
            view(&facts, &profiles::builtin(name), "builtin"),
        );
    }
    let mut custom_views = serde_json::Map::new();
    for profile in custom {
        custom_views.insert(profile.id.clone(), view(&facts, profile, "custom"));
    }
    let catalog: Vec<profiles::Profile> = profiles::NAMES
        .iter()
        .map(|id| profiles::builtin(id))
        .collect();
    let coverage = json!({"bodies":bodies.len(),"angles":angles.len(),"houseCusps":houses.len(),"pointPairs":relations.len(),"aspectMatching":"configuredRulesOnly"});
    json!({"chartKind":chart_kind,"subjectCount":1,"natal":natal,"context":{"rulership":rulership,"rulershipRulesVersion":"1.0","houseAssignment":"eclipticLongitude",
        "points":points,"houses":houses,"dispositors":dispositors,"relations":relations,"aspects":aspect_facts,
        "houseRulerRelations":ruler_relations,"advanced":advanced,"coverage":coverage},"domains":views,"customDomains":custom_views,"profileCatalog":catalog})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Value {
        json!({"operation":"natalDomains","utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297}})
    }
    fn run(r: Value) -> Value {
        serde_json::from_str(&calculate_json(&r.to_string())).unwrap()
    }
    #[test]
    fn complete_context_and_references() {
        let result = run(request());
        assert_eq!(result["errors"], json!([]));
        let data = &result["data"];
        let context = &data["context"];
        let points = context["points"].as_array().unwrap();
        let ids: HashSet<&str> = points.iter().map(|p| p["id"].as_str().unwrap()).collect();
        assert_eq!(points.len(), 26);
        assert_eq!(ids.len(), 26);
        let relations = context["relations"].as_array().unwrap();
        assert_eq!(relations.len(), 325);
        let relation_ids: HashSet<&str> = relations
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(relation_ids.len(), 325);
        let facts = context["aspects"].as_array().unwrap();
        for relation in relations {
            assert!(ids.contains(relation["point1"].as_str().unwrap()));
            assert!(ids.contains(relation["point2"].as_str().unwrap()));
            for index in relation["aspectIndexes"].as_array().unwrap() {
                let fact = &facts[index.as_u64().unwrap() as usize];
                assert_eq!(fact["point1"], relation["point1"]);
                assert_eq!(fact["point2"], relation["point2"]);
            }
        }
        for aspect in facts {
            let unknown = points.iter().any(|p| {
                (p["id"] == aspect["point1"] || p["id"] == aspect["point2"]) && p["kind"] != "body"
            });
            if unknown {
                assert!(aspect["applying"].is_null());
            }
        }
        for view in data["domains"].as_object().unwrap().values() {
            for id in view["pointIds"].as_array().unwrap() {
                assert!(ids.contains(id.as_str().unwrap()));
            }
            for id in view["relationIds"].as_array().unwrap() {
                assert!(relation_ids.contains(id.as_str().unwrap()));
            }
            for aspect in view["aspects"].as_array().unwrap() {
                let fact = &facts[aspect["index"].as_u64().unwrap() as usize];
                assert_eq!(fact["point1"], aspect["point1"]);
                assert!(!aspect["selectedEndpoints"].as_array().unwrap().is_empty());
            }
        }
    }
    #[test]
    fn selected_aspect_keeps_an_external_endpoint() {
        let longs = [0., 310., 40., 70., 115., 145., 190., 225., 260., 90.];
        let ids = [
            "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
            "pluto",
        ];
        let positions: Vec<Position> = ids
            .into_iter()
            .zip(longs)
            .map(|(id, longitude)| Position {
                id: id.into(),
                longitude,
                speed: Some(1.),
            })
            .collect();
        let cusps: Vec<f64> = (0..12).map(|n| n as f64 * 30.).collect();
        let houses: Vec<Value> = cusps
            .iter()
            .enumerate()
            .map(|(i, lon)| json!({"number":i+1,"longitude":lon,"sign":SIGNS[i],"degreeInSign":0}))
            .collect();
        let angles: Vec<Position> = [
            ("ascendant", 10.),
            ("midheaven", 280.),
            ("descendant", 190.),
            ("imumCoeli", 100.),
        ]
        .into_iter()
        .map(|(id, longitude)| Position {
            id: id.into(),
            longitude,
            speed: None,
        })
        .collect();
        let natal = json!({"placements":placements(&positions,Some(&cusps)),"angles":placements(&angles,None),"houses":houses});
        let data = build(
            natal,
            &default_rules(),
            &["career".into()],
            "traditional",
            &[],
        );
        let view = &data["domains"]["career"];
        assert!(!view["pointIds"]
            .as_array()
            .unwrap()
            .contains(&json!("pluto")));
        let aspect = view["aspects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["point1"] == "sun" && a["point2"] == "pluto" && a["angle"].as_f64() == Some(90.)
            })
            .unwrap();
        assert_eq!(aspect["selectedEndpoints"], json!(["sun"]));
    }
    #[test]
    fn whole_sign_rulers_and_real_mc() {
        let mut r = request();
        r["houseSystem"] = json!("wholeSign");
        let traditional = run(r.clone());
        r["rulership"] = json!("modern");
        let modern = run(r);
        assert_eq!(
            traditional["data"]["context"]["houses"][4]["sign"],
            "Scorpio"
        );
        assert_eq!(
            traditional["data"]["context"]["houses"][4]["rulerBodyId"],
            "mars"
        );
        assert_eq!(
            modern["data"]["context"]["houses"][4]["rulerBodyId"],
            "pluto"
        );
        assert_ne!(
            traditional["data"]["context"]["houses"][9]["longitude"],
            traditional["data"]["natal"]["angles"][1]["longitude"]
        );
        assert_eq!(
            traditional["calculation"]["aspectRules"]
                .as_array()
                .unwrap()
                .len(),
            11
        );
    }
    #[test]
    fn custom_rules_and_requested_domains() {
        let mut r = request();
        r["domains"] = json!(["finance"]);
        r["aspectRules"] = json!([]);
        let result = run(r.clone());
        assert_eq!(result["errors"], json!([]));
        assert_eq!(result["data"]["context"]["aspects"], json!([]));
        assert_eq!(
            result["data"]["context"]["relations"]
                .as_array()
                .unwrap()
                .len(),
            325
        );
        assert_eq!(result["data"]["domains"].as_object().unwrap().len(), 1);
        r.as_object_mut().unwrap().remove("aspectRules");
        r["aspectPreset"] = json!("major");
        let domain = run(r.clone());
        r["operation"] = json!("natal");
        r.as_object_mut().unwrap().remove("domains");
        r.as_object_mut().unwrap().remove("aspectPreset");
        assert_eq!(domain["data"]["natal"], run(r)["data"]);
    }
    #[test]
    fn invalid_settings_are_errors() {
        for (key, value) in [
            ("domains", json!([])),
            ("domains", json!(["love", "love"])),
            ("domains", json!(["health"])),
            ("domains", Value::Null),
            ("rulership", json!("both")),
            ("rulership", Value::Null),
            ("aspectPreset", json!("unknown")),
            ("aspectPreset", Value::Null),
        ] {
            let mut r = request();
            r[key] = value;
            assert_eq!(run(r)["errors"][0]["code"], "INVALID_INPUT");
        }
        let mut r = request();
        r["aspectRules"] = json!([]);
        r["aspectPreset"] = json!("major");
        assert_eq!(run(r)["errors"][0]["code"], "INVALID_INPUT");
        let mut r = request();
        r["operation"] = json!("natal");
        r["domains"] = json!(["love"]);
        assert_eq!(run(r)["errors"][0]["code"], "INVALID_INPUT");
    }
}
