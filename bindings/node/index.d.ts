export interface Position { id: string; longitude: number; speed?: number | null; }
export interface AspectRule { angle: number; maxOrb: number; }
export interface GeometryRequest {
  operation: 'chart' | 'harmonic' | 'synastry';
  positions: Position[];
  otherPositions?: Position[];
  houses?: number[];
  otherHouses?: number[];
  harmonic?: number;
  aspectRules?: AspectRule[];
}
export interface NatalRequest {
  operation: 'natal';
  /** Gregorian UTC; date/hour/minute must have integer values. Caller converts local time to UTC. */
  utc: { year: number; month: number; day: number; hour: number; minute: number; second?: number; };
  /** Degrees, north/east positive. */
  location: { latitude: number; longitude: number; };
  houseSystem?: 'placidus' | 'wholeSign';
  aspectRules?: AspectRule[];
}
export type DomainName = 'career' | 'love' | 'relationships' | 'family' | 'finance'
  | 'identity' | 'learning' | 'creativity' | 'innerLife' | 'dailyLife';
export type NatalBodyId = 'sun' | 'moon' | 'mercury' | 'venus' | 'mars' | 'jupiter'
  | 'saturn' | 'uranus' | 'neptune' | 'pluto';
export type NatalAngleId = 'ascendant' | 'midheaven' | 'descendant' | 'imumCoeli';
export interface ProfileSelectors {
  /** Unique integer-valued house numbers from 1 through 12. */
  houses?: number[]; bodies?: NatalBodyId[]; angles?: NatalAngleId[];
}
export interface ProfileSectionInput extends ProfileSelectors {
  /** ASCII identifier; prototype keys are reserved. At least one selector is required. */
  id: string;
}
export interface CustomProfileInput extends ProfileSelectors {
  /** Unique ASCII identifier, distinct from builtin IDs and prototype keys. */
  id: string;
  /** ASCII version label; default 1.0. */
  version?: string;
  /** Up to eight sections; each section may extend beyond the parent selectors. */
  sections?: ProfileSectionInput[];
}
export interface ProfileSectionDefinition {
  id: string; houses: number[]; bodies: NatalBodyId[]; angles: NatalAngleId[];
}
export interface ProfileDefinition {
  id: string; version: string; houses: number[]; bodies: NatalBodyId[];
  angles: NatalAngleId[]; sections: ProfileSectionDefinition[];
}
export interface NatalDomainsRequest extends Omit<NatalRequest, 'operation'> {
  operation: 'natalDomains';
  /** Absent selects all ten builtins. Empty is valid only with nonempty customProfiles. */
  domains?: DomainName[];
  /** One through eight custom profiles. Each profile requires at least one selector. */
  customProfiles?: CustomProfileInput[];
  rulership?: 'traditional' | 'modern';
  /** Default extended. Mutually exclusive with aspectRules. */
  aspectPreset?: 'major' | 'extended';
}
export type CoupleDomainName = 'attraction' | 'communication' | 'emotionalConnection'
  | 'longTerm' | 'sharedResources' | 'homeFamily';
export type CoupleChartId = 'A' | 'B';
export type NatalHouseId = 'H1' | 'H2' | 'H3' | 'H4' | 'H5' | 'H6'
  | 'H7' | 'H8' | 'H9' | 'H10' | 'H11' | 'H12';
export type NatalPointId = NatalBodyId | NatalAngleId | NatalHouseId;
/** IDs are namespaced because each person has their own Sun, houses and angles. */
export type CouplePointId = `${CoupleChartId}:${NatalPointId}`;
export type CoupleBodyId = `${CoupleChartId}:${NatalBodyId}`;
export type CoupleHouseId = `${CoupleChartId}:${NatalHouseId}`;
export interface BirthInput {
  utc: NatalRequest['utc']; location: NatalRequest['location'];
  houseSystem?: 'placidus' | 'wholeSign';
}
export interface CoupleRequestBase {
  operation: 'couple'; personA: BirthInput; personB: BirthInput;
  /** Absent selects all six pair builtins. Empty requires nonempty customProfiles. */
  domains?: CoupleDomainName[]; customProfiles?: CustomProfileInput[];
  rulership?: 'traditional' | 'modern';
  /** Opt in to symbolic chart C; omitted preserves the existing synastry payload. */
  composite?: CompositeOptions;
}
/** Aspect presets and explicit rules are mutually exclusive. Explicit [] selects no aspects. */
export type CoupleRequest = CoupleRequestBase & (
  | { aspectPreset?: 'major' | 'extended'; aspectRules?: never; }
  | { aspectRules: AspectRule[]; aspectPreset?: never; }
);
export type CompositeHouseMethod = 'midpoint' | 'wholeSignFromMidpointAscendant';
export type CompositeAntipodalPolicy = 'error' | 'lowerLongitude';
export interface CompositeOptions {
  /** Default midpoint. Invalid cusp order fails; cusps are never silently repaired. */
  houseMethod?: CompositeHouseMethod;
  /** Default error. lowerLongitude explicitly resolves exactly opposite source points. */
  antipodalPolicy?: CompositeAntipodalPolicy;
  /** Absent selects the same ten profile definitions as individual natal reports. */
  domains?: DomainName[];
  customProfiles?: CustomProfileInput[];
}
export interface CompositeRequestBase extends CompositeOptions {
  operation: 'composite'; personA: BirthInput; personB: BirthInput;
  rulership?: 'traditional' | 'modern';
}
export type CompositeRequest = CompositeRequestBase & (
  | { aspectPreset?: 'major' | 'extended'; aspectRules?: never; }
  | { aspectRules: AspectRule[]; aspectPreset?: never; }
);
/** Fixed civil UTC offset; no IANA timezone conversion or DST inference. */
export type ForecastPeriod =
  | { kind: 'day'; year: number; month: number; day: number; utcOffsetMinutes?: number; }
  | { kind: 'month'; year: number; month: number; day?: never; utcOffsetMinutes?: number; }
  | { kind: 'year'; year: number; month?: never; day?: never; utcOffsetMinutes?: number; };
export type GlobalEventType = 'ingress' | 'station' | 'lunarPhase' | 'planetaryAspect' | 'solarEclipse' | 'lunarEclipse';
export interface EventSearchOptions {
  period: ForecastPeriod;
  /** Unique, nonempty Sun–Pluto IDs; default all ten. */
  bodies?: NatalBodyId[];
  /** Default all six. [] disables global events. Phases/eclipses always use Sun/Moon. */
  eventTypes?: GlobalEventType[];
}
export type ForecastAspectSettings =
  | { aspectPreset?: 'major' | 'extended'; aspectRules?: never; }
  | { aspectRules: AspectRule[]; aspectPreset?: never; };
