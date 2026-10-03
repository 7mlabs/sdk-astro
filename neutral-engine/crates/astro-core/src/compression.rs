//! Deterministic, reversible representation and explicit scope selection for LLM context.
//! Shared facts are intentionally retained rather than guessing at evidence dependencies.
use crate::{error_json, MAX_CONTEXT_BYTES, VERSION};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};

const REF: &str = "$astroRef";
const TABLE: &str = "$astroTable";
const LITERAL: &str = "$astroLiteral";
const FORMAT: &str = "astro-context/1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default)]
    domains: Vec<String>,
    #[serde(default)]
    sections: Vec<String>,
    #[serde(default, deserialize_with = "budget_integer")]
    max_bytes: Option<u32>,
    #[serde(default, deserialize_with = "budget_integer")]
    max_tokens: Option<u32>,
}

fn default_mode() -> String {
    "compact".into()
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            domains: Vec::new(),
            sections: Vec::new(),
            max_bytes: None,
            max_tokens: None,
        }
    }
}

fn budget_integer<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    let result = crate::numbers::optional_u32(deserializer)?;
    if result.is_none() {
        return Err(serde::de::Error::custom(
            "Explicit null is not a numeric budget",
        ));
    }
    Ok(result)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    payload: Value,
    #[serde(default)]
    options: Options,
}

fn fail(code: &str, message: impl AsRef<str>) -> String {
    error_json(code, message.as_ref())
}

fn engine_envelope(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.keys().all(|key| {
        matches!(
            key.as_str(),
            "schemaVersion" | "engineVersion" | "data" | "calculation" | "warnings" | "errors"
        )
    }) && value["schemaVersion"] == "1.0"
        && value["engineVersion"].is_string()
        && object.contains_key("data")
        && value["warnings"]
            .as_array()
            .is_some_and(|a| a.iter().all(Value::is_string))
        && value["errors"].as_array().is_some_and(|a| {
            a.iter().all(|e| {
                e.as_object().is_some_and(|o| {
                    o.len() == 2 && o.contains_key("code") && o.contains_key("message")
                }) && e["code"].is_string()
                    && e["message"].is_string()
            })
        })
}

fn valid_ids(values: &[String]) -> bool {
    values.iter().all(|s| !s.is_empty())
        && values.iter().collect::<HashSet<_>>().len() == values.len()
}

