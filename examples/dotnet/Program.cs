using SevenMLabs.Astrology;
using System.Text.Json;
using System.Text.Json.Nodes;

if (args.Length > 0 && (args[0] == "--compress-json" || args[0] == "--expand-context-json"))
{
    var input = Console.In.ReadToEnd();
    if (args[0] == "--expand-context-json") Console.WriteLine(Engine.ExpandContextJson(input));
    else
    {
        using var wrapper = JsonDocument.Parse(input);
        var root = wrapper.RootElement;
        Console.WriteLine(Engine.CompressPayloadJson(root.GetProperty("payload").GetRawText(),
            root.TryGetProperty("options", out var options) ? options.GetRawText() : "{}"));
    }
    return;
}

const string queryOptionsJson = """
{
  "birth": {
    "utc": {
      "year": 2000,
      "month": 1,
      "day": 1,
      "hour": 12,
      "minute": 0
    },
    "location": {
      "latitude": 10.8231,
      "longitude": 106.6297
    },
    "houseSystem": "placidus"
  },
  "houseNumbers": [
    2,
    8
  ],
  "rulership": "modern"
}
""";
if (args.Length > 0 && args[0] == "--query")
{
    Console.WriteLine(Engine.Houses.Inspect(JsonSerializer.Deserialize<JsonElement>(queryOptionsJson)));
    return;
}

const string forecastRequestJson = """
{
  "operation": "forecast",
  "birth": {
    "utc": {
      "year": 2000,
      "month": 1,
      "day": 1,
      "hour": 12,
      "minute": 0
    },
    "location": {
      "latitude": 10.8231,
      "longitude": 106.6297
    },
    "houseSystem": "placidus"
  },
  "period": {
    "kind": "day",
    "year": 2026,
    "month": 3,
    "day": 3,
    "utcOffsetMinutes": 420
  }
}
""";
var forecastRequest = JsonSerializer.Deserialize<JsonElement>(forecastRequestJson);
if (args.Length > 0 && args[0] == "--forecast")
{
    Console.WriteLine(Engine.Calculate(forecastRequest));
    return;
}

const string compositeRequestJson = """
{
  "operation": "composite",
  "personA": {
    "utc": {
      "year": 2000,
      "month": 1,
      "day": 1,
      "hour": 12,
      "minute": 0
    },
    "location": {
      "latitude": 10.8231,
      "longitude": 106.6297
    },
    "houseSystem": "placidus"
  },
  "personB": {
    "utc": {
      "year": 1998,
      "month": 6,
      "day": 15,
      "hour": 6,
      "minute": 30
    },
    "location": {
      "latitude": 21.0278,
      "longitude": 105.8342
    },
    "houseSystem": "placidus"
  },
  "rulership": "traditional",
  "aspectPreset": "extended",
  "houseMethod": "midpoint",
  "antipodalPolicy": "error"
}
""";
var compositeRequest = JsonSerializer.Deserialize<JsonElement>(compositeRequestJson);
if (args.Length > 0 && args[0] == "--composite")
{
    Console.WriteLine(Engine.Calculate(compositeRequest));
    return;
}


const string coupleCompositeRequestJson = """
{
  "operation": "couple",
  "personA": {
    "utc": {
      "year": 2000,
      "month": 1,
      "day": 1,
      "hour": 12,
      "minute": 0
    },
    "location": {
      "latitude": 10.8231,
      "longitude": 106.6297
    },
    "houseSystem": "placidus"
  },
  "personB": {
    "utc": {
      "year": 1998,
      "month": 6,
      "day": 15,
      "hour": 6,
      "minute": 30
    },
    "location": {
      "latitude": 21.0278,
      "longitude": 105.8342
    },
    "houseSystem": "wholeSign"
  },
  "domains": [
    "attraction",
    "communication",
    "emotionalConnection",
    "longTerm",
    "sharedResources",
    "homeFamily"
  ],
  "rulership": "traditional",
  "aspectPreset": "extended",
  "composite": {
    "houseMethod": "wholeSignFromMidpointAscendant"
  }
}
""";
if (args.Length > 0 && args[0] == "--coupleComposite")
{
    Console.WriteLine(Engine.Calculate(JsonSerializer.Deserialize<JsonElement>(coupleCompositeRequestJson)));
    return;
}

