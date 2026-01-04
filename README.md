# FIX

Tools for improving readability of FIX messages.

# Contents

- [Introduction](#introduction)
- [Features](#features)
- [Roadmap](#roadmap)
- [Contributions](#contributions)

# Introduction

## Motivation

FIX messages can be hard to read and are often embedded in logs.
Most companies create their own small tools for making them easier to read and that's the motivation for this work: an open source tool that's better than the closed source alternatives.

## Design

This tool will be written in the Unix philosophy.
I.e. it will focus on making FIX messages easier to read by splitting tags on separate lines, apply colouring, etc. but rely on other tools such as `cat` and `grep` to input and filter the raw text into the tool.

## Naming

The tool command will be `nfix` and the idea is that you can say e.g. *grep'n'fix* when piping results from `grep` into `nfix`.

Originally, I wanted to call the tool `hfix` in the spirit of e.g. `htop` and the `h` would be for "human" but when I learned that the `h` in `htop` is short for "Hisham" ([htop](https://htop.dev/) was written by [Hisham Muhammad](https://hisham.hm/)) then what the heck - it's also called `nfix` because my first name is Nicholas. :-)

# Features

None yet!

# Roadmap

1. Parse data read on stdin, identify FIX messages and print one tag per line on stdout. Non-FIX message data is passed through unprocessed.
2. Add tag names to parsed tags and enum names, etc.
3. Group messages in order chains to display the lifecycle of each order (from the initial New Order Single messages to the final Execution Report.) These messages are often scattered across a log and interleaved with each other and can thus be quite hard to look at - a lot of jumping around based on ClOrdID (11) OrderID (37).
4. Hide often irrelevant tags such as BeginString (8), BodyLength (9), CheckSum (10) and specific tags per message. Show all tags by explicitly passing command line argument `-a`.
5. Apply colours with some scheme to make it easier to read the FIX messages.
6. Support custom FIX protocols by parsing the spec and generating a binary that "natively" understands e.g. custom tags.

# Contributions

You are welcome to create a Pull-Request or open an issue with suggestions for improvements.

Changes are accepted solely at my discretion and I will focus on whether the changes are a good fit for the purpose and design of this crate.