/** Major aspects by default; bounded root search fails rather than truncating. */
export type EventsRequest = EventSearchOptions & ForecastAspectSettings & { operation: 'events'; };
export type ForecastRequest = EventSearchOptions & ForecastAspectSettings & {
  operation: 'forecast'; birth: BirthInput; includeNatalTransits?: boolean;
  domains?: DomainName[]; customProfiles?: CustomProfileInput[]; rulership?: 'traditional' | 'modern';
};
export type QueryMotionMode = 'none' | 'relative' | 'fixedSecond';
export interface GeometryNormalizeRequest { operation: 'query'; group: 'geometry'; action: 'normalize'; longitude: number; }
export interface GeometrySeparationRequest { operation: 'query'; group: 'geometry'; action: 'separation'; longitude1: number; longitude2: number; }
export interface GeometryMidpointRequest { operation: 'query'; group: 'geometry'; action: 'midpoint'; longitude1: number; longitude2: number; antipodalPolicy?: CompositeAntipodalPolicy; }
export interface AspectsBetweenRequest {
  operation: 'query'; group: 'aspects'; action: 'between'; positions: [Position, Position]; rule: AspectRule;
  /** Default none. fixedSecond uses zero speed for the second point without changing its supplied metadata. */
  motionMode?: QueryMotionMode;
}
export interface HousesLocateRequest { operation: 'query'; group: 'houses'; action: 'locate'; longitude: number; houseCusps: number[]; }
export type QueryInspectionOptions = { birth: BirthInput; rulership?: 'traditional' | 'modern'; } & ForecastAspectSettings;
/** Default all 26; at least two distinct IDs. Both aspect endpoints must be selected. */
export type AspectsInspectRequest = QueryInspectionOptions & { operation: 'query'; group: 'aspects'; action: 'inspect'; pointIds?: NatalPointId[]; };
/** Default all 26. Relations/aspects require at least one selected endpoint. */
export type PointsInspectRequest = QueryInspectionOptions & { operation: 'query'; group: 'points'; action: 'inspect'; pointIds?: NatalPointId[]; };
/** Default all twelve; integer-valued house numbers from 1 through 12. */
export type HousesInspectRequest = QueryInspectionOptions & { operation: 'query'; group: 'houses'; action: 'inspect'; houseNumbers?: number[]; };
export type QueryRequest = GeometryNormalizeRequest | GeometrySeparationRequest | GeometryMidpointRequest
  | AspectsBetweenRequest | HousesLocateRequest | AspectsInspectRequest | PointsInspectRequest | HousesInspectRequest;
export type Request = GeometryRequest | NatalRequest | NatalDomainsRequest | CoupleRequest | CompositeRequest
  | EventsRequest | ForecastRequest | QueryRequest;
export interface House { number: number; longitude: number; sign: string; degreeInSign: number; }
export interface Placement {
  latitude?: number; distanceAu?: number;
  id: string; longitude: number; signIndex: number; sign: string; degreeInSign: number;
  speed: number | null; isRetrograde: boolean | null; house: number | null;
}
export interface Aspect {
  body1: string; body2: string; angle: number; separation: number; orb: number;
  maxOrb: number; applying: boolean | null;
}
export interface Midpoint { body1: string; body2: string; longitude: number | null; ambiguous: boolean; }
export interface NatalData {
  utc: NatalRequest['utc']; location: NatalRequest['location']; placements: Placement[];
  angles: Placement[]; houses: House[]; houseCusps: number[]; aspects: Aspect[];
}
export interface DomainHouse extends House {
  id: string; signIndex: number; occupants: Placement[]; rulerBodyId: string; rulerPlacement: Placement;
}
export interface ContextPoint {
  id: string; kind: 'body' | 'angle' | 'houseCusp'; longitude: number;
  signIndex: number; sign: string; degreeInSign: number; speed: number | null;
  house?: number | null; houseNumber?: number; isRetrograde?: boolean | null;
  latitude?: number; distanceAu?: number;
}
export interface ContextAspect {
  index: number; point1: string; point2: string; angle: number; separation: number;
  orb: number; maxOrb: number; applying: boolean | null;
}
export interface PointRelation {
  id: string; point1: string; point2: string; signedDelta: number; separation: number; aspectIndexes: number[];
}
export interface HouseRulerRelation {
  house1: string; house2: string; ruler1: string; ruler2: string; sameRuler: boolean;
  /** Null when both houses have the same ruler; otherwise a context.relations ID. */
  relationId: string | null; aspectIndexes: number[];
}
export interface EssentialDignity {
  system: 'traditional'; supported: boolean;
  /** Null for bodies outside the seven classical bodies; no dignity score is produced. */
  domicile: boolean | null; detriment: boolean | null; exaltation: boolean | null; fall: boolean | null;
}
export interface NatalBodyState {
  bodyId: string; element: 'fire' | 'earth' | 'air' | 'water';
  modality: 'cardinal' | 'fixed' | 'mutable'; polarity: 'positive' | 'negative';
  houseType: 'angular' | 'succedent' | 'cadent'; motion: 'direct' | 'retrograde' | 'stationary';
  dignity: EssentialDignity;
  /** Geometric longitude classification under the declared thresholds, not a prediction. */
  solarCondition: { separation: number | null; condition: 'notApplicable' | 'cazimi' | 'combust' | 'underSunbeams' | 'free'; };
}
export interface BodyDistribution { id: string; count: number; bodyIds: string[]; }
export interface DispositorChain {
  id: string; bodyId: string; path: string[]; termination: 'selfRuler' | 'cycle';
  terminalBodyId: string | null; cycleBodyIds: string[];
}
export interface MutualReception {
  id: string; bodyIds: string[]; type: 'mutualDomicile'; rulership: 'traditional' | 'modern';
}
export interface AspectPattern {
  id: string; type: 'grandTrine' | 'tSquare' | 'yod' | 'kite' | 'grandCross';
  bodyIds: string[]; aspectIndexes: number[]; apexBodyId: string | null;
}
export interface NatalAdvancedRules {
  id: string; version: string;
  dignity: { system: 'traditional'; scope: 'signOnly'; supportedBodyIds: string[]; excluded: string[]; };
  motion: { source: 'instantaneousLongitudeSpeed'; stationary: 'exactZero'; predictsStation: false; };
  solarCondition: {
    distance: 'shortestEclipticLongitude'; unit: 'degrees';
    inclusiveThresholds: { cazimi: number; combust: number; underSunbeams: number; };
    boundaryToleranceDegrees: number; classification: 'geometricOnly'; exemptBodyIds: string[];
  };
  dispositors: { rulership: 'traditional' | 'modern'; path: 'uniqueBodyIds'; cycle: 'repeatWithoutAppendingRepeatedBody'; };
  receptions: { type: 'mutualDomicile'; rulership: 'traditional' | 'modern'; };
  aspectPatterns: {
    points: 'bodiesOnly'; edges: 'configuredMatchedAspects'; angles: 'exactConfiguredAngle';
    orb: 'inheritedFromConfiguredAspectRules'; duplicateEdges: 'lowestGlobalAspectIndex';
    kiteApex: 'additionalBodyOppositeOneGrandTrineBody'; additionalMatchedEdges: 'allowed';
    uniqueBy: 'typeBodyIdsAndApex';
  };
  distributions: { population: 'allNatalBodies'; weights: 'onePerBody'; emptyGroups: 'retained'; };
}
export interface NatalAdvancedFacts {
  rules: NatalAdvancedRules; bodyStates: NatalBodyState[];
  distributions: { elements: BodyDistribution[]; modalities: BodyDistribution[]; polarities: BodyDistribution[]; houseTypes: BodyDistribution[]; };
  dispositorChains: DispositorChain[]; receptions: MutualReception[]; aspectPatterns: AspectPattern[];
}
export type ReportHouseRole = 'focus' | 'selectedBodyPlacement' | 'selectedBodyRulership'
  | 'dispositorPlacement' | 'dispositorRulership' | 'focusAnglePlacement'
  | 'aspectEndpointPlacement' | 'aspectEndpointRulership' | 'sectionFocus' | 'sectionSupport';
