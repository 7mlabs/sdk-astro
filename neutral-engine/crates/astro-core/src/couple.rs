//! Cross-chart geometry between two independently computed natal skies.
//! Optional symbolic composite reuses both computed skies; no score or interpretation.
use super::*;
use serde_json::value::RawValue;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoupleRequest {
    operation: String,
    person_a: Box<RawValue>,
    person_b: Box<RawValue>,
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
    #[serde(default, deserialize_with = "natal::present")]
    composite: Option<composite::Options>,
}

fn qualified(chart: &str, id: &str) -> String {
    format!("{chart}:{id}")
}

fn point_ids(context: &Value, selected: &HashSet<String>) -> Vec<Value> {
    context["points"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| selected.contains(p["id"].as_str().unwrap()))
        .map(|p| p["id"].clone())
        .collect()
}

fn local_context(natal: Value, rules: &[AspectRule], rulership: &str) -> (Value, Value) {
    let mut data = domains::build(natal, rules, &[], rulership, &[]);
    (data["natal"].take(), data["context"].take())
}

fn cross_relation_id(source_chart: &str, source: &str, target: &str) -> String {
    if source_chart == "A" {
        format!("A:{source}:B:{target}")
    } else {
        format!("A:{target}:B:{source}")
    }
}

fn overlays(
    source_chart: &str,
    target_chart: &str,
    subjects: &Value,
    context: &Value,
) -> Vec<Value> {
    let source = &subjects[source_chart];
    let target = &subjects[target_chart];
    let cusps: Vec<f64> = target["natal"]["houseCusps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    source["natal"]["placements"].as_array().unwrap().iter().map(|body| {
        let body_id = body["id"].as_str().unwrap();
        let number = house_for(body["longitude"].as_f64().unwrap(), Some(&cusps)).unwrap();
        let target_house = &target["context"]["houses"][number - 1];
        let target_id = target_house["id"].as_str().unwrap();
        let ruler_id = target_house["rulerBodyId"].as_str().unwrap();
        let cusp_relation = cross_relation_id(source_chart, body_id, target_id);
        let ruler_relation = cross_relation_id(source_chart, body_id, ruler_id);
        let point_id = qualified(source_chart, body_id);
        let target_house_id = qualified(target_chart, target_id);
        let mut aspect_indexes = Vec::new();
        for relation in context["relations"].as_array().unwrap().iter().filter(|r| r["id"] == cusp_relation || r["id"] == ruler_relation) {
            for index in relation["aspectIndexes"].as_array().unwrap() {
                if !aspect_indexes.contains(index) { aspect_indexes.push(index.clone()); }
            }
        }
        aspect_indexes.sort_by_key(|v| v.as_u64().unwrap());
        json!({"id":format!("{point_id}->{target_house_id}"),"sourceChartId":source_chart,"targetChartId":target_chart,
            "pointId":point_id,"localBodyId":body_id,"sourceHouseId":qualified(source_chart,&format!("H{}",body["house"])),
            "targetHouseId":target_house_id,"targetHouseNumber":number,"targetRulerPointId":qualified(target_chart,ruler_id),
            "targetCuspRelationId":cusp_relation,"targetRulerRelationId":ruler_relation,"aspectIndexes":aspect_indexes})
    }).collect()
}

