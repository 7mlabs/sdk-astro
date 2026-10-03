//! Contract tests across request parsing, shared natal facts and report references.
use super::*;

fn request() -> Value {
    json!({"operation":"natalDomains",
        "utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},
        "location":{"latitude":10.8231,"longitude":106.6297}})
}

fn run(extra: Value) -> Value {
    let mut input = request();
    for (key, value) in extra.as_object().unwrap() {
        input[key] = value.clone();
    }
    serde_json::from_str(&calculate_json(&input.to_string())).unwrap()
}

fn ids<'a>(values: &'a Value, key: &str) -> HashSet<&'a str> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value[key].as_str().unwrap())
        .collect()
}

fn refs(values: &Value) -> HashSet<&str> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect()
}

fn indexes(values: &Value) -> HashSet<u64> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap())
        .collect()
}

fn find<'a>(values: &'a Value, key: &str, id: &Value) -> &'a Value {
    values
        .as_array()
        .unwrap()
        .iter()
        .find(|value| &value[key] == id)
        .unwrap()
}

fn validate_view(data: &Value, view: &Value) {
    let context = &data["context"];
    let advanced = &context["advanced"];
    let global_points = ids(&context["points"], "id");
    let global_houses = ids(&context["houses"], "id");
    let global_bodies = ids(&data["natal"]["placements"], "id");
    let global_chains = ids(&advanced["dispositorChains"], "id");
    let global_relations = ids(&context["relations"], "id");
    let primary_points = refs(&view["pointIds"]);
    let primary_aspects: HashSet<u64> = view["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|aspect| aspect["index"].as_u64().unwrap())
        .collect();
    assert!(primary_points.is_subset(&global_points));
    assert!(refs(&view["relationIds"]).is_subset(&global_relations));
    for aspect in view["aspects"].as_array().unwrap() {
        assert!(!refs(&aspect["selectedEndpoints"]).is_empty());
        assert!(refs(&aspect["selectedEndpoints"]).is_subset(&primary_points));
    }

    let report = &view["report"];
    assert_eq!(report["chartKind"], "individualNatal");
    let report_points = ids(&report["points"], "id");
    let report_houses = ids(&report["houses"], "id");
    let report_bodies = ids(&report["bodyFacts"], "bodyId");
    let report_chains = ids(&report["dispositorChains"], "id");
    let report_receptions = ids(&report["receptions"], "id");
    let report_patterns = ids(&report["aspectPatterns"], "id");
    let section_ids = ids(&report["sections"], "id");
    let report_aspects: HashSet<u64> = report["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|aspect| aspect["index"].as_u64().unwrap())
        .collect();
    assert_eq!(report_points, refs(&report["pointIds"]));
    assert_eq!(report_houses, refs(&report["houseIds"]));
    assert_eq!(
        report_points.len(),
        report["points"].as_array().unwrap().len()
    );
    assert_eq!(
        report_houses.len(),
        report["houses"].as_array().unwrap().len()
    );
    assert_eq!(
        report_aspects.len(),
        report["aspects"].as_array().unwrap().len()
    );
    assert_eq!(
        section_ids.len(),
        report["sections"].as_array().unwrap().len()
    );
    assert!(primary_points.is_subset(&report_points));
    assert!(report_points.is_subset(&global_points));
    assert!(report_houses.is_subset(&global_houses));
    assert!(report_bodies.is_subset(&global_bodies));

    for point in report["points"].as_array().unwrap() {
        assert_eq!(point, find(&context["points"], "id", &point["id"]));
    }
    for house in report["houses"].as_array().unwrap() {
        let global = find(&context["houses"], "id", &house["id"]);
        for (key, value) in global.as_object().unwrap() {
            assert_eq!(&house[key], value);
        }
        assert!(global_houses.contains(house["rulerPlacementHouseId"].as_str().unwrap()));
        assert!(report_chains.contains(house["rulerDispositorChainId"].as_str().unwrap()));
        for index in house["aspectIndexes"].as_array().unwrap() {
            let aspect = &context["aspects"][index.as_u64().unwrap() as usize];
            assert!(aspect.is_object());
            assert!(report_aspects.contains(&index.as_u64().unwrap()));
        }
        for evidence in house["evidence"].as_array().unwrap() {
            for key in ["bodyId", "sourceBodyId"] {
                if let Some(id) = evidence[key].as_str() {
                    assert!(global_bodies.contains(id), "{key}: {id}");
                }
            }
            if let Some(id) = evidence["houseId"].as_str() {
                assert!(global_houses.contains(id));
            }
            if let Some(id) = evidence["pointId"].as_str() {
                assert!(global_points.contains(id));
            }
            if let Some(id) = evidence["chainId"].as_str() {
                assert!(global_chains.contains(id));
            }
            if let Some(id) = evidence["sectionId"].as_str() {
                assert!(section_ids.contains(id));
            }
            if let Some(index) = evidence["aspectIndex"].as_u64() {
                assert!(context["aspects"][index as usize].is_object());
            }
        }
    }
    for aspect in report["aspects"].as_array().unwrap() {
        let global = &context["aspects"][aspect["index"].as_u64().unwrap() as usize];
        for (key, value) in global.as_object().unwrap() {
            assert_eq!(&aspect[key], value);
        }
        assert!(report_points.contains(aspect["point1"].as_str().unwrap()));
        assert!(report_points.contains(aspect["point2"].as_str().unwrap()));
        assert!(refs(&aspect["relatedHouseIds"]).is_subset(&report_houses));
        assert_eq!(
            aspect["primaryContact"].as_bool().unwrap(),
            primary_aspects.contains(&aspect["index"].as_u64().unwrap())
        );
        for id in aspect["sectionIds"].as_array().unwrap() {
            let section = find(&report["sections"], "id", id);
            assert!(indexes(&section["aspectIndexes"]).contains(&aspect["index"].as_u64().unwrap()));
        }
    }
    for body in report["bodyFacts"].as_array().unwrap() {
        assert_eq!(
            body,
            find(&advanced["bodyStates"], "bodyId", &body["bodyId"])
        );
        assert!(report_points.contains(body["bodyId"].as_str().unwrap()));
    }
    for chain in report["dispositorChains"].as_array().unwrap() {
        assert_eq!(
            chain,
            find(&advanced["dispositorChains"], "id", &chain["id"])
        );
        assert!(report_points.contains(chain["bodyId"].as_str().unwrap()));
        assert!(refs(&chain["path"]).is_subset(&report_points));
    }
    for (key, global_key) in [
        ("receptions", "receptions"),
        ("aspectPatterns", "aspectPatterns"),
    ] {
        for fact in report[key].as_array().unwrap() {
            assert_eq!(fact, find(&advanced[global_key], "id", &fact["id"]));
            assert!(refs(&fact["bodyIds"]).is_subset(&report_points));
            if let Some(indexes) = fact["aspectIndexes"].as_array() {
                for index in indexes {
                    assert!(context["aspects"][index.as_u64().unwrap() as usize].is_object());
                }
            }
        }
    }
    for section in report["sections"].as_array().unwrap() {
        let definition = find(&view["definition"]["sections"], "id", &section["id"]);
        for key in ["houses", "bodies", "angles"] {
            assert_eq!(section["selectionRules"][key], definition[key]);
        }
        assert!(refs(&section["primaryPointIds"]).is_subset(&report_points));
        assert!(refs(&section["focusHouseIds"]).is_subset(&report_houses));
        assert!(refs(&section["relatedHouseIds"]).is_subset(&report_houses));
        assert!(indexes(&section["aspectIndexes"]).is_subset(&report_aspects));
        assert!(refs(&section["bodyFactIds"]).is_subset(&report_bodies));
        assert!(refs(&section["dispositorChainIds"]).is_subset(&report_chains));
        assert!(refs(&section["receptionIds"]).is_subset(&report_receptions));
        assert!(refs(&section["aspectPatternIds"]).is_subset(&report_patterns));
        for index in section["aspectIndexes"].as_array().unwrap() {
            let aspect = find(&report["aspects"], "index", index);
            assert!(aspect["sectionIds"]
                .as_array()
                .unwrap()
                .contains(&section["id"]));
        }
        for nested in [
            "report",
            "houses",
            "points",
            "aspects",
            "bodyFacts",
            "dispositorChains",
            "receptions",
            "aspectPatterns",
        ] {
            assert!(
                section.get(nested).is_none(),
                "Section contains nested {nested}"
            );
        }
    }
    assert_eq!(
        report["coverage"]["primaryAspectCount"],
        json!(primary_aspects.len())
    );
    assert_eq!(
        report["coverage"]["reportAspectCount"],
        json!(report_aspects.len())
    );
    assert_eq!(
        report["coverage"]["relatedHouseCount"],
        json!(report_houses.len())
    );
    assert_eq!(report["coverage"]["sectionCount"], json!(section_ids.len()));
}

#[test]
fn default_ten_profiles_and_thirty_sections_resolve_shared_facts() {
    let result = run(json!({"utc":{"year":1980,"month":1,"day":1,"hour":12,"minute":0}}));
    assert_eq!(result["errors"], json!([]));
    let data = &result["data"];
    assert_eq!(data["chartKind"], "individualNatal");
    assert_eq!(data["subjectCount"], 1);
    let catalog = &data["profileCatalog"];
    assert_eq!(catalog.as_array().unwrap().len(), 10);
    assert_eq!(ids(catalog, "id").len(), 10);
    assert_eq!(data["domains"].as_object().unwrap().len(), 10);
    assert_eq!(data["customDomains"], json!({}));
    assert!(!data["context"]["advanced"]["receptions"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!data["context"]["advanced"]["aspectPatterns"]
        .as_array()
        .unwrap()
        .is_empty());
    let mut section_count = 0;
    for definition in catalog.as_array().unwrap() {
        let view = &data["domains"][definition["id"].as_str().unwrap()];
        assert_eq!(view["origin"], "builtin");
        assert_eq!(view["definition"], *definition);
        assert_eq!(view["profileVersion"], "2.0");
        assert_eq!(view["profileId"], "sevenmlabs-domain-selection");
        validate_view(data, view);
        section_count += view["report"]["sections"].as_array().unwrap().len();
    }
    assert_eq!(section_count, 30);
}

#[test]
fn custom_only_request_normalizes_raw_house_literals_and_defaults() {
    let raw = r#"{"operation":"natalDomains","utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"domains":[],"customProfiles":[{"id":"study","houses":[1.0,9e0],"bodies":["mercury"],"sections":[{"id":"learning","houses":[3e0]}]}]}"#;
    let result: Value = serde_json::from_str(&calculate_json(raw)).unwrap();
    assert_eq!(result["errors"], json!([]));
    let data = &result["data"];
    assert_eq!(data["domains"], json!({}));
    assert_eq!(data["customDomains"].as_object().unwrap().len(), 1);
    let view = &data["customDomains"]["study"];
    assert_eq!(view["origin"], "custom");
    assert_eq!(view["profileId"], "study");
    assert_eq!(view["profileVersion"], "1.0");
    assert_eq!(
        view["definition"],
        json!({"id":"study","version":"1.0","houses":[1,9],"bodies":["mercury"],"angles":[],"sections":[{"id":"learning","houses":[3],"bodies":[],"angles":[]}]})
    );
    validate_view(data, view);
    assert_eq!(view["report"]["focusHouseIds"], json!(["H1", "H9"]));
}

#[test]
fn custom_selectors_matching_career_preserve_primary_geometry_and_natal() {
    let result = run(json!({"domains":["career"],"customProfiles":[{
        "id":"myCareer","houses":[2,6,10],"bodies":["sun","mercury","jupiter","saturn"],"angles":["midheaven"]}]}));
    assert_eq!(result["errors"], json!([]));
    let data = &result["data"];
    let builtin = &data["domains"]["career"];
    let custom = &data["customDomains"]["myCareer"];
    for key in [
        "selectionRules",
        "houses",
        "bodies",
        "angles",
        "pointIds",
        "relationIds",
        "aspects",
    ] {
        assert_eq!(builtin[key], custom[key], "Primary field {key}");
    }
    validate_view(data, custom);
    assert_eq!(custom["report"]["sections"], json!([]));
    let natal = run(
        json!({"operation":"natal","aspectRules":domains::extended_rules().iter().map(|r|json!({"angle":r.angle,"maxOrb":r.max_orb})).collect::<Vec<_>>()}),
    );
    assert_eq!(natal["errors"], json!([]));
    assert_eq!(data["natal"], natal["data"]);
}

#[test]
fn section_selectors_extend_report_without_changing_parent_primary_selection() {
    let base = json!({"domains":[],"aspectRules":[],"customProfiles":[{"id":"focused","angles":["ascendant"]}]});
    let baseline = run(base.clone());
    let mut extra = base;
    extra["customProfiles"][0]["sections"] =
        json!([{"id":"extended","houses":[9],"bodies":["pluto"],"angles":["imumCoeli"]}]);
    let expanded = run(extra);
    assert_eq!(baseline["errors"], json!([]));
    assert_eq!(expanded["errors"], json!([]));
    let before = &baseline["data"]["customDomains"]["focused"];
    let after = &expanded["data"]["customDomains"]["focused"];
    for key in [
        "selectionRules",
        "houses",
        "bodies",
        "angles",
        "pointIds",
        "relationIds",
        "aspects",
    ] {
        assert_eq!(before[key], after[key], "Parent field {key}");
    }
    assert_eq!(after["pointIds"], json!(["ascendant"]));
    assert_eq!(after["report"]["focusHouseIds"], json!([]));
    assert_eq!(after["report"]["aspects"], json!([]));
    assert_eq!(
        after["report"]["sections"][0]["focusHouseIds"],
        json!(["H9"])
    );
    assert!(!refs(&before["report"]["houseIds"]).contains("H9"));
    let ninth = find(&after["report"]["houses"], "id", &json!("H9"));
    assert!(ninth["roles"]
        .as_array()
        .unwrap()
        .contains(&json!("sectionFocus")));
    assert!(!ninth["roles"].as_array().unwrap().contains(&json!("focus")));
    assert!(ninth["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["rule"] == "profileHouse"
            && e["houseId"] == "H9"
            && e["sectionId"] == "extended"));
    validate_view(&expanded["data"], after);
}

#[test]
fn section_contacts_keep_parent_primary_contact_flags() {
    let result = run(json!({"domains":[],"customProfiles":[{
        "id":"focused","angles":["ascendant"],"sections":[{"id":"outer","bodies":["pluto"]}]}]}));
    assert_eq!(result["errors"], json!([]));
    let data = &result["data"];
    let view = &data["customDomains"]["focused"];
    validate_view(data, view);
    assert_eq!(view["pointIds"], json!(["ascendant"]));
    let section = &view["report"]["sections"][0];
    assert_eq!(section["primaryPointIds"], json!(["pluto"]));
    let distinct: Vec<&Value> = view["report"]["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| {
            (a["point1"] == "pluto" || a["point2"] == "pluto")
                && a["point1"] != "ascendant"
                && a["point2"] != "ascendant"
        })
        .collect();
    assert!(!distinct.is_empty());
    for aspect in distinct {
        assert_eq!(aspect["primaryContact"], false);
        assert!(aspect["sectionIds"]
            .as_array()
            .unwrap()
            .contains(&json!("outer")));
        assert!(indexes(&section["aspectIndexes"]).contains(&aspect["index"].as_u64().unwrap()));
    }
}

#[test]
fn runtime_rejects_invalid_custom_profile_settings() {
    let valid = json!({"id":"customA","houses":[1]});
    let mut too_many: Vec<Value> = (0..8)
        .map(|i| json!({"id":format!("custom{i}"),"houses":[1]}))
        .collect();
    too_many.push(json!({"id":"custom8","houses":[1]}));
    for extra in [
        json!({"domains":[],"customProfiles":[]}),
        json!({"customProfiles":[]}),
        json!({"customProfiles":null}),
        json!({"customProfiles":[valid.clone(),valid]}),
        json!({"customProfiles":[{"id":"career","houses":[1]}]}),
        json!({"customProfiles":[{"id":"constructor","houses":[1]}]}),
        json!({"customProfiles":[{"id":"x","houses":[1],"unknown":true}]}),
        json!({"customProfiles":[{"id":"x","houses":null}]}),
        json!({"customProfiles":[{"id":"x"}]}),
        json!({"customProfiles":too_many}),
        json!({"domains":["customA"],"customProfiles":[{"id":"customA","houses":[1]}]}),
    ] {
        let result = run(extra.clone());
        assert_eq!(result["errors"][0]["code"], "INVALID_INPUT", "{extra}");
    }
}

#[test]
fn basic_natal_rejects_custom_profiles_even_when_empty_or_null() {
    for custom in [json!([]), json!(null), json!([{"id":"x","houses":[1]}])] {
        assert_eq!(
            run(json!({"operation":"natal","customProfiles":custom}))["errors"][0]["code"],
            "INVALID_INPUT"
        );
    }
}
