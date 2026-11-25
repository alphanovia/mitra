# M028 relaxed: dry run (not on main)

`home` has functions (`target`, `tick`) and holds a working part
(`Heater`) and an environment (`Sensor`). On main this is M028.

    mitra check home.mitra
    mitra explain home.mitra --no-test --store evidence.mitra

Rules in this branch: M028 is off; M029 says an outer part rests on an
inner part only through its ensures (its own pieces as any part).
