//! Traceable report context for one person's natal; selection does not interpret facts.
use super::*;

#[derive(Default)]
struct HouseEvidence {
    roles: Vec<String>,
    evidence: Vec<Value>,
}

fn note(notes: &mut [HouseEvidence], number: usize, role: &str, evidence: Value) {
    let entry = &mut notes[number - 1];
    if !entry.roles.iter().any(|r| r == role) {
        entry.roles.push(role.into());
    }
    if !entry.evidence.contains(&evidence) {
        entry.evidence.push(evidence);
    }
}

fn body_houses(
    notes: &mut [HouseEvidence],
    houses: &[Value],
    bodies: &[Value],
    body_id: &str,
    roles: (&str, &str),
    evidence: Value,
) {
    let body = bodies.iter().find(|p| p["id"] == body_id).unwrap();
    note(
        notes,
        body["house"].as_u64().unwrap() as usize,
        roles.0,
        evidence.clone(),
    );
    for house in houses.iter().filter(|h| h["rulerBodyId"] == body_id) {
        note(
            notes,
            house["number"].as_u64().unwrap() as usize,
            roles.1,
            evidence.clone(),
        );
    }
}

fn touches(fact: &Value, ids: &HashSet<&str>) -> bool {
    ids.contains(fact["point1"].as_str().unwrap()) || ids.contains(fact["point2"].as_str().unwrap())
}