export type ReportHouseEvidence = ({ sectionId?: string; } & (
  | { rule: 'profileHouse'; houseId: string; }
  | { rule: 'selectedBody'; bodyId: string; }
  | { rule: 'dispositorChain'; bodyId: string; sourceBodyId: string; chainId: string; }
  | { rule: 'focusAngle'; pointId: string; }
  | { rule: 'aspectEndpoint'; pointId: string; aspectIndex: number; }
));
export interface ReportHouse extends DomainHouse {
  roles: ReportHouseRole[]; evidence: ReportHouseEvidence[]; aspectIndexes: number[];
  rulerPlacementHouseId: string; rulerDispositorChainId: string;
}
export interface ReportAspect extends ContextAspect {
  primaryContact: boolean; relatedHouseIds: string[]; sectionIds: string[];
}
export interface DomainSelectionRules {
  houses: number[]; bodies: NatalBodyId[]; angles: NatalAngleId[];
  aspectSelection: 'atLeastOneSelectedEndpoint';
}
/** References resolve against the shared context and the enclosing report's union of facts. */
export interface IndividualReportSection {
  id: string; selectionRules: DomainSelectionRules;
  primaryPointIds: string[]; focusHouseIds: string[]; relatedHouseIds: string[];
  aspectIndexes: number[]; bodyFactIds: NatalBodyId[]; dispositorChainIds: string[];
  receptionIds: string[]; aspectPatternIds: string[];
}
export interface IndividualDomainReport {
  version: string; chartKind: 'individualNatal'; selectionPolicy: 'primaryProfileWithTraceableSupport';
  focusHouseIds: string[]; houseIds: string[]; pointIds: string[];
  houses: ReportHouse[]; points: ContextPoint[]; aspects: ReportAspect[];
  bodyFacts: NatalBodyState[]; dispositorChains: DispositorChain[];
  receptions: MutualReception[]; aspectPatterns: AspectPattern[];
  sections: IndividualReportSection[];
  coverage: {
    primaryAspectCount: number; reportAspectCount: number; relatedHouseCount: number;
    supportSelection: 'primaryBodiesDispositorChainsAndPrimaryAspectEndpoints';
    houseContacts: 'cuspOccupantsAndRuler'; patternSelection: 'atLeastOnePrimaryBody';
    recursiveHouseExpansion: false; sectionCount: number;
    sectionSelection: 'independentSelectorsSharedNatal';
  };
}
export interface DomainView {
  profileId: string; profileVersion: string;
  origin: 'builtin' | 'custom'; definition: ProfileDefinition;
  selectionRules: DomainSelectionRules;
  houses: DomainHouse[];
  bodies: Array<{ placement: Placement; selectionReasons: Array<{ rule: 'profileBody' | 'houseOccupant' | 'houseRuler'; houseId?: string; }>; }>;
  angles: Placement[]; pointIds: string[]; relationIds: string[];
  aspects: Array<ContextAspect & { selectedEndpoints: string[]; }>;
  report: IndividualDomainReport;
}
export interface NatalContext {
  rulership: 'traditional' | 'modern'; rulershipRulesVersion: string; houseAssignment: 'eclipticLongitude';
  points: ContextPoint[]; houses: DomainHouse[]; relations: PointRelation[]; aspects: ContextAspect[];
  dispositors: Array<{ bodyId: string; sign: string; dispositorBodyId: string; dispositorPlacement: Placement; }>;
  /** All 66 unordered pairs of the twelve houses, including pairs with the same ruler. */
  houseRulerRelations: HouseRulerRelation[]; advanced: NatalAdvancedFacts;
  coverage: { bodies: number; angles: number; houseCusps: number; pointPairs: number; aspectMatching: string; };
}
export interface NatalDomainsData {
  chartKind: 'individualNatal'; subjectCount: 1;
  natal: NatalData; context: NatalContext; domains: Partial<Record<DomainName, DomainView>>;
  customDomains: Record<string, DomainView>; profileCatalog: ProfileDefinition[];
}
export interface Result {
  schemaVersion: string; engineVersion: string;
  calculation: { scope: string; provider: string; angleUnit: string; speedUnit: string; aspectRules: AspectRule[];
    providerVersion?: string; ephemeris?: string; zodiac?: string; houseSystem?: string;
    julianDayTt?: number; julianDayUt1?: number; [key: string]: unknown; };
  data: { natal?: NatalData; context?: NatalContext; domains?: Partial<Record<DomainName, DomainView>>;
    customDomains?: Record<string, DomainView>; profileCatalog?: ProfileDefinition[];
    utc?: NatalRequest["utc"]; location?: NatalRequest["location"];
    houses?: House[]; houseCusps?: number[]; angles?: Placement[]; placements?: Placement[]; aspects?: Aspect[]; midpoints?: Midpoint[]; harmonic?: number;
    personA?: Placement[]; personB?: Placement[]; crossAspects?: Aspect[];
    overlaysAtoB?: Placement[]; overlaysBtoA?: Placement[]; };
  warnings: string[]; errors: Array<{ code: string; message: string }>;
}
export interface NatalDomainsResult extends Result {
  calculation: Result['calculation'] & {
    scope: 'natal-domain-data'; chartKind: 'individualNatal';
    aspectPreset: 'major' | 'extended' | 'custom'; rulership: 'traditional' | 'modern';
    domainProfile: { id: string; version: string; }; reportProfile: { id: string; version: string; };
    customProfileCount: number;
  };
  data: NatalDomainsData;
}
export interface BasicNatalCalculation {
  scope: 'basic-natal'; provider: 'swiss-ephemeris'; providerVersion: string; ephemeris: 'moshier';
  zodiac: 'tropical'; coordinates: 'geocentric'; positionType: 'apparent'; referenceFrame: 'ecliptic-of-date';
  calendar: 'gregorian'; inputTimeScale: 'UTC'; houseTimeScale: 'UT1'; planetTimeScale: 'TT';
  timeModel: string; julianDayTt: number; julianDayUt1: number; houseSystem: 'placidus' | 'wholeSign';
  angleUnit: 'degrees'; speedUnit: 'degrees/day'; distanceUnit: 'AU'; aspectRules: AspectRule[];
}
export interface CoupleSubject {
  id: CoupleChartId; natal: NatalData; context: NatalContext; calculation: BasicNatalCalculation;
}
export interface CoupleContextPoint extends Omit<ContextPoint, 'id'> {
  id: CouplePointId; chartId: CoupleChartId; localId: NatalPointId;
}
export interface CouplePointRelation extends Omit<PointRelation, 'point1' | 'point2'> {
  point1: CouplePointId; point2: CouplePointId;
}
export interface CoupleAspect extends Omit<ContextAspect, 'point1' | 'point2' | 'applying'> {
  relationId: string; point1: CouplePointId; point2: CouplePointId;
  /** The points belong to different birth instants; applying/separating is undefined. */
  applying: null;
}
export interface CoupleOverlay {
  id: string; sourceChartId: CoupleChartId; targetChartId: CoupleChartId;
  pointId: CoupleBodyId; localBodyId: NatalBodyId; sourceHouseId: CoupleHouseId;
  targetHouseId: CoupleHouseId; targetHouseNumber: number; targetRulerPointId: CoupleBodyId;
  targetCuspRelationId: string; targetRulerRelationId: string; aspectIndexes: number[];
}
export interface CoupleHouseRulerRelation {
  id: string; house1: CoupleHouseId; house2: CoupleHouseId; ruler1: CoupleBodyId; ruler2: CoupleBodyId;
  /** Matching local body IDs still represent two different people and have a cross relation. */
  sameBodyId: boolean; relationId: string; aspectIndexes: number[];
}
export interface CoupleContext {
  points: CoupleContextPoint[]; relations: CouplePointRelation[]; aspects: CoupleAspect[];
  overlaysAtoB: CoupleOverlay[]; overlaysBtoA: CoupleOverlay[];
  houseRulerRelations: CoupleHouseRulerRelation[];
  coverage: {
    subjects: 2; points: 52; pointPairs: 676; bodyBodyPairs: 100; overlays: 20; houseRulerRelations: 144;
    aspectMatching: 'configuredRulesOnly'; houseAssignment: 'eclipticLongitude';
    crossApplying: 'undefinedForDifferentBirthInstants';
  };
}
export interface CoupleSelectionRules extends DomainSelectionRules {
  overlaySelection: 'selectedSourceBodyOrTargetFocusHouse';
  houseRulerSelection: 'selectedHouseOrRulerEndpoint';
}
export interface CoupleSubjectSelection {
  chartId: CoupleChartId; referenceScope: 'subjectLocalNatal'; selectionRules: DomainSelectionRules;
  houses: DomainHouse[]; bodies: DomainView['bodies']; angles: Placement[];
  pointIds: string[]; relationIds: string[]; aspects: DomainView['aspects'];
}
/** References are local to data.subjects[chartId].context, never cross-chart patterns. */
export interface CoupleNatalFactRef { chartId: CoupleChartId; id: string; }
export interface CoupleReportSection {
  id: string; selectionRules: CoupleSelectionRules; primaryPointIds: CouplePointId[];
  focusHouseIds: CoupleHouseId[]; pointIds: CouplePointId[]; houseIds: CoupleHouseId[];
  bodyFactIds: CoupleBodyId[]; aspectIndexes: number[]; overlayIds: string[]; houseRulerRelationIds: string[];
  dispositorChainRefs: CoupleNatalFactRef[]; natalReceptionRefs: CoupleNatalFactRef[];
  natalAspectPatternRefs: CoupleNatalFactRef[];
}
export interface CoupleReport {
  chartKind: 'coupleSynastry'; selectionPolicy: 'primaryProfileWithTraceableSupport';
  profileId: 'sevenmlabs-couple-report'; profileVersion: '1.0'; primaryPointIds: CouplePointId[];
  points: Array<CoupleContextPoint & { primary: boolean; sectionIds: string[]; }>;
  houses: Array<{ id: CoupleHouseId; chartId: CoupleChartId; localId: NatalHouseId; house: DomainHouse; primary: boolean; sectionIds: string[]; }>;
  bodyFacts: Array<{ id: CoupleBodyId; chartId: CoupleChartId; localId: NatalBodyId; facts: NatalBodyState; primary: boolean; sectionIds: string[]; }>;
  dispositorChainRefs: CoupleNatalFactRef[]; natalReceptionRefs: CoupleNatalFactRef[];
  natalAspectPatternRefs: CoupleNatalFactRef[];
  aspectFacts: Array<CoupleAspect & { selectedEndpoints: CouplePointId[]; primaryContact: boolean; sectionIds: string[]; }>;
  overlayIds: string[]; houseRulerRelationIds: string[]; sections: CoupleReportSection[];
  coverage: {
    pointCount: number; houseCount: number; bodyFactCount: number; primaryAspectCount: number;
    aspectCount: number; sectionCount: number; localAdvancedScope: 'subjectNatalOnly';
    sectionSelection: 'independentSymmetricSelectorsSharedNatals';
  };
}
export interface CoupleDomainView {
  profileId: string; profileVersion: string; origin: 'builtin' | 'custom'; definition: ProfileDefinition;
  selectionRules: CoupleSelectionRules; selections: { A: CoupleSubjectSelection; B: CoupleSubjectSelection; };
  pointIds: CouplePointId[]; relationIds: string[];
  aspects: Array<CoupleAspect & { selectedEndpoints: CouplePointId[]; }>;
  overlayIds: string[]; houseRulerRelationIds: string[]; report: CoupleReport;
}
export interface CoupleData {
  chartKind: 'coupleSynastry'; subjectCount: 2; subjects: { A: CoupleSubject; B: CoupleSubject; };
  context: CoupleContext; domains: Partial<Record<CoupleDomainName, CoupleDomainView>>;
  customDomains: Record<string, CoupleDomainView>; profileCatalog: ProfileDefinition[];
  /** Present together only when the request opts in to composite. */
  chartCount?: 3; composite?: CompositeData;
}
export interface CoupleCalculation extends Omit<BasicNatalCalculation, 'scope' | 'julianDayTt' | 'julianDayUt1' | 'houseSystem'> {
  scope: 'couple-synastry-data'; chartKind: 'coupleSynastry';
  aspectPreset: 'major' | 'extended' | 'custom'; rulership: 'traditional' | 'modern';
  domainProfile: { id: 'sevenmlabs-couple-selection'; version: '1.0'; };
  reportProfile: { id: 'sevenmlabs-couple-report'; version: '1.0'; }; customProfileCount: number;
  subjectCalculations: Record<CoupleChartId, Pick<BasicNatalCalculation, 'julianDayTt' | 'julianDayUt1' | 'houseSystem'>>;
  composite?: CompositeCalculation;
}
export interface CoupleResult extends Omit<Result, 'calculation' | 'data'> {
  calculation: CoupleCalculation; data: CoupleData;
}
/** A symbolic midpoint has no instantaneous motion, latitude, distance, date or location. */
export interface CompositePlacement extends Omit<Placement, 'latitude' | 'distanceAu' | 'speed' | 'isRetrograde'> {
  speed: null; isRetrograde: null;
}
export interface CompositeAspect extends Omit<Aspect, 'applying'> { applying: null; }
export interface CompositeContextAspect extends Omit<ContextAspect, 'applying'> { applying: null; }
export interface CompositeChart {
  placements: CompositePlacement[]; angles: CompositePlacement[];
  houses: House[]; houseCusps: number[]; aspects: CompositeAspect[];
}
export interface CompositeContextPoint extends Omit<ContextPoint, 'latitude' | 'distanceAu' | 'speed' | 'isRetrograde'> {
  speed: null; isRetrograde?: null;
}
export interface CompositeHouse extends Omit<DomainHouse, 'occupants' | 'rulerPlacement'> {
  occupants: CompositePlacement[]; rulerPlacement: CompositePlacement;
}
export interface CompositeBodyState extends Omit<NatalBodyState, 'motion'> { motion: 'notApplicable'; }
export interface CompositeAdvancedRules extends Omit<NatalAdvancedRules, 'motion' | 'distributions'> {
  id: 'sevenmlabs-composite-facts';
  motion: { source: 'symbolicMidpointConstruction'; stationary: 'notApplicable'; predictsStation: false; };
  distributions: { population: 'allCompositeBodies'; weights: 'onePerBody'; emptyGroups: 'retained'; };
}
export interface CompositeAdvancedFacts extends Omit<NatalAdvancedFacts, 'rules' | 'bodyStates'> {
  rules: CompositeAdvancedRules; bodyStates: CompositeBodyState[];
}
export interface CompositeContext extends Omit<NatalContext, 'points' | 'houses' | 'aspects' | 'dispositors' | 'advanced'> {
  points: CompositeContextPoint[]; houses: CompositeHouse[]; aspects: CompositeContextAspect[];
  dispositors: Array<{ bodyId: string; sign: string; dispositorBodyId: string; dispositorPlacement: CompositePlacement; }>;
  advanced: CompositeAdvancedFacts;
}
export interface CompositeReportHouse extends Omit<ReportHouse, 'occupants' | 'rulerPlacement'> {
  occupants: CompositePlacement[]; rulerPlacement: CompositePlacement;
}
export interface CompositeReportAspect extends Omit<ReportAspect, 'applying'> { applying: null; }
export interface CompositeDomainReport extends Omit<IndividualDomainReport,
  'chartKind' | 'houses' | 'points' | 'aspects' | 'bodyFacts'> {
  chartKind: 'midpointComposite'; houses: CompositeReportHouse[]; points: CompositeContextPoint[];
  aspects: CompositeReportAspect[]; bodyFacts: CompositeBodyState[];
}
export interface CompositeDomainView extends Omit<DomainView, 'houses' | 'bodies' | 'angles' | 'aspects' | 'report'> {
  houses: CompositeHouse[];
  bodies: Array<{ placement: CompositePlacement; selectionReasons: DomainView['bodies'][number]['selectionReasons']; }>;
  angles: CompositePlacement[];
  aspects: Array<CompositeContextAspect & { selectedEndpoints: string[]; }>;
  report: CompositeDomainReport;
}
export interface CompositeProvenancePoint {
  id: NatalPointId; kind: 'body' | 'angle' | 'houseCusp';
  sourcePointIds: [`A:${NatalPointId}`, `B:${NatalPointId}`]; sourceLongitudes: [number, number];
  longitude: number;
  construction: 'shortestArcMidpoint' | 'oppositeCompositeAscendant' | 'oppositeCompositeMidheaven'
    | 'wholeSignFromMidpointAscendant';
  antipodal: boolean; resolution: 'unambiguous' | 'lowerLongitude' | 'notRequiredForDerivedPoint';
}
export interface CompositeProvenance {
  method: 'shortestArcMidpoint'; houseMethod: CompositeHouseMethod;
  antipodalPolicy: CompositeAntipodalPolicy; toleranceDegrees: number;
  sourceHouseSystems: Record<CoupleChartId, 'placidus' | 'wholeSign'>;
  points: CompositeProvenancePoint[];
}
export interface CompositeData {
  id: 'C'; chartKind: 'midpointComposite'; sourceSubjectIds: ['A', 'B']; chart: CompositeChart;
  context: CompositeContext; domains: Partial<Record<DomainName, CompositeDomainView>>;
  customDomains: Record<string, CompositeDomainView>; profileCatalog: ProfileDefinition[];
  provenance: CompositeProvenance;
}
export interface CompositeCalculation {
  scope: 'midpoint-composite-data'; chartKind: 'midpointComposite'; construction: 'symbolic';
  method: 'shortestArcMidpoint'; houseMethod: CompositeHouseMethod;
  antipodalPolicy: CompositeAntipodalPolicy; toleranceDegrees: number;
  sourceProvider: 'swiss-ephemeris'; sourceProviderVersion: string; sourceEphemeris: 'moshier';
  zodiac: 'tropical'; angleUnit: 'degrees'; speedUnit: null; motion: 'notApplicable';
  aspectPreset: 'major' | 'extended' | 'custom'; aspectRules: AspectRule[];
  rulership: 'traditional' | 'modern';
  domainProfile: { id: 'sevenmlabs-domain-selection'; version: '2.0'; };
  reportProfile: { id: 'sevenmlabs-individual-report'; version: '1.1'; }; customProfileCount: number;
  sourceCalculations: Record<CoupleChartId, Pick<BasicNatalCalculation, 'julianDayTt' | 'julianDayUt1' | 'houseSystem'>>;
}
export interface CompositeSourceSubject {
  id: CoupleChartId; natal: NatalData; calculation: BasicNatalCalculation;
}
export interface CompositeRelationshipData {
  chartKind: 'compositeRelationship'; subjectCount: 2; chartCount: 3;
  subjects: Record<CoupleChartId, CompositeSourceSubject>; composite: CompositeData;
}
export interface CompositeResult extends Omit<Result, 'calculation' | 'data'> {
  calculation: CompositeCalculation; data: CompositeRelationshipData;
}
export interface CoupleWithCompositeResult extends CoupleResult {
  calculation: CoupleCalculation & { composite: CompositeCalculation; };
  data: CoupleData & { chartCount: 3; composite: CompositeData; };
}
export interface EventUtc {
  year: number; month: number; day: number; hour: number; minute: number; second: number;
}
export interface EventLocalTime extends EventUtc { utcOffsetMinutes: number; }
export interface NormalizedForecastPeriod {
  kind: 'day' | 'month' | 'year'; year: number; month: number | null; day: number | null;
  utcOffsetMinutes: number; startUtc: EventUtc; endUtc: EventUtc;
  startJulianDayUt1: number; endJulianDayUt1: number; durationDays: number;
}
/** Physical geocentric positions without a horizon or transit house calculation. */
export interface EventPosition {
  id: NatalBodyId; longitude: number; latitude: number; distanceAu: number;
  speed: number; isRetrograde: boolean; sign: string; degreeInSign: number;
}
export interface BracketedEventPrecision {
  method: 'bracketedRoot';
  bracketSeconds: number; residual: number; residualUnit: 'degrees' | 'degrees/day';
}
export interface SwissEclipsePrecision {
  method: 'swissEclipseSearch'; bracketSeconds: null; residual: null; residualUnit: null;
}
export type EventPrecision = BracketedEventPrecision | SwissEclipsePrecision;
export interface AstronomicalEventBase {
  id: string; utc: EventUtc; local: EventLocalTime; julianDayUt1: number;
  bodyIds: NatalBodyId[]; positions: EventPosition[]; precision: EventPrecision;
}
export type NatalForecastPointId = `N:${NatalPointId}`;
export type TransitForecastPointId = `T:${NatalBodyId}`;
export type IngressEvent = AstronomicalEventBase & { type: 'ingress'; details: {
  boundaryLongitude: number; fromSign: string; toSign: string; direction: 'direct' | 'retrograde';
}; };
export type StationEvent = AstronomicalEventBase & { type: 'station'; details: { direction: 'direct' | 'retrograde'; }; };
export type LunarPhaseEvent = AstronomicalEventBase & { type: 'lunarPhase'; details: {
  phase: 'newMoon' | 'firstQuarter' | 'fullMoon' | 'lastQuarter'; angle: 0 | 90 | 180 | 270;
}; };
export type PlanetaryAspectEvent = AstronomicalEventBase & { type: 'planetaryAspect'; details: { angle: number; branchLongitude: number; }; };
export type NatalTransitEvent = AstronomicalEventBase & { type: 'natalTransit'; details: {
  angle: number; branchLongitude: number; targetPointId: NatalForecastPointId; targetLongitude: number;
}; };
export type EclipseContactName = 'eclipseBegin' | 'eclipseEnd' | 'centralPhaseBegin' | 'centralPhaseEnd'
  | 'partialBegin' | 'partialEnd' | 'totalityBegin' | 'totalityEnd' | 'penumbralBegin' | 'penumbralEnd';
