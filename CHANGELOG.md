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