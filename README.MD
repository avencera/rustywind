# RustyWind [![Mean Bean CI](https://github.com/avencera/rustywind/workflows/Mean%20Bean%20CI/badge.svg)](https://github.com/avencera/rustywind/actions?query=workflow%3A%22Mean+Bean+CI%22) [![npm version](https://badge.fury.io/js/rustywind.svg)](https://badge.fury.io/js/rustywind)

## Install

Available via npm

`yarn global add rustywind`

or

`npm install -g rustywind`

or

Install from homebrew (mac and linux):

`brew install avencera/taps/rustywind`

or

Install from a github release:

`curl -LSfs https://avencera.github.io/rustywind/install.sh | sh -s -- --git avencera/rustywind`

or

Download a release directly from github: [github.com/avencera/rustywind/releases](https://github.com/avencera/rustywind/releases)

or

You can use the dockerized version

`docker run --rm -v $PWD:/app avencera/rustywind:latest <rustywind arguments>`

## Usage

Run rustywind with a path to output updated file contents to the terminal:

- `rustywind .`

If you want to reorganize all classes in place, and change the files run with the `--write` flag

- `rustywind --write .`

Run rustywind with a path and the `--dry-run` to get a list of files that will be changed:

- `rustywind --dry-run .`

Run rustywind on your STDIN:

- `echo "<FILE CONTENTS>" | rustywind --stdin`

Run in CI, exit with error if unsorted classes are found:

- `rustywind --check-formatted .`

Run RustyWind with a custom sorter. The `config_file.json` should have a top level entry of `sortOrder`
which is an array with the classes listed in the order you want them sorted.

- `rustywind --config-file config_file.json`

```shell
RustyWind 0.14.0
Praveen Perera <praveen@avencera.com>

USAGE:
    Run rustywind with a path to get a list of files that will be changed
      rustywind . --dry-run

    If you want to reorganize all classes in place, and change the files run with the `--write` flag
      rustywind --write .

    To print only the file names that would be changed run with the `--check-formatted` flag
      rustywind --check-formatted .

    If you want to run it on your STDIN, you can do:
      echo "<FILE CONTENTS>" | rustywind --stdin

    rustywind [FLAGS] <PATH>

ARGS:
    <PATH>...    A file or directory to run on

OPTIONS:
   --allow-duplicates
       When set, RustyWind will not delete duplicated classes

   --check-formatted
       Checks if the files are already formatted, exits with 1 if not formatted

   --config-file <CONFIG_FILE>
       When set, RustyWind will use the config file to derive configurations. The config file
       current only supports json with one property sortOrder,e.g. { "sortOrder": ["class1",
       ...] }

   --custom-regex <CUSTOM_REGEX>
       Uses a custom regex instead of default one

   --dry-run
       Prints out the new file content with the sorted classes to the terminal

    -h, --help
       Print help information

   --ignored-files <IGNORED_FILES>
       When set, RustyWind will ignore this list of files

   --stdin
       Uses stdin instead of a file or folder

    -V, --version
       Print version information

   --write
            Changes the files in place with the reorganized classes

```

## What

Inspired by [Ryan Heybourn's](https://github.com/heybourn) [headwind](https://github.com/heybourn/headwind)
vscode plugin. This is a CLI tool that will look through your project and sort all [Tailwind CSS](https://tailwindcss.com) classes.

It will also delete any duplicate classes it finds.