var coupleRequest = new { operation = "couple",
    personA = new {
        utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 },
        location = new { latitude = 10.8231, longitude = 106.6297 }, houseSystem = "placidus" },
    personB = new {
        utc = new { year = 1998, month = 6, day = 15, hour = 6, minute = 30 },
        location = new { latitude = 21.0278, longitude = 105.8342 }, houseSystem = "wholeSign" },
    domains = new[] { "attraction", "communication", "emotionalConnection", "longTerm", "sharedResources", "homeFamily" },
    rulership = "traditional", aspectPreset = "extended"
};
if (args.Length > 0 && args[0] == "--couple")
{
    Console.WriteLine(Engine.Calculate(coupleRequest));
    return;
}

var customRequest = new { operation = "natalDomains",
    utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 },
    location = new { latitude = 10.8231, longitude = 106.6297 },
    domains = Array.Empty<string>(), rulership = "traditional", aspectPreset = "extended",
    customProfiles = new[] { new { id = "personalGrowth", version = "1.0", houses = new[] { 1, 9 },
        bodies = new[] { "sun", "moon", "mercury" }, angles = new[] { "ascendant" },
        sections = new[] { new { id = "study", houses = new[] { 3, 9 }, bodies = new[] { "mercury" } } } } }
};
if (args.Length > 0 && args[0] == "--customProfiles")
{
    Console.WriteLine(Engine.Calculate(customRequest));
    return;
}

if (args.Length > 0 && args[0] == "--domains")
{
    Console.WriteLine(Engine.Calculate(new { operation = "natalDomains",
        utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 },
        location = new { latitude = 10.8231, longitude = 106.6297 },
        domains = new[] { "career", "love", "relationships", "family", "finance", "identity", "learning", "creativity", "innerLife", "dailyLife" },
        rulership = "traditional", aspectPreset = "extended" }));
    return;
}
if (args.Length > 0 && args[0] != "--test")
{
    Console.Write(Engine.CalculateJson(args[0]));
    return;
}
var request = new { operation = "chart", positions = new[] {
    new { id = "moon", longitude = 359, speed = 13 },
    new { id = "sun", longitude = 1, speed = 1 }
}};
var result = Engine.Calculate(request);
if (args.Length == 0) {
    Console.WriteLine(Engine.Calculate(new { operation = "natal",
        utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 },
        location = new { latitude = 10.8231, longitude = 106.6297 } }));
    return;
}
var data = result.GetProperty("data");
if (data.GetProperty("aspects")[0].GetProperty("orb").GetDouble() != 2
    || !data.GetProperty("aspects")[0].GetProperty("applying").GetBoolean()
    || data.GetProperty("midpoints")[0].GetProperty("longitude").GetDouble() != 0)
    throw new Exception("Unexpected geometry");
try { Engine.Calculate(new { operation = "natal", positions = request.positions }); throw new Exception("Expected validation error"); }
catch (EngineException error) when (error.Code == "INVALID_INPUT") { }
using var malformed = JsonDocument.Parse(Engine.CalculateJson("{"));
if (malformed.RootElement.GetProperty("errors")[0].GetProperty("code").GetString() != "INVALID_INPUT") throw new Exception("Expected JSON error");
Parallel.For(0, 1000, _ => {
    if (Engine.Calculate(request).GetRawText() != result.GetRawText()) throw new Exception("Concurrent result mismatch");
});
Console.WriteLine(".NET installed NuGet: geometry, errors, 1000 concurrent calls OK");

var natalRequests = new object[] {
    new { operation = "natal", utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 }, location = new { latitude = 10.8231, longitude = 106.6297 }, houseSystem = "placidus" },
    new { operation = "natal", utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 }, location = new { latitude = -33.86, longitude = 151.21 }, houseSystem = "wholeSign" }
};
var natalResults = natalRequests.Select(r => Engine.Calculate(r).GetRawText()).ToArray();
Parallel.For(0, 1000, i => {
    if (Engine.Calculate(natalRequests[i % 2]).GetRawText() != natalResults[i % 2]) throw new Exception("Concurrent natal mismatch");
});
Console.WriteLine(".NET installed NuGet: natal, 1000 concurrent calls OK");

