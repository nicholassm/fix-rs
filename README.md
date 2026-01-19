# FIX

Tools for improving readability of FIX messages.

# Contents

- [Introduction](#introduction)
- [Getting Started](#getting-started)
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

## AI

The code is carefully crafted by me with only a bit of AI intellisense (which was carefully reviewed).

# Getting Started

Download the code and run

    cargo build --release

Copy it to `/usr/local/bin` or similar depending on your OS.

To get help and see available options, run

    nfix -h

Example usage:

    head -1 test-data/messages.txt | nfix -s ';'
        2020/03/02-06:14:36,393 INFO :: TracingIncomingMessage[HFE_UAT_EQ_US]:
        35 : MsgType      = D
        49 : SenderCompID = IREACH
        56 : TargetCompID = MLPT
       115                = HARRISEUAP
        34 : MsgSeqNum    = 988
        52 : SendingTime  = 20200302-11:14:36.393
     75205                = 2
        11                = IREACH_t182_LLOYLN.nx83e.j#a1
        63                = 0
        21                = 2
      6401                = 45
      6408                = M
      9682                = BofAGlobalAlgoFixSpec_ProductionVersionJan2020
        55                = LLOYLN
        48                = 0870612
        22                = 2
       167                = CS
       207                = XLON
        54                = 1
        60                = 20200302-11:14:36
        38                = 1000
        40                = 2
        44                = 59.73
        15                = GBp
        59                = 0
      8202                = PCT
      8204                = ABS
      8205                = FAV
      8206                = ATS

# Features

1. Parse data read on stdin, identify FIX messages and print one tag per line on stdout. Non-FIX message data is passed through unprocessed.
2. Specify non-default field separator with the command line argument `-s`.
3. Hide often irrelevant tags such as BeginString (8), BodyLength (9), CheckSum (10). Show all tags by explicitly passing command line argument `-a`.

# Roadmap

1. Add tag names to parsed tags and enum names, etc.
2. Group messages in order chains to display the lifecycle of each order (from the initial New Order Single messages to the final Execution Report.) These messages are often scattered across a log and interleaved with each other and can thus be quite hard to look at - a lot of jumping around based on ClOrdID (11) OrderID (37).
3. Hide often irrelevant tags specific to a MsgType.
4. Apply colours with some scheme to make it easier to read the FIX messages.
5. Support custom FIX protocols by parsing the spec and generating a binary that "natively" understands e.g. custom tags.

# Contributions

You are welcome to create a Pull-Request or open an issue with suggestions for improvements.

Changes are accepted solely at my discretion and I will focus on whether the changes are a good fit for the purpose and design of this crate.
