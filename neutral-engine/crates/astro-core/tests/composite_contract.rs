use astro_core::{calculate_json, normalize, signed_difference};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

fn request() -> Value {
    serde_json::from_str(include_str!("../../../../examples/composite-request.json")).unwrap()
}
fn run(r: &Value) -> Value {
    serde_json::from_str(&calculate_json(&r.to_string())).unwrap()
}
fn success(r: &Value) -> Value {
    let v = run(r);
    assert_eq!(v["errors"], json!([]));
    v
}
fn list<'a>(v: &'a Value, field: &str) -> &'a [Value] {
    v[field].as_array().unwrap().as_slice()
}
fn ids<'a>(v: &'a Value, field: &str) -> HashSet<&'a str> {
    list(v, field)
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect()
}
fn close(a: f64, b: f64) {
    assert!(signed_difference(a, b).abs() < 1e-10, "{a} != {b}");
}
fn combined(r: &Value) -> Value {
    let mut r = r.clone();
    r["operation"] = json!("couple");
    let options = json!({"houseMethod":r["houseMethod"],"antipodalPolicy":r["antipodalPolicy"]});
    r.as_object_mut().unwrap().remove("houseMethod");
    r.as_object_mut().unwrap().remove("antipodalPolicy");
    r["domains"] = json!(["communication"]);
    r["composite"] = options;
    r
}

#[test]
fn third_chart_is_symbolic_and_has_ten_scoped_reports() {
    let output = success(&request());
    let data = &output["data"];
    let c = &data["composite"];
    assert_eq!(data["chartKind"], "compositeRelationship");
    assert_eq!(data["subjectCount"], 2);
    assert_eq!(data["chartCount"], 3);
    assert_eq!(c["id"], "C");
    assert_eq!(c["sourceSubjectIds"], json!(["A", "B"]));
    assert_eq!(c["chartKind"], "midpointComposite");
    assert_eq!(output["calculation"]["scope"], "midpoint-composite-data");
    for field in ["utc", "location", "julianDayTt", "julianDayUt1"] {
        assert!(c["chart"].get(field).is_none());
        assert!(output["calculation"].get(field).is_none());
    }
    for p in list(&c["chart"], "placements") {
        assert!(p["speed"].is_null());
        assert!(p["isRetrograde"].is_null());
        assert!(p.get("latitude").is_none());
        assert!(p.get("distanceAu").is_none());
    }
    let context = &c["context"];
    assert_eq!(list(context, "points").len(), 26);
    assert_eq!(ids(context, "points").len(), 26);
    assert_eq!(list(context, "relations").len(), 325);
    assert_eq!(ids(context, "relations").len(), 325);
    assert_eq!(list(context, "houseRulerRelations").len(), 66);
    assert!(list(context, "aspects")
        .iter()
        .all(|a| a["applying"].is_null()));
    assert!(list(&context["advanced"], "bodyStates")
        .iter()
        .all(|b| b["motion"] == "notApplicable"));
    assert_eq!(
        context["advanced"]["rules"]["distributions"]["population"],
        "allCompositeBodies"
    );
    assert_eq!(c["domains"].as_object().unwrap().len(), 10);
    assert_eq!(list(c, "profileCatalog").len(), 10);
    assert_eq!(
        c["domains"]
            .as_object()
            .unwrap()
            .values()
            .map(|v| list(&v["report"], "sections").len())
            .sum::<usize>(),
        30
    );
    for v in c["domains"].as_object().unwrap().values() {
        assert_eq!(v["report"]["chartKind"], "midpointComposite");
        assert!(list(&v["report"], "bodyFacts")
            .iter()
            .all(|b| b["motion"] == "notApplicable"));
    }
}

#[test]
fn source_charts_preserve_their_independent_natal_calculations() {
    let r = request();
    let output = success(&r);
    for (person, id) in [("personA", "A"), ("personB", "B")] {
        let mut natal = r[person].clone();
        natal["operation"] = json!("natal");
        natal["aspectRules"] = output["calculation"]["aspectRules"].clone();
        let single = success(&natal);
        let subject = &output["data"]["subjects"][id];
        assert_eq!(subject["natal"], single["data"]);
        assert_eq!(subject["calculation"], single["calculation"]);
        for key in ["julianDayTt", "julianDayUt1", "houseSystem"] {
            assert_eq!(
                output["calculation"]["sourceCalculations"][id][key],
                single["calculation"][key]
            );
        }
    }
}

