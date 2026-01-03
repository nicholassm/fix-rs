# FIX

Tools for improving readability of FIX messages.

# Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contributions](#contributions)
- [Roadmap](#roadmap)

# Introduction

## Motivation

FIX messages can be hard to read and are often embedded in logs.
Most companies create their own small tools for making them easier to read and that's the motivation for this work: an open source tool that's better than the closed source alternatives.

## Design

This tool will be written in the Unix philosophy.
I.e. it will focus on making FIX messages easier to read by splitting tags on separate lines, apply colouring, etc. but rely on other tools such as `cat` and `grep` to input the raw text into the tool.

## Naming

The tool command will be `nfix` and the idea is that you can say e.g. *grep'n'fix* when piping results from `grep` into `nfix`.

Originally, I wanted to call the tool `hfix` in the spirit of e.g. `htop` and the `h` would be for "human" but when I learned that the `h` in `htop` is short for "Hisham" ([htop](https://htop.dev/) was written by [Hisham Muhammad](https://hisham.hm/)) then what the heck -  It's also called `nfix` because my first name is Nicholas.

# Features

None yet!

# Contributions

# Roadmap
