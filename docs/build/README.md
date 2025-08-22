# Perky  🐿️

Perky is a library and command-line interface (CLI) for evaluating and optimizing keyboard layouts.

It is intended for use by designers, researchers, and enthusiasts who want a principled, data-driven method to quantitatively assess and improve the efficiency of keyboard layouts, using metrics derived from [n‑gram tables](#n-gram-tables) and the ergonomics of the hand.

Perky's unique feature is a high-performance, parallelized, and permutation-based analysis engine that can exhaustively determine the best keyboard layouts given specific constraints.

## Features

- 🔧 Customizable input

  Specify your layout, key assignments, and the n‑gram frequency data to be used for analysis. Defaults are included to get you started quickly.

- ⚖️ Comprehensive scoring engine

  Evaluate your layout using a wide range of [metrics](#metrics). Scores are calculated based on digit (finger or thumb) movement and n‑gram frequency data, reflecting real-world usage patterns.

- 🔢 High-performance permutation engine

  (Optionally) use placeholders for certain key assignments and provide the set of characters that should be assigned to those and Perky will permute all possibilities, ranking the (potentially trillions of) records according to your desired metric.

- 🎯 Expressive sorting, filtering, and selection

  Sort records by one or more metrics (each in ascending or descending order), filter by multiple criteria using a built-in mini-language, and select a specific subset for output.

- 📈 Rich output

  Export human-readable (and colourful) text or structured [JSON Lines](https://jsonlines.org), with optional metadata, details, and summaries.

## Documentation

This project includes a user manual which includes information on how to install this package.

The user manual is available here, in three formats:

- [Markdown](/docs/build/perky.md)
- [HTML](/docs/build/perky.html)
- [HTML Standalone](/docs/build/perky-standalone.html)

<!--
## Contributing
&#10;Pull requests are welcome. Please open an issue first to discuss what you would like to change.
-->

## Author and copyright

This project was written and copyrighted in 2025 by Chris McLaren ([@csmclaren](https://www.github.com/csmclaren)).

## License

Unless otherwise noted, all files in this project are licensed under the [MIT License](https://choosealicense.com/licenses/mit/). See the [LICENSE](/LICENSE.txt) file for details.
