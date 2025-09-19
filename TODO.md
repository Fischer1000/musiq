# Things to be done
- Adding nameday announcements
- Having authorization on the webpage in the form of an API key
- A "select all" option in the web UI
- Add status information logging to `stdout` _(WIP)_
- Adding the ability to add other scheduled events
- Send the server's current time to the web UI
- Remove features and instead use environment variables or command line arguments
- Have a translation of the web UI
- Finishing the event scheduling API
- Normalizing the output volume
- Refactoring warning-emitting code
- Removing duplicated code from `webserver.rs` at line 467 in the `for` loop
# Bugs to be fixed
- Output volume is inconsistent across songs
- Scheduled plays _may_ fail to fire