var domainRequests = new object[] {
    new { operation = "natalDomains", utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 }, location = new { latitude = 10.8231, longitude = 106.6297 }, houseSystem = "placidus", rulership = "traditional" },
    new { operation = "natalDomains", utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 }, location = new { latitude = 10.8231, longitude = 106.6297 }, houseSystem = "wholeSign", rulership = "modern" }
};
var domainResults = domainRequests.Select(r => Engine.Calculate(r).GetRawText()).ToArray();
Parallel.For(0, 20, i => {
    if (Engine.Calculate(domainRequests[i % 2]).GetRawText() != domainResults[i % 2]) throw new Exception("Concurrent domains mismatch");
});
Console.WriteLine(".NET installed NuGet: 10 domain views, 20 concurrent calls OK");

var customResult = Engine.Calculate(customRequest);
if (customResult.GetProperty("data").GetProperty("domains").EnumerateObject().Any()) throw new Exception("Expected custom-only output");
if (customResult.GetProperty("data").GetProperty("customDomains").GetProperty("personalGrowth").GetProperty("report").GetProperty("sections")[0].GetProperty("id").GetString() != "study") throw new Exception("Expected report section");
Parallel.For(0, 20, _ => {
    if (Engine.Calculate(customRequest).GetRawText() != customResult.GetRawText()) throw new Exception("Concurrent custom profile mismatch");
});
Console.WriteLine(".NET installed NuGet: custom-only profile, section, 20 concurrent calls OK");

var coupleResult = Engine.Calculate(coupleRequest);
var coupleData = coupleResult.GetProperty("data");
var crossContext = coupleData.GetProperty("context");
if (coupleData.GetProperty("chartKind").GetString() != "coupleSynastry"
    || coupleData.GetProperty("subjectCount").GetInt32() != 2
    || coupleData.GetProperty("domains").EnumerateObject().Count() != 6
    || coupleData.GetProperty("domains").EnumerateObject().Sum(p => p.Value.GetProperty("report").GetProperty("sections").GetArrayLength()) != 18
    || crossContext.GetProperty("points").GetArrayLength() != 52
    || crossContext.GetProperty("relations").GetArrayLength() != 676
    || crossContext.GetProperty("overlaysAtoB").GetArrayLength() != 10
    || crossContext.GetProperty("overlaysBtoA").GetArrayLength() != 10
    || crossContext.GetProperty("houseRulerRelations").GetArrayLength() != 144)
    throw new Exception("Unexpected couple coverage");
if (coupleData.GetProperty("subjects").GetProperty("A").GetProperty("calculation").GetProperty("houseSystem").GetString() != "placidus"
    || coupleData.GetProperty("subjects").GetProperty("B").GetProperty("calculation").GetProperty("houseSystem").GetString() != "wholeSign")
    throw new Exception("Subject house systems must remain independent");
var couplePointIds = crossContext.GetProperty("points").EnumerateArray().Select(p => p.GetProperty("id").GetString()!).ToHashSet();
if (couplePointIds.Count != 52 || crossContext.GetProperty("points").EnumerateArray().Any(p =>
    p.GetProperty("id").GetString() != $"{p.GetProperty("chartId").GetString()}:{p.GetProperty("localId").GetString()}"))
    throw new Exception("Qualified points must resolve without local ID ambiguity");
if (crossContext.GetProperty("relations").EnumerateArray().Any(r =>
    !r.GetProperty("point1").GetString()!.StartsWith("A:") || !r.GetProperty("point2").GetString()!.StartsWith("B:")
    || !couplePointIds.Contains(r.GetProperty("point1").GetString()!) || !couplePointIds.Contains(r.GetProperty("point2").GetString()!)))
    throw new Exception("Cross relation directions must remain A to B");
if (crossContext.GetProperty("aspects").GetArrayLength() == 0
    || crossContext.GetProperty("aspects").EnumerateArray().Any(a => a.GetProperty("applying").ValueKind != JsonValueKind.Null))
    throw new Exception("Cross applying is undefined across two birth instants");
foreach (var (source, target, field) in new[] { ("A", "B", "overlaysAtoB"), ("B", "A", "overlaysBtoA") })
    if (crossContext.GetProperty(field).EnumerateArray().Any(o =>
        o.GetProperty("sourceChartId").GetString() != source || o.GetProperty("targetChartId").GetString() != target
        || !o.GetProperty("pointId").GetString()!.StartsWith(source + ":")
        || o.GetProperty("targetHouseId").GetString() != $"{target}:H{o.GetProperty("targetHouseNumber").GetInt32()}"))
        throw new Exception("Overlay sender and receiving house must resolve correctly");
