# Refresh Deploy Hook Redgreen Progress

Task initialized for a refresh deployment that also initializes Hook red-green runtime on the server.

2026-05-19T07:00:00Z Step 1 done: remote services active, green unit absent, proxy include absent, baseline 37/37 before deployment.

2026-05-19T07:03:00Z Step 2 done: frontend static export succeeded; backend cross-compiled locally to x86_64 Linux with hash 17c627c39fd65a640e7b4b904cd073fc7d8d87cf40eb34227640bc4b380dd83e.

2026-05-19T07:06:00Z Step 3 done: initialized green systemd service and config, patched Nginx to proxy include, kept include on blue slot, and installed matching binaries to blue and green.

2026-05-19T08:44:00Z Step 4 done: stopped both slots, deleted 5 Redis hook keys, ran migration refresh, started green, switched Nginx to 5557, observed a brief public 502 during cutover, then public health recovered; blue was started as same-version standby.

2026-05-19T08:45:00Z Step 5 done: verified public health, site-info, auth-config, admin sign-in, direct origin block, Nginx proxy include to green, and both blue/green services active. Updated the downloaded deployment manual to reflect that red-green runtime is now initialized and traffic points to green.
