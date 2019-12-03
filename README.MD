# RustyWind [![Build Status Travis](https://travis-ci.org/avencera/rustywind.svg?branch=master)](https://travis-ci.org/avencera/rustywind) [![Build Status Appveyor](https://ci.appveyor.com/api/projects/status/at89lbs7n35wqy96?svg=true)](https://ci.appveyor.com/project/praveenperera/rustywind) [![npm version](https://badge.fury.io/js/rustywind.svg)](https://badge.fury.io/js/rustywind)


## Install

Available via npm

`yarn global add rustywind`

or

`npm install -g rustywind`

or 

Install from a github release:

`curl -LSfs https://avencera.github.io/rustywind/install.sh | sh -s -- --git avencera/rustywind`

or 

Download a release directly from github: [github.com/avencera/rustywind/releases](https://github.com/avencera/rustywind/releases)

## Usage

Run rustywind with a path to output updated file contents to the terminal:
  - `rustywind .`

If you want to reorganize all classes in place, and change the files run with the `--write` flag
  - `rustywind --write .`

Run rustywind with a path and the `--dry-run` to get a list of files that will be changed:
-  `rustywind --dry-run .`

For more options run help 
  - `rustywind --help`

## What

Inspired by [Ryan Heybourn's](https://github.com/heybourn) [headwind](https://github.com/heybourn/headwind)
vscode plugin. This is a CLI tool that will look through your project and sort all [Tailwind CSS](https://tailwindcss.com) classes.

It will also delete any duplicate classes it finds.

## Why?

When Ryan released his vscode extension some people expressed an interest in having a CLI version.

In addition, I've had an interest in learning rust. I usually learn by doing. I thought this would be
the perfect opportunity to make a small useful tool while learning rust.

Keep in mind this is my first rust project. My code here is most likely going to be ugly and not idiomatic rust.
If you are a rust expert you're help in making the code nicer and more idiomatic will be greatly appreciated.

Any and all feedback and criticism is very welcome. If im doing anything stupid please let me know.

## The Plan

### ✅ ~~MVP~~

1. ~~Run on current directory~~
2. ~~Order all classes in all files in directory (except for gitignored ones)~~

#### ✅ ~~Development Plan~~

1. ~~Get a list of all files in current directory recursively.~~ [#1](https://github.com/avencera/rustywind/pull/1)
2. ~~Ignore all files in .gitignore~~ [#1](https://github.com/avencera/rustywind/pull/1)
3. ~~Get all contents from the list of files 1 by 1~~ [#2](https://github.com/avencera/rustywind/pull/2)
4. ~~Run regex on all file contents and get all class names~~ [#3](https://github.com/avencera/rustywind/pull/3)
5. ~~Figure out how to sort class names using sorter array~~ [#4](https://github.com/avencera/rustywind/pull/4)
6. ~~Replace old class names with new list of class names~~ [#5](https://github.com/avencera/rustywind/pull/5)
7. ~~Save file~~ [#6](https://github.com/avencera/rustywind/pull/6)
8. ~~Release via NPM~~ (available via [npm](https://www.npmjs.com/package/rustywind) `npm install rustywind -g`)
   - ~~[How to Distribute a Rust Binary on NPM](https://dev.to/kennethlarsen/how-to-distribute-a-rust-binary-on-npm-75n)~~
   - ~~[Trust Repo](https://github.com/japaric/trust)~~
   - ~~[Example](https://github.com/kennethlarsen/baelte-npm)~~

### ✅ ~~Phase 2 (CLI)~~

1. ~~Add CLI parser~~ [#8](https://github.com/avencera/rustywind/pull/8)
2. ~~Use CLI parser to choose directory or file to run on~~ [#8](https://github.com/avencera/rustywind/pull/8)
3. ~~Add `--write` option which will change the files, change default to printing results to the terminal~~ [#8](https://github.com/avencera/rustywind/pull/8)

### ✅ ~~Phase 3 (More CLI)~~
1.  ~~Add `--allow-duplicates` to allow duplicate class name~~ [#10](https://github.com/avencera/rustywind/pull/10)
2.  ~~Add `--dry-run` option to print out all changes to console~~ [c975d1](https://github.com/avencera/rustywind/commit/c975d1f041f95b45e15760ccded24dbf62bf1f6f)

### ✅~~Phase 4 (Performance)~~

1. ~~Search and change files in parallel (parallel map?)~~ [#9](https://github.com/avencera/rustywind/pull/9)

### Phase 5 (Match Headwind Features)
1. Add custom sorter via CLI args `--customer-sorter=...`
2. Add custom regex via CLI args `--custom-regex=...`

### Later Possibilities...

- Add ability to add custom sorter
- Derive sorter from evaluation generated CSS from tailwind config file
- Add ability to exclude folders, or files using regex pattern
- Add argument for only searching in certain file types