var disabledCouple = Engine.Calculate(new { operation = "couple", coupleRequest.personA, coupleRequest.personB,
    domains = new[] { "communication" }, aspectRules = Array.Empty<object>() });
if (disabledCouple.GetProperty("data").GetProperty("context").GetProperty("aspects").GetArrayLength() != 0
    || disabledCouple.GetProperty("data").GetProperty("context").GetProperty("relations").GetArrayLength() != 676)
    throw new Exception("Disabled matching must retain cross relations");
try
{
    Engine.Calculate(new { operation = "couple", coupleRequest.personA, personB = new {
        utc = new { year = 1998, month = 2, day = 30, hour = 6, minute = 30 }, coupleRequest.personB.location } });
    throw new Exception("Expected person B calendar validation error");
}
catch (EngineException error) when (error.Code == "INVALID_INPUT") { }
var coupleRequests = new object[] {
    new { operation = "couple", coupleRequest.personA, coupleRequest.personB, domains = new[] { "communication" } },
    new { operation = "couple", personA = new { coupleRequest.personA.utc, coupleRequest.personA.location, houseSystem = "wholeSign" },
        personB = new { coupleRequest.personB.utc, coupleRequest.personB.location, houseSystem = "placidus" },
        domains = Array.Empty<string>(), rulership = "modern", aspectRules = new[] { new { angle = 0, maxOrb = 3 }, new { angle = 90, maxOrb = 3 } },
        customProfiles = new[] { new { id = "sharedLearning", houses = new[] { 3, 9 }, bodies = new[] { "mercury", "jupiter" },
            sections = new[] { new { id = "ideas", houses = new[] { 3 }, bodies = new[] { "mercury" } } } } } }
};
var coupleResults = coupleRequests.Select(r => Engine.Calculate(r).GetRawText()).ToArray();
Parallel.For(0, 20, i => {
    if (Engine.Calculate(coupleRequests[i % 2]).GetRawText() != coupleResults[i % 2]) throw new Exception("Concurrent couple mismatch");
});
Console.WriteLine(".NET installed NuGet: couple, qualified points, 676 relations, 20 overlays, 144 house-ruler pairs, validation, 20 concurrent calls OK");

var compositeResult = Engine.Calculate(compositeRequest);
var composite = compositeResult.GetProperty("data").GetProperty("composite");
var compositeContext = composite.GetProperty("context");
if (compositeResult.GetProperty("data").GetProperty("chartCount").GetInt32() != 3
    || composite.GetProperty("chartKind").GetString() != "midpointComposite"
    || composite.GetProperty("domains").EnumerateObject().Count() != 10
    || composite.GetProperty("domains").EnumerateObject().Sum(p => p.Value.GetProperty("report").GetProperty("sections").GetArrayLength()) != 30
    || compositeContext.GetProperty("points").GetArrayLength() != 26
    || compositeContext.GetProperty("relations").GetArrayLength() != 325
    || compositeContext.GetProperty("houseRulerRelations").GetArrayLength() != 66)
    throw new Exception("Unexpected third-chart report coverage");
if (composite.GetProperty("chart").TryGetProperty("utc", out _)
    || composite.GetProperty("chart").TryGetProperty("location", out _)
    || compositeResult.GetProperty("calculation").TryGetProperty("julianDayTt", out _)
    || composite.GetProperty("chart").GetProperty("placements").EnumerateArray().Any(p =>
        p.GetProperty("speed").ValueKind != JsonValueKind.Null || p.GetProperty("isRetrograde").ValueKind != JsonValueKind.Null
        || p.TryGetProperty("latitude", out _) || p.TryGetProperty("distanceAu", out _))
    || compositeContext.GetProperty("aspects").EnumerateArray().Any(a => a.GetProperty("applying").ValueKind != JsonValueKind.Null)
    || compositeContext.GetProperty("advanced").GetProperty("bodyStates").EnumerateArray().Any(b => b.GetProperty("motion").GetString() != "notApplicable"))
    throw new Exception("Symbolic composite must not invent physical birth data or motion");
