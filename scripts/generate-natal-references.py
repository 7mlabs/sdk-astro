"""Explicit fixture generator; never invoked by package tests.

Requires development-only pyswisseph==2.10.3.2, skyfield==1.49 and the JPL
DE421 file at artifacts/de421.bsp. Review changed fixtures before committing.
No reference dependency or BSP is included in consumer packages.
"""
import hashlib
import json
from pathlib import Path
import swisseph as swe
from skyfield.api import load, load_file
from skyfield.framelib import ecliptic_frame

ROOT = Path(__file__).resolve().parents[1]
bsp = ROOT / 'artifacts/de421.bsp'
eph = load_file(str(bsp))
ts = load.timescale(builtin=True)
ids = ['sun','moon','mercury','venus','mars','jupiter','saturn','uranus','neptune','pluto']
targets = ['sun','moon','mercury','venus','mars barycenter','jupiter barycenter','saturn barycenter','uranus barycenter','neptune barycenter','pluto barycenter']
fixtures = []
for year,month,day,hour,lat,lon,system in [
    (1950,6,15,3,51.5074,-0.1278,'placidus'),
    (2000,1,1,12,10.8231,106.6297,'placidus'),
    (2020,2,29,18,-33.86,151.21,'placidus'),
    (2000,1,1,12,80,0,'wholeSign'),
]:
    swe.close()
    swe.set_tid_acc(-25.580)
    jd_tt, jd_ut1 = swe.utc_to_jd(year,month,day,hour,0,0,swe.GREG_CAL)
    coords = [swe.calc_ut(jd_ut1, i, swe.FLG_MOSEPH | swe.FLG_SPEED)[0] for i in range(10)]
    cusps, angles = swe.houses_ex(jd_ut1,lat,lon,b'P' if system=='placidus' else b'W')
    # Compare physical positions at the same TT epoch, independent of Delta T models.
    t = ts.tt_jd(jd_tt)
    sky = []
    for target in targets:
        latitude,longitude,distance = eph['earth'].at(t).observe(eph[target]).apparent().frame_latlon(ecliptic_frame)
        sky.append({'longitude':longitude.degrees,'latitude':latitude.degrees,'distanceAu':distance.au})
    fixtures.append({'name':f'{year}-{month}-{day} {system}',
        'request':{'operation':'natal','utc':{'year':year,'month':month,'day':day,'hour':hour,'minute':0},'location':{'latitude':lat,'longitude':lon},'houseSystem':system},
        'swiss':{'julianDayTt':jd_tt,'julianDayUt1':jd_ut1,'positions':[{'id':id,'longitude':p[0],'latitude':p[1],'distanceAu':p[2],'speed':p[3]} for id,p in zip(ids,coords)],'houseCusps':list(cusps),'ascendant':angles[0],'midheaven':angles[1]},
        'skyfield':sky})
result = {'references':{'swiss':{'binding':'pyswisseph 2.10.3.2','providerVersion':swe.version,'ephemeris':'Moshier'},'independent':{'binding':'Skyfield 1.49','ephemeris':'JPL DE421','bspSha256':hashlib.sha256(bsp.read_bytes()).hexdigest(),'epochPolicy':'same TT as Swiss reference','frame':'apparent geocentric ecliptic-of-date'}},
    'tolerances':{'swissAngleDegrees':0.00001,'swissSpeedDegreesPerDay':0.00001,'swissDistanceAu':0.000001,'julianDay':0.0000001,'independentAngleDegrees':0.01},'cases':fixtures}
(ROOT/'tests/conformance/natal-references.json').write_text(json.dumps(result,indent=2)+'\n')
print('Generated',len(fixtures),'reference cases')
