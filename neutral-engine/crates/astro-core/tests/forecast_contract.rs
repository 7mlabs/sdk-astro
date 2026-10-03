use astro_core::{calculate_json, signed_difference};
use serde_json::{json, Value};
use std::collections::HashSet;
fn birth() -> Value {
    json!({"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"houseSystem":"placidus"})
}
fn request() -> Value {
    json!({"operation":"forecast","birth":birth(),"period":{"kind":"day","year":2026,"month":3,"day":3,"utcOffsetMinutes":420}})
}
fn run(r: &Value) -> Value {
    serde_json::from_str(&calculate_json(&r.to_string())).unwrap()
}
fn valid(r: &Value) -> Value {
    let v = run(r);
    assert_eq!(v["errors"], json!([]), "{}", v["errors"]);
    v
}
fn members(v: &Value) -> HashSet<&str> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect()
}
fn closure(d: &Value) {
    let context = &d["subject"]["context"];
    let ids: HashSet<String> = context["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| format!("N:{}", p["id"].as_str().unwrap()))
        .collect();
    let events = d["events"].as_array().unwrap();
    let event_ids: HashSet<&str> = events.iter().map(|e| e["id"].as_str().unwrap()).collect();
    assert_eq!(events.len(), event_ids.len());
    for e in events {
        let impact = &e["personalImpact"];
        for id in impact["affectedNatalPointIds"].as_array().unwrap() {
            assert!(ids.contains(id.as_str().unwrap()));
        }
        for (i, c) in impact["contacts"].as_array().unwrap().iter().enumerate() {
            assert_eq!(c["index"].as_u64().unwrap(), i as u64);
            assert!(ids.contains(c["natalPointId"].as_str().unwrap()));
        }
        for o in impact["houseOverlays"].as_array().unwrap() {
            assert!(ids.contains(o["natalHouseId"].as_str().unwrap()));
            assert!(ids.contains(o["natalRulerPointId"].as_str().unwrap()));
            for id in o["natalOccupantPointIds"].as_array().unwrap() {
                assert!(ids.contains(id.as_str().unwrap()));
            }
        }
    }
    for v in d["domains"]
        .as_object()
        .unwrap()
        .values()
        .chain(d["customDomains"].as_object().unwrap().values())
    {
        for id in v["natalPointIds"].as_array().unwrap() {
            assert!(ids.contains(id.as_str().unwrap()));
        }
        let refs: HashSet<&str> = v["eventReferences"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["eventId"].as_str().unwrap())
            .collect();
        assert!(refs.is_subset(&event_ids));
        assert_eq!(refs, members(&v["messageContext"]["eventIds"]));
        assert!(v["messageContext"]["narrative"].is_null());
        for section in v["sections"].as_array().unwrap() {
            assert!(members(&section["eventIds"]).is_subset(&refs));
            for index in section["snapshotAspectIndexes"].as_array().unwrap() {
                assert!(
                    (index.as_u64().unwrap() as usize)
                        < d["snapshot"]["aspects"].as_array().unwrap().len()
                );
            }
        }
    }
    assert_eq!(
        d["overview"]["eventCount"].as_u64().unwrap() as usize,
        events.len()
    );
    assert_eq!(
        d["overview"]["byType"]
            .as_object()
            .unwrap()
            .values()
            .map(|v| v.as_u64().unwrap())
            .sum::<u64>() as usize,
        events.len()
    );
    assert!(members(&d["overview"]["highlightEventIds"]).is_subset(&event_ids));
    for day in d["overview"]["days"].as_array().unwrap() {
        assert!(members(&day["eventIds"]).is_subset(&event_ids));
    }
}
#[test]
fn daily_forecast_has_one_natal_ten_views_thirty_sections_and_all_fixed_target_pairs() {
    let v = valid(&request());
    let d = &v["data"];
    assert_eq!(d["chartKind"], "individualForecast");
    assert_eq!(d["subjectCount"], 1);
    assert_eq!(d["subject"]["id"], "N");
    assert_eq!(
        d["subject"]["context"]["points"].as_array().unwrap().len(),
        26
    );
    assert_eq!(d["snapshot"]["relations"].as_array().unwrap().len(), 260);
    assert_eq!(d["snapshot"]["houseOverlays"].as_array().unwrap().len(), 10);
    assert_eq!(d["domains"].as_object().unwrap().len(), 10);
    assert_eq!(
        d["domains"]
            .as_object()
            .unwrap()
            .values()
            .map(|v| v["sections"].as_array().unwrap().len())
            .sum::<usize>(),
        30
    );
    assert!(d["domains"]
        .as_object()
        .unwrap()
        .values()
        .all(|v| v["messageContext"]["kind"] == "dailyMessage"));
    assert!(d["snapshot"].get("angles").is_none());
    assert!(d["snapshot"].get("houses").is_none());
    assert_eq!(
        v["calculation"]["transitTargetMotion"],
        "fixedNatalLongitudes"
    );
    assert!(d["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"] == "lunarEclipse"));
    closure(d);
}
#[test]
fn natal_is_the_same_calculation_as_individual_natal_and_snapshot_applying_uses_no_birth_speeds() {
    let v = valid(&request());
    let mut n = birth();
    n["operation"] = json!("natal");
    let standalone = valid(&n);
    assert_eq!(v["data"]["subject"]["natal"], standalone["data"]);
    assert_eq!(
        v["data"]["subject"]["calculation"],
        standalone["calculation"]
    );
    let snapshot = &v["data"]["snapshot"];
    for a in snapshot["aspects"].as_array().unwrap() {
        let local = a["transitPointId"]
            .as_str()
            .unwrap()
            .strip_prefix("T:")
            .unwrap();
        let p = snapshot["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == local)
            .unwrap();
        let id = a["natalPointId"]
            .as_str()
            .unwrap()
            .strip_prefix("N:")
            .unwrap();
        let n = v["data"]["subject"]["context"]["points"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap();
        let delta = signed_difference(
            p["longitude"].as_f64().unwrap(),
            n["longitude"].as_f64().unwrap(),
        );
        let speed = p["speed"].as_f64().unwrap();
        let orb = a["orb"].as_f64().unwrap();
        if orb > 1e-6 && speed.abs() > 1e-12 {
            assert_eq!(
                a["applying"],
                json!((delta.abs() - a["angle"].as_f64().unwrap()) * speed * delta.signum() < 0.)
            );
        }
    }
}
#[test]
fn month_events_retain_all_exact_hits_with_residual_and_natal_house_ruler_evidence() {
    let mut r = request();
    r["period"] = json!({"kind":"month","year":2026,"month":3,"utcOffsetMinutes":420});
    r["bodies"] = json!(["mercury"]);
    r["domains"] = json!(["career", "love"]);
    let v = valid(&r);
    let d = &v["data"];
    assert_eq!(d["snapshot"]["relations"].as_array().unwrap().len(), 26);
    assert_eq!(d["overview"]["months"].as_array().unwrap().len(), 1);
    assert_eq!(
        d["domains"]["career"]["messageContext"]["kind"],
        "monthlyOverview"
    );
    closure(d);
    let targets = &d["subject"]["context"]["points"];
    let exact: Vec<&Value> = d["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "natalTransit")
        .collect();
    assert!(!exact.is_empty());
    for e in exact {
        let id = e["details"]["targetPointId"]
            .as_str()
            .unwrap()
            .strip_prefix("N:")
            .unwrap();
        let p = targets
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap();
        let longitude = e["positions"][0]["longitude"].as_f64().unwrap();
        assert!(
            (signed_difference(longitude, p["longitude"].as_f64().unwrap()).abs()
                - e["details"]["angle"].as_f64().unwrap())
            .abs()
                <= 1e-6
        );
        assert_eq!(
            e["personalImpact"]["primaryNatalPointId"],
            e["details"]["targetPointId"]
        );
        assert!(!e["personalImpact"]["affectedNatalHouseIds"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
#[test]
fn annual_eclipses_have_independent_maxima_and_twelve_overview_months() {
    let mut r = request();
    r["period"] = json!({"kind":"year","year":2026});
    r["eventTypes"] = json!(["solarEclipse", "lunarEclipse"]);
    r["aspectRules"] = json!([]);
    r["bodies"] = json!(["saturn"]);
    let v = valid(&r);
    let d = &v["data"];
    assert_eq!(d["events"].as_array().unwrap().len(), 4);
    assert_eq!(d["overview"]["months"].as_array().unwrap().len(), 12);
    assert_eq!(
        d["overview"]["highlightEventIds"].as_array().unwrap().len(),
        4
    );
    for e in d["events"].as_array().unwrap() {
        assert_eq!(e["positions"].as_array().unwrap().len(), 2);
        assert_eq!(e["details"]["visibilityScope"], "global");
        assert_eq!(e["precision"]["method"], "swissEclipseSearch");
        assert!(e["precision"]["residual"].is_null());
        assert_eq!(
            e["personalImpact"]["houseOverlays"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
    closure(d);
}
#[test]
fn empty_aspects_and_disabled_personal_scan_keep_raw_pairs_and_custom_section_support() {
    let mut r = request();
    r["aspectRules"] = json!([]);
    r["eventTypes"] = json!([]);
    r["includeNatalTransits"] = json!(false);
    r["domains"] = json!([]);
    r["customProfiles"] = json!([{"id":"customFocus","bodies":["sun"],"sections":[{"id":"outsideParent","houses":[8]}]}]);
    let v = valid(&r);
    let d = &v["data"];
    assert_eq!(d["events"], json!([]));
    assert_eq!(d["snapshot"]["aspects"], json!([]));
    assert_eq!(d["snapshot"]["relations"].as_array().unwrap().len(), 260);
    assert!(d["domains"].as_object().unwrap().is_empty());
    assert!(members(&d["customDomains"]["customFocus"]["natalHouseIds"]).contains("N:H8"));
    assert!(d["customDomains"]["customFocus"]["primaryNatalHouseIds"]
        .as_array()
        .unwrap()
        .is_empty());
    closure(d);
}
#[test]
fn validation_is_exact_and_precedes_provider_without_partial_payloads() {
    for mutate in [0, 1, 2, 3, 4, 5, 6, 7] {
        let mut r = request();
        match mutate {
            0 => r["period"]["day"] = json!(32),
            1 => r["period"]["month"] = json!(2),
            2 => r["includeNatalTransits"] = Value::Null,
            3 => r["eventTypes"] = json!(["eclipse"]),
            4 => r["bodies"] = json!(["sun", "sun"]),
            5 => r["birth"]["utc"]["year"] = json!(2000.5),
            6 => r["domains"] = json!(["attraction"]),
            _ => r["period"]["utcOffsetMinutes"] = json!(841),
        };
        // February 3 is valid; use an impossible day for the calendar mutation.
        if mutate == 1 {
            r["period"]["day"] = json!(30);
        }
        let v = run(&r);
        assert_eq!(v["errors"][0]["code"], "INVALID_INPUT");
        assert!(v["data"].is_null());
    }
    let raw = r#"{"operation":"forecast","birth":{"utc":{"year":2000.0,"month":1e0,"day":1,"hour":12,"minute":0},"location":{"latitude":0,"longitude":0}},"period":{"kind":"day","year":2.026e3,"month":3e0,"day":3.0,"utcOffsetMinutes":420.0},"eventTypes":[],"aspectRules":[],"includeNatalTransits":false}"#;
    let v: Value = serde_json::from_str(&calculate_json(raw)).unwrap();
    assert_eq!(v["errors"], json!([]));
    let invalid = raw.replace("3.0,", "3.0000000000000001,");
    let v: Value = serde_json::from_str(&calculate_json(&invalid)).unwrap();
    assert_eq!(v["errors"][0]["code"], "INVALID_INPUT");
}
