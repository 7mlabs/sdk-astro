/** Compile-only contract check against an installed @7mlabs/astrology package. */
import {
  calculate, BirthInput, ForecastPeriod, EventsRequest, EventsResult, ForecastRequest,
  ForecastResult, ForecastView, ForecastEvent, EclipseEvent, EventPosition,
  NatalForecastPointId, TransitForecastPointId, EventPrecision, ForecastHighlightReason,
} from '@7mlabs/astrology';
const birth: BirthInput = { utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 } };
const period: ForecastPeriod = { kind: 'day', year: 2026, month: 3, day: 3, utcOffsetMinutes: 420 };
const request: ForecastRequest = { operation: 'forecast', birth, period,
  aspectPreset: 'major', domains: ['career', 'love'], customProfiles: [{ id: 'myDay',
    houses: [1], sections: [{ id: 'study', houses: [3, 9], bodies: ['mercury'] }] }] };
const result: ForecastResult = calculate(request);
const kind: 'individualForecast' = result.data.chartKind;
const count: 1 = result.data.subjectCount;
const scope: 'individual-forecast-data' = result.calculation.scope;
const motion: 'fixedNatalLongitudes' = result.calculation.transitTargetMotion;
const providerEclipseMethod: 'Swiss global eclipse maximum, no local visibility' = result.calculation.search.eclipseMethod;
const view: ForecastView | undefined = result.data.domains.career;
if (view) {
  const narrative: null = view.messageContext.narrative;
  const pointIds: NatalForecastPointId[] = view.primaryNatalPointIds;
  const indexes: number[] = view.snapshotAspectIndexes;
  void [narrative, pointIds, indexes];
}
for (const relation of result.data.snapshot.relations) {
  const transit: TransitForecastPointId = relation.transitPointId;
  const natal: NatalForecastPointId = relation.natalPointId;
  void [transit, natal];
}
for (const event of result.data.events) {
  const typed: ForecastEvent = event; const position: EventPosition | undefined = event.positions[0];
  const precision: EventPrecision = event.precision;
  const reasons: ForecastHighlightReason[] = event.highlightReasons;
  if (event.type === 'natalTransit') {
    const target: NatalForecastPointId = event.details.targetPointId; void target;
  }
  if (event.type === 'solarEclipse' || event.type === 'lunarEclipse') {
    const eclipse: EclipseEvent = event;
    const residual: null = eclipse.precision.residual;
    const visibility: 'global' = eclipse.details.visibilityScope;
    void [residual, visibility];
  }
  if (precision.method === 'bracketedRoot') {
    const residual: number = precision.residual; void residual;
  }
  void [typed, position, reasons];
}
const eventsRequest: EventsRequest = { operation: 'events', period: { kind: 'year', year: 2026 },
  eventTypes: ['solarEclipse', 'lunarEclipse'], bodies: ['saturn'] };
const events: EventsResult = calculate(eventsRequest);
const eventKind: 'astronomicalEvents' = events.data.chartKind;
const eventScope: 'astronomical-event-data' = events.calculation.scope;
// @ts-expect-error A year period does not accept month or day selectors.
const mixedPeriod: ForecastPeriod = { kind: 'year', year: 2026, month: 3 };
// @ts-expect-error A day period requires month and day.
const incompleteDay: ForecastPeriod = { kind: 'day', year: 2026 };
// @ts-expect-error Preset and explicit rules are mutually exclusive.
const conflicting: EventsRequest = { operation: 'events', period, aspectPreset: 'major', aspectRules: [] };
// @ts-expect-error Natal transits are an output family, not a global eventTypes selector.
const invalidGlobal: EventsRequest = { operation: 'events', period, eventTypes: ['natalTransit'] };
// @ts-expect-error Natal references require the N namespace.
const missingNamespace: NatalForecastPointId = 'sun';
// @ts-expect-error Transit horizon angles are not calculated.
const transitAngle: TransitForecastPointId = 'T:ascendant';
// @ts-expect-error Forecast message context contains no narrative text.
const inventedNarrative: string = view!.messageContext.narrative;
// @ts-expect-error Global events do not need a birth input.
const globalBirth: EventsRequest = { operation: 'events', period, birth };
void [kind, count, scope, motion, providerEclipseMethod, eventKind, eventScope, mixedPeriod,
  incompleteDay, conflicting, invalidGlobal, missingNamespace, transitAngle, inventedNarrative, globalBirth];
