//! Bounded chronological search for geometric astronomical events.
//! Civil periods use a fixed offset; roots are refined in provider UT1 time.
use super::*;
use astro_provider_swiss::{CivilUtc, EphemerisSession};
use std::collections::HashMap;

pub(super) const BODY_IDS: [&str; 10] = [
    "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune", "pluto",
];
const EVENT_TYPES: [&str; 6] = [
    "ingress",
    "station",
    "lunarPhase",
    "planetaryAspect",
    "solarEclipse",
    "lunarEclipse",
];
const SAMPLE_DAYS: f64 = 0.25;
const TIME_TOLERANCE: f64 = 0.25;
const ANGULAR_TOLERANCE: f64 = 1e-6;
const STATION_TOLERANCE: f64 = 1e-8;
const EXTREMA_TOLERANCE: f64 = 1e-7;
const TANGENCY_TOLERANCE: f64 = 1e-10;
const DUPLICATE_SECONDS: f64 = 0.001;
const MAX_ITERATIONS: usize = 80;
const MAX_EVENTS: usize = 30000;
const MAX_EVALUATIONS: usize = 2_000_000;

fn present_integer<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<i32>, D::Error> {
    numbers::i32(d).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Period {
    pub kind: String,
    #[serde(deserialize_with = "numbers::i32")]
    pub year: i32,
    #[serde(default, deserialize_with = "present_integer")]
    pub month: Option<i32>,
    #[serde(default, deserialize_with = "present_integer")]
    pub day: Option<i32>,
    #[serde(default, deserialize_with = "numbers::i32")]
    pub utc_offset_minutes: i32,
}

pub(super) struct PeriodBounds {
    period: Period,
    start: CivilUtc,
    end: CivilUtc,
}

// Gregorian days relative to 1970-01-01; integer civil arithmetic never assumes UT1=UTC.
fn civil_days(year: i32, month: i32, day: i32) -> i64 {
    let year = year as i64 - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let m = month as i64 + if month > 2 { -3 } else { 9 };
    era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + (153 * m + 2) / 5 + day as i64 - 1 - 719468
}
fn date_from_days(days: i64) -> (i32, i32, i32) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    ((y + i64::from(m <= 2)) as i32, m as i32, d as i32)
}
fn civil_from_milliseconds(milliseconds: i64) -> CivilUtc {
    let days = milliseconds.div_euclid(86_400_000);
    let remaining = milliseconds.rem_euclid(86_400_000);
    let (year, month, day) = date_from_days(days);
    CivilUtc {
        year,
        month,
        day,
        hour: (remaining / 3_600_000) as i32,
        minute: (remaining % 3_600_000 / 60_000) as i32,
        second: (remaining % 60_000) as f64 / 1000.,
    }
}
fn civil_milliseconds(utc: &CivilUtc) -> i64 {
    civil_days(utc.year, utc.month, utc.day) * 86_400_000
        + utc.hour as i64 * 3_600_000
        + utc.minute as i64 * 60_000
        + (utc.second * 1000.).round() as i64
}
fn utc_value(utc: &CivilUtc) -> Value {
    json!({"year":utc.year,"month":utc.month,"day":utc.day,"hour":utc.hour,"minute":utc.minute,"second":utc.second})
}
fn utc_args(utc: &CivilUtc) -> [i32; 5] {
    [utc.year, utc.month, utc.day, utc.hour, utc.minute]
}

impl Period {
    /// Validate every civil selector and offset before any provider calculation.
    pub(super) fn validate(&self) -> Result<PeriodBounds, String> {
        if !(1800..=2399).contains(&self.year) || !(-840..=840).contains(&self.utc_offset_minutes) {
            return Err("Period requires year 1800..2399 and utcOffsetMinutes -840..840".into());
        }
        let (month,day) = match self.kind.as_str() {
            "year" if self.month.is_none() && self.day.is_none() => (1,1),
            "month" if self.month.is_some() && self.day.is_none() => (self.month.unwrap(),1),
            "day" if self.month.is_some() && self.day.is_some() => (self.month.unwrap(),self.day.unwrap()),
            _ => return Err("Period year accepts only year; month requires month without day; day requires month and day".into()),
        };
        if !(1..=12).contains(&month) {
            return Err("Period month must be 1..12".into());
        }
        let leap = self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0);
        let month_days = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        if !(1..=month_days[(month - 1) as usize]).contains(&day) {
            return Err("Period Gregorian day is invalid".into());
        }
        let start_days = civil_days(self.year, month, day);
        let end_days = match self.kind.as_str() {
            "day" => start_days + 1,
            "month" if month == 12 => civil_days(self.year + 1, 1, 1),
            "month" => civil_days(self.year, month + 1, 1),
            _ => civil_days(self.year + 1, 1, 1),
        };
        let offset = self.utc_offset_minutes as i64 * 60_000;
        let start_ms = start_days * 86_400_000 - offset;
        let end_ms = end_days * 86_400_000 - offset;
        if start_ms < civil_days(1800, 1, 1) * 86_400_000
            || end_ms > civil_days(2400, 1, 1) * 86_400_000
        {
            return Err("Offset-adjusted UTC period must stay within 1800-01-01 inclusive and 2400-01-01 exclusive".into());
        }
        Ok(PeriodBounds {
            period: Period {
                kind: self.kind.clone(),
                year: self.year,
                month: self.month,
                day: self.day,
                utc_offset_minutes: self.utc_offset_minutes,
            },
            start: civil_from_milliseconds(start_ms),
            end: civil_from_milliseconds(end_ms),
        })
    }
}

pub(super) struct ScanOptions {
    pub bodies: Vec<String>,
    pub event_types: Vec<String>,
    pub rules: Vec<AspectRule>,
}
impl ScanOptions {
    pub(super) fn validate(&self) -> Result<(), String> {
        let mut bodies = HashSet::new();
        if self.bodies.is_empty()
            || self.bodies.len() > 10
            || self
                .bodies
                .iter()
                .any(|b| !BODY_IDS.contains(&b.as_str()) || !bodies.insert(b))
        {
            return Err("bodies requires 1 to 10 unique Sun-through-Pluto IDs".into());
        }
        let mut types = HashSet::new();
        if self.event_types.len() > 6
            || self
                .event_types
                .iter()
                .any(|v| !EVENT_TYPES.contains(&v.as_str()) || !types.insert(v))
        {
            return Err(
                "eventTypes accepts unique ingress, station, lunarPhase, planetaryAspect, solarEclipse and lunarEclipse".into(),
            );
        }
        validate_aspect_rules(&self.rules)
    }
    fn indices(&self) -> Vec<usize> {
        (0..10)
            .filter(|i| self.bodies.iter().any(|b| b == BODY_IDS[*i]))
            .collect()
    }
}
pub(super) struct Target {
    pub id: String,
    pub longitude: f64,
}
pub(super) struct ScanResult {
    pub events: Vec<Value>,
    pub metadata: Value,
    pub snapshot: Value,
    pub period: Value,
}

