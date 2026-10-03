import json
from sevenmlabs_astrology import houses
options = json.loads('{"birth":{"utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297},"houseSystem":"placidus"},"houseNumbers":[2,8],"rulership":"modern"}')
print(json.dumps(houses.inspect(options)))