fn pointer_part(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

fn omitted(omissions: &mut Vec<Value>, path: String, reason: &str) {
    omissions.push(json!({"path":path,"reason":reason}));
}

/// Select atomic report views; all source contexts, charts, aspects and events remain intact.
fn select_scope(
    payload: &mut Value,
    options: &Options,
) -> Result<(Vec<String>, Vec<Value>), String> {
    let mut retained = vec![String::new()];
    let mut omissions = Vec::new();
    if options.domains.is_empty() && options.sections.is_empty() {
        return Ok((retained, omissions));
    }
    let domain_ids: BTreeSet<&str> = options.domains.iter().map(String::as_str).collect();
    let section_ids: BTreeSet<&str> = options.sections.iter().map(String::as_str).collect();
    let mut found_domains = BTreeSet::new();
    let mut found_sections = BTreeSet::new();
    for container_path in ["/data", "/data/composite"] {
        let Some(container) = payload
            .pointer_mut(container_path)
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        let has_domains = ["domains", "customDomains"]
            .iter()
            .any(|key| container.get(*key).is_some_and(Value::is_object));
        if !has_domains {
            continue;
        }
        for key in ["domains", "customDomains"] {
            let Some(domains) = container.get_mut(key).and_then(Value::as_object_mut) else {
                continue;
            };
            let keys: Vec<String> = domains.keys().cloned().collect();
            for id in keys {
                let domain_path = format!("{container_path}/{key}/{}", pointer_part(&id));
                if !domain_ids.is_empty() && !domain_ids.contains(id.as_str()) {
                    domains.remove(&id);
                    omitted(&mut omissions, domain_path, "domainOutsideScope");
                    continue;
                }
                found_domains.insert(id.clone());
                retained.push(domain_path.clone());
                let domain = domains.get_mut(&id).unwrap();
                let section_relative_path = if domain.pointer("/report/sections").is_some() {
                    "/report/sections"
                } else {
                    "/sections"
                };
                let definition_section_ids: BTreeSet<String> = domain
                    .pointer("/definition/sections")
                    .and_then(Value::as_array)
                    .map(|sections| {
                        sections
                            .iter()
                            .filter_map(|s| s["id"].as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(sections) = domain
                    .pointer_mut(section_relative_path)
                    .and_then(Value::as_array_mut)
                {
                    if !section_ids.is_empty() {
                        // Parent facts retain original sectionIds provenance. When
                        // a source lacks reusable section definitions, retaining
                        // the section objects is safer than creating dangling IDs.
                        let supporting_definitions_exist = sections.iter().all(|s| {
                            s["id"]
                                .as_str()
                                .is_some_and(|id| definition_section_ids.contains(id))
                        });
                        if !supporting_definitions_exist {
                            for (index, section) in sections.iter().enumerate() {
                                if let Some(section_id) = section["id"]
                                    .as_str()
                                    .filter(|id| section_ids.contains(*id))
                                {
                                    found_sections.insert(section_id.to_owned());
                                    retained.push(format!(
                                        "{domain_path}{section_relative_path}/{index}"
                                    ));
                                }
                            }
                            continue;
                        }
                        let original = std::mem::take(sections);
                        for (index, section) in original.into_iter().enumerate() {
                            if let Some(section_id) = section["id"]
                                .as_str()
                                .filter(|id| section_ids.contains(*id))
                            {
                                found_sections.insert(section_id.to_owned());
                                // Paths refer to the retained payload after section selection.
                                retained.push(format!(
                                    "{domain_path}{section_relative_path}/{}",
                                    sections.len()
                                ));
                                sections.push(section);
                            } else {
                                omitted(
                                    &mut omissions,
                                    format!("{domain_path}{section_relative_path}/{index}"),
                                    "sectionOutsideScope",
                                );
                            }
                        }
                    }
                }
            }
        }
        if !domain_ids.is_empty() {
            if let Some(catalog) = container
                .get_mut("profileCatalog")
                .and_then(Value::as_array_mut)
            {
                let original = std::mem::take(catalog);
                for (index, profile) in original.into_iter().enumerate() {
                    if profile["id"]
                        .as_str()
                        .is_some_and(|id| domain_ids.contains(id))
                    {
                        catalog.push(profile);
                    } else {
                        omitted(
                            &mut omissions,
                            format!("{container_path}/profileCatalog/{index}"),
                            "profileOutsideScope",
                        );
                    }
                }
            }
        }
    }
    for id in &options.domains {
        if !found_domains.contains(id) {
            return Err(format!("Unknown domain selector: {id}"));
        }
    }
    for id in &options.sections {
        if !found_sections.contains(id) {
            return Err(format!(
                "Unknown section selector in the retained domains: {id}"
            ));
        }
    }
    retained.sort();
    omissions.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok((retained, omissions))
}

fn serialized_len(value: &Value) -> usize {
    serde_json::to_vec(value).unwrap().len()
}

fn exact_fields(value: &Value, names: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == names.len() && names.iter().all(|name| object.contains_key(*name))
    })
}

fn string_array(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|values| values.iter().all(Value::is_string))
}

fn nonnegative_integer(value: &Value) -> bool {
    value.as_u64().is_some()
        || value
            .as_f64()
            .is_some_and(|number| number.is_finite() && number >= 0. && number.fract() == 0.)
}

fn nullable_budget(value: &Value) -> bool {
    value.is_null()
        || value.as_f64().is_some_and(|number| {
            number >= 1. && number <= MAX_CONTEXT_BYTES as f64 && number.fract() == 0.
        })
}

/// Validate the public context metadata contract before interpreting any encoding.
/// These checks mirror compression-response.schema.json, including required
/// nullable budget fields and rejection of unknown metadata fields.
fn valid_context_metadata(data: &Value) -> bool {
    exact_fields(
        data,
        &[
            "format",
            "mode",
            "payload",
            "dictionary",
            "coverage",
            "omitted",
            "metrics",
            "budget",
        ],
    ) && data["format"] == FORMAT
        && data["mode"]
            .as_str()
            .is_some_and(|mode| matches!(mode, "compact" | "focused" | "budgeted"))
        && data["dictionary"].is_object()
        && exact_fields(&data["coverage"], &["complete", "retainedPaths", "scope"])
        && data["coverage"]["complete"].is_boolean()
        && string_array(&data["coverage"]["retainedPaths"])
        && exact_fields(&data["coverage"]["scope"], &["domains", "sections"])
        && string_array(&data["coverage"]["scope"]["domains"])
        && string_array(&data["coverage"]["scope"]["sections"])
        && data["omitted"].as_array().is_some_and(|omissions| {
            omissions.iter().all(|omission| {
                exact_fields(omission, &["path", "reason"])
                    && omission["path"].is_string()
                    && omission["reason"].is_string()
            })
        })
        && exact_fields(
            &data["metrics"],
            &[
                "inputBytes",
                "outputBytes",
                "estimatedTokens",
                "tokenEstimateMethod",
            ],
        )
        && nonnegative_integer(&data["metrics"]["inputBytes"])
        && nonnegative_integer(&data["metrics"]["outputBytes"])
        && nonnegative_integer(&data["metrics"]["estimatedTokens"])
        && data["metrics"]["tokenEstimateMethod"] == "utf8-bytes-conservative"
        && exact_fields(&data["budget"], &["maxBytes", "maxTokens", "exceeded"])
        && nullable_budget(&data["budget"]["maxBytes"])
        && nullable_budget(&data["budget"]["maxTokens"])
        && data["budget"]["exceeded"].is_boolean()
}

/// serde_json's default recursion allowance is 128, and the 128th nested
/// container is rejected. Inspect the actual wire tree without reparsing or
/// serializing subtrees; reserved-key escaping may triple source nesting.
fn valid_wire_depth(value: &Value) -> bool {
    let mut pending = vec![(value, 0usize)];
    while let Some((value, parent_depth)) = pending.pop() {
        match value {
            Value::Object(object) => {
                let depth = parent_depth + 1;
                if depth >= 128 {
                    return false;
                }
                pending.extend(object.values().map(|value| (value, depth)));
            }
            Value::Array(values) => {
                let depth = parent_depth + 1;
                if depth >= 128 {
                    return false;
                }
                pending.extend(values.iter().map(|value| (value, depth)));
            }
            _ => {}
        }
    }
    true
}

/// A dictionary adds decoder hops without necessarily adding wire nesting.
/// Account for those hops too, memoizing definitions so DAGs stay linear-time.
fn valid_decoder_depth(payload: &Value, dictionary: &Map<String, Value>) -> bool {
    fn height(
        value: &Value,
        dictionary: &Map<String, Value>,
        memo: &mut HashMap<String, usize>,
    ) -> usize {
        match value {
            Value::Object(object) if object.contains_key(REF) => {
                let id = object[REF].as_str().unwrap();
                let child_height = match memo.get(id) {
                    Some(height) => *height,
                    None => {
                        let child_height = height(&dictionary[id], dictionary, memo);
                        memo.insert(id.to_owned(), child_height);
                        child_height
                    }
                };
                child_height.saturating_add(1)
            }
            Value::Object(object) if object.contains_key(TABLE) => object[TABLE]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|row| row.as_array().unwrap())
                .map(|value| height(value, dictionary, memo).saturating_add(1))
                .max()
                .unwrap_or(0),
            Value::Object(object) if object.contains_key(LITERAL) => object[LITERAL]
                .as_array()
                .unwrap()
                .iter()
                .map(|pair| height(&pair[1], dictionary, memo).saturating_add(1))
                .max()
                .unwrap_or(0),
            Value::Object(object) => object
                .values()
                .map(|value| height(value, dictionary, memo).saturating_add(1))
                .max()
                .unwrap_or(0),
            Value::Array(values) => values
                .iter()
                .map(|value| height(value, dictionary, memo).saturating_add(1))
                .max()
                .unwrap_or(0),
            _ => 0,
        }
    }
    height(payload, dictionary, &mut HashMap::new()) <= 128
}

