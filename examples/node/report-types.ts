/** Compile-only smoke test after installing @7mlabs/astrology in this example project.
 * Run an available compiler: node "$TSC_JS" --strict --noEmit --target es2020 --module commonjs report-types.ts
 */
import {
  calculate, Result, NatalDomainsResult, NatalAdvancedFacts,
  IndividualDomainReport, ReportHouseEvidence, DomainName, ProfileDefinition,
  CustomProfileInput, IndividualReportSection,
} from '@7mlabs/astrology';

const result: NatalDomainsResult = calculate({
  operation: 'natalDomains',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  domains: ['identity', 'learning', 'creativity', 'innerLife', 'dailyLife'],
  customProfiles: [{
    id: 'personalOverview', houses: [1], bodies: ['sun', 'moon'], angles: ['ascendant'],
    sections: [{ id: 'career', houses: [6, 10], bodies: ['saturn'], angles: ['midheaven'] }],
  }],
});

// The domain overload retains the common result envelope and narrows individual metadata.
const envelope: Result = result;
const subjectCount: 1 = result.data.subjectCount;
const chartKind: 'individualNatal' = result.data.chartKind;
const calculationChartKind: 'individualNatal' = result.calculation.chartKind;
const reportProfileVersion: string = result.calculation.reportProfile.version;
const advanced: NatalAdvancedFacts = result.data.context.advanced;
const report: IndividualDomainReport | undefined = result.data.domains.career?.report;
const rulerRelationId: string | null = result.data.context.houseRulerRelations[0].relationId;
const builtinIds: DomainName[] = ['career', 'love', 'relationships', 'family', 'finance',
  'identity', 'learning', 'creativity', 'innerLife', 'dailyLife'];
const catalog: ProfileDefinition[] = result.data.profileCatalog;
const customReport: IndividualDomainReport | undefined = result.data.customDomains.personalOverview?.report;
const customCount: number = result.calculation.customProfileCount;
const section: IndividualReportSection | undefined = customReport?.sections[0];

if (section) {
  const indexRefs: number[] = section.aspectIndexes;
  const stringRefs: string[][] = [section.primaryPointIds, section.focusHouseIds,
    section.relatedHouseIds, section.bodyFactIds, section.dispositorChainIds,
    section.receptionIds, section.aspectPatternIds];
  void [indexRefs, stringRefs];
}

if (report) {
  const reportAspectCount: number = report.coverage.reportAspectCount;
  const recursiveHouseExpansion: false = report.coverage.recursiveHouseExpansion;
  for (const house of report.houses) {
    for (const evidence of house.evidence) {
      if (evidence.rule === 'dispositorChain') {
        const refs: string[] = [evidence.bodyId, evidence.sourceBodyId, evidence.chainId];
        void refs;
      }
      if (evidence.rule === 'aspectEndpoint') {
        const index: number = evidence.aspectIndex;
        void index;
      }
      const sectionId: string | undefined = evidence.sectionId;
      void sectionId;
    }
  }
  void [reportAspectCount, recursiveHouseExpansion];
}

// @ts-expect-error Domain reports describe one individual, not a chart pair.
const pair: 'synastry' = result.data.chartKind;
// @ts-expect-error Dispositor evidence requires a resolvable chain ID.
const incompleteEvidence: ReportHouseEvidence = { rule: 'dispositorChain', bodyId: 'sun', sourceBodyId: 'sun' };
// @ts-expect-error The subject count is one.
const twoSubjects: 2 = result.data.subjectCount;
// @ts-expect-error A custom ID belongs in customProfiles, not the builtin selector.
const customBuiltin: DomainName = 'personalOverview';
const invalidProfile: CustomProfileInput = {
  id: 'invalidProfile',
  // @ts-expect-error Only the ten supported body IDs are valid selectors.
  bodies: ['chiron'],
};
void [envelope, subjectCount, chartKind, calculationChartKind, reportProfileVersion,
  advanced, report, rulerRelationId, pair, incompleteEvidence, twoSubjects,
  builtinIds, catalog, customReport, customCount, section, customBuiltin, invalidProfile];
