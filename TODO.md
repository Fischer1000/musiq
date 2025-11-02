# Things to be done
- Adding nameday announcements
- Making the webpage use HTTPS
- A "select all" option in the web UI _(made obsolete
  by command line switches added in version 0.5.3)_
- Adding the ability to add other scheduled events
- Have a translation of the web UI
- Finishing the event scheduling API
- Add tests
- Add documentation
- Adding a `RAEDME.md` file
- Add a file (or link to a file), which stores the links
  to the approved songs and whose contents will be downloaded on each compilation
  - _Alternately query a link, which hosts an archive of the songs to be played_
  - Bake in the songs and unpack them to the `songs`
    directory on every run of the program
- Remove duplicated code from `lib.rs`
- Make command line switches combinable (eg. `-E -R` => `-ER`)
- Adding a file similar to `ENVVARS.md` with the purpose
  of tracking command line arguments
- Add a config for hosting _(address, port)_,
  and remove the mandatory command line argument
- Make the program automatically change timezones
- Making the main loop async _(probably needs the rethinking of the
  entire infrastructure)_
- Adding datetime to `time.rs` to handle the triggering of scheduled events, and
  moving time handling functionality from `events.rs`
- Adding feedback to web UI changes _(ex. "Operation successful")_
- Use `or_bad_request!()` macro more often in `webserver.rs`
- Make `webserver.rs` check whether a new event can be created instead of it
  deleting potentially conflicting events
- Add event-editing functionality to the web UI
- Make the MP3 playing functionality accept different sample rate files as well
  as stereo and mono sounds too, and support more devices
# Bugs to be fixed
- ~~Song randomization seems off~~