fn build_context(subjects: &Value, rules: &[AspectRule]) -> Value {
    let mut points = Vec::new();
    for chart in ["A", "B"] {
        for point in subjects[chart]["context"]["points"].as_array().unwrap() {
            let mut point = point.clone();
            let local_id = point["id"].as_str().unwrap().to_owned();
            point["id"] = json!(qualified(chart, &local_id));
            point["localId"] = json!(local_id);
            point["chartId"] = json!(chart);
            points.push(point);
        }
    }
    let positions: Vec<Position> = points
        .iter()
        .map(|p| Position {
            id: p["id"].as_str().unwrap().into(),
            longitude: p["longitude"].as_f64().unwrap(),
            speed: None,
        })
        .collect();
    let aspect_facts: Vec<Value> = aspects(&positions[..26], Some(&positions[26..]), rules).into_iter().enumerate().map(|(index, a)| {
        json!({"index":index,"relationId":format!("{}:{}",a.body1,a.body2),"point1":a.body1,"point2":a.body2,
            "angle":a.angle,"separation":a.separation,"orb":a.orb,"maxOrb":a.max_orb,"applying":null})
    }).collect();
    let relations: Vec<Value> = positions[..26].iter().flat_map(|a| {
        let aspect_facts = &aspect_facts;
        positions[26..].iter().map(move |b| {
            let delta = signed_difference(b.longitude, a.longitude);
            let id = format!("{}:{}",a.id,b.id);
            let indexes: Vec<&Value> = aspect_facts.iter().filter(|v| v["relationId"] == id).map(|v| &v["index"]).collect();
            json!({"id":id,"point1":a.id,"point2":b.id,"signedDelta":delta,"separation":delta.abs(),"aspectIndexes":indexes})
        })
    }).collect();
    let house_ruler_relations: Vec<Value> = subjects["A"]["context"]["houses"].as_array().unwrap().iter().flat_map(|first| {
        let relations = &relations;
        subjects["B"]["context"]["houses"].as_array().unwrap().iter().map(move |second| {
            let a = first["rulerBodyId"].as_str().unwrap();
            let b = second["rulerBodyId"].as_str().unwrap();
            let relation_id = cross_relation_id("A",a,b);
            let relation = relations.iter().find(|r| r["id"] == relation_id).unwrap();
            let h1 = qualified("A",first["id"].as_str().unwrap());
            let h2 = qualified("B",second["id"].as_str().unwrap());
            json!({"id":format!("{h1}:{h2}"),"house1":h1,"house2":h2,"ruler1":qualified("A",a),"ruler2":qualified("B",b),
                "sameBodyId":a == b,"relationId":relation_id,"aspectIndexes":relation["aspectIndexes"]})
        })
    }).collect();
    let mut context = json!({"points":points,"relations":relations,"aspects":aspect_facts,"houseRulerRelations":house_ruler_relations,
        "coverage":{"subjects":2,"points":52,"pointPairs":676,"bodyBodyPairs":100,"overlays":20,"houseRulerRelations":144,
            "aspectMatching":"configuredRulesOnly","houseAssignment":"eclipticLongitude","crossApplying":"undefinedForDifferentBirthInstants"}});
    context["overlaysAtoB"] = json!(overlays("A", "B", subjects, &context));
    context["overlaysBtoA"] = json!(overlays("B", "A", subjects, &context));
    context
}

struct Selection {
    local: Value,
    point_ids: HashSet<String>,
    focus_houses: HashSet<String>,
    aspect_indexes: HashSet<u64>,
    overlay_ids: HashSet<String>,
    ruler_ids: HashSet<String>,
}

fn selection_rules(profile: &profiles::Profile) -> Value {
    json!({"houses":profile.houses,"bodies":profile.bodies,"angles":profile.angles,
        "aspectSelection":"atLeastOneSelectedEndpoint","overlaySelection":"selectedSourceBodyOrTargetFocusHouse",
        "houseRulerSelection":"selectedHouseOrRulerEndpoint"})
}

fn all_overlays(context: &Value) -> impl Iterator<Item = &Value> {
    context["overlaysAtoB"]
        .as_array()
        .unwrap()
        .iter()
        .chain(context["overlaysBtoA"].as_array().unwrap())
}

