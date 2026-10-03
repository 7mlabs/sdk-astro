//! Local Swiss Ephemeris, explicitly selecting the built-in Moshier model.
use std::ffi::{c_char, c_double, c_int, CStr};
use std::sync::{Mutex, MutexGuard};

static PROVIDER: Mutex<()> = Mutex::new(());
const FLAGS: c_int = 4 | 256; // SEFLG_MOSEPH | SEFLG_SPEED

extern "C" {
    fn swe_utc_to_jd(
        y: c_int,
        m: c_int,
        d: c_int,
        h: c_int,
        min: c_int,
        sec: c_double,
        greg: c_int,
        jd: *mut c_double,
        err: *mut c_char,
    ) -> c_int;
    fn swe_calc_ut(
        jd: c_double,
        body: c_int,
        flags: c_int,
        pos: *mut c_double,
        err: *mut c_char,
    ) -> c_int;
    fn swe_houses_ex(
        jd: c_double,
        flags: c_int,
        lat: c_double,
        lon: c_double,
        system: c_int,
        cusps: *mut c_double,
        angles: *mut c_double,
    ) -> c_int;
    fn swe_jdut1_to_utc(
        jd: c_double,
        greg: c_int,
        y: *mut c_int,
        m: *mut c_int,
        d: *mut c_int,
        h: *mut c_int,
        min: *mut c_int,
        sec: *mut c_double,
    );
    fn swe_sol_eclipse_when_glob(
        start: c_double,
        flags: c_int,
        eclipse_types: c_int,
        times: *mut c_double,
        backward: c_int,
        err: *mut c_char,
    ) -> c_int;
    fn swe_lun_eclipse_when(
        start: c_double,
        flags: c_int,
        eclipse_types: c_int,
        times: *mut c_double,
        backward: c_int,
        err: *mut c_char,
    ) -> c_int;
    fn swe_set_tid_acc(value: c_double);
    fn swe_close();
}

pub struct Sky {
    pub jd_tt: f64,
    pub jd_ut1: f64,
    /// Longitude, latitude, distance AU, and their rates per day, Sun through Pluto.
    pub bodies: Vec<[f64; 6]>,
    pub houses: Vec<f64>,
    pub ascendant: f64,
    pub midheaven: f64,
}