#[test]
fn correspondence_midpoints_and_provenance_resolve_back_to_both_sources() {
    let output = success(&request());
    let data = &output["data"];
    let c = &data["composite"];
    let points: HashMap<&str, &Value> = list(&c["context"], "points")
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p))
        .collect();
    assert_eq!(list(&c["provenance"], "points").len(), 26);
    for p in list(&c["provenance"], "points") {
        let id = p["id"].as_str().unwrap();
        close(
            p["longitude"].as_f64().unwrap(),
            points[id]["longitude"].as_f64().unwrap(),
        );
        assert_eq!(
            p["sourcePointIds"],
            json!([format!("A:{id}"), format!("B:{id}")])
        );
        if p["construction"] == "shortestArcMidpoint" {
            let a = p["sourceLongitudes"][0].as_f64().unwrap();
            let b = p["sourceLongitudes"][1].as_f64().unwrap();
            assert_eq!(p["antipodal"], false);
            let actual = p["longitude"].as_f64().unwrap();
            close(actual, normalize(a + signed_difference(b, a) / 2.0));
            assert!(
                (signed_difference(actual, a).abs() - signed_difference(actual, b).abs()).abs()
                    < 1e-10
            );
        }
    }
}

#[test]
fn same_birth_is_an_identity_for_geometry_but_never_for_physical_motion() {
    let mut r = request();
    r["personB"] = r["personA"].clone();
    let output = success(&r);
    let natal = &output["data"]["subjects"]["A"]["natal"];
    let chart = &output["data"]["composite"]["chart"];
    for key in ["placements", "angles", "houses"] {
        for (source, target) in list(natal, key).iter().zip(list(chart, key)) {
            close(
                source["longitude"].as_f64().unwrap(),
                target["longitude"].as_f64().unwrap(),
            );
        }
    }
    assert!(list(chart, "placements")
        .iter()
        .all(|p| p["speed"].is_null() && p["isRetrograde"].is_null()));
}

#[test]
fn source_swap_preserves_composite_longitudes_and_house_assignments() {
    let r = request();
    let before = success(&r);
    let mut swap = r.clone();
    swap["personA"] = r["personB"].clone();
    swap["personB"] = r["personA"].clone();
    let after = success(&swap);
    for key in ["placements", "angles", "houses"] {
        for (a, b) in list(&before["data"]["composite"]["chart"], key)
            .iter()
            .zip(list(&after["data"]["composite"]["chart"], key))
        {
            close(
                a["longitude"].as_f64().unwrap(),
                b["longitude"].as_f64().unwrap(),
            );
            assert_eq!(a.get("house"), b.get("house"));
            assert_eq!(a.get("sign"), b.get("sign"));
        }
    }
}

#[test]
fn optional_couple_chart_matches_standalone_and_keeps_the_original_pair_context() {
    let r = request();
    let standalone = success(&r);
    let with_c = combined(&r);
    let output = success(&with_c);
    assert_eq!(output["data"]["chartCount"], 3);
    assert_eq!(output["data"]["composite"], standalone["data"]["composite"]);
    assert_eq!(
        output["calculation"]["composite"],
        standalone["calculation"]
    );
    let mut plain = with_c.clone();
    plain.as_object_mut().unwrap().remove("composite");
    let original = success(&plain);
    for key in [
        "subjects",
        "context",
        "domains",
        "customDomains",
        "profileCatalog",
    ] {
        assert_eq!(output["data"][key], original["data"][key]);
    }
    assert!(original["data"].get("composite").is_none());
    assert!(original["data"].get("chartCount").is_none());
}