fn select(subjects: &Value, context: &Value, profile: &profiles::Profile) -> Selection {
    let mut local = json!({});
    let mut point_ids = HashSet::new();
    let mut focus_houses = HashSet::new();
    for chart in ["A", "B"] {
        let mut selected = domains::select_context(
            &subjects[chart]["natal"],
            &subjects[chart]["context"],
            profile,
        );
        point_ids.extend(
            selected["pointIds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| qualified(chart, id.as_str().unwrap())),
        );
        focus_houses.extend(
            selected["houses"]
                .as_array()
                .unwrap()
                .iter()
                .map(|house| qualified(chart, house["id"].as_str().unwrap())),
        );
        selected["chartId"] = json!(chart);
        selected["referenceScope"] = json!("subjectLocalNatal");
        local[chart] = selected;
    }
    let aspect_indexes = context["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| touches(a, &point_ids))
        .map(|a| a["index"].as_u64().unwrap())
        .collect();
    let overlay_ids = all_overlays(context)
        .filter(|o| {
            point_ids.contains(o["pointId"].as_str().unwrap())
                || focus_houses.contains(o["targetHouseId"].as_str().unwrap())
        })
        .map(|o| o["id"].as_str().unwrap().into())
        .collect();
    let ruler_ids = context["houseRulerRelations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            focus_houses.contains(r["house1"].as_str().unwrap())
                || focus_houses.contains(r["house2"].as_str().unwrap())
                || point_ids.contains(r["ruler1"].as_str().unwrap())
                || point_ids.contains(r["ruler2"].as_str().unwrap())
        })
        .map(|r| r["id"].as_str().unwrap().into())
        .collect();
    Selection {
        local,
        point_ids,
        focus_houses,
        aspect_indexes,
        overlay_ids,
        ruler_ids,
    }
}

fn touches(aspect: &Value, point_ids: &HashSet<String>) -> bool {
    point_ids.contains(aspect["point1"].as_str().unwrap())
        || point_ids.contains(aspect["point2"].as_str().unwrap())
}

fn selected_aspect(aspect: &Value, point_ids: &HashSet<String>) -> Value {
    let mut fact = aspect.clone();
    fact["selectedEndpoints"] = json!(["point1", "point2"]
        .into_iter()
        .filter(|key| point_ids.contains(aspect[*key].as_str().unwrap()))
        .map(|key| aspect[key].clone())
        .collect::<Vec<_>>());
    fact
}

fn selected_ids<'a>(
    values: impl Iterator<Item = &'a Value>,
    selected: &HashSet<String>,
) -> Vec<Value> {
    values
        .filter(|v| selected.contains(v["id"].as_str().unwrap()))
        .map(|v| v["id"].clone())
        .collect()
}

struct Support {
    point_ids: HashSet<String>,
    house_ids: HashSet<String>,
    body_ids: HashSet<String>,
    chain_refs: Vec<Value>,
    reception_refs: Vec<Value>,
    pattern_refs: Vec<Value>,
}