/// Tables are used only when all rows contain exactly the same keys, and save bytes.
/// Reserved input keys are escaped before representation markers are introduced.
fn tables(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            if object
                .keys()
                .any(|k| matches!(k.as_str(), REF | TABLE | LITERAL))
            {
                return json!({LITERAL:object.iter().map(|(key,value)| json!([key,tables(value)])).collect::<Vec<_>>()});
            }
            Value::Object(
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), tables(value)))
                    .collect(),
            )
        }
        Value::Array(values) => {
            let encoded: Vec<Value> = values.iter().map(tables).collect();
            if values.len() >= 2 {
                if let Some(first) = values[0].as_object() {
                    let columns: Vec<String> = first.keys().cloned().collect();
                    if !columns.is_empty()
                        && !columns
                            .iter()
                            .any(|k| matches!(k.as_str(), REF | TABLE | LITERAL))
                        && values
                            .iter()
                            .all(|v| v.as_object().is_some_and(|o| o.keys().eq(first.keys())))
                    {
                        let rows: Vec<Value> = encoded
                            .iter()
                            .map(|v| {
                                Value::Array(columns.iter().map(|key| v[key].clone()).collect())
                            })
                            .collect();
                        let table = json!({TABLE:{"columns":columns,"rows":rows}});
                        if serialized_len(&table) < serde_json::to_vec(&encoded).unwrap().len() {
                            return table;
                        }
                    }
                }
            }
            Value::Array(encoded)
        }
        _ => value.clone(),
    }
}

struct Candidate<'a> {
    example: &'a Value,
    count: usize,
    size: usize,
}

#[derive(Default)]
struct Census<'a> {
    candidates: Vec<Candidate<'a>>,
    hashes: HashMap<u64, Vec<usize>>,
    nodes: HashMap<usize, usize>,
}

impl<'a> Census<'a> {
    // Structural hashes avoid serializing and storing every large subtree. Equality
    // is checked after a hash match, so collisions can never merge different facts.
    fn count(&mut self, value: &'a Value) -> (u64, usize) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let size = match value {
            Value::Array(values) => {
                0u8.hash(&mut hasher);
                values.len().hash(&mut hasher);
                let mut length = 2 + values.len().saturating_sub(1);
                for child in values {
                    let (hash, size) = self.count(child);
                    hash.hash(&mut hasher);
                    length += size;
                }
                length
            }
            Value::Object(object) => {
                1u8.hash(&mut hasher);
                let mut length = 2 + object.len().saturating_sub(1);
                for (key, child) in object {
                    key.hash(&mut hasher);
                    let (hash, size) = self.count(child);
                    hash.hash(&mut hasher);
                    length += serialized_len(&Value::String(key.clone())) + 1 + size;
                }
                length
            }
            _ => {
                2u8.hash(&mut hasher);
                let text = serde_json::to_string(value).unwrap();
                text.hash(&mut hasher);
                text.len()
            }
        };
        let hash = hasher.finish();
        if size >= 64 && (value.is_object() || value.is_array()) {
            let existing = self.hashes.get(&hash).and_then(|indices| {
                indices
                    .iter()
                    .copied()
                    .find(|i| self.candidates[*i].example == value)
            });
            let index = existing.unwrap_or_else(|| {
                let index = self.candidates.len();
                self.candidates.push(Candidate {
                    example: value,
                    count: 0,
                    size,
                });
                self.hashes.entry(hash).or_default().push(index);
                index
            });
            self.candidates[index].count += 1;
            self.nodes.insert(value as *const Value as usize, index);
        }
        (hash, size)
    }
}

struct Encoder<'a> {
    census: Census<'a>,
    ids: HashMap<usize, String>,
    dictionary: Map<String, Value>,
}

impl<'a> Encoder<'a> {
    fn encode(&mut self, value: &'a Value, bypass: Option<usize>) -> Value {
        if let Some(index) = self
            .census
            .nodes
            .get(&(value as *const Value as usize))
            .copied()
        {
            let candidate = &self.census.candidates[index];
            if bypass != Some(index) && candidate.count > 1 && candidate.size > 80 {
                if let Some(id) = self.ids.get(&index) {
                    return json!({REF:id});
                }
                let id = format!("r{}", self.ids.len());
                self.ids.insert(index, id.clone());
                let definition = self.encode(value, Some(index));
                self.dictionary.insert(id.clone(), definition);
                return json!({REF:id});
            }
        }
        match value {
            Value::Object(object) if object.contains_key(TABLE) => {
                // The marker's structural body, column list, row list and row
                // containers stay concrete. Only cell values may use references.
                let body = &object[TABLE];
                let rows: Vec<Value> = body["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| {
                        Value::Array(
                            row.as_array()
                                .unwrap()
                                .iter()
                                .map(|cell| self.encode(cell, None))
                                .collect(),
                        )
                    })
                    .collect();
                json!({TABLE:{"columns":body["columns"].clone(),"rows":rows}})
            }
            Value::Object(object) if object.contains_key(LITERAL) => {
                let pairs: Vec<Value> = object[LITERAL]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|pair| json!([pair[0].clone(), self.encode(&pair[1], None)]))
                    .collect();
                json!({LITERAL:pairs})
            }
            Value::Object(object) => Value::Object(
                object
                    .iter()
                    .map(|(key, child)| (key.clone(), self.encode(child, None)))
                    .collect(),
            ),
            Value::Array(values) => Value::Array(
                values
                    .iter()
                    .map(|child| self.encode(child, None))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }
}

fn count_refs(value: &Value, counts: &mut BTreeMap<String, usize>) {
    if let Some(id) = value
        .as_object()
        .filter(|o| o.len() == 1)
        .and_then(|o| o.get(REF))
        .and_then(Value::as_str)
    {
        *counts.entry(id.to_owned()).or_default() += 1;
        return;
    }
    match value {
        Value::Array(values) => {
            for value in values {
                count_refs(value, counts);
            }
        }
        Value::Object(object) => {
            for value in object.values() {
                count_refs(value, counts);
            }
        }
        _ => {}
    }
}

fn inline_ref(value: &mut Value, id: &str, definition: &Value) {
    if value
        .as_object()
        .is_some_and(|o| o.len() == 1 && o.get(REF).and_then(Value::as_str) == Some(id))
    {
        *value = definition.clone();
        return;
    }
    match value {
        Value::Array(values) => {
            for value in values {
                inline_ref(value, id, definition);
            }
        }
        Value::Object(object) => {
            for value in object.values_mut() {
                inline_ref(value, id, definition);
            }
        }
        _ => {}
    }
}