struct Sampler {
    session: EphemerisSession,
    cache: HashMap<(u64, usize), [f64; 6]>,
    evaluations: usize,
    window: Option<(f64, f64, i64, i64)>,
}
impl Sampler {
    fn position(&mut self, jd: f64, body: usize) -> Result<[f64; 6], String> {
        let key = (jd.to_bits(), body);
        if let Some(p) = self.cache.get(&key) {
            return Ok(*p);
        }
        if self.evaluations >= MAX_EVALUATIONS {
            return Err("Event search exceeded the explicit provider evaluation budget; no partial payload is returned".into());
        }
        let p = self.session.positions(jd, &[body])?[0];
        self.evaluations += 1;
        self.cache.insert(key, p);
        Ok(p)
    }
    fn times(&mut self, jd: f64, offset: i32) -> Result<(Value, Value), String> {
        let source = self.session.utc_from_ut1(jd)?;
        let (utc, mut local) = if source.second >= 60. {
            // A true leap second belongs to the preceding minute, including at local offset.
            let mut utc = source;
            utc.second = ((utc.second * 1000.).round() / 1000.).min(60.999);
            let minute_start = civil_days(utc.year, utc.month, utc.day) * 86_400_000
                + utc.hour as i64 * 3_600_000
                + utc.minute as i64 * 60_000;
            let mut local = civil_from_milliseconds(minute_start + offset as i64 * 60_000);
            local.second = utc.second;
            (utc, utc_value(&local))
        } else {
            let mut milliseconds = civil_milliseconds(&source);
            if let Some((start, end, start_ms, end_ms)) = self.window {
                if jd >= start && jd < end {
                    milliseconds = milliseconds.clamp(start_ms, end_ms - 1);
                }
            }
            let utc = civil_from_milliseconds(milliseconds);
            let local = civil_from_milliseconds(milliseconds + offset as i64 * 60_000);
            (utc, utc_value(&local))
        };
        local["utcOffsetMinutes"] = json!(offset);
        Ok((utc_value(&utc), local))
    }
}
fn position_value(body: usize, p: [f64; 6]) -> Value {
    let longitude = normalize(p[0]);
    json!({"id":BODY_IDS[body],"longitude":longitude,"latitude":p[1],"distanceAu":p[2],"speed":p[3],"isRetrograde":p[3]<0.,"sign":SIGNS[(longitude/30.).floor() as usize],"degreeInSign":longitude%30.})
}
#[derive(Clone, Copy)]
struct Model {
    first: usize,
    second: Option<usize>,
    fixed: f64,
}
impl Model {
    fn evaluate(&self, sampler: &mut Sampler, t: f64) -> Result<(f64, f64), String> {
        let first = sampler.position(t, self.first)?;
        let (longitude, speed) = if let Some(body) = self.second {
            let other = sampler.position(t, body)?;
            (other[0], other[3])
        } else {
            (self.fixed, 0.)
        };
        Ok((normalize(first[0] - longitude), first[3] - speed))
    }
}
#[derive(Clone, Copy)]
struct Root {
    t: f64,
    bracket_seconds: f64,
    residual: f64,
    // Proof from the original sign-changing bracket, before rate noise near zero.
    crossing_direction: f64,
}
fn refine<F>(
    mut left: f64,
    mut right: f64,
    mut fl: f64,
    mut fr: f64,
    tolerance: f64,
    mut evaluate: F,
) -> Result<Root, String>
where
    F: FnMut(f64) -> Result<f64, String>,
{
    let crossing_direction = if fl * fr < 0. { (fr - fl).signum() } else { 0. };
    if fl == 0. {
        return Ok(Root {
            t: left,
            bracket_seconds: 0.,
            residual: 0.,
            crossing_direction,
        });
    }
    if fr == 0. {
        return Ok(Root {
            t: right,
            bracket_seconds: 0.,
            residual: 0.,
            crossing_direction,
        });
    }
    if fl.signum() == fr.signum() {
        return Err("Root refinement requires a genuine sign bracket".into());
    }
    let mut best = if fl.abs() < fr.abs() {
        (left, fl)
    } else {
        (right, fr)
    };
    for _ in 0..MAX_ITERATIONS {
        if (right - left) * 86400. <= TIME_TOLERANCE && best.1.abs() <= tolerance {
            return Ok(Root {
                t: best.0,
                bracket_seconds: (right - left) * 86400.,
                residual: best.1.abs(),
                crossing_direction,
            });
        }
        let secant = left + (right - left) * (-fl) / (fr - fl);
        let span = right - left;
        let t = if secant > left + span * 0.1 && secant < right - span * 0.1 {
            secant
        } else {
            left + span / 2.
        };
        if t == left || t == right {
            break;
        }
        let f = evaluate(t)?;
        if !f.is_finite() {
            return Err("Provider returned a non-finite root function".into());
        }
        if f == 0. {
            return Ok(Root {
                t,
                bracket_seconds: 0.,
                residual: 0.,
                crossing_direction,
            });
        }
        if f.signum() == fl.signum() {
            left = t;
            fl = f;
        } else {
            right = t;
            fr = f;
        }
        best = if fl.abs() < fr.abs() {
            (left, fl)
        } else {
            (right, fr)
        };
    }
    if (right - left) * 86400. <= TIME_TOLERANCE && best.1.abs() <= tolerance {
        Ok(Root {
            t: best.0,
            bracket_seconds: (right - left) * 86400.,
            residual: best.1.abs(),
            crossing_direction,
        })
    } else {
        Err(format!("Event root did not meet declared tolerances: bracketSeconds={}, residual={}, requiredResidual={}, UT1 range=[{},{}]; no partial payload is returned", (right-left)*86400.,best.1.abs(),tolerance,left,right))
    }
}
fn extrema(
    sampler: &mut Sampler,
    model: Model,
    left: f64,
    right: f64,
) -> Result<Vec<Root>, String> {
    let times = [left, left + (right - left) / 2., right];
    let mut roots = Vec::new();
    let mut speeds = Vec::new();
    for t in times {
        speeds.push(model.evaluate(sampler, t)?.1);
    }
    for i in 0..2 {
        if speeds[i] == 0. {
            roots.push(Root {
                t: times[i],
                bracket_seconds: 0.,
                residual: 0.,
                crossing_direction: 0.,
            });
        }
        if speeds[i] * speeds[i + 1] < 0. {
            roots.push(
                refine(
                    times[i],
                    times[i + 1],
                    speeds[i],
                    speeds[i + 1],
                    if model.second.is_some() {
                        EXTREMA_TOLERANCE
                    } else {
                        STATION_TOLERANCE
                    },
                    |t| Ok(model.evaluate(sampler, t)?.1),
                )
                .map_err(|e| {
                    format!(
                        "Velocity extremum {} / {:?}: {e}",
                        BODY_IDS[model.first],
                        model.second.map(|i| BODY_IDS[i])
                    )
                })?,
            );
        }
    }
    if speeds[2] == 0. {
        roots.push(Root {
            t: right,
            bracket_seconds: 0.,
            residual: 0.,
            crossing_direction: 0.,
        });
    }
    roots.sort_by(|a, b| a.t.total_cmp(&b.t));
    roots.dedup_by(|a, b| (a.t - b.t).abs() * 86400. < DUPLICATE_SECONDS);
    Ok(roots)
}
struct Spec {
    kind: &'static str,
    model: Model,
    branches: Vec<(f64, f64)>,
    target: Option<(String, f64)>,
}
fn aspect_branches(rules: &[AspectRule]) -> Vec<(f64, f64)> {
    let mut branches = Vec::new();
    for r in rules {
        branches.push((r.angle, r.angle));
        if r.angle != 0. && r.angle != 180. {
            branches.push((360. - r.angle, r.angle));
        }
    }
    branches
}
fn specs(options: &ScanOptions, targets: &[Target]) -> Vec<Spec> {
    let selected = options.indices();
    let mut all = Vec::new();
    if options.event_types.iter().any(|v| v == "ingress") {
        for body in &selected {
            all.push(Spec {
                kind: "ingress",
                model: Model {
                    first: *body,
                    second: None,
                    fixed: 0.,
                },
                branches: (0..12).map(|i| (i as f64 * 30., i as f64 * 30.)).collect(),
                target: None,
            });
        }
    }
    if options.event_types.iter().any(|v| v == "lunarPhase") {
        all.push(Spec {
            kind: "lunarPhase",
            model: Model {
                first: 1,
                second: Some(0),
                fixed: 0.,
            },
            branches: vec![(0., 0.), (90., 90.), (180., 180.), (270., 270.)],
            target: None,
        });
    }
    if options.event_types.iter().any(|v| v == "planetaryAspect") {
        for (i, first) in selected.iter().enumerate() {
            for second in &selected[i + 1..] {
                all.push(Spec {
                    kind: "planetaryAspect",
                    model: Model {
                        first: *first,
                        second: Some(*second),
                        fixed: 0.,
                    },
                    branches: aspect_branches(&options.rules),
                    target: None,
                });
            }
        }
    }
    for first in selected {
        for target in targets {
            all.push(Spec {
                kind: "natalTransit",
                model: Model {
                    first,
                    second: None,
                    fixed: target.longitude,
                },
                branches: aspect_branches(&options.rules),
                target: Some((target.id.clone(), target.longitude)),
            });
        }
    }
    all
}
fn event(
    sampler: &mut Sampler,
    spec: &Spec,
    root: Root,
    branch: f64,
    angle: f64,
    direction: f64,
    offset: i32,
) -> Result<Value, String> {
    let (utc, local) = sampler.times(root.t, offset)?;
    let mut body_indices = vec![spec.model.first];
    if let Some(second) = spec.model.second {
        body_indices.push(second);
    }
    body_indices.sort_unstable();
    let mut positions = Vec::new();
    for body in &body_indices {
        positions.push(position_value(*body, sampler.position(root.t, *body)?));
    }
    let details = match spec.kind {
        "ingress" => {
            let upper = (branch / 30.).round() as usize % 12;
            let lower = (upper + 11) % 12;
            let (from, to) = if direction > 0. {
                (lower, upper)
            } else {
                (upper, lower)
            };
            json!({"boundaryLongitude":normalize(branch),"fromSign":SIGNS[from],"toSign":SIGNS[to],"direction":if direction>0.{"direct"}else{"retrograde"}})
        }
        "lunarPhase" => {
            json!({"phase":match angle as usize {0=>"newMoon",90=>"firstQuarter",180=>"fullMoon",_=>"lastQuarter"},"angle":angle})
        }
        "planetaryAspect" => json!({"angle":angle,"branchLongitude":normalize(branch)}),
        "natalTransit" => {
            let (id, longitude) = spec.target.as_ref().unwrap();
            json!({"angle":angle,"branchLongitude":normalize(branch),"targetPointId":id,"targetLongitude":longitude})
        }
        "station" => json!({"direction":if direction>0.{"direct"}else{"retrograde"}}),
        _ => unreachable!(),
    };
    let body_ids: Vec<&str> = body_indices.iter().map(|i| BODY_IDS[*i]).collect();
    let label = if let Some((id, _)) = &spec.target {
        format!("{id}:{branch}")
    } else {
        format!("{branch}")
    };
    let id = format!(
        "{}:{}:{}:ut1-{:.8}",
        spec.kind,
        body_ids.join("+"),
        label,
        root.t
    );
    Ok(
        json!({"id":id,"type":spec.kind,"utc":utc,"local":local,"julianDayUt1":root.t,"bodyIds":body_ids,"positions":positions,"details":details,"precision":{"method":"bracketedRoot","bracketSeconds":root.bracket_seconds,"residual":root.residual,"residualUnit":if spec.kind=="station"{"degrees/day"}else{"degrees"}}}),
    )
}
fn append(events: &mut Vec<Value>, value: Value, start: f64, end: f64) -> Result<(), String> {
    let t = value["julianDayUt1"].as_f64().unwrap();
    if t < start || t >= end {
        return Ok(());
    }
    // Same root can touch two monotonic subintervals; include it only once.
    let duplicate = events.iter().rev().take(500).any(|v| {
        v["type"] == value["type"]
            && v["bodyIds"] == value["bodyIds"]
            && v["details"] == value["details"]
            && (v["julianDayUt1"].as_f64().unwrap() - t).abs() * 86400. <= DUPLICATE_SECONDS
    });
    if !duplicate {
        if events.len() >= MAX_EVENTS {
            return Err("Event search exceeded the explicit 30000 event budget; no partial payload is returned".into());
        }
        events.push(value);
    }
    Ok(())
}