export type EclipseEvent = AstronomicalEventBase & { type: 'solarEclipse' | 'lunarEclipse';
  precision: SwissEclipsePrecision;
  details: { eclipseType: 'total' | 'annular' | 'partial' | 'hybrid' | 'penumbral';
    visibilityScope: 'global'; providerFlags: number;
    contacts: Array<{ name: EclipseContactName; utc: EventUtc; local: EventLocalTime; julianDayUt1: number; }>; };
};
export type GlobalAstronomicalEvent = IngressEvent | StationEvent | LunarPhaseEvent | PlanetaryAspectEvent | EclipseEvent;
export type AstronomicalEvent = GlobalAstronomicalEvent | NatalTransitEvent;
export interface EventSnapshot {
  utc: EventUtc; local: EventLocalTime; julianDayUt1: number; positions: EventPosition[];
}
export interface EventSearchMetadata {
  samplingHours: 6; timeToleranceSeconds: 0.25; angularToleranceDegrees: 0.000001;
  stationToleranceDegreesPerDay: 0.00000001; maximumRefinementIterations: 80;
  rootIsolation: 'velocityExtremaPartitioned'; interval: 'startInclusiveEndExclusive';
  truncated: false; eventLimit: 30000; gridIntervals: number;
  eclipseMethod: 'Swiss global eclipse maximum, no local visibility';
  velocityExtremaToleranceDegreesPerDay: 0.0000001; tangencyToleranceDegrees: 0.0000000001;
  duplicateTimeToleranceSeconds: 0.001; maximumProviderBodyEvaluations: 2000000;
}
export interface EventTypeCounts {
  ingress: number; station: number; lunarPhase: number; planetaryAspect: number; natalTransit: number;
  solarEclipse: number; lunarEclipse: number;
}
export interface EventsData {
  chartKind: 'astronomicalEvents'; period: NormalizedForecastPeriod; snapshot: EventSnapshot;
  events: GlobalAstronomicalEvent[]; coverage: { eventCount: number; byType: EventTypeCounts; };
}
export interface EventCalculationBase {
  provider: 'swiss-ephemeris'; providerVersion: string; ephemeris: 'moshier'; zodiac: 'tropical';
  coordinates: 'geocentric'; positionType: 'apparent'; referenceFrame: 'ecliptic-of-date';
  calendar: 'gregorian'; inputTimeScale: 'UTC'; planetTimeScale: 'TT'; searchTimeScale: 'UT1';
  timeModel: string; angleUnit: 'degrees'; speedUnit: 'degrees/day'; distanceUnit: 'AU';
  bodyIds: NatalBodyId[]; eventTypes: GlobalEventType[];
  aspectPreset: 'major' | 'extended' | 'custom'; aspectRules: AspectRule[]; search: EventSearchMetadata;
}
export interface EventsCalculation extends EventCalculationBase { scope: 'astronomical-event-data'; }
export interface EventsResult extends Omit<Result, 'calculation' | 'data'> {
  calculation: EventsCalculation; data: EventsData;
}
export interface ForecastSnapshotPosition extends EventPosition { pointId: TransitForecastPointId; }
export interface ForecastRelation {
  id: string; transitPointId: TransitForecastPointId; natalPointId: NatalForecastPointId;
  signedDelta: number; separation: number; aspectIndexes: number[];
}
export interface ForecastAspect {
  index: number; relationId: string; transitPointId: TransitForecastPointId; natalPointId: NatalForecastPointId;
  angle: number; separation: number; orb: number; maxOrb: number;
  /** Transit velocity relative to fixed natal longitude; null at exact geometry. */
  applying: boolean | null;
}
export interface ForecastHouseOverlay {
  transitPointId: TransitForecastPointId; natalHouseId: `N:${NatalHouseId}`; natalHouseNumber: number;
  natalRulerPointId: `N:${NatalBodyId}`; natalOccupantPointIds: Array<`N:${NatalBodyId}`>;
}
export interface ForecastSnapshot extends Omit<EventSnapshot, 'positions'> {
  positions: ForecastSnapshotPosition[]; relations: ForecastRelation[]; aspects: ForecastAspect[];
  houseOverlays: ForecastHouseOverlay[];
}
export interface ForecastPersonalImpact {
  primaryNatalPointId: NatalForecastPointId | null; primaryAngle: number | null;
  houseOverlays: ForecastHouseOverlay[]; contacts: ForecastAspect[];
  affectedNatalPointIds: NatalForecastPointId[]; affectedNatalHouseIds: Array<`N:${NatalHouseId}`>;
  rulerLinks: Array<{ natalPointId: NatalForecastPointId; natalHouseIds: Array<`N:${NatalHouseId}`>; }>;
}
export type ForecastHighlightReason = 'station' | 'newOrFullMoon' | 'slowBodyIngress'
  | 'slowBodyPlanetaryAspect' | 'nonLunarNatalTransit' | 'eclipse';
