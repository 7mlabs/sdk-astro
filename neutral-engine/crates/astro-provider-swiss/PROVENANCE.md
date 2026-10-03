# Swiss Ephemeris provider

C source and headers from https://github.com/aloistr/swisseph,
commit `aacf962d19d79f8bc921dbcdaacf306d85be1917`, provider version 2.10.03.
Source fetched 2026-10-02. `vendor/LICENSE` and `vendor/LICENSE.TXT` retain
upstream licensing terms (AGPL or commercial Professional License).
Local patch: `sweph.c:swi_fopen` returns NULL when the build flag
`ASTRO_MOSHIER_ONLY` is set. This disables external ephemeris and time-table
files, so the selected Moshier model and compiled leap-second/Delta T tables
are reproducible regardless of working directory or SE_EPHE_PATH.
No ephemeris data files are required by the selected Moshier mode.
Local integration does not settle licensing of the entire derived distribution.
Public release requires the license decision described in docs/distribution.md.
