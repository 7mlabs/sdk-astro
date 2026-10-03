/** Compile-only contract check after installing @7mlabs/astrology. */
import {
  calculate, BirthInput, CompositeRequest, CompositeResult, CompositeData,
  CompositeDomainReport, CompositeCalculation, CompositeHouseMethod,
  CompositePlacement, CompositeBodyState, CoupleResult, CoupleWithCompositeResult,
  CoupleRequest, DomainName, Request, Result, EventsResult, ForecastResult, QueryResult,
} from '@7mlabs/astrology';
const personA: BirthInput = {
  utc: { year: 1980, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }, houseSystem: 'placidus',
};
const personB: BirthInput = { ...personA, utc: { year: 1990, month: 6, day: 15, hour: 3, minute: 30 } };
const input: CompositeRequest = {
  operation: 'composite', personA, personB, houseMethod: 'wholeSignFromMidpointAscendant',
  aspectPreset: 'extended', domains: ['identity', 'love', 'career'],
  customProfiles: [{ id: 'ourLearning', houses: [3, 9], bodies: ['mercury'],
    sections: [{ id: 'home', houses: [4], angles: ['imumCoeli'] }] }],
};
const result: CompositeResult = calculate(input);
const c: CompositeData = result.data.composite;
const metadata: CompositeCalculation = result.calculation;
const count: 3 = result.data.chartCount;
const subjectCount: 2 = result.data.subjectCount;
const kind: 'midpointComposite' = c.chartKind;
const houseMethod: CompositeHouseMethod = metadata.houseMethod;
const sourceJd: number = metadata.sourceCalculations.A.julianDayTt;
const placement: CompositePlacement = c.chart.placements[0];
const speed: null = placement.speed;
const retrograde: null = placement.isRetrograde;
const bodyState: CompositeBodyState = c.context.advanced.bodyStates[0];
const motion: 'notApplicable' = bodyState.motion;
const population: 'allCompositeBodies' = c.context.advanced.rules.distributions.population;
const report: CompositeDomainReport | undefined = c.domains.love?.report;
if (report) {
  const reportKind: 'midpointComposite' = report.chartKind;
  const aspectApplying: null | undefined = report.aspects[0]?.applying;
  void [reportKind, aspectApplying];
}
const combined: CoupleWithCompositeResult = calculate({
  operation: 'couple', personA, personB, composite: { domains: ['career', 'finance'] },
});
const combinedC: CompositeData = combined.data.composite;
const combinedMetadata: CompositeCalculation = combined.calculation.composite;
const pair: CoupleResult = calculate({ operation: 'couple', personA, personB });
const dynamicPair: CoupleRequest = { operation: 'couple', personA, personB,
  composite: Math.random() ? {} : undefined };
const optionalC: CompositeData | undefined = calculate(dynamicPair).data.composite;
function callUnknownOperation(request: Request): Result | CoupleResult | CompositeResult | EventsResult | ForecastResult | QueryResult {
  return calculate(request);
}
// @ts-expect-error C has no astronomical timestamp.
const syntheticTime = c.chart.utc;
// @ts-expect-error C has no physical latitude or distance.
const latitude = placement.latitude;
// @ts-expect-error C metadata has source Julian days rather than one physical Julian day.
const syntheticJd = metadata.julianDayTt;
// @ts-expect-error Composite selects the ten individual domain IDs, not pair-profile IDs.
const pairDomain: DomainName = 'attraction';
// @ts-expect-error Both source birth inputs are required.
const missingSource: CompositeRequest = { operation: 'composite', personA };
// @ts-expect-error Preset and explicit aspect rules cannot be specified together.
const conflicting: CompositeRequest = { operation: 'composite', personA, personB,
  aspectPreset: 'major', aspectRules: [] };
// @ts-expect-error Combined C inherits parent aspects; there are no nested aspect settings.
const nestedAspect: CoupleRequest = { operation: 'couple', personA, personB, composite: { aspectRules: [] } };
void [count, subjectCount, kind, houseMethod, sourceJd, speed, retrograde, motion, population,
  combinedC, combinedMetadata, pair, optionalC, callUnknownOperation, syntheticTime, latitude,
  syntheticJd, pairDomain, missingSource, conflicting, nestedAspect];