export type ForecastEvent = AstronomicalEvent & {
  personalImpact: ForecastPersonalImpact; highlightReasons: ForecastHighlightReason[];
};
export interface ForecastEventReference {
  eventId: string; primaryContact: boolean; sectionIds: string[];
  reasons: Array<{ code: 'natalContact' | 'exactNatalTarget' | 'transitThroughFocusHouse';
    natalPointIds: NatalForecastPointId[]; natalHouseIds: Array<`N:${NatalHouseId}`>; aspectIndexes: number[]; }>;
}
export interface ForecastSection {
  id: string; natalPointIds: NatalForecastPointId[]; natalHouseIds: Array<`N:${NatalHouseId}`>;
  snapshotAspectIndexes: number[]; snapshotOverlayPointIds: TransitForecastPointId[];
  eventIds: string[]; highlightEventIds: string[];
}
export interface ForecastView {
  id: string; origin: 'builtin' | 'custom'; profileVersion: string; definition: ProfileDefinition;
  primaryNatalPointIds: NatalForecastPointId[]; primaryNatalHouseIds: Array<`N:${NatalHouseId}`>;
  natalPointIds: NatalForecastPointId[]; natalHouseIds: Array<`N:${NatalHouseId}`>;
  snapshotAspectIndexes: number[]; snapshotOverlayPointIds: TransitForecastPointId[];
  eventReferences: ForecastEventReference[]; highlightEventIds: string[]; sections: ForecastSection[];
  messageContext: { kind: 'dailyMessage' | 'monthlyOverview' | 'yearlyOverview';
    snapshotAspectIndexes: number[]; eventIds: string[]; highlightEventIds: string[];
    natalHouseIds: Array<`N:${NatalHouseId}`>; narrative: null; };
}
export interface ForecastOverview {
  eventCount: number; byType: EventTypeCounts; natalHouseEventCounts: Record<`N:${NatalHouseId}`, number>;
  domainEventCounts: Record<string, number>; highlightEventIds: string[];
  days: Array<{ date: string; eventIds: string[]; highlightEventIds: string[]; }>;
  months: Array<{ year: number; month: number; eventIds: string[]; highlightEventIds: string[]; }>;
}
export interface ForecastData {
  chartKind: 'individualForecast'; subjectCount: 1;
  subject: { id: 'N'; natal: NatalData; context: NatalContext; calculation: BasicNatalCalculation; };
  period: NormalizedForecastPeriod; snapshot: ForecastSnapshot; events: ForecastEvent[];
  domains: Partial<Record<DomainName, ForecastView>>; customDomains: Record<string, ForecastView>;
  profileCatalog: ProfileDefinition[]; overview: ForecastOverview;
}
export interface ForecastCalculation extends EventCalculationBase {
  scope: 'individual-forecast-data'; chartKind: 'individualForecast'; includeNatalTransits: boolean;
  rulership: 'traditional' | 'modern'; domainProfile: { id: 'sevenmlabs-domain-selection'; version: '2.0'; };
  forecastProfile: { id: 'sevenmlabs-individual-forecast'; version: '1.0'; }; customProfileCount: number;
  transitTargetMotion: 'fixedNatalLongitudes'; snapshotPolicy: 'periodMidpoint';
  highlightPolicy: { id: 'sevenmlabs-calendar-highlights'; version: '1.0'; slowBodyIds: NatalBodyId[]; };
}
export interface ForecastResult extends Omit<Result, 'calculation' | 'data'> {
  calculation: ForecastCalculation; data: ForecastData;
}
export interface QueryZodiacPosition { longitude: number; signIndex: number; sign: string; degreeInSign: number; }
export interface GeometryNormalizeData extends QueryZodiacPosition { group: 'geometry'; action: 'normalize'; inputLongitude: number; }
export interface GeometrySeparationData { group: 'geometry'; action: 'separation'; longitude1: number; longitude2: number; signedDelta: number; separation: number; }
export interface GeometryMidpointData { group: 'geometry'; action: 'midpoint'; longitude1: number; longitude2: number; longitude: number; antipodal: boolean; antipodalPolicy: CompositeAntipodalPolicy; resolution: 'unambiguous' | 'lowerLongitude'; }
/** Supplied longitude geometry has no physical latitude or distance. */
export interface QueryPlacement extends Omit<Placement, 'latitude' | 'distanceAu' | 'house'> { house: null; }
export interface QueryPairRelation { id: string; point1: string; point2: string; signedDelta: number; separation: number; }
export interface QueryPairAspect { point1: string; point2: string; angle: number; orb: number; maxOrb: number; matched: boolean; applying: boolean | null; }
export interface AspectsBetweenData {
  group: 'aspects'; action: 'between'; positions: [QueryPlacement, QueryPlacement]; rule: AspectRule;
  relation: QueryPairRelation; aspect: QueryPairAspect; motionMode: QueryMotionMode; distanceRate: number | null;
}
export interface HousesLocateData {
  group: 'houses'; action: 'locate'; longitude: number; houseCusps: number[];
  house: House & { id: NatalHouseId; signIndex: number; widthDegrees: number; };
  boundaryRule: 'cuspInclusiveNextCuspExclusive';
}
export type QueryInspectionGroup = 'aspects' | 'points' | 'houses';
/** Local aspect indexes resolve in this response; supporting points make every endpoint resolvable. */
export interface QueryInspectionData<G extends QueryInspectionGroup = QueryInspectionGroup> {
  group: G; action: 'inspect'; chartKind: 'individualNatal';
  selection: { pointIds: NatalPointId[]; houseNumbers: number[]; aspectSelection: G extends 'aspects' ? 'bothSelectedEndpoints' : 'atLeastOneSelectedEndpoint'; };
  primaryPointIds: NatalPointId[]; primaryHouseIds: NatalHouseId[];
  points: Array<ContextPoint & { id: NatalPointId; }>; houses: Array<DomainHouse & { id: NatalHouseId; }>;
  relations: PointRelation[]; aspects: ContextAspect[];
  coverage: { primaryPoints: number; supportingPoints: number; selectedHouses: number; pointPairs: number; matchedAspects: number; };
}
export interface PureQueryCalculation<G extends 'geometry' | 'aspects' | 'houses' = 'geometry' | 'aspects' | 'houses', A extends string = string> {
  scope: 'on-demand-query'; computationalScope: 'pureGeometry'; group: G; action: A;
  provider: 'supplied-positions'; angleUnit: 'degrees';
}
export interface QueryMidpointCalculation extends PureQueryCalculation<'geometry', 'midpoint'> { antipodalPolicy: CompositeAntipodalPolicy; toleranceDegrees: 1e-10; }
export interface QueryBetweenCalculation extends PureQueryCalculation<'aspects', 'between'> { speedUnit: 'degrees/day'; motionMode: QueryMotionMode; }
export interface QueryInspectionCalculation<G extends QueryInspectionGroup = QueryInspectionGroup> extends Omit<BasicNatalCalculation, 'scope'> {
  scope: 'on-demand-query'; sourceScope: 'basic-natal'; computationalScope: 'fullNatalThenSelection'; group: G; action: 'inspect';
  rulership: 'traditional' | 'modern'; aspectPreset: 'major' | 'extended' | 'custom';
  selectionPolicy: G extends 'aspects' ? 'bothSelectedEndpoints' : 'atLeastOneSelectedEndpoint';
}
export type QueryData = GeometryNormalizeData | GeometrySeparationData | GeometryMidpointData | AspectsBetweenData | HousesLocateData | QueryInspectionData;
export type QueryCalculation = PureQueryCalculation<'geometry', 'normalize' | 'separation'> | PureQueryCalculation<'houses', 'locate'> | QueryMidpointCalculation | QueryBetweenCalculation | QueryInspectionCalculation;
export interface QueryResult<D extends QueryData = QueryData, C extends QueryCalculation = QueryCalculation> extends Omit<Result, 'calculation' | 'data'> { calculation: C; data: D; }
export type GeometryNormalizeResult = QueryResult<GeometryNormalizeData, PureQueryCalculation<'geometry', 'normalize'>>;
export type GeometrySeparationResult = QueryResult<GeometrySeparationData, PureQueryCalculation<'geometry', 'separation'>>;
export type GeometryMidpointResult = QueryResult<GeometryMidpointData, QueryMidpointCalculation>;
export type AspectsBetweenResult = QueryResult<AspectsBetweenData, QueryBetweenCalculation>;
export type HousesLocateResult = QueryResult<HousesLocateData, PureQueryCalculation<'houses', 'locate'>>;
export type AspectsInspectResult = QueryResult<QueryInspectionData<'aspects'>, QueryInspectionCalculation<'aspects'>>;
export type PointsInspectResult = QueryResult<QueryInspectionData<'points'>, QueryInspectionCalculation<'points'>>;
export type HousesInspectResult = QueryResult<QueryInspectionData<'houses'>, QueryInspectionCalculation<'houses'>>;
/** Distributive omission preserves the mutually exclusive aspect settings on inspection options. */
export type QueryMethodOptions<R extends QueryRequest> = R extends unknown ? Omit<R, 'operation' | 'group' | 'action'> : never;
/** Typed convenience namespaces route into the same native calculation core. */
export const geometry: {
  normalize(options: QueryMethodOptions<GeometryNormalizeRequest>): GeometryNormalizeResult;
  separation(options: QueryMethodOptions<GeometrySeparationRequest>): GeometrySeparationResult;
  midpoint(options: QueryMethodOptions<GeometryMidpointRequest>): GeometryMidpointResult;
};
export const aspects: {
  between(options: QueryMethodOptions<AspectsBetweenRequest>): AspectsBetweenResult;
  inspect(options: QueryMethodOptions<AspectsInspectRequest>): AspectsInspectResult;
};
export const houses: {
  locate(options: QueryMethodOptions<HousesLocateRequest>): HousesLocateResult;
  inspect(options: QueryMethodOptions<HousesInspectRequest>): HousesInspectResult;
};
export const points: { inspect(options: QueryMethodOptions<PointsInspectRequest>): PointsInspectResult; };
export function calculate(input: GeometryNormalizeRequest): GeometryNormalizeResult;
export function calculate(input: GeometrySeparationRequest): GeometrySeparationResult;
export function calculate(input: GeometryMidpointRequest): GeometryMidpointResult;
export function calculate(input: AspectsBetweenRequest): AspectsBetweenResult;
export function calculate(input: HousesLocateRequest): HousesLocateResult;
export function calculate(input: AspectsInspectRequest): AspectsInspectResult;
export function calculate(input: PointsInspectRequest): PointsInspectResult;
export function calculate(input: HousesInspectRequest): HousesInspectResult;
export function calculate(input: QueryRequest): QueryResult;
export function calculate(input: EventsRequest): EventsResult;
export function calculate(input: ForecastRequest): ForecastResult;
export function calculate(input: CoupleRequest & { composite: CompositeOptions; }): CoupleWithCompositeResult;
export function calculate(input: CoupleRequest): CoupleResult;
export function calculate(input: CompositeRequest): CompositeResult;
export function calculate(input: NatalDomainsRequest): NatalDomainsResult;
/** Synchronous local calculation; validation errors throw with .code and .result. */
export function calculate(input: GeometryRequest | NatalRequest | NatalDomainsRequest): Result;
export function calculate(input: Request): Result | CoupleResult | CompositeResult | EventsResult | ForecastResult | QueryResult;
/** Low-level JSON envelope, including errors; malformed requests do not throw. */
export function calculateJson(input: string): string;