fn message(buffer: &[c_char; 256]) -> String {
    // Swiss guarantees a NUL-terminated diagnostic of at most AS_MAXCH bytes.
    unsafe {
        CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}

/// Caller validates Gregorian civil UTC and coordinates before entering this boundary.
pub fn natal(utc: [i32; 5], second: f64, lat: f64, lon: f64, system: u8) -> Result<Sky, String> {
    let _guard = PROVIDER.lock().map_err(|_| "Provider lock is poisoned")?;
    // All stateful calls, including time conversion and cleanup, share the lock.
    unsafe {
        swe_close();
        swe_set_tid_acc(-25.580);
    }
    struct Cleanup;
    impl Drop for Cleanup {
        fn drop(&mut self) {
            unsafe {
                swe_close();
            }
        }
    }
    let _cleanup = Cleanup;
    let mut jd = [0.; 2];
    let mut err = [0 as c_char; 256];
    let status = unsafe {
        swe_utc_to_jd(
            utc[0],
            utc[1],
            utc[2],
            utc[3],
            utc[4],
            second,
            1,
            jd.as_mut_ptr(),
            err.as_mut_ptr(),
        )
    };
    if status < 0 {
        return Err(message(&err));
    }
    let mut bodies = Vec::with_capacity(10);
    for body in 0..10 {
        let mut pos = [0.; 6];
        let flags = unsafe { swe_calc_ut(jd[1], body, FLAGS, pos.as_mut_ptr(), err.as_mut_ptr()) };
        if flags < 0 {
            return Err(message(&err));
        }
        if flags & 7 != 4 || pos.iter().any(|n| !n.is_finite()) {
            return Err("Provider did not return finite Moshier coordinates".into());
        }
        bodies.push(pos);
    }
    let mut cusps = [0.; 13];
    let mut angles = [0.; 10];
    let status = unsafe {
        swe_houses_ex(
            jd[1],
            0,
            lat,
            lon,
            system.into(),
            cusps.as_mut_ptr(),
            angles.as_mut_ptr(),
        )
    };
    // Swiss may return Porphyry as a fallback on ERR. Never expose that silently.
    if status < 0 {
        return Err(
            "Requested house system cannot be calculated at this latitude; try wholeSign".into(),
        );
    }
    if cusps[1..]
        .iter()
        .chain(angles[..2].iter())
        .any(|n| !n.is_finite())
    {
        return Err("Provider returned non-finite houses or angles".into());
    }
    Ok(Sky {
        jd_tt: jd[0],
        jd_ut1: jd[1],
        bodies,
        houses: cusps[1..].to_vec(),
        ascendant: angles[0],
        midheaven: angles[1],
    })
}

/// One bounded scan owns the stateful Swiss context. No houses are calculated.
/// The original natal entry point retains its independent initialization path.
pub struct EphemerisSession {
    _guard: MutexGuard<'static, ()>,
}

#[derive(Clone, Debug)]
pub struct CivilUtc {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: f64,
}

impl EphemerisSession {
    pub fn new() -> Result<Self, String> {
        let guard = PROVIDER.lock().map_err(|_| "Provider lock is poisoned")?;
        unsafe {
            swe_close();
            swe_set_tid_acc(-25.580);
        }
        Ok(Self { _guard: guard })
    }

    /// UTC is validated by the engine; returned order is TT, UT1.
    pub fn julian_days(&mut self, utc: [i32; 5], second: f64) -> Result<[f64; 2], String> {
        let mut jd = [0.; 2];
        let mut err = [0 as c_char; 256];
        let status = unsafe {
            swe_utc_to_jd(
                utc[0],
                utc[1],
                utc[2],
                utc[3],
                utc[4],
                second,
                1,
                jd.as_mut_ptr(),
                err.as_mut_ptr(),
            )
        };
        if status < 0 {
            return Err(message(&err));
        }
        if jd.iter().any(|v| !v.is_finite()) {
            return Err("Provider returned non-finite time coordinates".into());
        }
        Ok(jd)
    }

    pub fn utc_from_ut1(&mut self, jd: f64) -> Result<CivilUtc, String> {
        if !jd.is_finite() {
            return Err("Julian day must be finite".into());
        }
        let (mut year, mut month, mut day, mut hour, mut minute, mut second) = (0, 0, 0, 0, 0, 0.);
        unsafe {
            swe_jdut1_to_utc(
                jd,
                1,
                &mut year,
                &mut month,
                &mut day,
                &mut hour,
                &mut minute,
                &mut second,
            );
        }
        if !second.is_finite() {
            return Err("Provider returned non-finite UTC".into());
        }
        Ok(CivilUtc {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }

    /// Search a true global eclipse maximum, with the same forced Moshier model.
    /// Times are UT1; the event-family-specific contact mapping is public Swiss API.
    pub fn next_eclipse(
        &mut self,
        start_ut1: f64,
        solar: bool,
    ) -> Result<(i32, [f64; 10]), String> {
        if !start_ut1.is_finite() {
            return Err("Eclipse search requires finite UT1 JD".into());
        }
        let mut times = [0.; 10];
        let mut err = [0 as c_char; 256];
        let flags = unsafe {
            if solar {
                swe_sol_eclipse_when_glob(start_ut1, 4, 0, times.as_mut_ptr(), 0, err.as_mut_ptr())
            } else {
                swe_lun_eclipse_when(start_ut1, 4, 0, times.as_mut_ptr(), 0, err.as_mut_ptr())
            }
        };
        if flags < 0 {
            return Err(message(&err));
        }
        if flags == 0
            || !times[0].is_finite()
            || times[0] <= start_ut1
            || times.iter().any(|v| !v.is_finite())
        {
            return Err(
                "Provider eclipse search did not return a finite, increasing global maximum".into(),
            );
        }
        Ok((flags, times))
    }

    /// Body indices are Swiss Sun=0 through Pluto=9. Real rates are degrees/day.
    pub fn positions(&mut self, jd_ut1: f64, indices: &[usize]) -> Result<Vec<[f64; 6]>, String> {
        if !jd_ut1.is_finite() || indices.iter().any(|i| *i >= 10) {
            return Err("Require finite UT1 JD and body indices 0..9".into());
        }
        indices
            .iter()
            .map(|body| {
                let mut pos = [0.; 6];
                let mut err = [0 as c_char; 256];
                let flags = unsafe {
                    swe_calc_ut(
                        jd_ut1,
                        *body as c_int,
                        FLAGS,
                        pos.as_mut_ptr(),
                        err.as_mut_ptr(),
                    )
                };
                if flags < 0 {
                    return Err(message(&err));
                }
                if flags & 7 != 4 || pos.iter().any(|v| !v.is_finite()) {
                    return Err("Provider did not return finite Moshier coordinates".into());
                }
                Ok(pos)
            })
            .collect()
    }
}

impl Drop for EphemerisSession {
    fn drop(&mut self) {
        unsafe {
            swe_close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanning_positions_equal_natal_positions_without_house_calculation() {
        let sky = natal([2000, 1, 1, 12, 0], 0., 10.8231, 106.6297, b'P').unwrap();
        let mut session = EphemerisSession::new().unwrap();
        let days = session.julian_days([2000, 1, 1, 12, 0], 0.).unwrap();
        assert_eq!(days, [sky.jd_tt, sky.jd_ut1]);
        assert_eq!(
            session
                .positions(days[1], &(0..10).collect::<Vec<_>>())
                .unwrap(),
            sky.bodies
        );
        let utc = session.utc_from_ut1(days[1]).unwrap();
        assert_eq!(
            (utc.year, utc.month, utc.day, utc.hour, utc.minute),
            (2000, 1, 1, 12, 0)
        );
        assert!(utc.second.abs() < 0.001);
    }
}
