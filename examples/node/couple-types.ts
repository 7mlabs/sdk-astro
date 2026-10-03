/** Compile-only contract check after installing @7mlabs/astrology.
 * node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs couple-types.ts
 */
import {
  calculate, BirthInput, CoupleRequest, CoupleResult, CoupleReport, CoupleReportSection,
  CouplePointId, CoupleBodyId, CoupleHouseId, CoupleChartId, CoupleDomainName, CoupleNatalFactRef,
  NatalContext, NatalAdvancedFacts, NatalDomainsResult, Result, Request,
} from '@7mlabs/astrology';
const personA: BirthInput = {
  utc: { year: 1980, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }, houseSystem: 'placidus',
};
const personB: BirthInput = { ...personA, utc: { year: 1990, month: 6, day: 15, hour: 3, minute: 30 }, houseSystem: 'wholeSign' };
const request: CoupleRequest = {
  operation: 'couple', personA, personB, aspectPreset: 'extended', domains: ['attraction', 'longTerm'],
  customProfiles: [{ id: 'ourStudy', houses: [3, 9], bodies: ['mercury'], sections: [{ id: 'home', houses: [4], angles: ['imumCoeli'] }] }],
};
const result: CoupleResult = calculate(request);
const count: 2 = result.data.subjectCount;
const kind: 'coupleSynastry' = result.data.chartKind;
const scope: 'couple-synastry-data' = result.calculation.scope;
const localContext: NatalContext = result.data.subjects.A.context;
const localAdvanced: NatalAdvancedFacts = localContext.advanced;
const julianDay: number = result.calculation.subjectCalculations.B.julianDayTt;
const report: CoupleReport | undefined = result.data.domains.attraction?.report;
const customReport: CoupleReport | undefined = result.data.customDomains.ourStudy?.report;
const ids: CoupleDomainName[] = ['attraction', 'communication', 'emotionalConnection', 'longTerm', 'sharedResources', 'homeFamily'];
for (const point of result.data.context.points) {
  const pointId: CouplePointId = point.id; const chartId: CoupleChartId = point.chartId;
  void [pointId, chartId];
}
for (const aspect of result.data.context.aspects) {
  const applying: null = aspect.applying; const relationId: string = aspect.relationId;
  void [applying, relationId];
}
for (const overlay of result.data.context.overlaysBtoA) {
  const houseId: CoupleHouseId = overlay.targetHouseId; const bodyId: CoupleBodyId = overlay.pointId;
  void [houseId, bodyId];
}
if (report) {
  const section: CoupleReportSection | undefined = report.sections[0];
  const localRefs: CoupleNatalFactRef[] = report.dispositorChainRefs;
  const localScope: 'subjectNatalOnly' = report.coverage.localAdvancedScope;
  const policy: 'primaryProfileWithTraceableSupport' = report.selectionPolicy;
  const reportKind: 'coupleSynastry' = report.chartKind;
  if (section) {
    const pointIds: CouplePointId[] = section.pointIds;
    const houseIds: CoupleHouseId[] = section.focusHouseIds;
    const bodyIds: CoupleBodyId[] = section.bodyFactIds;
    void [pointIds, houseIds, bodyIds];
  }
  void [localRefs, localScope, policy, reportKind];
}
const explicitRules: CoupleResult = calculate({ operation: 'couple', personA, personB, aspectRules: [] });
const individual: NatalDomainsResult = calculate({ operation: 'natalDomains', ...personA });
const geometry: Result = calculate({ operation: 'chart', positions: [{ id: 'sun', longitude: 0 }] });
const unionRequest: Request = request;
const unionResponse: Result | CoupleResult = calculate(unionRequest);
// @ts-expect-error Pair requests require both people.
const missingPerson: CoupleRequest = { operation: 'couple', personA };
// @ts-expect-error Aspect rules and a preset cannot be specified together.
const conflicting: CoupleRequest = { operation: 'couple', personA, personB, aspectPreset: 'major', aspectRules: [] };
// @ts-expect-error Individual domain IDs do not select pair profiles.
const badDomain: CoupleDomainName = 'love';
// @ts-expect-error Pair point IDs require the owning chart namespace.
const unqualified: CouplePointId = 'sun';
// @ts-expect-error House IDs are restricted to the twelve supported houses.
const badHouse: CoupleHouseId = 'B:H13';
// @ts-expect-error There is no single Julian day for two independent births.
const singleJulianDay = result.calculation.julianDayTt;
void [count, kind, scope, localAdvanced, julianDay, customReport, ids, explicitRules, individual,
  geometry, unionResponse, missingPerson, conflicting, badDomain, unqualified, badHouse, singleJulianDay];