/** Context preparation is independent of astronomical calculation and runs in the native core. */
export interface CompressionOptions {
  /** compact preserves all facts; focused/budgeted report every deliberate omission. */
  mode?: 'compact' | 'focused' | 'budgeted';
  /** Profile IDs to retain. Empty/absent leaves the domain scope unrestricted. */
  domains?: string[];
  /** Section IDs within the selected domains. */
  sections?: string[];
  /** Positive integer byte budget; mandatory evidence is never silently truncated. */
  maxBytes?: number;
  /** Positive integer estimate budget. This is not an exact model-specific token count. */
  maxTokens?: number;
}
export type ContextJsonValue = null | boolean | number | string | ContextJsonValue[]
  | { [key: string]: ContextJsonValue };
export interface ContextData {
  format: 'astro-context/1';
  mode: 'compact' | 'focused' | 'budgeted';
  /** Encoded retained engine envelope; use expandContext to recover its JSON structure. */
  payload: ContextJsonValue;
  dictionary: Record<string, ContextJsonValue>;
  coverage: {
    complete: boolean;
    retainedPaths: string[];
    scope: { domains: string[]; sections: string[]; };
  };
  omitted: Array<{ path: string; reason: string; }>;
  metrics: {
    inputBytes: number; outputBytes: number; estimatedTokens: number;
    tokenEstimateMethod: 'utf8-bytes-conservative';
  };
  budget: { maxBytes: number | null; maxTokens: number | null; exceeded: boolean; };
}
export interface ContextResult {
  schemaVersion: string;
  engineVersion: string;
  data: ContextData;
  warnings: string[];
  errors: Array<{ code: string; message: string; }>;
}
/** A scoped context may retain only part of the original data structure. */
export interface RetainedPayloadResult {
  schemaVersion: string; engineVersion: string;
  calculation?: ContextJsonValue;
  data: ContextJsonValue;
  warnings: string[];
  errors: Array<{ code: string; message: string; }>;
  [key: string]: ContextJsonValue | undefined;
}
/** JSON-compatible envelope; rejects non-finite numbers/unsupported JSON values. Validation errors throw with .code and .result. Does not mutate payload. */
export function compressPayload(payload: object, options?: CompressionOptions): ContextResult;
/** Preserves raw JSON number literals. Validation errors are returned in the envelope. */
export function compressPayloadJson(payloadJson: string, optionsJson?: string): string;
/** Reconstruct the retained engine envelope. Omitted facts cannot be restored. */
export function expandContext(context: ContextResult): RetainedPayloadResult;
export function expandContextJson(contextJson: string): string;
/** Synchronous calculation followed by context preparation. Keeps the original result for UI/audit. */
export type CalculationResultFor<R extends Request> =
  R extends GeometryNormalizeRequest ? GeometryNormalizeResult :
  R extends GeometrySeparationRequest ? GeometrySeparationResult :
  R extends GeometryMidpointRequest ? GeometryMidpointResult :
  R extends AspectsBetweenRequest ? AspectsBetweenResult :
  R extends HousesLocateRequest ? HousesLocateResult :
  R extends AspectsInspectRequest ? AspectsInspectResult :
  R extends PointsInspectRequest ? PointsInspectResult :
  R extends HousesInspectRequest ? HousesInspectResult :
  R extends EventsRequest ? EventsResult :
  R extends ForecastRequest ? ForecastResult :
  R extends CoupleRequest & { composite: CompositeOptions; } ? CoupleWithCompositeResult :
  R extends CoupleRequest ? CoupleResult :
  R extends CompositeRequest ? CompositeResult :
  R extends NatalDomainsRequest ? NatalDomainsResult : Result;
export function calculateWithContext<R extends Request>(input: R, options?: CompressionOptions): {
  result: CalculationResultFor<R>;
  context: ContextResult;
};
