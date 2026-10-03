use astro_core::{calculate_json, normalize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

fn request() -> Value {
    json!({"operation":"couple",
        "personA":{"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},
            "location":{"latitude":10.8231,"longitude":106.6297}},
        "personB":{"utc":{"year":1998,"month":6,"day":15,"hour":6,"minute":30},
            "location":{"latitude":21.0285,"longitude":105.8542},"houseSystem":"wholeSign"}})
}
fn run(r: &Value) -> Value {
    serde_json::from_str(&calculate_json(&r.to_string())).unwrap()
}
fn success(r: &Value) -> Value {
    let result = run(r);
    assert_eq!(result["errors"], json!([]));
    result
}
fn list<'a>(v: &'a Value, field: &str) -> &'a [Value] {
    v[field].as_array().unwrap().as_slice()
}
fn ids(v: &Value, field: &str) -> HashSet<String> {
    list(v, field)
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn two_birth_charts_equal_individual_calculations_with_separate_epochs() {
    let r = request();
    let paired = success(&r);
    assert_eq!(paired["data"]["chartKind"], "coupleSynastry");
    assert_eq!(paired["data"]["subjectCount"], 2);
    assert_eq!(paired["calculation"]["scope"], "couple-synastry-data");
    for (person, chart) in [("personA", "A"), ("personB", "B")] {
        let mut natal = r[person].clone();
        natal["operation"] = json!("natal");
        natal["aspectRules"] = paired["calculation"]["aspectRules"].clone();
        let single = success(&natal);
        let subject = &paired["data"]["subjects"][chart];
        assert_eq!(subject["natal"], single["data"]);
        for field in ["julianDayTt", "julianDayUt1", "houseSystem"] {
            assert_eq!(subject["calculation"][field], single["calculation"][field]);
        }
        natal["operation"] = json!("natalDomains");
        natal["domains"] = json!(["identity"]);
        let individual = success(&natal);
        assert_eq!(subject["context"], individual["data"]["context"]);
    }
    assert_ne!(
        paired["data"]["subjects"]["A"]["calculation"]["julianDayTt"],
        paired["data"]["subjects"]["B"]["calculation"]["julianDayTt"]
    );
    assert!(paired["calculation"].get("julianDayTt").is_none());
}

#[test]
fn all_cross_pairs_are_qualified_and_join_to_matched_aspects() {
    let result = success(&request());
    let context = &result["data"]["context"];
    let points = list(context, "points");
    let point_ids = ids(context, "points");
    assert_eq!(points.len(), 52);
    assert_eq!(point_ids.len(), 52);
    for p in points {
        let chart = p["chartId"].as_str().unwrap();
        let local = p["localId"].as_str().unwrap();
        assert_eq!(p["id"], format!("{chart}:{local}"));
        assert!(["A", "B"].contains(&chart));
    }
    let relations = list(context, "relations");
    assert_eq!(relations.len(), 676);
    assert_eq!(ids(context, "relations").len(), 676);
    let aspect_facts = list(context, "aspects");
    let mut referenced = HashSet::new();
    for relation in relations {
        let a = relation["point1"].as_str().unwrap();
        let b = relation["point2"].as_str().unwrap();
        assert!(a.starts_with("A:") && b.starts_with("B:"));
        assert!(point_ids.contains(a) && point_ids.contains(b));
        for index in list(relation, "aspectIndexes") {
            let i = index.as_u64().unwrap() as usize;
            let aspect = &aspect_facts[i];
            assert_eq!(aspect["index"], i);
            assert_eq!(aspect["relationId"], relation["id"]);
            assert_eq!(aspect["point1"], relation["point1"]);
            assert_eq!(aspect["point2"], relation["point2"]);
            assert!(aspect["applying"].is_null());
            assert!(referenced.insert(i));
        }
    }
    assert_eq!(referenced.len(), aspect_facts.len());
    assert_eq!(list(context, "overlaysAtoB").len(), 10);
    assert_eq!(list(context, "overlaysBtoA").len(), 10);
    assert_eq!(list(context, "houseRulerRelations").len(), 144);
    let domains = result["data"]["domains"].as_object().unwrap();
    assert_eq!(domains.len(), 6);
    assert_eq!(list(&result["data"], "profileCatalog").len(), 6);
    assert_eq!(
        domains
            .values()
            .map(|v| list(&v["report"], "sections").len())
            .sum::<usize>(),
        18
    );
}

#[test]
fn body_cross_contacts_match_existing_position_synastry() {
    let paired = success(&request());
    let a = &paired["data"]["subjects"]["A"]["natal"];
    let b = &paired["data"]["subjects"]["B"]["natal"];
    let positions = |subject: &Value| {
        list(subject, "placements")
            .iter()
            .map(|p| json!({"id":p["id"],"longitude":p["longitude"],"speed":p["speed"]}))
            .collect::<Vec<_>>()
    };
    let old = success(&json!({"operation":"synastry", "positions":positions(a),
        "otherPositions":positions(b),"houses":a["houseCusps"],"otherHouses":b["houseCusps"],
        "aspectRules":paired["calculation"]["aspectRules"]}));
    let body_ids: HashSet<&str> = list(a, "placements")
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    let body_aspects: Vec<Value> = list(&paired["data"]["context"], "aspects")
        .iter()
        .filter(|p| {
            body_ids.contains(&p["point1"].as_str().unwrap()[2..])
                && body_ids.contains(&p["point2"].as_str().unwrap()[2..])
        })
        .map(|p| {
            json!({"body1":&p["point1"].as_str().unwrap()[2..],
            "body2":&p["point2"].as_str().unwrap()[2..],"angle":p["angle"],
            "separation":p["separation"],"orb":p["orb"],"maxOrb":p["maxOrb"],"applying":null})
        })
        .collect();
    let old_aspects = list(&old["data"], "crossAspects");
    assert_eq!(body_aspects.len(), old_aspects.len());
    for (actual, expected) in body_aspects.iter().zip(old_aspects) {
        for field in ["body1", "body2", "angle", "maxOrb", "applying"] {
            assert_eq!(actual[field], expected[field]);
        }
        // A JSON round trip of supplied longitudes may differ at f64 precision.
        for field in ["separation", "orb"] {
            assert!(
                (actual[field].as_f64().unwrap() - expected[field].as_f64().unwrap()).abs() < 1e-10
            );
        }
    }
}

#[test]
fn swapping_subjects_preserves_separations_and_reverses_delta() {
    let r = request();
    let before = success(&r);
    let mut swapped = r.clone();
    swapped["personA"] = r["personB"].clone();
    swapped["personB"] = r["personA"].clone();
    let after = success(&swapped);
    let reverse: HashMap<(String, String), &Value> = list(&after["data"]["context"], "relations")
        .iter()
        .map(|p| {
            (
                (
                    p["point1"].as_str().unwrap()[2..].to_owned(),
                    p["point2"].as_str().unwrap()[2..].to_owned(),
                ),
                p,
            )
        })
        .collect();
    for p in list(&before["data"]["context"], "relations") {
        let key = (
            p["point2"].as_str().unwrap()[2..].to_owned(),
            p["point1"].as_str().unwrap()[2..].to_owned(),
        );
        let other = reverse[&key];
        assert!(
            (p["separation"].as_f64().unwrap() - other["separation"].as_f64().unwrap()).abs()
                < 1e-10
        );
        let total = p["signedDelta"].as_f64().unwrap() + other["signedDelta"].as_f64().unwrap();
        assert!(normalize(total).min(360.0 - normalize(total)) < 1e-10);
        assert_eq!(
            list(p, "aspectIndexes").len(),
            list(other, "aspectIndexes").len()
        );
    }
}

#[test]
fn identical_births_keep_distinct_charts_and_disabled_aspects_keep_geometry() {
    let mut r = request();
    r["personB"] = r["personA"].clone();
    r["aspectRules"] = json!([{"angle":0,"maxOrb":0}]);
    let result = success(&r);
    let context = &result["data"]["context"];
    let identical = list(context, "aspects")
        .iter()
        .filter(|a| a["point1"].as_str().unwrap()[2..] == a["point2"].as_str().unwrap()[2..])
        .count();
    assert_eq!(identical, 26);
    r["aspectRules"] = json!([]);
    let empty = success(&r);
    let context = &empty["data"]["context"];
    assert_eq!(list(context, "relations").len(), 676);
    assert_eq!(list(context, "aspects").len(), 0);
    assert_eq!(list(context, "houseRulerRelations").len(), 144);
    for relation in list(context, "relations") {
        assert_eq!(relation["aspectIndexes"], json!([]));
    }
    for domain in empty["data"]["domains"].as_object().unwrap().values() {
        assert_eq!(domain["aspects"], json!([]));
        assert_eq!(domain["report"]["aspectFacts"], json!([]));
    }
}

#[test]
fn both_subjects_validate_before_provider_calculation_and_no_partial_success() {
    let mut r = request();
    r["personA"]["location"]["latitude"] = json!(80);
    r["personB"]["utc"]["day"] = json!(32);
    let error = run(&r);
    assert_eq!(error["errors"][0]["code"], "INVALID_INPUT");
    assert!(error["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("personB"));
    assert!(error["data"].is_null());
    r["personB"]["utc"]["day"] = json!(15);
    let error = run(&r);
    assert_eq!(error["errors"][0]["code"], "CALCULATION_FAILED");
    assert!(error["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("personA"));
    assert!(error["data"].is_null());
}

#[test]
fn raw_integer_notations_normalize_before_two_chart_computation() {
    let r = request();
    let expected = success(&r);
    let raw = r
        .to_string()
        .replace("\"year\":2000", "\"year\":2e3")
        .replace("\"year\":1998", "\"year\":1998.0");
    let actual: Value = serde_json::from_str(&calculate_json(&raw)).unwrap();
    assert_eq!(actual, expected);
    let raw = r
        .to_string()
        .replace("\"year\":1998", "\"year\":1998.0000000000000001");
    let error: Value = serde_json::from_str(&calculate_json(&raw)).unwrap();
    assert_eq!(error["errors"][0]["code"], "INVALID_INPUT");
    assert!(error["data"].is_null());
}

#[test]
fn directional_overlays_and_cross_house_rulers_resolve_to_the_right_chart() {
    let paired = success(&request());
    let data = &paired["data"];
    let context = &data["context"];
    let relations: HashMap<&str, &Value> = list(context, "relations")
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r))
        .collect();
    for (direction, source, target) in [("overlaysAtoB", "A", "B"), ("overlaysBtoA", "B", "A")] {
        for overlay in list(context, direction) {
            assert_eq!(overlay["sourceChartId"], source);
            assert_eq!(overlay["targetChartId"], target);
            let body = overlay["localBodyId"].as_str().unwrap();
            assert_eq!(overlay["pointId"], format!("{source}:{body}"));
            let number = overlay["targetHouseNumber"].as_u64().unwrap() as usize;
            assert!((1..=12).contains(&number));
            assert_eq!(overlay["targetHouseId"], format!("{target}:H{number}"));
            let natal = &data["subjects"][source]["natal"];
            let placement = list(natal, "placements")
                .iter()
                .find(|p| p["id"] == body)
                .unwrap();
            let target_natal = &data["subjects"][target]["natal"];
            let cusps = list(target_natal, "houseCusps");
            let longitude = placement["longitude"].as_f64().unwrap();
            let left = cusps[number - 1].as_f64().unwrap();
            let right = cusps[number % 12].as_f64().unwrap();
            assert!(normalize(longitude - left) < normalize(right - left));
            let house = &list(&data["subjects"][target]["context"], "houses")[number - 1];
            assert_eq!(
                overlay["targetRulerPointId"],
                format!("{target}:{}", house["rulerBodyId"].as_str().unwrap())
            );
            assert_eq!(
                overlay["sourceHouseId"],
                format!("{source}:H{}", placement["house"])
            );
            for field in ["targetCuspRelationId", "targetRulerRelationId"] {
                let relation = relations[overlay[field].as_str().unwrap()];
                let expected_other = if field == "targetCuspRelationId" {
                    &overlay["targetHouseId"]
                } else {
                    &overlay["targetRulerPointId"]
                };
                assert!(
                    (relation["point1"] == overlay["pointId"]
                        && relation["point2"] == *expected_other)
                        || (relation["point2"] == overlay["pointId"]
                            && relation["point1"] == *expected_other)
                );
            }
        }
    }
    for link in list(context, "houseRulerRelations") {
        for (chart, house_field, ruler_field) in
            [("A", "house1", "ruler1"), ("B", "house2", "ruler2")]
        {
            let local_house = &link[house_field].as_str().unwrap()[2..];
            let house = list(&data["subjects"][chart]["context"], "houses")
                .iter()
                .find(|h| h["id"] == local_house)
                .unwrap();
            assert_eq!(
                link[ruler_field],
                format!("{chart}:{}", house["rulerBodyId"].as_str().unwrap())
            );
        }
        let relation = relations[link["relationId"].as_str().unwrap()];
        assert_eq!(relation["point1"], link["ruler1"]);
        assert_eq!(relation["point2"], link["ruler2"]);
        assert_eq!(relation["aspectIndexes"], link["aspectIndexes"]);
    }
}

#[test]
fn custom_sections_extend_support_without_changing_parent_primary_contacts() {
    let mut r = request();
    r["personA"]["utc"]["year"] = json!(1980);
    r["domains"] = json!([]);
    r["customProfiles"] = json!([{"id":"coreBond","bodies":["sun"],"sections":[
        {"id":"broader","houses":[8],"bodies":["sun","moon","mercury","venus","mars","jupiter","saturn","uranus","neptune","pluto"]}]}]);
    let paired = success(&r);
    let data = &paired["data"];
    assert_eq!(data["domains"], json!({}));
    let profile = &data["customDomains"]["coreBond"];
    assert_eq!(profile["definition"]["version"], "1.0");
    assert_eq!(profile["pointIds"], json!(["A:sun", "B:sun"]));
    let report = &profile["report"];
    assert_eq!(report["chartKind"], "coupleSynastry");
    assert_eq!(report["primaryPointIds"], profile["pointIds"]);
    assert_eq!(list(report, "sections").len(), 1);
    let global_points = ids(&data["context"], "points");
    let report_points = ids(report, "points");
    let report_houses = ids(report, "houses");
    let report_bodies = ids(report, "bodyFacts");
    assert!(report_points.len() > 2);
    let primary_aspects: HashSet<u64> = list(profile, "aspects")
        .iter()
        .map(|a| a["index"].as_u64().unwrap())
        .collect();
    let mut all_aspects = HashSet::new();
    for point in list(report, "points") {
        assert!(global_points.contains(point["id"].as_str().unwrap()));
        let primary = point["id"] == "A:sun" || point["id"] == "B:sun";
        assert_eq!(point["primary"], primary);
    }
    for aspect in list(report, "aspectFacts") {
        let index = aspect["index"].as_u64().unwrap();
        assert!(all_aspects.insert(index));
        assert_eq!(aspect["primaryContact"], primary_aspects.contains(&index));
        assert!(report_points.contains(aspect["point1"].as_str().unwrap()));
        assert!(report_points.contains(aspect["point2"].as_str().unwrap()));
        let global = &list(&data["context"], "aspects")[index as usize];
        for field in ["point1", "point2", "angle", "orb", "relationId"] {
            assert_eq!(aspect[field], global[field]);
        }
    }
    let overlays: HashSet<String> = ids(&data["context"], "overlaysAtoB")
        .into_iter()
        .chain(ids(&data["context"], "overlaysBtoA"))
        .collect();
    let ruler_ids = ids(&data["context"], "houseRulerRelations");
    let check_refs = |owner: &Value| {
        for (field, advanced_field) in [
            ("dispositorChainRefs", "dispositorChains"),
            ("natalReceptionRefs", "receptions"),
            ("natalAspectPatternRefs", "aspectPatterns"),
        ] {
            for reference in list(owner, field) {
                let chart = reference["chartId"].as_str().unwrap();
                assert!(["A", "B"].contains(&chart));
                assert!(list(
                    &data["subjects"][chart]["context"]["advanced"],
                    advanced_field
                )
                .iter()
                .any(|fact| fact["id"] == reference["id"]));
            }
        }
        for id in list(owner, "overlayIds") {
            assert!(overlays.contains(id.as_str().unwrap()));
        }
        for id in list(owner, "houseRulerRelationIds") {
            assert!(ruler_ids.contains(id.as_str().unwrap()));
        }
    };
    check_refs(report);
    assert!(!list(report, "natalReceptionRefs").is_empty());
    assert!(!list(report, "natalAspectPatternRefs").is_empty());
    for section in list(report, "sections") {
        check_refs(section);
        for (field, pool) in [
            ("pointIds", &report_points),
            ("houseIds", &report_houses),
            ("bodyFactIds", &report_bodies),
        ] {
            for id in list(section, field) {
                assert!(pool.contains(id.as_str().unwrap()));
            }
        }
        for index in list(section, "aspectIndexes") {
            assert!(all_aspects.contains(&index.as_u64().unwrap()));
        }
    }
}
