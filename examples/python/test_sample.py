import json
import copy
import unittest
from concurrent.futures import ThreadPoolExecutor
from sevenmlabs_astrology import calculate, calculate_json, EngineError, geometry, aspects, houses, points

REQUEST = {"operation": "chart", "positions": [{"id": "moon", "longitude": 359, "speed": 13}, {"id": "sun", "longitude": 1, "speed": 1}]}

class PackageTests(unittest.TestCase):
    def test_geometry(self):
        result = calculate(REQUEST)
        self.assertEqual(result["data"]["aspects"][0]["orb"], 2)
        self.assertTrue(result["data"]["aspects"][0]["applying"])
        self.assertEqual(result["data"]["midpoints"][0]["longitude"], 0)

    def test_natal_and_concurrency(self):
        request = {"operation": "natal", "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0}, "location": {"latitude": 10.8231, "longitude": 106.6297}}
        requests = [request, {**request, "houseSystem": "wholeSign", "location": {"latitude": -33.86, "longitude": 151.21}}]
        expected = list(map(calculate, requests))
        self.assertEqual(len(expected[0]["data"]["placements"]), 10)
        self.assertEqual(len(expected[0]["data"]["houses"]), 12)
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, result in enumerate(pool.map(lambda i: calculate(requests[i % 2]), range(1000))):
                self.assertEqual(result, expected[i % 2])

    def test_domain_payloads_and_concurrency(self):
        request = {"operation": "natalDomains", "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0}, "location": {"latitude": 10.8231, "longitude": 106.6297}}
        requests = [request, {**request, "domains": ["love"], "houseSystem": "wholeSign", "rulership": "modern"}]
        expected = list(map(calculate, requests))
        self.assertEqual(len(expected[0]["data"]["domains"]), 10)
        self.assertEqual(len(expected[0]["data"]["context"]["relations"]), 325)
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, result in enumerate(pool.map(lambda i: calculate(requests[i % 2]), range(20))):
                self.assertEqual(result, expected[i % 2])

    def test_errors(self):
        with self.assertRaises(EngineError) as error:
            calculate({**REQUEST, "operation": "natal"})
        self.assertEqual(error.exception.code, "INVALID_INPUT")
        self.assertEqual(json.loads(calculate_json("{"))["errors"][0]["code"], "INVALID_INPUT")
        with self.assertRaises(ValueError):
            calculate_json("{}\0")

    def test_custom_profiles_and_concurrency(self):
        request = {"operation": "natalDomains", "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
            "location": {"latitude": 10.8231, "longitude": 106.6297}, "domains": [],
            "customProfiles": [{"id": "personalGrowth", "houses": [1, 9], "bodies": ["sun", "moon", "mercury"],
                "angles": ["ascendant"], "sections": [{"id": "study", "houses": [3, 9], "bodies": ["mercury"]}]}]}
        other = {**request, "houseSystem": "wholeSign", "rulership": "modern"}
        requests = [request, other]
        expected = list(map(calculate, requests))
        self.assertEqual(expected[0]["data"]["domains"], {})
        profile = expected[0]["data"]["customDomains"]["personalGrowth"]
        self.assertEqual(profile["origin"], "custom")
        self.assertEqual(profile["report"]["sections"][0]["id"], "study")
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, result in enumerate(pool.map(lambda i: calculate(requests[i % 2]), range(20))):
                self.assertEqual(result, expected[i % 2])

    def test_repeated_and_concurrent_calls(self):
        expected = calculate(REQUEST)
        with ThreadPoolExecutor(max_workers=8) as pool:
            for result in pool.map(lambda _: calculate(REQUEST), range(1000)):
                self.assertEqual(result, expected)

    def test_couple_and_concurrency(self):
        request = {"operation": "couple", "personA": {
            "utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0},
            "location": {"latitude": 10.8231, "longitude": 106.6297}, "houseSystem": "placidus"
        }, "personB": {
            "utc": {"year": 1998, "month": 6, "day": 15, "hour": 6, "minute": 30},
            "location": {"latitude": 21.0278, "longitude": 105.8342}, "houseSystem": "wholeSign"
        }}
        data = calculate(request)["data"]
        self.assertEqual(data["chartKind"], "coupleSynastry")
        self.assertEqual(data["subjectCount"], 2)
        self.assertEqual(len(data["domains"]), 6)
        self.assertEqual(sum(len(view["report"]["sections"]) for view in data["domains"].values()), 18)
        self.assertEqual(data["subjects"]["A"]["calculation"]["houseSystem"], "placidus")
        self.assertEqual(data["subjects"]["B"]["calculation"]["houseSystem"], "wholeSign")
        cross = data["context"]
        self.assertEqual(len(cross["points"]), 52)
        self.assertEqual(len({point["id"] for point in cross["points"]}), 52)
        self.assertTrue(all(point["id"] == f'{point["chartId"]}:{point["localId"]}' for point in cross["points"]))
        self.assertEqual(len(cross["relations"]), 676)
        self.assertTrue(all(relation["point1"].startswith("A:") and relation["point2"].startswith("B:") for relation in cross["relations"]))
        self.assertTrue(cross["aspects"])
        self.assertTrue(all(aspect["applying"] is None for aspect in cross["aspects"]))
        self.assertEqual(len(cross["overlaysAtoB"]), 10)
        self.assertEqual(len(cross["overlaysBtoA"]), 10)
        self.assertTrue(all(overlay["pointId"].startswith("A:") and overlay["targetHouseId"] == f'B:H{overlay["targetHouseNumber"]}' for overlay in cross["overlaysAtoB"]))
        self.assertTrue(all(overlay["pointId"].startswith("B:") and overlay["targetHouseId"] == f'A:H{overlay["targetHouseNumber"]}' for overlay in cross["overlaysBtoA"]))
        self.assertEqual(len(cross["houseRulerRelations"]), 144)
        disabled = calculate({**request, "domains": ["communication"], "aspectRules": []})["data"]["context"]
        self.assertEqual(disabled["aspects"], [])
        self.assertEqual(len(disabled["relations"]), 676)
        with self.assertRaises(EngineError) as error:
            calculate({**request, "personB": {**request["personB"], "utc": {"year": 1998, "month": 2, "day": 30, "hour": 6, "minute": 30}}})
        self.assertEqual(error.exception.code, "INVALID_INPUT")
        requests = [{**request, "domains": ["communication"]}, {
            **request, "personA": {**request["personA"], "houseSystem": "wholeSign"},
            "personB": {**request["personB"], "houseSystem": "placidus"},
            "domains": [], "rulership": "modern", "aspectRules": [{"angle": 0, "maxOrb": 3}, {"angle": 90, "maxOrb": 3}],
            "customProfiles": [{"id": "sharedLearning", "houses": [3, 9], "bodies": ["mercury", "jupiter"],
                "sections": [{"id": "ideas", "houses": [3], "bodies": ["mercury"]}]}]
        }]
        expected = list(map(calculate, requests))
        self.assertEqual(expected[1]["data"]["customDomains"]["sharedLearning"]["report"]["sections"][0]["id"], "ideas")
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, result in enumerate(pool.map(lambda i: calculate(requests[i % 2]), range(20))):
                self.assertEqual(result, expected[i % 2])

    def test_composite_and_combined_concurrency(self):
        request = json.loads('{"operation": "composite", "personA": {"utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0}, "location": {"latitude": 10.8231, "longitude": 106.6297}, "houseSystem": "placidus"}, "personB": {"utc": {"year": 1998, "month": 6, "day": 15, "hour": 6, "minute": 30}, "location": {"latitude": 21.0278, "longitude": 105.8342}, "houseSystem": "placidus"}, "rulership": "traditional", "aspectPreset": "extended", "houseMethod": "midpoint", "antipodalPolicy": "error"}')
        result = calculate(request)
        c = result["data"]["composite"]
        self.assertEqual(result["data"]["chartCount"], 3)
        self.assertEqual(c["chartKind"], "midpointComposite")
        self.assertEqual(len(c["domains"]), 10)
        self.assertEqual(sum(len(v["report"]["sections"]) for v in c["domains"].values()), 30)
        self.assertEqual(len(c["context"]["points"]), 26)
        self.assertEqual(len(c["context"]["relations"]), 325)
        self.assertEqual(len(c["context"]["houseRulerRelations"]), 66)
        self.assertNotIn("utc", c["chart"])
        self.assertNotIn("location", c["chart"])
        self.assertNotIn("julianDayTt", result["calculation"])
        self.assertTrue(all(p["speed"] is None and p["isRetrograde"] is None for p in c["chart"]["placements"]))
        self.assertTrue(all(a["applying"] is None for a in c["context"]["aspects"]))
        self.assertTrue(all(b["motion"] == "notApplicable" for b in c["context"]["advanced"]["bodyStates"]))
        combined = {"operation": "couple", "personA": request["personA"], "personB": request["personB"],
                    "domains": ["communication"], "aspectPreset": request["aspectPreset"], "rulership": request["rulership"],
                    "composite": {"houseMethod": request["houseMethod"], "antipodalPolicy": request["antipodalPolicy"]}}
        combined_result = calculate(combined)
        self.assertEqual(combined_result["data"]["composite"], c)
        self.assertEqual(combined_result["calculation"]["composite"], result["calculation"])
        invalid = {**combined, "composite": {"domains": ["attraction"]}}
        with self.assertRaises(EngineError):
            calculate(invalid)
        disabled = copy.deepcopy(request)
        disabled.pop("aspectPreset")
        disabled.update({"houseMethod": "wholeSignFromMidpointAscendant", "aspectRules": [], "rulership": "modern", "domains": [],
                         "customProfiles": [{"id": "sharedLearning", "houses": [3, 9], "bodies": ["mercury", "jupiter"]}]})
        requests = [combined, disabled]
        expected = list(map(calculate, requests))
        self.assertEqual(expected[1]["data"]["composite"]["context"]["aspects"], [])
        self.assertEqual(len(expected[1]["data"]["composite"]["context"]["relations"]), 325)
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, response in enumerate(pool.map(lambda i: calculate(requests[i % 2]), range(20))):
                self.assertEqual(response, expected[i % 2])


    def test_forecast_and_concurrent_calls(self):
        request = json.loads('{"operation": "forecast", "birth": {"utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0}, "location": {"latitude": 10.8231, "longitude": 106.6297}, "houseSystem": "placidus"}, "period": {"kind": "day", "year": 2026, "month": 3, "day": 3, "utcOffsetMinutes": 420}}')
        expected = calculate(request)
        d = expected["data"]
        self.assertEqual(d["chartKind"], "individualForecast")
        self.assertEqual(len(d["snapshot"]["relations"]), 260)
        self.assertEqual(len(d["domains"]), 10)
        self.assertEqual(sum(len(v["sections"]) for v in d["domains"].values()), 30)
        self.assertTrue(any(e["type"] == "lunarEclipse" for e in d["events"]))
        self.assertTrue(any(e["type"] == "natalTransit" for e in d["events"]))
        self.assertEqual(d["overview"]["eventCount"], len(d["events"]))
        self.assertTrue(all(v["messageContext"]["narrative"] is None for v in d["domains"].values()))
        points = {"N:"+p["id"] for p in d["subject"]["context"]["points"]}
        for event in d["events"]:
            self.assertTrue(all(id in points for id in event["personalImpact"]["affectedNatalPointIds"]))
            self.assertTrue(all(o["natalHouseId"] in points and o["natalRulerPointId"] in points for o in event["personalImpact"]["houseOverlays"]))
        quiet = copy.deepcopy(request)
        quiet.update({"eventTypes": [], "includeNatalTransits": False, "aspectRules": [], "domains": [], "bodies": ["mercury"], "rulership": "modern",
                      "customProfiles": [{"id": "quietFocus", "houses": [3,9], "bodies": ["mercury","jupiter"]}]})
        quiet_result = calculate(quiet)
        self.assertEqual(quiet_result["data"]["events"], [])
        self.assertEqual(len(quiet_result["data"]["snapshot"]["relations"]), 26)
        with self.assertRaises(EngineError):
            calculate({**request, "period": {"kind": "day", "year": 2026, "month": 2, "day": 30}})
        requests = [request, quiet]
        results = [expected, quiet_result]
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, result in enumerate(pool.map(lambda i: calculate(requests[i%2]), range(20))):
                self.assertEqual(result, results[i%2])

    def test_grouped_queries_and_concurrent_calls(self):
        birth = {"utc": {"year": 2000, "month": 1, "day": 1, "hour": 12, "minute": 0}, "location": {"latitude": 10.8231, "longitude": 106.6297}, "houseSystem": "placidus"}
        calls = [
            ("geometry", "normalize", {"longitude": -10}),
            ("geometry", "separation", {"longitude1": 359, "longitude2": 1}),
            ("geometry", "midpoint", {"longitude1": 359, "longitude2": 1}),
            ("aspects", "between", {"positions": [{"id": "moving", "longitude": 350, "speed": 1}, {"id": "fixed", "longitude": 28, "speed": 4}], "rule": {"angle": 37.5, "maxOrb": 1}, "motionMode": "fixedSecond"}),
            ("houses", "locate", {"longitude": 359, "houseCusps": list(range(0, 360, 30))}),
            ("houses", "inspect", {"birth": birth, "houseNumbers": [2, 8], "rulership": "modern"}),
            ("points", "inspect", {"birth": birth, "pointIds": ["sun", "midheaven", "H7"], "aspectRules": []}),
            ("aspects", "inspect", {"birth": birth, "pointIds": ["sun", "moon"], "aspectRules": [{"angle": 37.5, "maxOrb": 15}]}),
        ]
        groups = {"geometry": geometry, "aspects": aspects, "houses": houses, "points": points}
        expected = [calculate({"operation": "query", "group": g, "action": a, **o}) for g, a, o in calls]
        def invoke(i):
            g, a, o = calls[i % len(calls)]
            return getattr(groups[g], a)(o)
        for i in range(len(calls)):
            self.assertEqual(invoke(i), expected[i])
        for options in (None, [], 1, "bad", {"operation": "natal"}, {"group": "houses"}, {"action": "inspect"}):
            with self.assertRaises(TypeError): geometry.normalize(options)
        with self.assertRaises(EngineError): houses.inspect({"birth": birth, "houseNumbers": [13]})
        with ThreadPoolExecutor(max_workers=8) as pool:
            for i, result in enumerate(pool.map(invoke, range(20))):
                self.assertEqual(result, expected[i % len(calls)])

if __name__ == "__main__":
    unittest.main()
