# IN PROGRESS
- Internal compile-time warnings were addressed
# 0.3.7
- Added logging into a hard-coded path (`./latest.log`)
- Finished some previously unfinished work
- Realized `build.rs` was already ran at the intended time
# 0.3.6
- Hotfix for improving song randomization
- Deprecated some functions
# 0.3.5
- Made so that the playback functionality doesn't decode on-the-fly but instead
ahead-of-time, and also regard the output device's parameters
- Removed hard-coded song padding
- Made playback start announcing also state the time of day
- Removed some duplicated code
# 0.3.4
- Tweaked the web UI's design
- Added a new tab called 'Scheduled Events'. _It is currently in development,
so it is of no use for now._
# 0.3.3
- The server's UTC offset value is now presented to the web UI
- Typo corrected in `CHANGELOG.md` _(3.0.1 -> 0.3.1, 3.0.2 -> 0.3.2)_
- Minor web UI style changes
# 0.3.2
- Selecting break times now actually changes the server's state without any errors
- Autoplay functionality fixed to accommodate seconds (`3.0.1` broke it)
- Internal refactorings
# 0.3.1
- All seconds of the day can now be selected at break time selection
- Added a built in `config.musiq` generator
- Changed the format of `config.musiq`
- Internal refactorings
- Some bugfixes
- The server now sends second information too at `/data/breaks.csv`
- `build.rs` now runs every time (temporal solution)
# 0.3.0
- Some bugfixes
- Added a feature whether to use a local-to-machine or a local-on-network address
# 0.2.4
- Moved HTTP content encoding behind a feature called `use-encoding`
- Fixed a bug on Linux machines, where the songs' directory cannot have been resolved
- Fixed a bug where the web UI was only accessible from the same machine
as it was hosted on
- Small internal changes
# 0.2.3
- Fix for Linux systems not being able to open the songs' directory
- Added the automatic creation of a song directory if there was none detected
# 0.2.2
- Added some quality-of-life changes on the first run of the program,
such as automatically continuing with a pre-defined `config.musiq` file
or not updating the database from the disk if the file is not present
- Added a cargo feature to control whether some debug features are accessible
# 0.2.1
- Added this changelog
- Added a build script, which compresses the website's static files
- To improve network times, some `GET` requests are now replied to with Brotli encoding
- Added a command line argument to select the port to host on