fn compact(payload: &Value) -> (Value, Value) {
    let encoded = tables(payload);
    let mut census = Census::default();
    census.count(&encoded);
    let mut encoder = Encoder {
        census,
        ids: HashMap::new(),
        dictionary: Map::new(),
    };
    let mut result = encoder.encode(&encoded, None);
    // Parent deduplication can leave child definitions referenced once. Recompute
    // actual wire savings and inline such definitions rather than bloating a dictionary.
    loop {
        let mut counts = BTreeMap::new();
        count_refs(&result, &mut counts);
        for value in encoder.dictionary.values() {
            count_refs(value, &mut counts);
        }
        let remove = encoder.dictionary.iter().find_map(|(id, definition)| {
            let count = *counts.get(id).unwrap_or(&0);
            let size = serialized_len(definition);
            let reference_size = serialized_len(&json!({REF:id}));
            let entry_cost = serialized_len(&Value::String(id.clone())) + 2 + size;
            if count.saturating_mul(size) <= entry_cost + count.saturating_mul(reference_size) {
                Some(id.clone())
            } else {
                None
            }
        });
        let Some(id) = remove else { break };
        let definition = encoder.dictionary.remove(&id).unwrap();
        inline_ref(&mut result, &id, &definition);
        for value in encoder.dictionary.values_mut() {
            inline_ref(value, &id, &definition);
        }
    }
    let dictionary = Value::Object(encoder.dictionary);
    if serialized_len(&result) + serialized_len(&dictionary) >= serialized_len(&encoded) + 2 {
        (encoded, json!({}))
    } else {
        (result, dictionary)
    }
}

fn measure(envelope: &mut Value) -> String {
    // The serialized size includes its own decimal size fields; converge on their
    // digit width rather than measuring a partial payload or excluding metadata.
    for _ in 0..16 {
        let output = serde_json::to_string(envelope).unwrap();
        let bytes = output.len() as u64;
        if envelope["data"]["metrics"]["outputBytes"].as_u64() == Some(bytes) {
            return output;
        }
        envelope["data"]["metrics"]["outputBytes"] = json!(bytes);
        envelope["data"]["metrics"]["estimatedTokens"] = json!(bytes);
    }
    unreachable!("serialized size digit widths converge within 16 iterations")
}

/// Compress `{payload: engineEnvelope, options?: ...}` without calculating a chart.
pub fn compress_json(input: &str) -> String {
    if input.len() > MAX_CONTEXT_BYTES {
        return fail(
            "COMPRESSION_INPUT_TOO_LARGE",
            "Compression input exceeds 256 MiB",
        );
    }
    let request: Request = match serde_json::from_str(input) {
        Ok(value) => value,
        Err(error) => return fail("COMPRESSION_JSON_INVALID", error.to_string()),
    };
    if !engine_envelope(&request.payload) {
        return fail(
            "COMPRESSION_PAYLOAD_INVALID",
            "payload must be a standard engine response envelope",
        );
    }
    let mode = request.options.mode.as_str();
    if !matches!(mode, "compact" | "focused" | "budgeted")
        || !valid_ids(&request.options.domains)
        || !valid_ids(&request.options.sections)
        || request.options.max_bytes == Some(0)
        || request.options.max_tokens == Some(0)
        || request
            .options
            .max_bytes
            .is_some_and(|n| n as usize > MAX_CONTEXT_BYTES)
        || request
            .options
            .max_tokens
            .is_some_and(|n| n as usize > MAX_CONTEXT_BYTES)
        || (mode == "compact"
            && (!request.options.domains.is_empty() || !request.options.sections.is_empty()))
    {
        return fail("COMPRESSION_OPTIONS_INVALID","Use compact, focused or budgeted; unique nonempty selectors; integer budgets from 1 to 268435456. Selectors require focused or budgeted mode.");
    }
    let input_bytes = serialized_len(&request.payload);
    let mut payload = request.payload;
    let (retained_paths, omissions) = match select_scope(&mut payload, &request.options) {
        Ok(scope) => scope,
        Err(error) => return fail("COMPRESSION_SELECTOR_INVALID", error),
    };
    let (payload, dictionary) = compact(&payload);
    let mut envelope = json!({
        "schemaVersion":"1.0","engineVersion":VERSION,
        "data":{
            "format":FORMAT,"mode":mode,"payload":payload,"dictionary":dictionary,
            "coverage":{"complete":omissions.is_empty(),"retainedPaths":retained_paths,"scope":{"domains":request.options.domains,"sections":request.options.sections}},
            "omitted":omissions,
            "metrics":{"inputBytes":input_bytes,"outputBytes":0,"estimatedTokens":0,"tokenEstimateMethod":"utf8-bytes-conservative"},
            "budget":{"maxBytes":request.options.max_bytes,"maxTokens":request.options.max_tokens,"exceeded":false}
        },"warnings":[],"errors":[]
    });
    if !valid_wire_depth(&envelope)
        || !valid_decoder_depth(
            &envelope["data"]["payload"],
            envelope["data"]["dictionary"].as_object().unwrap(),
        )
    {
        return fail(
            "COMPRESSION_OUTPUT_TOO_DEEP",
            "Encoded context exceeds supported nesting; simplify the payload structure",
        );
    }
    let mut output = measure(&mut envelope);
    if request
        .options
        .max_bytes
        .is_some_and(|budget| output.len() as u64 > budget as u64)
        || request
            .options
            .max_tokens
            .is_some_and(|budget| output.len() as u64 > budget as u64)
    {
        envelope["data"]["budget"]["exceeded"] = json!(true);
        envelope["warnings"] = json!(["COMPRESSION_BUDGET_EXCEEDED: The retained evidence and metadata exceed the requested budget. No facts were truncated; select a smaller domain scope or raise the budget. Token counts are a conservative UTF-8 byte estimate."]);
        output = measure(&mut envelope);
    }
    if output.len() > MAX_CONTEXT_BYTES {
        return fail(
            "COMPRESSION_OUTPUT_TOO_LARGE",
            "Encoded context exceeds 256 MiB; select a smaller domain scope",
        );
    }
    output
}

struct Decoder<'a> {
    dictionary: &'a Map<String, Value>,
    active: HashSet<String>,
    bytes: usize,
}

