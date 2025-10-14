# Changelog
## 0.5.3
- Reworked the randomization code, it should now function as intended
- Added mutually exclusive command line switches:
  - `-E`: Enables all songs and exits
  - `-D`: Disables all songs and exits
  - `-R`: Resets the played status of all songs and exits
- Some minor changes
## 0.5.2
- Fixed a bug, where songs containing commas in their filenames
  had caused a startup crash, and `NaN` was displayed on the web UI.
- Made the program display the generated file's directory if it is
  started with the `DEBUG` environment variable set to `true`
- Modified default breaks to play music in to include the 1st break.
## 0.5.1
- Added `gzip` encoding
- Made `gzip` encoding the default instead of `none`
- Modified the style of `CHANGELOG.md` and `ENVVARS.md`
- Moved `embedded_files.rs`'s contents into `generated.rs` and removed it
## 0.5.0
- Reworked the build script
- Removed feature `use-encoding`. Use `ENCODING` optional compile-time
  environment variable instead.
- Removed feature `no-logging`. Use `LOGGING` optional runtime
  environment variable instead.
- Removed feature `debug-access`. Use `DEBUG` optional runtime
  environment variable instead.
- Removed feature `only-local`. Use the first command line argument
  to set the IP address to host on instead.
- Changed the first command line argument's function to not only set the port
  but also the whole IP address to host on
## 0.4.1
- Moved hard-coded target loudness value to an optional
  compile-time environment variable of name `TARGET_VOLUME`
- Modified `build.rs` to be able to generate a rust file to store
  pre-evaluated constants and statics. These are imported into the `generated` module
- Added `ENVVARS.md` to track expected environment variables
- Made song playing auto-detect the default device's best configuration and use it.
## 0.4.0
- Internal compile-time warnings were addressed
- Added volume normalization using the RMS (root-mean-square) method
- Added a featured called `no-logging`. This disables the logging functionality
  to write to the log file.
- Made default configs one minute late to prevent playing early, and
also disabled 6th and 7th breaks by default
## 0.3.7
- Added logging into a hard-coded path (`./latest.log`)
- Finished some previously unfinished work
- Realized `build.rs` was already ran at the intended time
## 0.3.6
- Hotfix for improving song randomization
- Deprecated some functions
## 0.3.5
- Made so that the playback functionality doesn't decode on-the-fly but instead
ahead-of-time, and also regard the output device's parameters
- Removed hard-coded song padding
- Made playback start announcing also state the time of day
- Removed some duplicated code
## 0.3.4
- Tweaked the web UI's design
- Added a new tab called 'Scheduled Events'. _It is currently in development,
so it is of no use for now._
## 0.3.3
- The server's UTC offset value is now presented to the web UI
- Typo corrected in `CHANGELOG.md` _(3.0.1 -> 0.3.1, 3.0.2 -> 0.3.2)_
- Minor web UI style changes
## 0.3.2
- Selecting break times now actually changes the server's state without any errors
- Autoplay functionality fixed to accommodate seconds (`3.0.1` broke it)
- Internal refactorings
## 0.3.1
- All seconds of the day can now be selected at break time selection
- Added a built in `config.musiq` generator
- Changed the format of `config.musiq`
- Internal refactorings
- Some bugfixes
- The server now sends second information too at `/data/breaks.csv`
- `build.rs` now runs every time (temporal solution)
## 0.3.0
- Some bugfixes
- Added a feature whether to use a local-to-machine or a local-on-network address
## 0.2.4
- Moved HTTP content encoding behind a feature called `use-encoding`
- Fixed a bug on Linux machines, where the songs' directory cannot have been resolved
- Fixed a bug where the web UI was only accessible from the same machine
as it was hosted on
- Small internal changes
## 0.2.3
- Fix for Linux systems not being able to open the songs' directory
- Added the automatic creation of a song directory if there was none detected
## 0.2.2
- Added some quality-of-life changes on the first run of the program,
such as automatically continuing with a pre-defined `config.musiq` file
or not updating the database from the disk if the file is not present
- Added a cargo feature to control whether some debug features are accessible
## 0.2.1
- Added this changelog
- Added a build script, which compresses the website's static files
- To improve network times, some `GET` requests are now replied to with Brotli encoding
- Added a command line argument to select the port to host on