pub(super) fn build_with_kind(
    chart_kind: &str,
    natal: &Value,
    houses: &[Value],
    points: &[Value],
    aspects: &[Value],
    advanced: &Value,
    view: &Value,
) -> Value {
    let bodies = natal["placements"].as_array().unwrap();
    let cusps: Vec<f64> = houses
        .iter()
        .map(|h| h["longitude"].as_f64().unwrap())
        .collect();
    let primary_ids: HashSet<&str> = view["pointIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let selected_bodies: Vec<&str> = view["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["placement"]["id"].as_str().unwrap())
        .collect();
    let mut notes: Vec<HouseEvidence> = (0..12).map(|_| HouseEvidence::default()).collect();
    for house in view["houses"].as_array().unwrap() {
        note(
            &mut notes,
            house["number"].as_u64().unwrap() as usize,
            "focus",
            json!({"rule":"profileHouse","houseId":house["id"]}),
        );
    }
    let chains = advanced["dispositorChains"].as_array().unwrap();
    for id in &selected_bodies {
        body_houses(
            &mut notes,
            houses,
            bodies,
            id,
            ("selectedBodyPlacement", "selectedBodyRulership"),
            json!({"rule":"selectedBody","bodyId":id}),
        );
        let chain = chains.iter().find(|c| c["bodyId"] == *id).unwrap();
        for node in chain["path"].as_array().unwrap() {
            let node_id = node.as_str().unwrap();
            body_houses(
                &mut notes,
                houses,
                bodies,
                node_id,
                ("dispositorPlacement", "dispositorRulership"),
                json!({"rule":"dispositorChain","bodyId":node_id,"sourceBodyId":id,"chainId":chain["id"]}),
            );
        }
    }
    for angle in view["angles"].as_array().unwrap() {
        let number = house_for(angle["longitude"].as_f64().unwrap(), Some(&cusps)).unwrap();
        note(
            &mut notes,
            number,
            "focusAnglePlacement",
            json!({"rule":"focusAngle","pointId":angle["id"]}),
        );
    }
    // Only primary contacts select support houses. There is no recursive profile expansion.
    for aspect in view["aspects"].as_array().unwrap() {
        for key in ["point1", "point2"] {
            let id = aspect[key].as_str().unwrap();
            let point = points.iter().find(|p| p["id"] == id).unwrap();
            let evidence =
                json!({"rule":"aspectEndpoint","pointId":id,"aspectIndex":aspect["index"]});
            if point["kind"] == "body" {
                body_houses(
                    &mut notes,
                    houses,
                    bodies,
                    id,
                    ("aspectEndpointPlacement", "aspectEndpointRulership"),
                    evidence,
                );
            } else {
                let number = if point["kind"] == "houseCusp" {
                    point["houseNumber"].as_u64().unwrap() as usize
                } else {
                    house_for(point["longitude"].as_f64().unwrap(), Some(&cusps)).unwrap()
                };
                note(&mut notes, number, "aspectEndpointPlacement", evidence);
            }
        }
    }
    let mut report_houses = Vec::new();
    let mut report_ids = primary_ids.clone();
    let mut house_point_ids = Vec::new();
    for (house, evidence) in houses.iter().zip(notes) {
        if evidence.roles.is_empty() {
            continue;
        }
        let mut ids: HashSet<&str> = HashSet::new();
        ids.insert(house["id"].as_str().unwrap());
        ids.insert(house["rulerBodyId"].as_str().unwrap());
        ids.extend(
            house["occupants"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b["id"].as_str().unwrap()),
        );
        let indexes: Vec<&Value> = aspects
            .iter()
            .filter(|a| touches(a, &ids))
            .map(|a| &a["index"])
            .collect();
        report_ids.extend(ids.iter().copied());
        let mut card = house.clone();
        card["roles"] = json!(evidence.roles);
        card["evidence"] = json!(evidence.evidence);
        card["aspectIndexes"] = json!(indexes);
        card["rulerPlacementHouseId"] = json!(format!("H{}", house["rulerPlacement"]["house"]));
        card["rulerDispositorChainId"] = json!(format!(
            "dispositor:{}",
            house["rulerBodyId"].as_str().unwrap()
        ));
        report_houses.push(card);
        house_point_ids.push(ids);
    }
    let primary_indexes: HashSet<u64> = view["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["index"].as_u64().unwrap())
        .collect();
    let mut report_aspects = Vec::new();
    for aspect in aspects {
        let house_ids: Vec<&Value> = report_houses
            .iter()
            .zip(&house_point_ids)
            .filter(|(_, ids)| touches(aspect, ids))
            .map(|(h, _)| &h["id"])
            .collect();
        let primary = primary_indexes.contains(&aspect["index"].as_u64().unwrap());
        if primary || !house_ids.is_empty() {
            let mut fact = aspect.clone();
            fact["primaryContact"] = json!(primary);
            fact["relatedHouseIds"] = json!(house_ids);
            report_ids.insert(aspect["point1"].as_str().unwrap());
            report_ids.insert(aspect["point2"].as_str().unwrap());
            report_aspects.push(fact);
        }
    }
    let patterns: Vec<&Value> = advanced["aspectPatterns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| {
            p["bodyIds"]
                .as_array()
                .unwrap()
                .iter()
                .any(|id| primary_ids.contains(id.as_str().unwrap()))
        })
        .collect();
    let receptions: Vec<&Value> = advanced["receptions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| {
            p["bodyIds"]
                .as_array()
                .unwrap()
                .iter()
                .any(|id| primary_ids.contains(id.as_str().unwrap()))
        })
        .collect();
    for pattern in patterns.iter().chain(&receptions) {
        report_ids.extend(
            pattern["bodyIds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_str().unwrap()),
        );
    }
    // Include referenced chain nodes without selecting additional support houses/contacts.
    loop {
        let before = report_ids.len();
        let extra: Vec<&str> = chains
            .iter()
            .filter(|c| report_ids.contains(c["bodyId"].as_str().unwrap()))
            .flat_map(|c| {
                c["path"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap())
            })
            .collect();
        report_ids.extend(extra);
        if report_ids.len() == before {
            break;
        }
    }
    let report_points: Vec<&Value> = points
        .iter()
        .filter(|p| report_ids.contains(p["id"].as_str().unwrap()))
        .collect();
    let facts: Vec<&Value> = advanced["bodyStates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| report_ids.contains(f["bodyId"].as_str().unwrap()))
        .collect();
    let report_chains: Vec<&Value> = chains
        .iter()
        .filter(|c| report_ids.contains(c["bodyId"].as_str().unwrap()))
        .collect();
    json!({
        "version":"1.1","chartKind":chart_kind,
        "selectionPolicy":"primaryProfileWithTraceableSupport",
        "focusHouseIds":view["houses"].as_array().unwrap().iter().map(|h| &h["id"]).collect::<Vec<_>>(),
        "houseIds":report_houses.iter().map(|h| &h["id"]).collect::<Vec<_>>(),
        "pointIds":report_points.iter().map(|p| &p["id"]).collect::<Vec<_>>(),
        "houses":report_houses,"points":report_points,"aspects":report_aspects,
        "bodyFacts":facts,"dispositorChains":report_chains,"receptions":receptions,"aspectPatterns":patterns,
        "coverage":{"primaryAspectCount":view["aspects"].as_array().unwrap().len(),
            "reportAspectCount":report_aspects.len(),"relatedHouseCount":report_houses.len(),
            "supportSelection":"primaryBodiesDispositorChainsAndPrimaryAspectEndpoints",
            "houseContacts":"cuspOccupantsAndRuler","patternSelection":"atLeastOnePrimaryBody",
            "recursiveHouseExpansion":false}
    })
}

/// Merge independently selected sections into one report; references avoid nested payload copies.
pub(super) fn merge_sections(
    mut report: Value,
    sections: &[(String, Value, Value)],
    points: &[Value],
    advanced: &Value,
) -> Value {
    for aspect in report["aspects"].as_array_mut().unwrap() {
        aspect["sectionIds"] = json!([]);
    }
    let mut point_ids: HashSet<String> = report["pointIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().into())
        .collect();
    let mut pattern_ids: HashSet<String> = report["aspectPatterns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().into())
        .collect();
    let mut reception_ids: HashSet<String> = report["receptions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().into())
        .collect();
    let mut summaries = Vec::new();
    for (section_id, selected, source) in sections {
        point_ids.extend(
            source["pointIds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_str().unwrap().to_owned()),
        );
        pattern_ids.extend(
            source["aspectPatterns"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["id"].as_str().unwrap().to_owned()),
        );
        reception_ids.extend(
            source["receptions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["id"].as_str().unwrap().to_owned()),
        );
        for card in source["houses"].as_array().unwrap() {
            let cards = report["houses"].as_array_mut().unwrap();
            let index = cards
                .iter()
                .position(|h| h["id"] == card["id"])
                .unwrap_or_else(|| {
                    let mut extra = card.clone();
                    extra["roles"] = json!([]);
                    extra["evidence"] = json!([]);
                    cards.push(extra);
                    cards.len() - 1
                });
            let target = &mut cards[index];
            let role = if card["roles"].as_array().unwrap().contains(&json!("focus")) {
                "sectionFocus"
            } else {
                "sectionSupport"
            };
            if !target["roles"].as_array().unwrap().contains(&json!(role)) {
                target["roles"].as_array_mut().unwrap().push(json!(role));
            }
            for evidence in card["evidence"].as_array().unwrap() {
                let mut evidence = evidence.clone();
                evidence["sectionId"] = json!(section_id);
                if !target["evidence"].as_array().unwrap().contains(&evidence) {
                    target["evidence"].as_array_mut().unwrap().push(evidence);
                }
            }
        }
        for aspect in source["aspects"].as_array().unwrap() {
            let facts = report["aspects"].as_array_mut().unwrap();
            let index = facts
                .iter()
                .position(|a| a["index"] == aspect["index"])
                .unwrap_or_else(|| {
                    let mut extra = aspect.clone();
                    // primaryContact belongs to the parent profile, not to a subsection.
                    extra["primaryContact"] = json!(false);
                    extra["sectionIds"] = json!([]);
                    facts.push(extra);
                    facts.len() - 1
                });
            let target = &mut facts[index];
            target["sectionIds"]
                .as_array_mut()
                .unwrap()
                .push(json!(section_id));
            for house_id in aspect["relatedHouseIds"].as_array().unwrap() {
                if !target["relatedHouseIds"]
                    .as_array()
                    .unwrap()
                    .contains(house_id)
                {
                    target["relatedHouseIds"]
                        .as_array_mut()
                        .unwrap()
                        .push(house_id.clone());
                }
            }
            target["relatedHouseIds"]
                .as_array_mut()
                .unwrap()
                .sort_by_key(|id| id.as_str().unwrap()[1..].parse::<u8>().unwrap());
        }
        summaries.push(json!({"id":section_id,"selectionRules":selected["selectionRules"],
            "primaryPointIds":selected["pointIds"],"focusHouseIds":source["focusHouseIds"],"relatedHouseIds":source["houseIds"],
            "aspectIndexes":source["aspects"].as_array().unwrap().iter().map(|a|&a["index"]).collect::<Vec<_>>(),
            "bodyFactIds":source["bodyFacts"].as_array().unwrap().iter().map(|f|&f["bodyId"]).collect::<Vec<_>>(),
            "dispositorChainIds":source["dispositorChains"].as_array().unwrap().iter().map(|f|&f["id"]).collect::<Vec<_>>(),
            "receptionIds":source["receptions"].as_array().unwrap().iter().map(|f|&f["id"]).collect::<Vec<_>>(),
            "aspectPatternIds":source["aspectPatterns"].as_array().unwrap().iter().map(|f|&f["id"]).collect::<Vec<_>>() }));
    }
    report["houses"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|h| h["number"].as_u64().unwrap());
    report["aspects"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|a| a["index"].as_u64().unwrap());
    report["points"] = json!(points
        .iter()
        .filter(|p| point_ids.contains(p["id"].as_str().unwrap()))
        .collect::<Vec<_>>());
    report["pointIds"] = json!(report["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| &p["id"])
        .collect::<Vec<_>>());
    report["houseIds"] = json!(report["houses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| &h["id"])
        .collect::<Vec<_>>());
    report["bodyFacts"] = json!(advanced["bodyStates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| point_ids.contains(f["bodyId"].as_str().unwrap()))
        .collect::<Vec<_>>());
    report["dispositorChains"] = json!(advanced["dispositorChains"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| point_ids.contains(f["bodyId"].as_str().unwrap()))
        .collect::<Vec<_>>());
    report["receptions"] = json!(advanced["receptions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| reception_ids.contains(f["id"].as_str().unwrap()))
        .collect::<Vec<_>>());
    report["aspectPatterns"] = json!(advanced["aspectPatterns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| pattern_ids.contains(f["id"].as_str().unwrap()))
        .collect::<Vec<_>>());
    report["sections"] = json!(summaries);
    report["coverage"]["reportAspectCount"] = json!(report["aspects"].as_array().unwrap().len());
    report["coverage"]["relatedHouseCount"] = json!(report["houses"].as_array().unwrap().len());
    report["coverage"]["sectionCount"] = json!(sections.len());
    report["coverage"]["sectionSelection"] = json!("independentSelectorsSharedNatal");
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(extra: Value) -> Value {
        let mut request = json!({"operation":"natalDomains",
            "utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},
            "location":{"latitude":10.8231,"longitude":106.6297}});
        for (k, v) in extra.as_object().unwrap() {
            request[k] = v.clone();
        }
        serde_json::from_str(&calculate_json(&request.to_string())).unwrap()
    }

    fn validate_reports(result: &Value) {
        assert_eq!(result["errors"], json!([]));
        assert_eq!(result["data"]["chartKind"], "individualNatal");
        assert_eq!(result["data"]["subjectCount"], 1);
        assert_eq!(result["calculation"]["chartKind"], "individualNatal");
        let context = &result["data"]["context"];
        let houses = context["houses"].as_array().unwrap();
        let aspects = context["aspects"].as_array().unwrap();
        let bodies = result["data"]["natal"]["placements"].as_array().unwrap();
        let ruler_relations = context["houseRulerRelations"].as_array().unwrap();
        assert_eq!(ruler_relations.len(), 66);
        for pair in ruler_relations {
            if pair["sameRuler"] == true {
                assert_eq!(pair["ruler1"], pair["ruler2"]);
                assert!(pair["relationId"].is_null());
                assert_eq!(pair["aspectIndexes"], json!([]));
            } else {
                assert!(context["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["id"] == pair["relationId"]
                        && r["aspectIndexes"] == pair["aspectIndexes"]));
            }
        }
        for view in result["data"]["domains"].as_object().unwrap().values() {
            let report = &view["report"];
            assert_eq!(report["chartKind"], "individualNatal");
            assert_eq!(report["coverage"]["recursiveHouseExpansion"], false);
            let cards = report["houses"].as_array().unwrap();
            let point_ids: HashSet<&str> = report["pointIds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            let card_ids: HashSet<&str> = cards.iter().map(|h| h["id"].as_str().unwrap()).collect();
            assert_eq!(cards.len(), card_ids.len());
            assert_eq!(point_ids.len(), report["points"].as_array().unwrap().len());
            for house in view["houses"].as_array().unwrap() {
                let card = cards.iter().find(|c| c["id"] == house["id"]).unwrap();
                assert!(card["roles"].as_array().unwrap().contains(&json!("focus")));
            }
            for selected in view["bodies"].as_array().unwrap() {
                let id = &selected["placement"]["id"];
                let number = selected["placement"]["house"].as_u64().unwrap();
                let card = cards.iter().find(|h| h["number"] == number).unwrap();
                assert!(card["roles"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("selectedBodyPlacement")));
                for ruled in houses.iter().filter(|h| &h["rulerBodyId"] == id) {
                    assert!(card_ids.contains(ruled["id"].as_str().unwrap()));
                }
            }
            for card in cards {
                let global = houses.iter().find(|h| h["id"] == card["id"]).unwrap();
                for (key, value) in global.as_object().unwrap() {
                    assert_eq!(&card[key], value);
                }
                assert!(!card["evidence"].as_array().unwrap().is_empty());
                let roles = card["roles"].as_array().unwrap();
                assert_eq!(
                    roles.len(),
                    roles
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect::<HashSet<_>>()
                        .len()
                );
                for index in card["aspectIndexes"].as_array().unwrap() {
                    let fact = &aspects[index.as_u64().unwrap() as usize];
                    let contains_endpoint = |key: &str| {
                        fact[key] == card["id"]
                            || fact[key] == card["rulerBodyId"]
                            || card["occupants"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|p| p["id"] == fact[key])
                    };
                    assert!(contains_endpoint("point1") || contains_endpoint("point2"));
                }
            }
            for aspect in report["aspects"].as_array().unwrap() {
                let fact = &aspects[aspect["index"].as_u64().unwrap() as usize];
                assert_eq!(fact["point1"], aspect["point1"]);
                assert_eq!(fact["point2"], aspect["point2"]);
                assert!(point_ids.contains(aspect["point1"].as_str().unwrap()));
                assert!(point_ids.contains(aspect["point2"].as_str().unwrap()));
            }
            for aspect in view["aspects"].as_array().unwrap() {
                assert!(report["aspects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| a["index"] == aspect["index"] && a["primaryContact"] == true));
            }
            for chain in report["dispositorChains"].as_array().unwrap() {
                assert!(point_ids.contains(chain["bodyId"].as_str().unwrap()));
                for id in chain["path"].as_array().unwrap() {
                    assert!(point_ids.contains(id.as_str().unwrap()));
                }
            }
            for fact in report["bodyFacts"].as_array().unwrap() {
                assert!(bodies.iter().any(|p| p["id"] == fact["bodyId"]));
            }
        }
    }

    #[test]
    fn individual_reports_have_traceable_related_houses_and_contacts() {
        validate_reports(&run(json!({})));
        validate_reports(&run(
            json!({"rulership":"modern","houseSystem":"wholeSign"}),
        ));
    }

    #[test]
    fn disabled_aspects_keep_structural_support_without_aspect_evidence() {
        let result = run(json!({"aspectRules":[],"domains":["career"]}));
        validate_reports(&result);
        let report = &result["data"]["domains"]["career"]["report"];
        assert_eq!(report["aspects"], json!([]));
        assert_eq!(report["aspectPatterns"], json!([]));
        assert_eq!(
            result["data"]["context"]["relations"]
                .as_array()
                .unwrap()
                .len(),
            325
        );
        assert_eq!(result["data"]["domains"].as_object().unwrap().len(), 1);
        for card in report["houses"].as_array().unwrap() {
            assert!(!card["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["rule"] == "aspectEndpoint"));
        }
    }

    #[test]
    fn natal_domains_rejects_second_subject_fields() {
        for extra in [
            json!({"otherUtc":{}}),
            json!({"otherPositions":[]}),
            json!({"personB":{}}),
        ] {
            assert_eq!(run(extra)["errors"][0]["code"], "INVALID_INPUT");
        }
    }
}
