# 0.2.2
- Added some quality-of-life changes on the first run of the program,
such as automatically continuing with a pre-defined `config.musiq` file
or not updating the database from the disk if the file is not present.
- Added a cargo feature to control whether some debug features are accessible
# 0.2.1
- Added this changelog
- Added a build script, which compresses the website's static files
- To improve network times, some `GET` requests are now replied to with Brotli encoding
- Added a command line argument to select the port to host on