var combinedCompositeRequest = JsonNode.Parse(compositeRequestJson)!.AsObject();
combinedCompositeRequest["operation"] = "couple";
combinedCompositeRequest.Remove("houseMethod");
combinedCompositeRequest.Remove("antipodalPolicy");
combinedCompositeRequest["domains"] = new JsonArray(JsonValue.Create("communication"));
combinedCompositeRequest["composite"] = JsonNode.Parse("{\"houseMethod\":\"midpoint\",\"antipodalPolicy\":\"error\"}");
var combinedCompositeResult = Engine.Calculate(combinedCompositeRequest);
if (combinedCompositeResult.GetProperty("data").GetProperty("composite").GetRawText() != composite.GetRawText()
    || combinedCompositeResult.GetProperty("calculation").GetProperty("composite").GetRawText() != compositeResult.GetProperty("calculation").GetRawText())
    throw new Exception("Combined and standalone composite must match");
var invalidCompositeRequest = combinedCompositeRequest.DeepClone().AsObject();
invalidCompositeRequest["composite"] = JsonNode.Parse("{\"domains\":[\"attraction\"]}");
try { Engine.Calculate(invalidCompositeRequest); throw new Exception("Expected composite options error"); }
catch (EngineException error) when (error.Code == "INVALID_INPUT") { }
var disabledCompositeRequest = JsonNode.Parse(compositeRequestJson)!.AsObject();
disabledCompositeRequest.Remove("aspectPreset");
disabledCompositeRequest["houseMethod"] = "wholeSignFromMidpointAscendant";
disabledCompositeRequest["rulership"] = "modern";
disabledCompositeRequest["aspectRules"] = new JsonArray();
disabledCompositeRequest["domains"] = new JsonArray();
disabledCompositeRequest["customProfiles"] = JsonNode.Parse("[{\"id\":\"sharedLearning\",\"houses\":[3,9],\"bodies\":[\"mercury\",\"jupiter\"]}]");
var compositeRequests = new object[] { combinedCompositeRequest, disabledCompositeRequest };
var compositeResults = compositeRequests.Select(r => Engine.Calculate(r).GetRawText()).ToArray();
using (var disabledComposite = JsonDocument.Parse(compositeResults[1]))
    if (disabledComposite.RootElement.GetProperty("data").GetProperty("composite").GetProperty("context").GetProperty("aspects").GetArrayLength() != 0)
        throw new Exception("Disabled composite aspects must be empty");
Parallel.For(0, 20, i => {
    if (Engine.Calculate(compositeRequests[i % 2]).GetRawText() != compositeResults[i % 2]) throw new Exception("Concurrent composite mismatch");
});
Console.WriteLine(".NET installed package: composite 10/30 reports, symbolic facts, combined reuse, 20 concurrent calls OK");

var forecastResult = Engine.Calculate(forecastRequest);
var forecast = forecastResult.GetProperty("data");
if (forecast.GetProperty("chartKind").GetString() != "individualForecast"
    || forecast.GetProperty("snapshot").GetProperty("relations").GetArrayLength() != 260
    || forecast.GetProperty("snapshot").GetProperty("houseOverlays").GetArrayLength() != 10
    || forecast.GetProperty("domains").EnumerateObject().Count() != 10
    || forecast.GetProperty("domains").EnumerateObject().Sum(p => p.Value.GetProperty("sections").GetArrayLength()) != 30
    || !forecast.GetProperty("events").EnumerateArray().Any(e => e.GetProperty("type").GetString() == "lunarEclipse")
    || !forecast.GetProperty("events").EnumerateArray().Any(e => e.GetProperty("type").GetString() == "natalTransit")
    || forecast.GetProperty("domains").EnumerateObject().Any(p => p.Value.GetProperty("messageContext").GetProperty("narrative").ValueKind != JsonValueKind.Null))
    throw new Exception("Forecast must contain one natal, complete pairs, 10/30 views and real eclipse/natal events");
var quietForecastRequest = JsonNode.Parse(forecastRequestJson)!.AsObject();
quietForecastRequest["eventTypes"] = new JsonArray();
quietForecastRequest["includeNatalTransits"] = false;
quietForecastRequest["aspectRules"] = new JsonArray();
quietForecastRequest["domains"] = new JsonArray();
quietForecastRequest["bodies"] = JsonNode.Parse("[\"mercury\"]");
quietForecastRequest["rulership"] = "modern";
quietForecastRequest["customProfiles"] = JsonNode.Parse("[{\"id\":\"quietFocus\",\"houses\":[3,9],\"bodies\":[\"mercury\",\"jupiter\"]}]");
var quietForecastResult = Engine.Calculate(quietForecastRequest);
if (quietForecastResult.GetProperty("data").GetProperty("events").GetArrayLength() != 0
    || quietForecastResult.GetProperty("data").GetProperty("snapshot").GetProperty("relations").GetArrayLength() != 26)
    throw new Exception("Disabled search retains natal and snapshot raw pairs");