#[test]
fn invalid_midpoint_houses_fail_without_fallback_and_whole_sign_is_explicit() {
    let mut r: Value = serde_json::from_str(include_str!(
        "../../../../tests/conformance/composite-invalid-houses-request.json"
    ))
    .unwrap();
    let error = run(&r);
    assert_eq!(error["errors"][0]["code"], "CALCULATION_FAILED");
    assert!(error["data"].is_null());
    let integrated = run(&combined(&r));
    assert_eq!(integrated["errors"][0]["code"], "CALCULATION_FAILED");
    assert!(integrated["data"].is_null());
    r["houseMethod"] = json!("wholeSignFromMidpointAscendant");
    let output = success(&r);
    let c = &output["data"]["composite"];
    let asc = list(&c["chart"], "angles")
        .iter()
        .find(|a| a["id"] == "ascendant")
        .unwrap()["longitude"]
        .as_f64()
        .unwrap();
    let start = (asc / 30.0).floor() * 30.0;
    for (i, cusp) in list(&c["chart"], "houseCusps").iter().enumerate() {
        close(cusp.as_f64().unwrap(), normalize(start + i as f64 * 30.0));
    }
    for p in list(&c["provenance"], "points")
        .iter()
        .filter(|p| p["kind"] == "houseCusp")
    {
        assert_eq!(p["construction"], "wholeSignFromMidpointAscendant");
        assert_eq!(p["resolution"], "notRequiredForDerivedPoint");
    }
}

#[test]
fn custom_c_sections_keep_parent_primary_contacts_and_reference_closure() {
    let mut r = request();
    r["domains"] = json!([]);
    r["customProfiles"] = json!([{"id":"bond","bodies":["sun"],"sections":[{"id":"resources","houses":[2,8],"bodies":["venus","jupiter","saturn"]}]}]);
    let output = success(&r);
    let c = &output["data"]["composite"];
    let view = &c["customDomains"]["bond"];
    let report = &view["report"];
    assert_eq!(c["domains"], json!({}));
    assert_eq!(view["pointIds"], json!(["sun"]));
    let points = ids(&c["context"], "points");
    let report_points: HashSet<&str> = list(report, "pointIds")
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    let primary: HashSet<u64> = list(view, "aspects")
        .iter()
        .map(|a| a["index"].as_u64().unwrap())
        .collect();
    let report_aspects: HashSet<u64> = list(report, "aspects")
        .iter()
        .map(|a| a["index"].as_u64().unwrap())
        .collect();
    for aspect in list(report, "aspects") {
        assert_eq!(
            aspect["primaryContact"],
            primary.contains(&aspect["index"].as_u64().unwrap())
        );
        for key in ["point1", "point2"] {
            let id = aspect[key].as_str().unwrap();
            assert!(points.contains(id) && report_points.contains(id));
        }
        assert!(aspect["applying"].is_null());
    }
    let houses = ids(report, "houses");
    let chains = ids(&c["context"]["advanced"], "dispositorChains");
    for section in list(report, "sections") {
        for id in list(section, "relatedHouseIds") {
            assert!(houses.contains(id.as_str().unwrap()));
        }
        for index in list(section, "aspectIndexes") {
            assert!(report_aspects.contains(&index.as_u64().unwrap()));
        }
        for id in list(section, "dispositorChainIds") {
            assert!(chains.contains(id.as_str().unwrap()));
        }
    }
}

#[test]
fn composite_options_and_both_births_validate_before_calculation() {
    let mut r = combined(&request());
    r["personA"]["location"]["latitude"] = json!(80);
    r["composite"]["domains"] = json!(["attraction"]);
    let error = run(&r);
    assert_eq!(error["errors"][0]["code"], "INVALID_INPUT");
    assert!(error["data"].is_null());
    r["composite"]["domains"] = json!(["love"]);
    r["personB"]["utc"]["day"] = json!(32);
    let error = run(&r);
    assert_eq!(error["errors"][0]["code"], "INVALID_INPUT");
    assert!(error["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("personB"));
    let raw = request()
        .to_string()
        .replace("\"year\":2000", "\"year\":2e3");
    let normalized: Value = serde_json::from_str(&calculate_json(&raw)).unwrap();
    assert_eq!(normalized, success(&request()));
    let raw = request()
        .to_string()
        .replace("\"year\":1998", "\"year\":1998.0000000000000001");
    let error: Value = serde_json::from_str(&calculate_json(&raw)).unwrap();
    assert_eq!(error["errors"][0]["code"], "INVALID_INPUT");
}