fn append_eclipses(
    sampler: &mut Sampler,
    events: &mut Vec<Value>,
    start: f64,
    end: f64,
    offset: i32,
    solar: bool,
) -> Result<(), String> {
    let family = if solar {
        "solarEclipse"
    } else {
        "lunarEclipse"
    };
    // Swiss excludes a maximum within 0.0001 day of its search start.
    let mut cursor = start - 0.001;
    for _ in 0..64 {
        let (flags, times) = sampler.session.next_eclipse(cursor, solar)?;
        let maximum = times[0];
        if maximum >= end {
            return Ok(());
        }
        if maximum < start {
            cursor = maximum + 0.001;
            continue;
        }
        let eclipse_type = if flags & 32 != 0 {
            "hybrid"
        } else if flags & 4 != 0 {
            "total"
        } else if flags & 8 != 0 {
            "annular"
        } else if flags & 16 != 0 {
            "partial"
        } else if flags & 64 != 0 {
            "penumbral"
        } else {
            return Err("Provider returned an unsupported global eclipse classification".into());
        };
        let mapping: &[(usize, &str)] = if solar {
            &[
                (2, "eclipseBegin"),
                (3, "eclipseEnd"),
                (4, "centralPhaseBegin"),
                (5, "centralPhaseEnd"),
            ]
        } else {
            &[
                (2, "partialBegin"),
                (3, "partialEnd"),
                (4, "totalityBegin"),
                (5, "totalityEnd"),
                (6, "penumbralBegin"),
                (7, "penumbralEnd"),
            ]
        };
        let mut contacts = Vec::new();
        for (index, name) in mapping {
            if times[*index] > 0. {
                let (utc, local) = sampler.times(times[*index], offset)?;
                contacts.push(
                    json!({"name":name,"utc":utc,"local":local,"julianDayUt1":times[*index]}),
                );
            }
        }
        contacts.sort_by(|a, b| {
            a["julianDayUt1"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["julianDayUt1"].as_f64().unwrap())
        });
        let (utc, local) = sampler.times(maximum, offset)?;
        let positions = vec![
            position_value(0, sampler.position(maximum, 0)?),
            position_value(1, sampler.position(maximum, 1)?),
        ];
        let value = json!({"id":format!("{family}:sun+moon:{eclipse_type}:ut1-{maximum:.8}"),"type":family,"utc":utc,"local":local,"julianDayUt1":maximum,"bodyIds":["sun","moon"],"positions":positions,
            "details":{"eclipseType":eclipse_type,"visibilityScope":"global","providerFlags":flags,"contacts":contacts},
            "precision":{"method":"swissEclipseSearch","bracketSeconds":null,"residual":null,"residualUnit":null}});
        append(events, value, start, end)?;
        cursor = maximum + 0.001;
    }
    Err("Global eclipse search exceeded its explicit iteration budget; no partial payload is returned".into())
}