fn validate_encoding(
    value: &Value,
    references: &mut BTreeSet<String>,
    depth: usize,
) -> Result<(), String> {
    if depth > 128 {
        return Err("Encoded payload nesting exceeds 128 levels".into());
    }
    match value {
        Value::Object(object) if object.contains_key(REF) => {
            let id = object
                .get(REF)
                .and_then(Value::as_str)
                .filter(|id| object.len() == 1 && !id.is_empty())
                .ok_or("Invalid $astroRef marker")?;
            references.insert(id.to_owned());
        }
        Value::Object(object) if object.contains_key(TABLE) => {
            let table = object
                .get(TABLE)
                .and_then(Value::as_object)
                .filter(|t| {
                    object.len() == 1
                        && t.len() == 2
                        && t.contains_key("columns")
                        && t.contains_key("rows")
                })
                .ok_or("Invalid $astroTable marker")?;
            let columns = table["columns"]
                .as_array()
                .ok_or("Table columns must be a concrete array")?;
            let mut unique = HashSet::new();
            for column in columns {
                let name = column.as_str().ok_or("Table columns must be strings")?;
                if !unique.insert(name) {
                    return Err("Duplicate table column".into());
                }
            }
            let rows = table["rows"]
                .as_array()
                .ok_or("Table rows must be a concrete array")?;
            for row in rows {
                let row = row.as_array().ok_or("Table rows must be concrete arrays")?;
                if row.len() != columns.len() {
                    return Err("Table row width differs from columns".into());
                }
                for cell in row {
                    validate_encoding(cell, references, depth + 1)?;
                }
            }
        }
        Value::Object(object) if object.contains_key(LITERAL) => {
            let pairs = object
                .get(LITERAL)
                .and_then(Value::as_array)
                .filter(|_| object.len() == 1)
                .ok_or("Invalid $astroLiteral marker")?;
            let mut unique = HashSet::new();
            for pair in pairs {
                let pair = pair
                    .as_array()
                    .filter(|p| p.len() == 2)
                    .ok_or("Literal entries must be concrete key/value pairs")?;
                let key = pair[0].as_str().ok_or("Literal keys must be strings")?;
                if !unique.insert(key) {
                    return Err("Duplicate literal key".into());
                }
                validate_encoding(&pair[1], references, depth + 1)?;
            }
        }
        Value::Object(object) => {
            for value in object.values() {
                validate_encoding(value, references, depth + 1)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_encoding(value, references, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_reference_graph(
    payload: &Value,
    dictionary: &Map<String, Value>,
) -> Result<(), String> {
    fn visit<'a>(
        id: &'a str,
        graph: &'a BTreeMap<String, BTreeSet<String>>,
        states: &mut HashMap<&'a str, u8>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 128 {
            return Err("Dictionary reference nesting exceeds 128 levels".into());
        }
        match states.get(id) {
            Some(1) => return Err(format!("Cyclic dictionary reference: {id}")),
            Some(2) => return Ok(()),
            _ => {}
        }
        let references = graph
            .get(id)
            .ok_or_else(|| format!("Missing dictionary reference: {id}"))?;
        states.insert(id, 1);
        for next in references {
            visit(next, graph, states, depth + 1)?;
        }
        states.insert(id, 2);
        Ok(())
    }
    let mut payload_refs = BTreeSet::new();
    validate_encoding(payload, &mut payload_refs, 0)?;
    let mut graph = BTreeMap::new();
    for (id, definition) in dictionary {
        let mut references = BTreeSet::new();
        validate_encoding(definition, &mut references, 0)?;
        graph.insert(id.clone(), references);
    }
    let mut states = HashMap::new();
    // Validate unused entries too: a malformed or cyclic dictionary is not a
    // valid context even when its current payload happens not to reach the entry.
    for id in graph.keys().chain(payload_refs.iter()) {
        visit(id, &graph, &mut states, 0)?;
    }
    Ok(())
}

impl Decoder<'_> {
    fn add(&mut self, bytes: usize) -> Result<(), String> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or("Expanded payload size overflow")?;
        if self.bytes > MAX_CONTEXT_BYTES {
            return Err("Expanded payload exceeds 256 MiB".into());
        }
        Ok(())
    }

    fn decode(&mut self, value: &Value, depth: usize) -> Result<Value, String> {
        if depth > 128 {
            return Err("Reference or payload nesting exceeds 128 levels".into());
        }
        match value {
            Value::Object(object) if object.contains_key(REF) => {
                let Some(id) = object
                    .get(REF)
                    .and_then(Value::as_str)
                    .filter(|_| object.len() == 1)
                else {
                    return Err("Invalid $astroRef marker".into());
                };
                if !self.active.insert(id.to_owned()) {
                    return Err(format!("Cyclic dictionary reference: {id}"));
                }
                let definition = self
                    .dictionary
                    .get(id)
                    .ok_or_else(|| format!("Missing dictionary reference: {id}"))?;
                let output = self.decode(definition, depth + 1);
                self.active.remove(id);
                output
            }
            Value::Object(object) if object.contains_key(TABLE) => {
                let Some(table) = object.get(TABLE).and_then(Value::as_object).filter(|t| {
                    object.len() == 1
                        && t.len() == 2
                        && t.contains_key("columns")
                        && t.contains_key("rows")
                }) else {
                    return Err("Invalid $astroTable marker".into());
                };
                let columns = table["columns"]
                    .as_array()
                    .ok_or("Table columns must be an array")?;
                let mut names = Vec::new();
                let mut unique = HashSet::new();
                for column in columns {
                    let name = column.as_str().ok_or("Table columns must be strings")?;
                    if !unique.insert(name) {
                        return Err("Duplicate table column".into());
                    }
                    names.push(name);
                }
                let rows = table["rows"]
                    .as_array()
                    .ok_or("Table rows must be an array")?;
                self.add(2 + rows.len().saturating_sub(1))?;
                let mut result = Vec::new();
                for row in rows {
                    let row = row.as_array().ok_or("Table row must resolve to an array")?;
                    if row.len() != names.len() {
                        return Err("Table row width differs from columns".into());
                    }
                    self.add(2 + names.len().saturating_sub(1))?;
                    let mut object = Map::new();
                    for (name, value) in names.iter().zip(row) {
                        self.add(serialized_len(&Value::String((*name).into())) + 1)?;
                        object.insert((*name).into(), self.decode(value, depth + 1)?);
                    }
                    result.push(Value::Object(object));
                }
                Ok(Value::Array(result))
            }
            Value::Object(object) if object.contains_key(LITERAL) => {
                let Some(pairs) = object
                    .get(LITERAL)
                    .and_then(Value::as_array)
                    .filter(|_| object.len() == 1)
                else {
                    return Err("Invalid $astroLiteral marker".into());
                };
                self.add(2 + pairs.len().saturating_sub(1))?;
                let mut output = Map::new();
                for pair in pairs {
                    let pair = pair.as_array().ok_or("Literal entries must be pairs")?;
                    if pair.len() != 2 {
                        return Err("Literal entries must be key/value pairs".into());
                    }
                    let key = pair[0].as_str().ok_or("Literal keys must be strings")?;
                    if output.contains_key(key) {
                        return Err("Duplicate literal key".into());
                    }
                    self.add(serialized_len(&Value::String(key.into())) + 1)?;
                    let decoded = self.decode(&pair[1], depth + 1)?;
                    output.insert(key.into(), decoded);
                }
                Ok(Value::Object(output))
            }
            Value::Object(object) => {
                self.add(2 + object.len().saturating_sub(1))?;
                let mut result = Map::new();
                for (key, value) in object {
                    self.add(serialized_len(&Value::String(key.clone())) + 1)?;
                    result.insert(key.clone(), self.decode(value, depth + 1)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(values) => {
                self.add(2 + values.len().saturating_sub(1))?;
                values
                    .iter()
                    .map(|value| self.decode(value, depth + 1))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::Array)
            }
            _ => {
                self.add(serialized_len(value))?;
                Ok(value.clone())
            }
        }
    }
}

/// Expand a successful compression envelope to the retained original engine envelope.
pub fn expand_context_json(input: &str) -> String {
    if input.len() > MAX_CONTEXT_BYTES {
        return fail(
            "COMPRESSION_INPUT_TOO_LARGE",
            "Context input exceeds 256 MiB",
        );
    }
    let context: Value = match serde_json::from_str(input) {
        Ok(context) => context,
        Err(error) => return fail("COMPRESSION_CONTEXT_INVALID", error.to_string()),
    };
    if !engine_envelope(&context)
        || context["errors"].as_array().is_none_or(|a| !a.is_empty())
        || !valid_context_metadata(&context["data"])
    {
        return fail(
            "COMPRESSION_CONTEXT_INVALID",
            "Expected a successful astro-context/1 engine envelope",
        );
    }
    let Some(dictionary) = context["data"]["dictionary"].as_object() else {
        return fail(
            "COMPRESSION_CONTEXT_INVALID",
            "Context dictionary must be an object",
        );
    };
    let Some(payload) = context["data"].get("payload") else {
        return fail("COMPRESSION_CONTEXT_INVALID", "Context payload is missing");
    };
    if let Err(error) = validate_reference_graph(payload, dictionary) {
        return fail("COMPRESSION_CONTEXT_INVALID", error);
    }
    let mut decoder = Decoder {
        dictionary,
        active: HashSet::new(),
        bytes: 0,
    };
    let output = match decoder.decode(payload, 0) {
        Ok(output) => output,
        Err(error) => return fail("COMPRESSION_CONTEXT_INVALID", error),
    };
    if !engine_envelope(&output) {
        return fail(
            "COMPRESSION_CONTEXT_INVALID",
            "Expanded payload is not a standard engine response envelope",
        );
    }
    let output = serde_json::to_string(&output).unwrap();
    if output.len() > MAX_CONTEXT_BYTES {
        return fail(
            "COMPRESSION_CONTEXT_INVALID",
            "Expanded payload exceeds 256 MiB",
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(data: Value) -> Value {
        json!({"schemaVersion":"1.0","engineVersion":"source-version","data":data,"warnings":["unknown applying retained"],"errors":[]})
    }

    fn compress(payload: Value, options: Value) -> Value {
        serde_json::from_str(&compress_json(
            &json!({"payload":payload,"options":options}).to_string(),
        ))
        .unwrap()
    }

    fn expand(context: &Value) -> Value {
        serde_json::from_str(&expand_context_json(&context.to_string())).unwrap()
    }

    fn first_difference(left: &Value, right: &Value, path: &str) -> Option<String> {
        if left == right {
            return None;
        }
        if let (Some(a), Some(b)) = (left.as_object(), right.as_object()) {
            for key in a.keys().chain(b.keys()) {
                if let Some(result) = first_difference(
                    &left[key],
                    &right[key],
                    &format!("{path}/{}", pointer_part(key)),
                ) {
                    return Some(result);
                }
            }
        }
        if let (Some(a), Some(b)) = (left.as_array(), right.as_array()) {
            if a.len() != b.len() {
                return Some(format!("{path}: array length {} != {}", a.len(), b.len()));
            }
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                if let Some(result) = first_difference(a, b, &format!("{path}/{index}")) {
                    return Some(result);
                }
            }
        }
        Some(format!(
            "{path}: {} != {}",
            left.to_string().chars().take(180).collect::<String>(),
            right.to_string().chars().take(180).collect::<String>()
        ))
    }

    #[test]
    fn compact_roundtrip_preserves_precision_nulls_and_reserved_keys() {
        let row = json!({"body1":"A:sun","body2":"B:moon","applying":null,"longitude":280.36891967534336,"orb":0.000000000023423});
        let payload = envelope(json!({
            "facts":[row.clone(),row.clone(),row.clone(),row],
            "differentKeys":[{"known":null},{"unknown":null}],
            "collision":{"$astroRef":"literal","$astroTable":{"columns":[],"rows":[]},"$astroLiteral":[["k","v"]]},
            "nestedCollision":{"$astroRef":{"$astroRef":"literal"}}
        }));
        let context = compress(payload.clone(), json!({}));
        assert_eq!(context["errors"], json!([]));
        assert_eq!(expand(&context), payload);
        assert_eq!(context["data"]["coverage"]["complete"], true);
        assert_eq!(
            context["data"]["metrics"]["outputBytes"].as_u64().unwrap(),
            context.to_string().len() as u64
        );
        assert_eq!(
            context["data"]["metrics"]["estimatedTokens"],
            context["data"]["metrics"]["outputBytes"]
        );
    }

    #[test]
    fn dictionary_cost_and_determinism() {
        let fact = json!({"label":"a sufficiently long repeated fact definition preserving a shared source","precision":1.234567890123456,"value":null});
        let payload = envelope(json!({"a":fact,"b":fact,"c":fact}));
        let a = compress(payload.clone(), json!({}));
        let b = compress(payload.clone(), json!({}));
        assert_eq!(a, b);
        assert!(!a["data"]["dictionary"].as_object().unwrap().is_empty());
        assert!(
            serialized_len(&a["data"]["payload"]) + serialized_len(&a["data"]["dictionary"])
                < serialized_len(&payload)
        );
        assert_eq!(expand(&a), payload);
    }

    #[test]
    fn focused_scope_preserves_context_and_marks_all_omissions() {
        let payload = envelope(
            json!({"context":{"points":[{"id":"A:sun"},{"id":"B:sun"}],"aspects":[{"point1":"A:sun","point2":"B:sun","applying":null}]},"domains":{
            "attraction":{"definition":{"sections":[{"id":"chemistry"},{"id":"romance"}]},"report":{"sections":[{"id":"chemistry","aspectIndexes":[0]},{"id":"romance","aspectIndexes":[0]}],"aspects":[{"index":0,"sectionIds":["chemistry","romance"]}],"points":[{"id":"A:sun"},{"id":"B:sun"}]}},
            "communication":{"report":{"sections":[{"id":"conversation"}]}}
        },"profileCatalog":[{"id":"attraction"},{"id":"communication"}]}),
        );
        let context = compress(
            payload.clone(),
            json!({"mode":"focused","domains":["attraction"],"sections":["chemistry"]}),
        );
        let restored = expand(&context);
        assert_eq!(restored["data"]["context"], payload["data"]["context"]);
        assert_eq!(
            restored["data"]["domains"]["attraction"]["report"]["sections"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(restored["data"]["domains"].as_object().unwrap().len(), 1);
        assert_eq!(context["data"]["coverage"]["complete"], false);
        assert_eq!(context["data"]["omitted"].as_array().unwrap().len(), 3);
        assert_eq!(
            restored["data"]["domains"]["attraction"]["definition"]["sections"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            restored["data"]["domains"]["attraction"]["report"]["aspects"][0]["sectionIds"],
            json!(["chemistry", "romance"])
        );
    }

    #[test]
    fn section_selection_without_definitions_keeps_supporting_sections() {
        let payload = envelope(
            json!({"domains":{"career":{"report":{"sections":[{"id":"profession"},{"id":"resources"}],"aspects":[{"sectionIds":["profession","resources"]}]}}}}),
        );
        let context = compress(
            payload.clone(),
            json!({"mode":"focused","domains":["career"],"sections":["profession"]}),
        );
        assert_eq!(expand(&context), payload);
        assert_eq!(context["data"]["coverage"]["complete"], true);
        assert_eq!(
            compress(
                payload,
                json!({"mode":"focused","domains":["career"],"sections":["unknown"]})
            )["errors"][0]["code"],
            "COMPRESSION_SELECTOR_INVALID"
        );
    }

    #[test]
    fn selection_keeps_couple_and_composite_namespaces_and_escapes_pointer_ids() {
        let payload = envelope(json!({
            "subjects":{"A":{"id":"A","pointIds":["A:sun"]},"B":{"id":"B","pointIds":["B:sun"]}},
            "context":{"points":[{"id":"A:sun"},{"id":"B:sun"}]},
            "domains":{"attraction":{"pointIds":["A:sun","B:sun"]},"communication":{}},
            "composite":{"id":"C","context":{"points":[{"id":"sun"}]},"domains":{"career":{"pointIds":["sun"]},"love":{}},"customDomains":{"my~/scope":{"pointIds":["sun"]}}}
        }));
        let context = compress(
            payload.clone(),
            json!({"mode":"focused","domains":["attraction","career","my~/scope"]}),
        );
        let restored = expand(&context);
        assert_eq!(restored["data"]["subjects"], payload["data"]["subjects"]);
        assert_eq!(restored["data"]["context"], payload["data"]["context"]);
        assert_eq!(
            restored["data"]["composite"]["context"],
            payload["data"]["composite"]["context"]
        );
        assert_eq!(restored["data"]["domains"].as_object().unwrap().len(), 1);
        assert_eq!(
            restored["data"]["composite"]["domains"]
                .as_object()
                .unwrap()
                .len(),
            1
        );
        assert!(context["data"]["coverage"]["retainedPaths"]
            .as_array()
            .unwrap()
            .contains(&json!("/data/composite/customDomains/my~0~1scope")));
    }

    #[test]
    fn budgets_follow_exact_decimal_integer_normalization() {
        let payload = envelope(json!({"placements":[]})).to_string();
        for number in ["2000", "2000.0", "2e3", "20000e-1"] {
            let result: Value = serde_json::from_str(&compress_json(&format!(
                "{{\"payload\":{payload},\"options\":{{\"maxBytes\":{number}}}}}"
            )))
            .unwrap();
            assert_eq!(result["errors"], json!([]), "{number}");
            assert_eq!(result["data"]["budget"]["maxBytes"], 2000, "{number}");
        }
        for number in [
            "2000.0000000000000001",
            "1999.9999999999999999",
            "1e-100",
            "null",
            "0",
            "-1",
            "268435457",
            "\"2000\"",
        ] {
            let result: Value = serde_json::from_str(&compress_json(&format!(
                "{{\"payload\":{payload},\"options\":{{\"maxTokens\":{number}}}}}"
            )))
            .unwrap();
            assert!(!result["errors"].as_array().unwrap().is_empty(), "{number}");
        }
    }

    #[test]
    fn reserved_key_nesting_never_produces_unexpandable_success() {
        for count in [20, 60] {
            let mut nested = json!("leaf");
            for _ in 0..count {
                nested = json!({REF:nested});
            }
            let payload = envelope(nested);
            let output = compress_json(&json!({"payload":payload}).to_string());
            let context: Value = serde_json::from_str(&output).unwrap();
            if count == 20 {
                assert_eq!(context["errors"], json!([]));
                assert_eq!(expand(&context), payload);
            } else {
                assert_eq!(context["errors"][0]["code"], "COMPRESSION_OUTPUT_TOO_DEEP");
            }
        }
    }

    #[test]
    fn expansion_requires_complete_metadata_contract() {
        let base = compress(envelope(json!({"facts":[]})), json!({}));
        let mut minimal = base.clone();
        minimal["data"] = json!({"format":FORMAT,"payload":base["data"]["payload"],"dictionary":base["data"]["dictionary"]});
        assert_eq!(
            expand(&minimal)["errors"][0]["code"],
            "COMPRESSION_CONTEXT_INVALID"
        );
        for path in [
            "/data/mode",
            "/data/coverage/complete",
            "/data/coverage/retainedPaths",
            "/data/coverage/scope/domains",
            "/data/omitted",
            "/data/metrics/inputBytes",
            "/data/budget/maxBytes",
        ] {
            let mut context = base.clone();
            let (parent, key) = path.rsplit_once('/').unwrap();
            context
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert_eq!(
                expand(&context)["errors"][0]["code"],
                "COMPRESSION_CONTEXT_INVALID",
                "missing {path}"
            );
        }
        for (path, value) in [
            ("/data/mode", json!("unknown")),
            ("/data/coverage/complete", json!(1)),
            ("/data/coverage/retainedPaths", json!([false])),
            ("/data/coverage/scope/sections", json!([1])),
            (
                "/data/omitted",
                json!([{"path":"/x","reason":"outside","extra":true}]),
            ),
            ("/data/metrics/inputBytes", json!(-1)),
            ("/data/metrics/outputBytes", json!(2.5)),
            ("/data/metrics/tokenEstimateMethod", json!("characters")),
            ("/data/budget/maxBytes", json!(0)),
            ("/data/budget/maxTokens", json!(MAX_CONTEXT_BYTES + 1)),
            ("/data/budget/exceeded", json!(null)),
        ] {
            let mut context = base.clone();
            *context.pointer_mut(path).unwrap() = value;
            assert_eq!(
                expand(&context)["errors"][0]["code"],
                "COMPRESSION_CONTEXT_INVALID",
                "invalid {path}"
            );
        }
        for path in [
            "/data",
            "/data/coverage",
            "/data/coverage/scope",
            "/data/metrics",
            "/data/budget",
        ] {
            let mut context = base.clone();
            context
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unexpected".into(), json!(true));
            assert_eq!(
                expand(&context)["errors"][0]["code"],
                "COMPRESSION_CONTEXT_INVALID",
                "extra {path}"
            );
        }
        let mut integer_notation = base.clone();
        integer_notation["data"]["metrics"]["inputBytes"] = json!(2000.0);
        integer_notation["data"]["budget"]["maxTokens"] = json!(2000.0);
        assert_eq!(expand(&integer_notation), expand(&base));
    }

    #[test]
    fn errors_unknown_selectors_and_budget_overflow_are_explicit() {
        let payload = envelope(json!({"placements":[{"id":"sun","applying":null}]}));
        assert_eq!(
            compress(
                payload.clone(),
                json!({"mode":"focused","domains":["career"]})
            )["errors"][0]["code"],
            "COMPRESSION_SELECTOR_INVALID"
        );
        assert_eq!(
            compress(payload.clone(), json!({"domains":["career"]}))["errors"][0]["code"],
            "COMPRESSION_OPTIONS_INVALID"
        );
        let context = compress(
            payload.clone(),
            json!({"mode":"budgeted","maxTokens":1,"maxBytes":1}),
        );
        assert_eq!(context["data"]["budget"]["exceeded"], true);
        assert_eq!(context["warnings"].as_array().unwrap().len(), 1);
        assert_eq!(expand(&context), payload);
        assert_eq!(
            context["data"]["metrics"]["outputBytes"].as_u64().unwrap(),
            context.to_string().len() as u64
        );
        let source_error: Value =
            serde_json::from_str(&error_json("SOURCE_ERROR", "source failed")).unwrap();
        assert_eq!(
            expand(&compress(source_error.clone(), json!({}))),
            source_error
        );
    }

    #[test]
    fn expansion_rejects_cycles_unknown_refs_and_malformed_tables() {
        let base = compress(envelope(json!({})), json!({}));
        for (payload, dictionary) in [
            (json!({REF:"missing"}), json!({})),
            (json!({REF:"r0"}), json!({"r0":{REF:"r1"},"r1":{REF:"r0"}})),
            (
                json!({TABLE:{"columns":["x","x"],"rows":[[1,2]]}}),
                json!({}),
            ),
            (json!({TABLE:{"columns":["x"],"rows":[[1,2]]}}), json!({})),
            (json!({LITERAL:[["x",1],["x",2]]}), json!({})),
            (
                base["data"]["payload"].clone(),
                json!({"unused":{REF:"unused"}}),
            ),
            (
                base["data"]["payload"].clone(),
                json!({"unused":{TABLE:{"columns":["x"],"rows":[[]]}}}),
            ),
            (
                json!({TABLE:{"columns":{REF:"r0"},"rows":[[1]]}}),
                json!({"r0":["x"]}),
            ),
        ] {
            let mut context = base.clone();
            context["data"]["payload"] = payload;
            context["data"]["dictionary"] = dictionary;
            assert_eq!(
                expand(&context)["errors"][0]["code"],
                "COMPRESSION_CONTEXT_INVALID"
            );
        }
        let empty = Map::new();
        let mut decoder = Decoder {
            dictionary: &empty,
            active: HashSet::new(),
            bytes: MAX_CONTEXT_BYTES,
        };
        assert!(decoder.decode(&json!("next"), 0).is_err());
    }

    #[test]
    fn real_natal_couple_composite_forecast_fixtures_roundtrip_and_focus() {
        for (name, domain, section) in [
            ("natal-domains", "career", "profession"),
            ("couple", "attraction", "chemistry"),
            ("composite", "career", "profession"),
            ("forecast-day", "career", "profession"),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../../examples/{name}-result.json"));
            let payload: Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let context = compress(payload.clone(), json!({}));
            assert_eq!(context["errors"], json!([]), "{name}");
            let restored = expand(&context);
            assert!(
                restored == payload,
                "{name}: {}",
                first_difference(&restored, &payload, "").unwrap_or_default()
            );
            assert!(
                serialized_len(&context["data"]["payload"])
                    + serialized_len(&context["data"]["dictionary"])
                    < serialized_len(&payload),
                "{name}"
            );
            let focused = compress(
                payload.clone(),
                json!({"mode":"focused","domains":[domain],"sections":[section]}),
            );
            assert_eq!(focused["errors"], json!([]), "{name}");
            let restored = expand(&focused);
            assert_eq!(restored["calculation"], payload["calculation"], "{name}");
            assert_eq!(restored["warnings"], payload["warnings"], "{name}");
            let base = if name == "composite" {
                "/data/composite"
            } else {
                "/data"
            };
            assert_eq!(
                restored.pointer(&format!("{base}/context")),
                payload.pointer(&format!("{base}/context")),
                "{name}"
            );
            assert_eq!(
                restored
                    .pointer(&format!("{base}/domains"))
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .len(),
                1,
                "{name}"
            );
            if name == "forecast-day" {
                assert_eq!(restored["data"]["subject"], payload["data"]["subject"]);
                assert_eq!(restored["data"]["snapshot"], payload["data"]["snapshot"]);
                assert_eq!(restored["data"]["events"], payload["data"]["events"]);
            }
        }
    }
}