var forecastRequests = new object[] { forecastRequest, quietForecastRequest };
var forecastResults = new[] { forecastResult.GetRawText(), quietForecastResult.GetRawText() };
Parallel.For(0,20,i => { if (Engine.Calculate(forecastRequests[i%2]).GetRawText() != forecastResults[i%2]) throw new Exception("Concurrent forecast mismatch"); });
Console.WriteLine(".NET installed package: daily forecast, 10/30 views, exact natal events, eclipse impact, 20 concurrent calls OK");

var groupedQueryCalls = new (string Group, string Action, JsonElement Options, Func<JsonElement, JsonElement> Invoke)[]
{
    ("geometry", "normalize", JsonSerializer.Deserialize<JsonElement>("""{"longitude":-370}"""), o => Engine.Geometry.Normalize(o)),
    ("geometry", "separation", JsonSerializer.Deserialize<JsonElement>("""{"longitude1":359,"longitude2":1}"""), o => Engine.Geometry.Separation(o)),
    ("geometry", "midpoint", JsonSerializer.Deserialize<JsonElement>("""{"longitude1":359,"longitude2":1}"""), o => Engine.Geometry.Midpoint(o)),
    ("aspects", "between", JsonSerializer.Deserialize<JsonElement>("""{"positions":[{"id":"moving","longitude":350,"speed":1},{"id":"fixed","longitude":28,"speed":4}],"rule":{"angle":37.5,"maxOrb":1},"motionMode":"fixedSecond"}"""), o => Engine.Aspects.Between(o)),
    ("houses", "locate", JsonSerializer.Deserialize<JsonElement>("""{"longitude":359,"houseCusps":[0,30,60,90,120,150,180,210,240,270,300,330]}"""), o => Engine.Houses.Locate(o)),
    ("houses", "inspect", JsonSerializer.Deserialize<JsonElement>("""{"birth":{"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"houseSystem":"placidus"},"houseNumbers":[2,8],"rulership":"modern"}"""), o => Engine.Houses.Inspect(o)),
    ("points", "inspect", JsonSerializer.Deserialize<JsonElement>("""{"birth":{"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"houseSystem":"placidus"},"pointIds":["sun","midheaven","H7"]}"""), o => Engine.Points.Inspect(o)),
    ("aspects", "inspect", JsonSerializer.Deserialize<JsonElement>("""{"birth":{"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"houseSystem":"placidus"},"pointIds":["sun","moon"],"aspectRules":[{"angle":37.5,"maxOrb":15}]}"""), o => Engine.Aspects.Inspect(o)),
};
var groupedQueryExpected = groupedQueryCalls.Select(c => {
    var request = JsonNode.Parse(c.Options.GetRawText())!.AsObject();
    request["operation"] = "query"; request["group"] = c.Group; request["action"] = c.Action;
    return Engine.Calculate(request).GetRawText();
}).ToArray();
for (var i=0; i<groupedQueryCalls.Length; i++)
    if (groupedQueryCalls[i].Invoke(groupedQueryCalls[i].Options).GetRawText() != groupedQueryExpected[i])
        throw new Exception("Grouped query differs from direct core call");
foreach (var raw in new[] { "null", "[]", "1", "\"bad\"", "{\"operation\":\"natal\"}", "{\"group\":\"houses\"}", "{\"action\":\"inspect\"}" })
{
    try { Engine.Geometry.Normalize(JsonSerializer.Deserialize<JsonElement>(raw)); throw new Exception("Invalid query wrapper input accepted"); }
    catch (ArgumentException) { }
}
Parallel.For(0,20,i => { var c=groupedQueryCalls[i%groupedQueryCalls.Length]; if (c.Invoke(c.Options).GetRawText()!=groupedQueryExpected[i%groupedQueryCalls.Length]) throw new Exception("Concurrent grouped query differs"); });
Console.WriteLine(".NET installed package: eight grouped query methods, invalid routing/options and20 concurrent calls OK");
