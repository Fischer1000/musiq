# Things to be done
- Adding nameday announcements
- Making the webpage use HTTPS
- A "select all" option in the web UI _(made obsolete
  by command line switches added in version 0.5.3)_
- Adding the ability to add other scheduled events
- Send the server's current time to the web UI
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
# Bugs to be fixed
- ~~Song randomization seems off~~