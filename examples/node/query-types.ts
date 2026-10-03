/** Compile-only contract checks against the installed Node package. */
import { calculate, geometry, aspects, houses, points, BirthInput, QueryRequest, QueryResult,
  GeometryNormalizeResult, GeometryMidpointResult, AspectsBetweenResult, AspectsInspectResult,
  PointsInspectResult, HousesInspectResult, HousesLocateResult, NatalPointId, QueryMotionMode,
  AspectsInspectRequest, PointsInspectRequest } from '@7mlabs/astrology';
const birth: BirthInput = { utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 } };
const normalized: GeometryNormalizeResult = geometry.normalize({ longitude: -10 });
const group: 'geometry' = normalized.data.group;
const longitude: number = normalized.data.longitude;
const provider: 'supplied-positions' = normalized.calculation.provider;
const midpoint: GeometryMidpointResult = calculate({ operation: 'query', group: 'geometry',
  action: 'midpoint', longitude1: 350, longitude2: 10, antipodalPolicy: 'lowerLongitude' });
const tolerance: 1e-10 = midpoint.calculation.toleranceDegrees;
const between: AspectsBetweenResult = aspects.between({ positions: [
  { id: 'moving', longitude: 22, speed: 1 }, { id: 'fixed', longitude: 100, speed: 2 }],
  rule: { angle: 77, maxOrb: 2 }, motionMode: 'fixedSecond' });
const matched: boolean = between.data.aspect.matched;
const rate: number | null = between.data.distanceRate;
const house: HousesLocateResult = houses.locate({ longitude: 350,
  houseCusps: [350, 20, 50, 80, 110, 140, 170, 200, 230, 260, 290, 320] });
const boundary: 'cuspInclusiveNextCuspExclusive' = house.data.boundaryRule;
const selectedHouses: HousesInspectResult = houses.inspect({ birth, houseNumbers: [2, 7], rulership: 'modern' });
const selectedPoints: PointsInspectResult = points.inspect({ birth, pointIds: ['sun', 'ascendant', 'H7'] });
const selectedAspects: AspectsInspectResult = aspects.inspect({ birth, pointIds: ['sun', 'moon', 'H7'],
  aspectRules: [{ angle: 77, maxOrb: 2 }] });
const fullScope: 'fullNatalThenSelection' = selectedPoints.calculation.computationalScope;
const sourceScope: 'basic-natal' = selectedAspects.calculation.sourceScope;
const selectedPolicy: 'bothSelectedEndpoints' = selectedAspects.calculation.selectionPolicy;
const pointsPolicy: 'atLeastOneSelectedEndpoint' = selectedPoints.data.selection.aspectSelection;
const selectedId: NatalPointId = selectedPoints.data.primaryPointIds[0];
function callDynamic(input: QueryRequest): QueryResult { return calculate(input); }
// @ts-expect-error Supplied positions do not contain physical latitude.
const latitude = between.data.positions[0].latitude;
// @ts-expect-error Routing keys cannot be supplied to grouped convenience methods.
geometry.normalize({ longitude: 10, operation: 'query' });
// @ts-expect-error A between query accepts exactly two positions.
aspects.between({ positions: [{ id: 'one', longitude: 1 }], rule: { angle: 77, maxOrb: 2 } });
// @ts-expect-error Unknown point selectors are rejected.
const unknownPoint: PointsInspectRequest = { operation: 'query', group: 'points', action: 'inspect', birth, pointIds: ['chiron'] };
// @ts-expect-error Aspect preset and rules are mutually exclusive in raw query requests.
const conflict: AspectsInspectRequest = { operation: 'query', group: 'aspects', action: 'inspect', birth, aspectPreset: 'major', aspectRules: [] };
// @ts-expect-error The same mutual exclusion applies to grouped methods.
aspects.inspect({ birth, aspectPreset: 'major', aspectRules: [] });
// @ts-expect-error Motion modes use an explicit none value and reject null.
const nullMotion: QueryMotionMode = null;
// @ts-expect-error House selectors belong to houses.inspect.
points.inspect({ birth, houseNumbers: [7] });
// @ts-expect-error Query actions remain grouped and cannot be mixed.
const mismatched: QueryRequest = { operation: 'query', group: 'points', action: 'locate', longitude: 0, houseCusps: [] };
void [group, longitude, provider, tolerance, matched, rate, boundary, selectedHouses, fullScope,
  sourceScope, selectedPolicy, pointsPolicy, selectedId, callDynamic, latitude, unknownPoint, conflict, nullMotion, mismatched];
