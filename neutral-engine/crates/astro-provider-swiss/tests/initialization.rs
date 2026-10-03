use astro_provider_swiss::{natal, EphemerisSession, Sky};

fn same_sky(actual: &Sky, expected: &Sky) {
    assert_eq!(actual.jd_tt, expected.jd_tt);
    assert_eq!(actual.jd_ut1, expected.jd_ut1);
    assert_eq!(actual.bodies, expected.bodies);
    assert_eq!(actual.houses, expected.houses);
    assert_eq!(actual.ascendant, expected.ascendant);
    assert_eq!(actual.midheaven, expected.midheaven);
}

#[test]
fn cold_historical_natal_matches_repeated_and_session_interleaved_calls() {
    // This integration test has its own executable and is its only test: the
    // first call must exercise Swiss's cold initialization, not a warmed cache.
    let historical = natal([1800, 1, 1, 0, 0], 0., 10.8231, 106.6297, b'P').unwrap();
    for utc in [
        [2000, 1, 1, 12, 0],
        [2399, 12, 31, 23, 59],
        [1950, 6, 15, 6, 30],
    ] {
        natal(utc, 0., -33.8688, 151.2093, b'W').unwrap();
        let repeated = natal([1800, 1, 1, 0, 0], 0., 10.8231, 106.6297, b'P').unwrap();
        same_sky(&repeated, &historical);

        let mut session = EphemerisSession::new().unwrap();
        let days = session.julian_days([1800, 1, 1, 0, 0], 0.).unwrap();
        assert_eq!(days, [historical.jd_tt, historical.jd_ut1]);
        assert_eq!(
            session
                .positions(days[1], &(0..10).collect::<Vec<_>>())
                .unwrap(),
            historical.bodies
        );
        drop(session);
        same_sky(
            &natal([1800, 1, 1, 0, 0], 0., 10.8231, 106.6297, b'P').unwrap(),
            &historical,
        );
    }
}