fn note_house_for_point(
    subjects: &Value,
    chart: &str,
    local_id: &str,
    houses: &mut HashSet<String>,
) {
    let subject = &subjects[chart];
    let point = subject["context"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == local_id)
        .unwrap();
    let number = if point["kind"] == "houseCusp" {
        point["houseNumber"].as_u64().unwrap() as usize
    } else {
        let cusps: Vec<f64> = subject["natal"]["houseCusps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        house_for(point["longitude"].as_f64().unwrap(), Some(&cusps)).unwrap()
    };
    houses.insert(qualified(chart, &format!("H{number}")));
    if point["kind"] == "body" {
        for house in subject["context"]["houses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["rulerBodyId"] == local_id)
        {
            houses.insert(qualified(chart, house["id"].as_str().unwrap()));
        }
    }
}

fn support(subjects: &Value, context: &Value, selection: &Selection) -> Support {
    let mut points = selection.point_ids.clone();
    let mut houses = selection.focus_houses.clone();
    for aspect in context["aspects"].as_array().unwrap().iter().filter(|a| {
        selection
            .aspect_indexes
            .contains(&a["index"].as_u64().unwrap())
    }) {
        points.insert(aspect["point1"].as_str().unwrap().into());
        points.insert(aspect["point2"].as_str().unwrap().into());
    }
    for overlay in
        all_overlays(context).filter(|o| selection.overlay_ids.contains(o["id"].as_str().unwrap()))
    {
        for field in ["pointId", "targetHouseId", "targetRulerPointId"] {
            points.insert(overlay[field].as_str().unwrap().into());
        }
        for field in ["sourceHouseId", "targetHouseId"] {
            houses.insert(overlay[field].as_str().unwrap().into());
        }
    }
    for relation in context["houseRulerRelations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| selection.ruler_ids.contains(r["id"].as_str().unwrap()))
    {
        for field in ["house1", "house2"] {
            houses.insert(relation[field].as_str().unwrap().into());
        }
        for field in ["house1", "house2", "ruler1", "ruler2"] {
            points.insert(relation[field].as_str().unwrap().into());
        }
    }
    // Support facts do not select additional cross contacts. Natal chain paths are local only.
    for chart in ["A", "B"] {
        let selected_bodies: HashSet<String> = subjects[chart]["natal"]["placements"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| points.contains(&qualified(chart, p["id"].as_str().unwrap())))
            .map(|p| p["id"].as_str().unwrap().into())
            .collect();
        for chain in subjects[chart]["context"]["advanced"]["dispositorChains"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| selected_bodies.contains(c["bodyId"].as_str().unwrap()))
        {
            for id in chain["path"].as_array().unwrap() {
                points.insert(qualified(chart, id.as_str().unwrap()));
            }
        }
    }
    for id in &points {
        let (chart, local_id) = id.split_once(':').unwrap();
        note_house_for_point(subjects, chart, local_id, &mut houses);
    }
    for chart in ["A", "B"] {
        for house in subjects[chart]["context"]["houses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| houses.contains(&qualified(chart, h["id"].as_str().unwrap())))
        {
            points.insert(qualified(chart, house["id"].as_str().unwrap()));
            points.insert(qualified(chart, house["rulerBodyId"].as_str().unwrap()));
            for occupant in house["occupants"].as_array().unwrap() {
                points.insert(qualified(chart, occupant["id"].as_str().unwrap()));
            }
        }
    }
    let body_ids: HashSet<String> = context["points"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["kind"] == "body" && points.contains(p["id"].as_str().unwrap()))
        .map(|p| p["id"].as_str().unwrap().into())
        .collect();
    let mut chain_refs = Vec::new();
    let mut reception_refs = Vec::new();
    let mut pattern_refs = Vec::new();
    for chart in ["A", "B"] {
        let advanced = &subjects[chart]["context"]["advanced"];
        for chain in advanced["dispositorChains"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| body_ids.contains(&qualified(chart, c["bodyId"].as_str().unwrap())))
        {
            chain_refs.push(json!({"chartId":chart,"id":chain["id"]}));
        }
        for (key, output) in [
            ("receptions", &mut reception_refs),
            ("aspectPatterns", &mut pattern_refs),
        ] {
            for fact in advanced[key].as_array().unwrap().iter().filter(|v| {
                v["bodyIds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|id| body_ids.contains(&qualified(chart, id.as_str().unwrap())))
            }) {
                output.push(json!({"chartId":chart,"id":fact["id"]}));
            }
        }
    }
    Support {
        point_ids: points,
        house_ids: houses,
        body_ids,
        chain_refs,
        reception_refs,
        pattern_refs,
    }
}

fn merge_refs(target: &mut Vec<Value>, source: &[Value]) {
    for reference in source {
        if !target.contains(reference) {
            target.push(reference.clone());
        }
    }
}

fn build_report(
    subjects: &Value,
    context: &Value,
    profile: &profiles::Profile,
    primary: &Selection,
) -> Value {
    let mut merged = support(subjects, context, primary);
    let mut union_primary = primary.point_ids.clone();
    let mut aspect_indexes = primary.aspect_indexes.clone();
    let mut overlay_ids = primary.overlay_ids.clone();
    let mut ruler_ids = primary.ruler_ids.clone();
    let mut sections = Vec::new();
    for section in &profile.sections {
        let section_profile = profiles::Profile {
            id: section.id.clone(),
            version: profile.version.clone(),
            houses: section.houses.clone(),
            bodies: section.bodies.clone(),
            angles: section.angles.clone(),
            sections: Vec::new(),
        };
        let selected = select(subjects, context, &section_profile);
        let evidence = support(subjects, context, &selected);
        merged.point_ids.extend(evidence.point_ids.iter().cloned());
        merged.house_ids.extend(evidence.house_ids.iter().cloned());
        merged.body_ids.extend(evidence.body_ids.iter().cloned());
        merge_refs(&mut merged.chain_refs, &evidence.chain_refs);
        merge_refs(&mut merged.reception_refs, &evidence.reception_refs);
        merge_refs(&mut merged.pattern_refs, &evidence.pattern_refs);
        union_primary.extend(selected.point_ids.iter().cloned());
        aspect_indexes.extend(selected.aspect_indexes.iter().copied());
        overlay_ids.extend(selected.overlay_ids.iter().cloned());
        ruler_ids.extend(selected.ruler_ids.iter().cloned());
        sections.push((section.id.clone(), section_profile, selected, evidence));
    }
    let points: Vec<Value> = context["points"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| merged.point_ids.contains(p["id"].as_str().unwrap()))
        .map(|p| {
            let id = p["id"].as_str().unwrap();
            let mut p = p.clone();
            p["primary"] = json!(primary.point_ids.contains(id));
            p["sectionIds"] = json!(sections
                .iter()
                .filter(|(_, _, _, s)| s.point_ids.contains(id))
                .map(|(id, _, _, _)| id)
                .collect::<Vec<_>>());
            p
        })
        .collect();
    let mut houses = Vec::new();
    let mut body_facts = Vec::new();
    for chart in ["A", "B"] {
        for house in subjects[chart]["context"]["houses"].as_array().unwrap() {
            let local_id = house["id"].as_str().unwrap();
            let id = qualified(chart, local_id);
            if merged.house_ids.contains(&id) {
                houses.push(json!({"id":id,"chartId":chart,"localId":local_id,"house":house,"primary":primary.focus_houses.contains(&id),
                    "sectionIds":sections.iter().filter(|(_,_,_,s)| s.house_ids.contains(&id)).map(|(id,_,_,_)| id).collect::<Vec<_>>()}));
            }
        }
        for fact in subjects[chart]["context"]["advanced"]["bodyStates"]
            .as_array()
            .unwrap()
        {
            let local_id = fact["bodyId"].as_str().unwrap();
            let id = qualified(chart, local_id);
            if merged.body_ids.contains(&id) {
                body_facts.push(json!({"id":id,"chartId":chart,"localId":local_id,"facts":fact,"primary":primary.point_ids.contains(&id),
                    "sectionIds":sections.iter().filter(|(_,_,_,s)| s.body_ids.contains(&id)).map(|(id,_,_,_)| id).collect::<Vec<_>>()}));
            }
        }
    }
    let aspect_facts: Vec<Value> = context["aspects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| aspect_indexes.contains(&a["index"].as_u64().unwrap()))
        .map(|a| {
            let index = a["index"].as_u64().unwrap();
            let mut a = selected_aspect(a, &union_primary);
            a["primaryContact"] = json!(primary.aspect_indexes.contains(&index));
            a["sectionIds"] = json!(sections
                .iter()
                .filter(|(_, _, s, _)| s.aspect_indexes.contains(&index))
                .map(|(id, _, _, _)| id)
                .collect::<Vec<_>>());
            a
        })
        .collect();
    let report_sections: Vec<Value> = sections.iter().map(|(id,profile,selected,evidence)| {
        json!({"id":id,"selectionRules":selection_rules(profile),"primaryPointIds":point_ids(context,&selected.point_ids),
            "focusHouseIds":point_ids(context,&selected.focus_houses),"pointIds":point_ids(context,&evidence.point_ids),
            "houseIds":point_ids(context,&evidence.house_ids),"bodyFactIds":point_ids(context,&evidence.body_ids),
            "aspectIndexes":context["aspects"].as_array().unwrap().iter().filter(|a| selected.aspect_indexes.contains(&a["index"].as_u64().unwrap())).map(|a| &a["index"]).collect::<Vec<_>>(),
            "overlayIds":selected_ids(all_overlays(context),&selected.overlay_ids),
            "houseRulerRelationIds":selected_ids(context["houseRulerRelations"].as_array().unwrap().iter(),&selected.ruler_ids),
            "dispositorChainRefs":evidence.chain_refs,"natalReceptionRefs":evidence.reception_refs,"natalAspectPatternRefs":evidence.pattern_refs})
    }).collect();
    json!({"chartKind":"coupleSynastry","selectionPolicy":"primaryProfileWithTraceableSupport","profileId":"sevenmlabs-couple-report","profileVersion":"1.0","primaryPointIds":point_ids(context,&primary.point_ids),
        "points":points,"houses":houses,"bodyFacts":body_facts,"dispositorChainRefs":merged.chain_refs,
        "natalReceptionRefs":merged.reception_refs,"natalAspectPatternRefs":merged.pattern_refs,"aspectFacts":aspect_facts,
        "overlayIds":selected_ids(all_overlays(context),&overlay_ids),
        "houseRulerRelationIds":selected_ids(context["houseRulerRelations"].as_array().unwrap().iter(),&ruler_ids),"sections":report_sections,
        "coverage":{"pointCount":points.len(),"houseCount":houses.len(),"bodyFactCount":body_facts.len(),
            "primaryAspectCount":primary.aspect_indexes.len(),"aspectCount":aspect_facts.len(),"sectionCount":profile.sections.len(),
            "localAdvancedScope":"subjectNatalOnly","sectionSelection":"independentSymmetricSelectorsSharedNatals"}})
}

fn view(subjects: &Value, context: &Value, profile: &profiles::Profile, origin: &str) -> Value {
    let selected = select(subjects, context, profile);
    let report = build_report(subjects, context, profile, &selected);
    json!({"profileId":if origin == "builtin" { "sevenmlabs-couple-selection" } else { &profile.id },
        "profileVersion":profile.version,"origin":origin,"definition":profile,"selectionRules":selection_rules(profile),
        "selections":selected.local,"pointIds":point_ids(context,&selected.point_ids),
        "relationIds":context["relations"].as_array().unwrap().iter().filter(|r| touches(r,&selected.point_ids)).map(|r| &r["id"]).collect::<Vec<_>>(),
        "aspects":context["aspects"].as_array().unwrap().iter().filter(|a| selected.aspect_indexes.contains(&a["index"].as_u64().unwrap())).map(|a| selected_aspect(a,&selected.point_ids)).collect::<Vec<_>>(),
        "overlayIds":selected_ids(all_overlays(context),&selected.overlay_ids),
        "houseRulerRelationIds":selected_ids(context["houseRulerRelations"].as_array().unwrap().iter(),&selected.ruler_ids),"report":report})
}

pub(super) fn calculate_json(raw: &str) -> String {
    let r: CoupleRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &e.to_string()),
    };
    if r.operation != "couple" {
        return error_json("INVALID_INPUT", "Expected couple operation");
    }
    let person_a: natal::BirthInput = match serde_json::from_str(r.person_a.get()) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &format!("personA: {e}")),
    };
    let person_b: natal::BirthInput = match serde_json::from_str(r.person_b.get()) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &format!("personB: {e}")),
    };
    let preset = r.aspect_preset.as_deref().unwrap_or("extended");
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
    if let Err(e) = validate_aspect_rules(&rules) {
        return error_json("INVALID_INPUT", &e);
    }
    let rulership = r.rulership.as_deref().unwrap_or("traditional");
    let names = r.domains.unwrap_or_else(|| {
        couple_profiles::NAMES
            .into_iter()
            .map(String::from)
            .collect()
    });
    let custom = r.custom_profiles.as_deref().unwrap_or(&[]);
    if r.custom_profiles.is_some() && custom.is_empty() {
        return error_json(
            "INVALID_INPUT",
            "customProfiles must contain 1 to 8 profiles when provided",
        );
    }
    if let Err(e) = couple_profiles::validate_selection(&names, custom, rulership) {
        return error_json("INVALID_INPUT", &e);
    }
    if let Some(options) = &r.composite {
        if let Err(e) = options.validate(rulership) {
            return error_json("INVALID_INPUT", &format!("composite: {e}"));
        }
    }
    // Validate both subjects and all policy options before consulting the provider.
    let system_a = match natal::validate_birth(&person_a) {
        Ok(s) => s,
        Err(e) => return error_json("INVALID_INPUT", &format!("personA: {e}")),
    };
    let system_b = match natal::validate_birth(&person_b) {
        Ok(s) => s,
        Err(e) => return error_json("INVALID_INPUT", &format!("personB: {e}")),
    };
    let (natal_a, calculation_a) = match natal::compute_birth(&person_a, system_a, &rules) {
        Ok(values) => values,
        Err(e) => return error_json("CALCULATION_FAILED", &format!("personA: {e}")),
    };
    let (natal_b, calculation_b) = match natal::compute_birth(&person_b, system_b, &rules) {
        Ok(values) => values,
        Err(e) => return error_json("CALCULATION_FAILED", &format!("personB: {e}")),
    };
    let composite_data = if let Some(options) = &r.composite {
        match composite::build_from_natals(
            &natal_a,
            &calculation_a,
            &natal_b,
            &calculation_b,
            options,
            &rules,
            rulership,
        ) {
            Ok(data) => Some(data),
            Err(e) => return error_json("CALCULATION_FAILED", &format!("composite: {e}")),
        }
    } else {
        None
    };
    let (natal_a, context_a) = local_context(natal_a, &rules, rulership);
    let (natal_b, context_b) = local_context(natal_b, &rules, rulership);
    let subjects = json!({"A":{"id":"A","natal":natal_a,"context":context_a,"calculation":calculation_a},
        "B":{"id":"B","natal":natal_b,"context":context_b,"calculation":calculation_b}});
    let context = build_context(&subjects, &rules);
    let mut domain_views = serde_json::Map::new();
    for name in names {
        domain_views.insert(
            name.clone(),
            view(
                &subjects,
                &context,
                &couple_profiles::builtin(&name),
                "builtin",
            ),
        );
    }
    let mut custom_views = serde_json::Map::new();
    for profile in custom {
        custom_views.insert(
            profile.id.clone(),
            view(&subjects, &context, profile, "custom"),
        );
    }
    let catalog: Vec<profiles::Profile> = couple_profiles::NAMES
        .iter()
        .map(|id| couple_profiles::builtin(id))
        .collect();
    let mut calculation = calculation_a.clone();
    for key in ["julianDayTt", "julianDayUt1", "houseSystem"] {
        calculation.as_object_mut().unwrap().remove(key);
    }
    calculation["scope"] = json!("couple-synastry-data");
    calculation["chartKind"] = json!("coupleSynastry");
    calculation["aspectPreset"] = json!(if r.aspect_rules.provided {
        "custom"
    } else {
        preset
    });
    calculation["rulership"] = json!(rulership);
    calculation["domainProfile"] = json!({"id":"sevenmlabs-couple-selection","version":"1.0"});
    calculation["reportProfile"] = json!({"id":"sevenmlabs-couple-report","version":"1.0"});
    calculation["customProfileCount"] = json!(custom.len());
    calculation["subjectCalculations"] = json!({"A":{"julianDayTt":calculation_a["julianDayTt"],"julianDayUt1":calculation_a["julianDayUt1"],"houseSystem":calculation_a["houseSystem"]},
        "B":{"julianDayTt":calculation_b["julianDayTt"],"julianDayUt1":calculation_b["julianDayUt1"],"houseSystem":calculation_b["houseSystem"]}});
    let mut result = json!({"schemaVersion":"1.0","engineVersion":VERSION,"calculation":calculation,
        "data":{"chartKind":"coupleSynastry","subjectCount":2,"subjects":subjects,"context":context,"domains":domain_views,"customDomains":custom_views,"profileCatalog":catalog},
        "warnings":["Moshier analytical ephemeris; no JPL/Swiss data files. Future UTC uses the provider's built-in time model, not live Earth-orientation data.",
            "Couple domains follow the declared symmetric selection profile; no scores or predictions are produced.",
            "Cross-chart applying is undefined for independent birth instants; all cross applying fields are null. Advanced chains, receptions and aspect patterns remain subject-local natal facts."],"errors":[]});
    if let (Some(data), Some(options)) = (composite_data, &r.composite) {
        result["data"]["composite"] = data;
        result["data"]["chartCount"] = json!(3);
        result["calculation"]["composite"] = composite::metadata(
            options,
            &rules,
            rulership,
            if r.aspect_rules.provided {
                "custom"
            } else {
                preset
            },
            &calculation_a,
            &calculation_b,
        );
        result["warnings"]
            .as_array_mut()
            .unwrap()
            .push(json!(composite::WARNING));
    }
    result.to_string()
}
