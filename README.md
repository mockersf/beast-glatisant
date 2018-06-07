# Beast Glatisant

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/mockersf/beast-glatisant.svg?branch=master)](https://travis-ci.org/mockersf/beast-glatisant)

Github bot to check code samples in issues.

## Goal

To help diagnose issues and check if they are fixed by another change, this bot will find code samples in Github issues, send them to the Rust Playground to compile / run / test / run clippy / run rustfmt, and add the result to the Github issue