pub(super) fn scan(
    bounds: &PeriodBounds,
    options: &ScanOptions,
    targets: &[Target],
) -> Result<ScanResult, String> {
    options.validate()?;
    let mut ids = HashSet::new();
    if targets.len() > 26
        || targets
            .iter()
            .any(|t| t.id.is_empty() || !t.longitude.is_finite() || !ids.insert(&t.id))
    {
        return Err("Require at most 26 unique finite fixed natal target points".into());
    }
    let mut sampler = Sampler {
        session: EphemerisSession::new()?,
        cache: HashMap::new(),
        evaluations: 0,
        window: None,
    };
    let start = sampler
        .session
        .julian_days(utc_args(&bounds.start), bounds.start.second)?[1];
    let end = sampler
        .session
        .julian_days(utc_args(&bounds.end), bounds.end.second)?[1];
    sampler.window = Some((
        start,
        end,
        civil_milliseconds(&bounds.start),
        civil_milliseconds(&bounds.end),
    ));
    let offset = bounds.period.utc_offset_minutes;
    let all_specs = specs(options, targets);
    let selected = options.indices();
    let mut events = Vec::new();
    let mut grid_intervals = 0;
    let mut left = start;
    while left < end {
        let right = (left + SAMPLE_DAYS).min(end);
        grid_intervals += 1;
        let mut turns: HashMap<(usize, Option<usize>), Vec<Root>> = HashMap::new();
        for spec in &all_specs {
            let key = (spec.model.first, spec.model.second);
            if let std::collections::hash_map::Entry::Vacant(entry) = turns.entry(key) {
                entry.insert(extrema(&mut sampler, spec.model, left, right)?);
            }
            let extrema = &turns[&key];
            let (origin, _) = spec.model.evaluate(&mut sampler, left)?;
            let mut divisions = vec![left];
            divisions.extend(
                extrema
                    .iter()
                    .filter(|r| r.t > left && r.t < right)
                    .map(|r| r.t),
            );
            divisions.push(right);
            let mut crossed_branches = HashSet::new();
            for segment in divisions.windows(2) {
                let (a, sa) = spec.model.evaluate(&mut sampler, segment[0])?;
                let (b, sb) = spec.model.evaluate(&mut sampler, segment[1])?;
                if (segment[1] - segment[0]) * sa.abs().max(sb.abs()) >= 90. {
                    return Err("Angular path exceeds the bounded unwrapping domain; no partial payload is returned".into());
                }
                let a = origin + signed_difference(a, origin);
                let b = origin + signed_difference(b, origin);
                for (branch, angle) in &spec.branches {
                    let low = a.min(b);
                    let high = a.max(b);
                    let first = ((low - branch) / 360.).ceil() as i32;
                    let last = ((high - branch) / 360.).floor() as i32;
                    for cycle in first..=last {
                        let target = branch + cycle as f64 * 360.;
                        let fl = a - target;
                        let fr = b - target;
                        if fl == 0. && fr == 0. {
                            continue;
                        }
                        let root =
                            refine(segment[0], segment[1], fl, fr, ANGULAR_TOLERANCE, |t| {
                                Ok(origin
                                    + signed_difference(
                                        spec.model.evaluate(&mut sampler, t)?.0,
                                        origin,
                                    )
                                    - target)
                            })?;
                        crossed_branches.insert(branch.to_bits());
                        if spec.kind == "ingress"
                            && spec.model.evaluate(&mut sampler, root.t)?.1.abs()
                                <= STATION_TOLERANCE
                        {
                            let before = signed_difference(
                                spec.model.evaluate(&mut sampler, root.t - 1. / 86400.)?.0,
                                *branch,
                            );
                            let after = signed_difference(
                                spec.model.evaluate(&mut sampler, root.t + 1. / 86400.)?.0,
                                *branch,
                            );
                            if before * after >= 0. {
                                continue;
                            }
                        }
                        let value =
                            event(&mut sampler, spec, root, *branch, *angle, b - a, offset)?;
                        append(&mut events, value, start, end)?;
                    }
                }
            }
            // An exact tangency is a contact, but does not constitute a sign ingress.
            if spec.kind != "ingress" {
                for turn in extrema {
                    let value = spec.model.evaluate(&mut sampler, turn.t)?.0;
                    for (branch, angle) in &spec.branches {
                        let residual = signed_difference(value, *branch).abs();
                        // Convergence tolerance is never an orb for creating a candidate.
                        if residual <= TANGENCY_TOLERANCE
                            && !crossed_branches.contains(&branch.to_bits())
                        {
                            let root = Root { residual, ..*turn };
                            let value =
                                event(&mut sampler, spec, root, *branch, *angle, 0., offset)?;
                            append(&mut events, value, start, end)?;
                        }
                    }
                }
            }
        }
        if options.event_types.iter().any(|v| v == "station") {
            for body in &selected {
                let model = Model {
                    first: *body,
                    second: None,
                    fixed: 0.,
                };
                let key = (*body, None);
                if let std::collections::hash_map::Entry::Vacant(entry) = turns.entry(key) {
                    entry.insert(extrema(&mut sampler, model, left, right)?);
                }
                for root in &turns[&key] {
                    let direction = if root.crossing_direction != 0. {
                        root.crossing_direction
                    } else {
                        // Exact sample zeros have no bracket; compare one hour away.
                        // Pluto's second-scale rates are below the provider's noise floor.
                        let before = model.evaluate(&mut sampler, root.t - 1. / 24.)?.1;
                        let after = model.evaluate(&mut sampler, root.t + 1. / 24.)?.1;
                        if before * after < 0. {
                            (after - before).signum()
                        } else {
                            0.
                        }
                    };
                    if direction != 0. {
                        let spec = Spec {
                            kind: "station",
                            model,
                            branches: Vec::new(),
                            target: None,
                        };
                        let value = event(&mut sampler, &spec, *root, 0., 0., direction, offset)?;
                        append(&mut events, value, start, end)?;
                    }
                }
            }
        }
        left = right;
    }
    for solar in [true, false] {
        let family = if solar {
            "solarEclipse"
        } else {
            "lunarEclipse"
        };
        if options.event_types.iter().any(|v| v == family) {
            append_eclipses(&mut sampler, &mut events, start, end, offset, solar)?;
        }
    }
    events.sort_by(|a, b| {
        a["julianDayUt1"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["julianDayUt1"].as_f64().unwrap())
            .then_with(|| a["id"].as_str().unwrap().cmp(b["id"].as_str().unwrap()))
    });
    let mut last_by_identity: HashMap<String, f64> = HashMap::new();
    events.retain(|event| {
        let key = format!(
            "{}:{}:{}",
            event["type"], event["bodyIds"], event["details"]
        );
        let t = event["julianDayUt1"].as_f64().unwrap();
        let duplicate = last_by_identity
            .get(&key)
            .is_some_and(|previous| (t - previous).abs() * 86400. <= DUPLICATE_SECONDS);
        if !duplicate {
            last_by_identity.insert(key, t);
        }
        !duplicate
    });
    let midpoint = start + (end - start) / 2.;
    let (utc, local) = sampler.times(midpoint, offset)?;
    let mut positions = Vec::new();
    for body in selected {
        positions.push(position_value(body, sampler.position(midpoint, body)?));
    }
    let period = &bounds.period;
    Ok(ScanResult {
        events,
        snapshot: json!({"utc":utc,"local":local,"julianDayUt1":midpoint,"positions":positions}),
        period: json!({"kind":period.kind,"year":period.year,"month":period.month,"day":period.day,"utcOffsetMinutes":offset,"startUtc":utc_value(&bounds.start),"endUtc":utc_value(&bounds.end),"startJulianDayUt1":start,"endJulianDayUt1":end,"durationDays":end-start}),
        metadata: json!({"samplingHours":6,"timeToleranceSeconds":TIME_TOLERANCE,"angularToleranceDegrees":ANGULAR_TOLERANCE,"stationToleranceDegreesPerDay":STATION_TOLERANCE,"velocityExtremaToleranceDegreesPerDay":EXTREMA_TOLERANCE,"tangencyToleranceDegrees":TANGENCY_TOLERANCE,"duplicateTimeToleranceSeconds":DUPLICATE_SECONDS,"maximumProviderBodyEvaluations":MAX_EVALUATIONS,"maximumRefinementIterations":MAX_ITERATIONS,"rootIsolation":"velocityExtremaPartitioned","interval":"startInclusiveEndExclusive","truncated":false,"eventLimit":MAX_EVENTS,"gridIntervals":grid_intervals,"eclipseMethod":"Swiss global eclipse maximum, no local visibility"}),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EventRequest {
    operation: String,
    period: Period,
    #[serde(default, deserialize_with = "natal::present")]
    bodies: Option<Vec<String>>,
    #[serde(default, deserialize_with = "natal::present")]
    event_types: Option<Vec<String>>,
    #[serde(default)]
    aspect_rules: natal::AspectSettings,
    #[serde(default, deserialize_with = "natal::present")]
    aspect_preset: Option<String>,
}
pub(super) fn coverage(events: &[Value]) -> Value {
    let mut types = json!({"ingress":0,"station":0,"lunarPhase":0,"planetaryAspect":0,"natalTransit":0,"solarEclipse":0,"lunarEclipse":0});
    for event in events {
        let key = event["type"].as_str().unwrap();
        let count = types[key].as_u64().unwrap();
        types[key] = json!(count + 1);
    }
    json!({"eventCount":events.len(),"byType":types})
}
pub(super) fn calculate_json(raw: &str) -> String {
    let r: EventRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => return error_json("INVALID_INPUT", &e.to_string()),
    };
    if r.operation != "events" {
        return error_json("INVALID_INPUT", "Expected events operation");
    }
    let preset = r.aspect_preset.as_deref().unwrap_or("major");
    if !["major", "extended"].contains(&preset)
        || (r.aspect_preset.is_some() && r.aspect_rules.provided)
    {
        return error_json(
            "INVALID_INPUT",
            "aspectPreset must be major or extended and cannot be combined with aspectRules",
        );
    }
    let options = ScanOptions {
        bodies: r
            .bodies
            .unwrap_or_else(|| BODY_IDS.into_iter().map(String::from).collect()),
        event_types: r
            .event_types
            .unwrap_or_else(|| EVENT_TYPES.into_iter().map(String::from).collect()),
        rules: if r.aspect_rules.provided {
            r.aspect_rules.rules
        } else if preset == "extended" {
            domains::extended_rules()
        } else {
            default_rules()
        },
    };
    if let Err(e) = options.validate() {
        return error_json("INVALID_INPUT", &e);
    }
    let period = match r.period.validate() {
        Ok(p) => p,
        Err(e) => return error_json("INVALID_INPUT", &e),
    };
    let result = match scan(&period, &options, &[]) {
        Ok(r) => r,
        Err(e) => return error_json("CALCULATION_FAILED", &e),
    };
    let calculation = json!({"scope":"astronomical-event-data","provider":"swiss-ephemeris","providerVersion":"2.10.03","ephemeris":"moshier","zodiac":"tropical","coordinates":"geocentric","positionType":"apparent","referenceFrame":"ecliptic-of-date","calendar":"gregorian","inputTimeScale":"UTC","planetTimeScale":"TT","searchTimeScale":"UT1","timeModel":"Swiss built-in leap seconds and Delta T; pre-1972 civil input treated as UT1","angleUnit":"degrees","speedUnit":"degrees/day","distanceUnit":"AU","bodyIds":options.indices().iter().map(|i|BODY_IDS[*i]).collect::<Vec<_>>(),"eventTypes":options.event_types,"aspectPreset":if r.aspect_rules.provided{"custom"}else{preset},"aspectRules":options.rules.iter().map(|r|json!({"angle":r.angle,"maxOrb":r.max_orb})).collect::<Vec<_>>(),"search":result.metadata});
    json!({"schemaVersion":"1.0","engineVersion":VERSION,"calculation":calculation,"data":{"chartKind":"astronomicalEvents","period":result.period,"snapshot":result.snapshot,"coverage":coverage(&result.events),"events":result.events},"warnings":["Moshier analytical ephemeris; no JPL/Swiss data files. Future UTC uses the provider's built-in time model, not live Earth-orientation data.","Periods use a fixed UTC offset, without timezone or daylight-saving inference. Lunar phases and global eclipses use Sun and Moon independently of the selected bodies. Eclipse maxima determine period membership; associated contacts may be outside the period. No local visibility is inferred. Exact events are geometric roots, not predictions of life outcomes."],"errors":[]}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn period(value: Value) -> PeriodBounds {
        serde_json::from_value::<Period>(value)
            .unwrap()
            .validate()
            .unwrap()
    }
    fn options(bodies: &[&str], types: &[&str], rules: Vec<AspectRule>) -> ScanOptions {
        ScanOptions {
            bodies: bodies.iter().map(|v| v.to_string()).collect(),
            event_types: types.iter().map(|v| v.to_string()).collect(),
            rules,
        }
    }
    fn run(value: Value) -> Value {
        serde_json::from_str(&calculate_json(&value.to_string())).unwrap()
    }
    #[test]
    fn calendar_period_offset_and_exact_integer_validation() {
        let leap = period(json!({"kind":"month","year":2000,"month":2,"utcOffsetMinutes":420}));
        assert_eq!(
            (
                leap.start.year,
                leap.start.month,
                leap.start.day,
                leap.start.hour
            ),
            (2000, 1, 31, 17)
        );
        assert_eq!((leap.end.month, leap.end.day, leap.end.hour), (2, 29, 17));
        assert_eq!(civil_days(2000, 3, 1) - civil_days(2000, 2, 1), 29);
        for raw in [
            r#"{"kind":"year","year":2000.0}"#,
            r#"{"kind":"year","year":2e3,"utcOffsetMinutes":0.0}"#,
        ] {
            assert!(serde_json::from_str::<Period>(raw)
                .unwrap()
                .validate()
                .is_ok());
        }
        for value in [
            json!({"kind":"year","year":2026,"month":1}),
            json!({"kind":"month","year":2026}),
            json!({"kind":"day","year":2026,"month":2,"day":29}),
            json!({"kind":"year","year":1800,"utcOffsetMinutes":420}),
            json!({"kind":"year","year":2399,"utcOffsetMinutes":-1}),
        ] {
            assert!(serde_json::from_value::<Period>(value)
                .unwrap()
                .validate()
                .is_err());
        }
        for raw in [
            r#"{"kind":"year","year":2000.0000000000000001}"#,
            r#"{"kind":"month","year":2026,"month":null}"#,
        ] {
            assert!(serde_json::from_str::<Period>(raw).is_err());
        }
        assert!(
            serde_json::from_value::<Period>(json!({"kind":"year","year":2399}))
                .unwrap()
                .validate()
                .is_ok()
        );
    }
    #[test]
    fn full_year_lunar_phases_match_all_fifty_usno_reference_rows() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/usno-moon-phases-2026.json"))
                .unwrap();
        // https://aa.usno.navy.mil/api/moon/phases/year?year=2026, read 2026-10-03.
        let scanned = scan(
            &period(json!({"kind":"year","year":2026})),
            &options(&["mercury"], &["lunarPhase"], vec![]),
            &[],
        )
        .unwrap();
        assert_eq!(scanned.events.len(), 50);
        let mut maximum_difference: f64 = 0.;
        for (event, expected) in scanned
            .events
            .iter()
            .zip(fixture["phases"].as_array().unwrap())
        {
            assert_eq!(event["details"]["phase"], expected["phase"]);
            assert_eq!(event["utc"]["month"], expected["month"]);
            assert_eq!(event["utc"]["day"], expected["day"]);
            let actual = event["utc"]["hour"].as_f64().unwrap() * 3600.
                + event["utc"]["minute"].as_f64().unwrap() * 60.
                + event["utc"]["second"].as_f64().unwrap();
            let reference = expected["hour"].as_f64().unwrap() * 3600.
                + expected["minute"].as_f64().unwrap() * 60.;
            maximum_difference = maximum_difference.max((actual - reference).abs());
            assert!((actual - reference).abs() <= 90., "{event} vs {expected}");
            assert!(event["precision"]["residual"].as_f64().unwrap() <= ANGULAR_TOLERANCE);
            assert!(event["precision"]["bracketSeconds"].as_f64().unwrap() <= TIME_TOLERANCE);
            assert_eq!(event["bodyIds"], json!(["sun", "moon"]));
        }
        println!("USNO 50 phases maximum difference: {maximum_difference:.3} seconds (source rounded to minute)");
        assert_eq!(scanned.snapshot["positions"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn solar_zero_wrap_ingress_matches_nasa_equinox_minute_reference() {
        // https://science.nasa.gov/solar-system/skywatching/night-sky-network/embracing-the-equinox/
        // NASA reports 2026-03-20 14:46 UTC, read 2026-10-03; minute-rounded reference.
        let scanned = scan(
            &period(json!({"kind":"day","year":2026,"month":3,"day":20,"utcOffsetMinutes":420})),
            &options(&["sun"], &["ingress"], vec![]),
            &[],
        )
        .unwrap();
        assert_eq!(scanned.events.len(), 1);
        let e = &scanned.events[0];
        assert_eq!(e["details"]["boundaryLongitude"], 0.);
        assert_eq!(e["details"]["fromSign"], "Pisces");
        assert_eq!(e["details"]["toSign"], "Aries");
        assert_eq!(e["local"]["hour"], 21);
        let actual = e["utc"]["hour"].as_f64().unwrap() * 3600.
            + e["utc"]["minute"].as_f64().unwrap() * 60.
            + e["utc"]["second"].as_f64().unwrap();
        assert!((actual - (14. * 3600. + 46. * 60.)).abs() < 90.);
    }
    #[test]
    fn mercury_stations_are_real_velocity_sign_changes_and_targets_repeat() {
        let bounds = period(json!({"kind":"year","year":2026}));
        let scanned = scan(
            &bounds,
            &options(
                &["mercury"],
                &["station"],
                vec![AspectRule {
                    angle: 0.,
                    max_orb: 8.,
                }],
            ),
            &[Target {
                id: "sun".into(),
                longitude: 350.,
            }],
        )
        .unwrap();
        let stations: Vec<&Value> = scanned
            .events
            .iter()
            .filter(|e| e["type"] == "station")
            .collect();
        assert_eq!(stations.len(), 6);
        let mut provider = EphemerisSession::new().unwrap();
        for station in stations {
            let t = station["julianDayUt1"].as_f64().unwrap();
            let before = provider.positions(t - 600. / 86400., &[2]).unwrap()[0][3];
            let after = provider.positions(t + 600. / 86400., &[2]).unwrap()[0][3];
            assert!(before * after < 0.);
            assert_eq!(
                station["details"]["direction"],
                if before > 0. { "retrograde" } else { "direct" }
            );
            assert!(station["positions"][0]["speed"].as_f64().unwrap().abs() <= STATION_TOLERANCE);
        }
        drop(provider);
        let repeats: Vec<&Value> = scanned
            .events
            .iter()
            .filter(|e| e["type"] == "natalTransit" && e["utc"]["month"].as_u64().unwrap() <= 4)
            .collect();
        assert_eq!(repeats.len(), 3);
        assert!(repeats[0]["positions"][0]["speed"].as_f64().unwrap() > 0.);
        assert!(repeats[1]["positions"][0]["speed"].as_f64().unwrap() < 0.);
        assert!(repeats[2]["positions"][0]["speed"].as_f64().unwrap() > 0.);
    }
    #[test]
    fn all_2026_stations_retain_their_velocity_bracket_proof() {
        let scanned = scan(
            &period(json!({"kind":"year","year":2026,"utcOffsetMinutes":420})),
            &options(&BODY_IDS, &["station"], vec![]),
            &[],
        )
        .unwrap();
        let expected = [0, 0, 6, 2, 0, 2, 2, 2, 2, 2];
        assert_eq!(scanned.events.len(), expected.iter().sum::<usize>());
        let mut provider = EphemerisSession::new().unwrap();
        for (body, count) in expected.into_iter().enumerate() {
            let stations: Vec<_> = scanned
                .events
                .iter()
                .filter(|e| e["bodyIds"][0] == BODY_IDS[body])
                .collect();
            assert_eq!(stations.len(), count, "{}", BODY_IDS[body]);
            for station in stations {
                let t = station["julianDayUt1"].as_f64().unwrap();
                // A one-hour bracket is well above the slow-planet velocity noise floor.
                let before = provider.positions(t - 1. / 24., &[body]).unwrap()[0][3];
                let after = provider.positions(t + 1. / 24., &[body]).unwrap()[0][3];
                assert!(before * after < 0., "{station}");
                assert_eq!(
                    station["details"]["direction"],
                    if before > 0. { "retrograde" } else { "direct" }
                );
                assert!(station["precision"]["bracketSeconds"].as_f64().unwrap() <= TIME_TOLERANCE);
                assert!(station["precision"]["residual"].as_f64().unwrap() <= STATION_TOLERANCE);
            }
        }
        let pluto = scanned
            .events
            .iter()
            .find(|e| e["bodyIds"][0] == "pluto" && e["utc"]["month"] == 5)
            .unwrap();
        assert_eq!(pluto["utc"]["day"], 6);
        assert_eq!(pluto["details"]["direction"], "retrograde");
    }
    #[test]
    fn annual_slow_body_forecast_preserves_pluto_station_after_cache_reuse() {
        // The release build previously discarded this genuine crossing after checking
        // noisy velocities only one second either side of its already refined root.
        let result: Value = serde_json::from_str(&crate::calculate_json(
            &json!({
                "operation":"forecast",
                "birth":{
                    "utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},
                    "location":{"latitude":10.8231,"longitude":106.6297},
                    "houseSystem":"placidus"
                },
                "period":{"kind":"year","year":2026,"utcOffsetMinutes":420},
                "bodies":["jupiter","saturn","uranus","neptune","pluto"]
            })
            .to_string(),
        ))
        .unwrap();
        assert_eq!(result["errors"], json!([]));
        let events = result["data"]["events"].as_array().unwrap();
        assert_eq!(events.len(), 143);
        assert_eq!(events.iter().filter(|e| e["type"] == "station").count(), 10);
        assert!(events.iter().any(|e| {
            e["type"] == "station"
                && e["bodyIds"][0] == "pluto"
                && e["utc"]["month"] == 5
                && e["utc"]["day"] == 6
                && e["details"]["direction"] == "retrograde"
        }));
    }
    #[test]
    fn exact_planetary_aspects_and_half_open_boundaries() {
        let result = run(
            json!({"operation":"events","period":{"kind":"month","year":2026,"month":1},"bodies":["sun","moon"],"eventTypes":["planetaryAspect"],"aspectRules":[{"angle":180,"maxOrb":0}]}),
        );
        assert_eq!(result["errors"], json!([]));
        let events = result["data"]["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert_eq!(e["utc"]["day"], 3);
        let positions = e["positions"].as_array().unwrap();
        let separation = signed_difference(
            positions[0]["longitude"].as_f64().unwrap(),
            positions[1]["longitude"].as_f64().unwrap(),
        )
        .abs();
        assert!((separation - 180.).abs() <= ANGULAR_TOLERANCE);
        let mut values = Vec::new();
        append(&mut values, json!({"id":"end","julianDayUt1":1.}), 0., 1.).unwrap();
        assert!(values.is_empty());
        append(&mut values, json!({"id":"start","julianDayUt1":0.}), 0., 1.).unwrap();
        assert_eq!(values.len(), 1);
    }
    #[test]
    fn period_partition_and_determinism_preserve_events() {
        let opts = options(&["moon"], &["ingress"], vec![]);
        let month = scan(
            &period(json!({"kind":"month","year":2026,"month":1})),
            &opts,
            &[],
        )
        .unwrap();
        let repeated = scan(
            &period(json!({"kind":"month","year":2026,"month":1})),
            &opts,
            &[],
        )
        .unwrap();
        assert_eq!(month.events, repeated.events);
        let mut days = Vec::new();
        for day in 1..=31 {
            days.extend(
                scan(
                    &period(json!({"kind":"day","year":2026,"month":1,"day":day})),
                    &opts,
                    &[],
                )
                .unwrap()
                .events,
            );
        }
        assert_eq!(month.events.len(), days.len());
        for (a, b) in month.events.iter().zip(days) {
            assert_eq!(a["details"], b["details"]);
            assert!(
                (a["julianDayUt1"].as_f64().unwrap() - b["julianDayUt1"].as_f64().unwrap()).abs()
                    * 86400.
                    < TIME_TOLERANCE
            );
        }
    }
    #[test]
    fn empty_families_and_rules_have_no_exact_events_and_input_errors_are_structured() {
        let result = run(
            json!({"operation":"events","period":{"kind":"day","year":2026,"month":1,"day":1},"eventTypes":[],"aspectRules":[]}),
        );
        assert_eq!(result["errors"], json!([]));
        assert_eq!(result["data"]["events"], json!([]));
        assert_eq!(
            result["data"]["snapshot"]["positions"]
                .as_array()
                .unwrap()
                .len(),
            10
        );
        for extra in [
            json!({"bodies":[]}),
            json!({"eventTypes":["eclipse"]}),
            json!({"bodies":["sun","sun"]}),
            json!({"aspectPreset":"major","aspectRules":[]}),
            json!({"bodies":null}),
        ] {
            let mut request =
                json!({"operation":"events","period":{"kind":"day","year":2026,"month":1,"day":1}});
            for (key, value) in extra.as_object().unwrap() {
                request[key] = value.clone();
            }
            let result = run(request);
            assert_eq!(result["errors"][0]["code"], "INVALID_INPUT");
            assert!(result["data"].is_null());
        }
    }
}

#[cfg(test)]
mod eclipse_tests {
    use super::*;
    #[test]
    fn all_four_true_global_eclipses_match_nasa_types_dates_and_minutes() {
        // https://eclipse.gsfc.nasa.gov/OH/OH2026.html, retrieved 2026-10-03.
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/nasa-eclipses-2026.json"))
                .unwrap();
        let period: Period = serde_json::from_value(json!({"kind":"year","year":2026})).unwrap();
        let options = ScanOptions {
            bodies: vec!["mercury".into()],
            event_types: vec!["solarEclipse".into(), "lunarEclipse".into()],
            rules: vec![],
        };
        let result = scan(&period.validate().unwrap(), &options, &[]).unwrap();
        assert_eq!(result.events.len(), 4);
        let mut maximum_difference: f64 = 0.;
        for (event, reference) in result
            .events
            .iter()
            .zip(fixture["eclipses"].as_array().unwrap())
        {
            assert_eq!(event["type"], reference["type"]);
            assert_eq!(event["details"]["eclipseType"], reference["eclipseType"]);
            assert_eq!(event["utc"]["month"], reference["month"]);
            assert_eq!(event["utc"]["day"], reference["day"]);
            let actual = event["utc"]["hour"].as_f64().unwrap() * 3600.
                + event["utc"]["minute"].as_f64().unwrap() * 60.
                + event["utc"]["second"].as_f64().unwrap();
            let expected = reference["hour"].as_f64().unwrap() * 3600.
                + reference["minute"].as_f64().unwrap() * 60.;
            maximum_difference = maximum_difference.max((actual - expected).abs());
            assert!(
                (actual - expected).abs() <= 120.,
                "{event} versus {reference}"
            );
            assert_eq!(event["bodyIds"], json!(["sun", "moon"]));
            assert_eq!(event["precision"]["method"], "swissEclipseSearch");
            assert!(event["precision"]["residual"].is_null());
            assert!(event["precision"]["bracketSeconds"].is_null());
            let contacts = event["details"]["contacts"].as_array().unwrap();
            assert!(!contacts.is_empty());
            assert!(contacts
                .windows(2)
                .all(|p| p[0]["julianDayUt1"].as_f64().unwrap()
                    < p[1]["julianDayUt1"].as_f64().unwrap()));
            assert!(
                contacts.first().unwrap()["julianDayUt1"].as_f64().unwrap()
                    < event["julianDayUt1"].as_f64().unwrap()
            );
            assert!(
                contacts.last().unwrap()["julianDayUt1"].as_f64().unwrap()
                    > event["julianDayUt1"].as_f64().unwrap()
            );
            if event["type"] == "solarEclipse" {
                assert!(contacts.iter().any(|c| c["name"] == "centralPhaseBegin"));
                assert!(!contacts.iter().any(|c| c["name"] == "totalityBegin"));
            }
        }
        println!("NASA 4 eclipses maximum difference: {maximum_difference:.3} seconds (minute reference, independent Delta T models)");
    }
}

#[cfg(test)]
mod edge_search_tests {
    use super::*;
    #[test]
    fn two_close_contacts_on_one_station_interval_do_not_gain_a_false_tangent() {
        let day: Period =
            serde_json::from_value(json!({"kind":"day","year":2026,"month":2,"day":26})).unwrap();
        let options = ScanOptions {
            bodies: vec!["mercury".into()],
            event_types: vec!["station".into()],
            rules: vec![AspectRule {
                angle: 0.,
                max_orb: 8.,
            }],
        };
        let station = scan(&day.validate().unwrap(), &options, &[])
            .unwrap()
            .events
            .remove(0);
        let maximum = station["positions"][0]["longitude"].as_f64().unwrap();
        let target = Target {
            id: "sun".into(),
            longitude: maximum - 1e-8,
        };
        let result = scan(
            &day.validate().unwrap(),
            &ScanOptions {
                event_types: vec![],
                ..options
            },
            &[target],
        )
        .unwrap();
        assert_eq!(result.events.len(), 2, "{}", json!(result.events));
        let first = result.events[0]["julianDayUt1"].as_f64().unwrap();
        let last = result.events[1]["julianDayUt1"].as_f64().unwrap();
        assert!((last - first) * 24. < 6.);
        assert!(result.events[0]["positions"][0]["speed"].as_f64().unwrap() > 0.);
        assert!(result.events[1]["positions"][0]["speed"].as_f64().unwrap() < 0.);
    }
    #[test]
    fn full_year_all_bodies_twenty_six_targets_and_extended_rules_fit_declared_budget() {
        let period: Period = serde_json::from_value(json!({"kind":"year","year":2026})).unwrap();
        let options = ScanOptions {
            bodies: BODY_IDS.into_iter().map(String::from).collect(),
            event_types: EVENT_TYPES.into_iter().map(String::from).collect(),
            rules: domains::extended_rules(),
        };
        let targets: Vec<Target> = (0..26)
            .map(|i| Target {
                id: format!("fixed{i}"),
                longitude: i as f64 * 360. / 26.,
            })
            .collect();
        let begin = std::time::Instant::now();
        let result = scan(&period.validate().unwrap(), &options, &targets).unwrap();
        assert!(result.events.len() > 5000 && result.events.len() < MAX_EVENTS);
        assert_eq!(result.metadata["truncated"], false);
        assert!(result
            .events
            .windows(2)
            .all(|pair| pair[0]["julianDayUt1"].as_f64().unwrap()
                <= pair[1]["julianDayUt1"].as_f64().unwrap()));
        println!(
            "Year all10bodies/26fixedtargets/extendedrules: {} exact events in {:.3}s debug",
            result.events.len(),
            begin.elapsed().as_secs_f64()
        );
    }
}

#[cfg(test)]
mod time_output_tests {
    use super::*;
    #[test]
    fn display_rounding_stays_in_civil_window_and_leap_seconds_remain_explicit() {
        let period: Period = serde_json::from_value(
            json!({"kind":"month","year":2026,"month":9,"utcOffsetMinutes":420}),
        )
        .unwrap();
        let bounds = period.validate().unwrap();
        let mut session = EphemerisSession::new().unwrap();
        let start = session.julian_days(utc_args(&bounds.start), 0.).unwrap()[1];
        let end = session.julian_days(utc_args(&bounds.end), 0.).unwrap()[1];
        let mut sampler = Sampler {
            session,
            cache: HashMap::new(),
            evaluations: 0,
            window: Some((
                start,
                end,
                civil_milliseconds(&bounds.start),
                civil_milliseconds(&bounds.end),
            )),
        };
        let (_, first) = sampler.times(start, 420).unwrap();
        assert_eq!(first["month"], 9);
        assert_eq!(first["day"], 1);
        let (_, last) = sampler.times(end - 0.0002 / 86400., 420).unwrap();
        assert_eq!(last["month"], 9);
        assert_eq!(last["day"], 30);
        assert_eq!(last["hour"], 23);
        assert_eq!(last["minute"], 59);
        assert_eq!(last["second"], 59.999);
        drop(sampler);
        let mut session = EphemerisSession::new().unwrap();
        let leap = session.julian_days([2016, 12, 31, 23, 59], 60.5).unwrap()[1];
        let mut sampler = Sampler {
            session,
            cache: HashMap::new(),
            evaluations: 0,
            window: None,
        };
        let (utc, local) = sampler.times(leap, 420).unwrap();
        assert_eq!(utc["year"], 2016);
        assert_eq!(utc["month"], 12);
        assert_eq!(utc["day"], 31);
        assert_eq!(utc["minute"], 59);
        assert!(utc["second"].as_f64().unwrap() >= 60.);
        assert_eq!(local["year"], 2017);
        assert_eq!(local["month"], 1);
        assert_eq!(local["day"], 1);
        assert_eq!(local["hour"], 6);
        assert_eq!(local["minute"], 59);
        assert_eq!(local["second"], utc["second"]);
    }
}
