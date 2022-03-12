# Changelog

## Unreleased

## 1.2.0 - 2022-03-11

- Add content type for `.js` files

## 1.1.0 - 2022-02-28

- Filter out files/directories that start with a `.` from the file watcher. This prevents serena from reloading the page when, for example, a file in the `.git` directory changes.

## 1.0.0 - 2022-02-27

- serena serves static files from a directory on disk
- `-p`/`--port` option to specify the port (default is `3000`)
- `--open` option to open the default browser to the website.
- Automatic browser refresh is the default behavior as that's what I personally find most useful. `--no-auto-refresh` disables auto-refreshing if you don't like that.
