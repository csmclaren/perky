# Perky  🐿️

Perky is a library and command-line interface (CLI) for evaluating and optimizing keyboard layouts.

It is intended for use by designers, researchers, and enthusiasts who want a principled, data-driven method to quantitatively assess and improve the efficiency of keyboard layouts, using metrics derived from [n‑gram tables](#n-gram-tables) and the ergonomics of the hand.

Perky's unique feature is a high-performance, parallelized, and permutation-based analysis engine that can exhaustively determine the best keyboard layouts given specific constraints.

## Table of contents

- [Features](#features)
- [Installation](#installation)
  - [Install the binary from crates.io](#install-the-binary-from-cratesio)
  - [Install the binary from source](#install-the-binary-from-source)
  - [Add the library to another project as a dependency](#add-the-library-to-another-project-as-a-dependency)
- [Usage](#usage)
  - [Introduction](#introduction)
  - [Layout tables](#layout-tables)
    - [Format](#format)
    - [Examples](#examples)
  - [Key tables](#key-tables)
    - [Format](#format-1)
    - [Examples](#examples-1)
  - [N-gram tables](#n-gram-tables)
    - [Default tables](#default-tables)
    - [Included tables](#included-tables)
    - [Specifying tables](#specifying-tables)
    - [Format](#format-2)
    - [Examples](#examples-2)
  - [Scoring](#scoring)
    - [Metrics](#metrics)
      - [Unigram metrics](#unigram-metrics)
      - [Bigram metrics](#bigram-metrics)
        - [Fsb - Full Scissor Bigram](#fsb---full-scissor-bigram)
        - [Hsb - Half Scissor Bigram](#hsb---half-scissor-bigram)
        - [Irb - Inward Roll Bigram](#irb---inward-roll-bigram)
        - [Lsb - Lateral Stretch Bigram](#lsb---lateral-stretch-bigram)
        - [Orb - Outward Roll Bigram](#orb---outward-roll-bigram)
        - [Sfb - Same Finger Bigram](#sfb---same-finger-bigram)
      - [Trigram metrics](#trigram-metrics)
        - [Alt - Alternating Trigram](#alt---alternating-trigram)
        - [One - One-Handed Trigram](#one---one-handed-trigram)
        - [Red - Redirect Trigram](#red---redirect-trigram)
        - [Rol - Roll Trigram](#rol---roll-trigram)
    - [Calculation](#calculation)
    - [Summary report](#summary-report)
      - [Examples](#examples-3)
    - [Detail reports](#detail-reports)
      - [Examples](#examples-4)
  - [Permuting](#permuting)
    - [Metric](#metric)
    - [Goal](#goal)
    - [Weight](#weight)
    - [Progress and metadata reporting](#progress-and-metadata-reporting)
    - [Examples](#examples-5)
    - [Parallelization](#parallelization)
      - [Examples](#examples-6)
    - [Practical limits](#practical-limits)
    - [Truncating](#truncating)
      - [Examples](#examples-7)
    - [Deduplicating](#deduplicating)
    - [Sorting](#sorting)
      - [Syntax](#syntax)
      - [Examples](#examples-8)
    - [Filtering](#filtering)
      - [Syntax](#syntax-1)
        - [Operators by precedence](#operators-by-precedence)
      - [Examples](#examples-9)
    - [Selecting](#selecting)
      - [Syntax](#syntax-2)
      - [Examples](#examples-10)
  - [Printing](#printing)
    - [Metadata](#metadata)
    - [Selected records](#selected-records)
    - [Format](#format-3)
    - [Examples](#examples-11)
- [Author and copyright](#author-and-copyright)
- [License](#license)

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

## Installation

Perky can run on macOS, Linux, and Windows.

To install Perky, first ensure that you have installed [Rust](https://www.rust-lang.org/tools/install), which includes [Cargo](https://doc.rust-lang.org/cargo/).

> Note: Perky requires Rust version 1.89.0 or later.

Then choose one of the following options based on the type of installation you prefer:

### Install the binary from crates.io

To install the binary from [crates.io](https://crates.io/crates/perky), follow these steps:

1.  Install the binary to your Cargo bin directory (typically `$HOME/.cargo/bin`):

    ``` sh
    cargo install perky
    ```

2.  (Optional) Test that the installation was successful by running Perky (from the directory containing the binary) with the `--help` argument.

    ``` sh
    perky --help
    ```

### Install the binary from source

To install the binary from source, follow these steps:

1.  Clone the [official repository](https://github.com/csmclaren/perky) from GitHub

    ``` sh
    git clone https://github.com/csmclaren/perky.git
    cd perky
    ```

2.  Build the binary, which will be placed in your target directory (typically `target/release/perky`):

    ``` sh
    cargo build --release
    ```

3.  (Optional) Copy the binary to your Cargo bin directory (typically `$HOME/.cargo/bin`):

    ``` sh
    cp target/release/perky ~/.cargo/bin/
    ```

4.  (Optional) Test that the installation was successful by running Perky (from the directory containing the binary) with the `--help` argument.

    ``` sh
    perky --help
    ```

### Add the library to another project as a dependency

To include Perky as a library in your own Rust project, add the following to your `Cargo.toml` file:

``` toml
[dependencies]
perky = "0.1"  # Replace with the latest available version on crates.io
```

The latest version can be found on [crates.io](https://crates.io/crates/perky).

## Usage

> Note: The test machine used to run all examples in this section is an Apple MacBook Pro, 16 inch, Nov 2023, M3 Max, 16-core, 128 GB RAM.

### Introduction

Perky is designed primarily to be run as a command-line tool.

It requires two inputs:

- A layout table (specified using `--layout-table`, or `-l`) defines the structure of a keyboard, including which digits (fingers and thumbs) are recommended for pressing each physical key.

- A key table (specified using `--key-table`, or `-k`), defines the mapping of logical keys (letters, numbers, symbols, etc.) to physical keys.

The format of these files will be explained in subsequent sections and there are a number of example files of each type in the [official repository](https://github.com/csmclaren/perky), but for purposes of this introductory example, here is an example file of each type:

- [example.lt.json](/examples/docs/example.lt.json) - A layout table in the shape of the central part of a standard [ANSI or ISO keyboard layout](https://en.wikipedia.org/wiki/Keyboard_layout).

- [example-introduction.kt.json](/examples/docs/example-introduction.kt.json) - A key table corresponding to the shape of the layout table and representing a standard [QWERTY](https://en.wikipedia.org/wiki/QWERTY) keyboard.

After downloading these to your working directory, you can run Perky as follows:

``` sh
perky -l example.lt.json -k example-introduction.kt.json
```

> Note: If either or both of these arguments are omitted, Perky will look for the files `default.lt.json` and/or `default.kt.json`, respectively, in your working directory.

Given the above example files and no further arguments, Perky will produce the following output:

    Q W E R T Y U I O P [ ] \
    A S D F G H J K L ; '
    Z X C V B N M , . /

    Unigram summaries:
    Lt ↑: 0, 0.000%, 0, 0.000%
    Li ↑: 796991792591, 22.365%, 796991792591, 22.365%
    Lm ↑: 700328467558, 19.653%, 700328467558, 19.653%
    Lr ↓: 300164341769, 8.423%, 300164341769, 8.423%
    Lp ↓: 294025744233, 8.251%, 294025744233, 8.251%
    Lh ↑: 2091510346151, 58.692%, 2091510346151, 58.692%
    Rt ↑: 0, 0.000%, 0, 0.000%
    Ri ↑: 689614872654, 19.352%, 689614872654, 19.352%
    Rm ↑: 288992871918, 8.110%, 288992871918, 8.110%
    Rr ↓: 417275087248, 11.710%, 417275087248, 11.710%
    Rp ↓: 76112599849, 2.136%, 76112599849, 2.136%
    Rh ↑: 1471995431669, 41.308%, 1471995431669, 41.308%
    TOTALS: 3563505777820, 3563505777820

    Bigram summaries:
    Fsb ↓: 29561207813, 1.075%, 76155339763, 1.809%
    Hsb ↓: 223473410253, 8.129%, 449964114055, 10.688%
    Irb ↑: 107185097537, 3.899%, 107185097537, 2.546%
    Lsb ↓: 195655811929, 7.117%, 490785251171, 11.658%
    Orb ↑: 98315980469, 3.576%, 98315980469, 2.335%
    Sfb ↓: 195686888871, 7.119%, 270316925501, 6.421%
    TOTALS: 2748955989650, 4209833679358

    Trigram summaries:
    Alt ↓: 547770117520, 28.921%, 547770117520, 12.191%
    One ↓: 58813867491, 3.105%, 164484663333, 3.661%
    Red ↓: 138042336404, 7.288%, 798736382372, 17.777%
    Rol ↓: 783296865779, 41.357%, 1866401805671, 41.539%
    TOTALS: 1893998506314, 4493113390239

> Note: Output shown herein is rendered without colour. When run in a terminal, Perky automatically uses colour and text effects for improved readability, unless piped into another command or explicitly disabled.

By default, Perky outputs a textual view of the keyboard and summaries of how that keyboard scored against a series of metrics, a complete description of which will be provided in subsequent sections.

### Layout tables

A layout table defines the structure of a keyboard, including which digits (fingers and thumbs) are recommended for pressing each physical key. This mapping is critical for ergonomic evaluation. For example, it helps assess whether or not two consecutive keys are typed with the same digit, a key requires a stretch or contraction, or the total effort is well balanced across the hands.

#### Format

Layout tables are JSON objects with the following structure:

``` json
{
  "data": [...],
  "version": 1
}
```

`data` is a 2-dimensional matrix:

- Each cell must contain null or a string of exactly two characters representing a digit. Null indicates the absence of a key in that position. A string defines how the key in that position would typically be pressed. The first character must be either "l" or "r", for the left or right hand, respectively. The second character must be "p", "r", "m", "i", or "t" for the pinky, ring, middle, index, or thumb digit, respectively.

- The size of the matrix is 16 columns by 8 rows. Any row may contain fewer than 16 columns, in which case the trailing cells of that row are treated as if they contained `null`. Any table may contain fewer than 8 rows, in which case all cells of the trailing rows of that table are treated as if they contained `null`.

`version` must be 1.

#### Examples

This example describes the digit assignments for the central part of a ANSI or ISO keyboard layout:

[example.lt.json](/examples/docs/example.lt.json)

``` json
{
  "data": [
    ["lp", "lr", "lm", "li", "li", "ri", "ri", "rm", "rr", "rp", "rp", "rp", "rp"],
    ["lp", "lr", "lm", "li", "li", "ri", "ri", "rm", "rr", "rp", "rp"],
    ["lp", "lr", "lm", "li", "li", "ri", "ri", "rm", "rr", "rp"]
  ],
  "version": 1
}
```

### Key tables

A key table defines the mapping of logical keys (letters, numbers, symbols, etc.) to physical keys. Together with the layout table, it forms the basis for ergonomic and statistical analysis of a keyboard layout. The key table identifies what is being typed and the layout table determines how it is typed. Key tables may include placeholders to allow permutation of many alternatives.

#### Format

Key tables are JSON objects with the following structure:

``` json
{
  "data": [...],
  "version": 1
}
```

`data` is a 2-dimensional matrix.

- Each cell must contain null; a string of exactly one character; or the number 1, 2, or 3.

  - Null indicates the absence of a key in that position.

  - A string assigns a character to that key. Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved.

  - A number represents a *placeholder*.

    > Note: The numbers 1, 2, and 3 are different from the strings "1", "2", or "3", which represent characters.

- Placeholders of the same numeric value form a `region`. The size of a region is defined as the number of placeholders in that region. Placeholders in the same region do not need to be adjacent to one another. It is their numeric value (1, 2, or 3), not their physical location, that binds them to a region. Regions are used for permutation, which will be described in subsequent sections.

- The size of the matrix is 16 columns by 8 rows. Any row may contain fewer than 16 columns, in which case the trailing cells of that row are treated as if they contained `null`. Any table may contain fewer than 8 rows, in which case all cells of the trailing rows of that table are treated as if they contained `null`.

`version` must be 1.

Perky expects the layout table and key table to have matching structures; that is, if a cell in the layout table defines a digit, the corresponding cell in the key table must not be null.

#### Examples

This is an example of the central part of an ANSI or ISO keyboard layout in the standard QWERTY configuration:

[example-introduction.kt.json](/examples/docs/example-introduction.kt.json)

``` json
{
  "data": [
    ["Q",  "W",  "E",  "R",  "T",  "Y",  "U",  "I",  "O",  "P",  "[",  "]",  "\\"],
    ["A",  "S",  "D",  "F",  "G",  "H",  "J",  "K",  "L",  ";",  "'"],
    ["Z",  "X",  "C",  "V",  "B",  "N",  "M",  ",",  ".",  "/"]
  ],
  "version": 1
}
```

This is an example of the central part of an ANSI or ISO keyboard layout in the standard QWERTY configuration, where the home row has been replaced by a single permutation region of size 9:

[example-permuting.kt.json](/examples/docs/example-permuting.kt.json)

``` json
{
  "data": [
    ["Q",  "W",  "E",  "R",  "T",  "Y",  "U",  "I",  "O",  "P",  "[",  "]",  "\\"],
    [ 1,    1,    1,    1,    1,    1,    1,    1,    1,   ";",  "'"],
    ["Z",  "X",  "C",  "V",  "B",  "N",  "M",  ",",  ".",  "/"]
  ],
  "version": 1
}
```

If one or more placeholders for a particular region are present in a key table, and Perky is given a set of possible characters for that region, Perky will permute all possible combinations of those characters in that region. This feature is explained in detail in subsequent sections.

### N-gram tables

An [n‑gram](https://en.wikipedia.org/wiki/N-gram) is a contiguous sequence of *n* characters drawn from a larger text (a "corpus").

Perky supports three kinds of n‑grams:

- Unigrams (or "1-grams"): one-character sequences like "E", "T", and "A".
- Bigrams (or "2-grams"): two-character sequences like "TH", "HE", and "IN".
- Trigrams (or "3-grams"): three-character sequences like "THE", "AND", and "ING".

An n‑gram table is one or more pairs of n‑grams and the number of times that n‑gram occurred in the corpus.

Unigram tables show the relative frequency of characters in a corpus. This information can be used, for example, to assign commonly used characters to stronger or more accessible digits.

Bigram and trigram tables show the relative frequency of sequences of characters in a corpus. This information can be used, for example, to assign commonly used sequences of characters to the most ergonomic movements.

Perky uses n‑grams tables to analyze how often individual characters and sequences of characters appear in real-world usage. This allows it to score keyboard layouts based on realistic workloads.

#### Default tables

By default, Perky uses n‑gram data derived from [Peter Norvig’s analysis of the Google Books corpus](https://norvig.com/mayzner.html) and copied from [charfreq-google](https://github.com/csmclaren/charfreq-google). These tables are part of Perky itself, providing a high-quality and standard starting point for most analyses out-of-the-box.

#### Included tables

In the [resources](/resources/) folder of the official repository, various n‑gram tables are included:

- [charfreq-dfko](https://github.com/csmclaren/charfreq-dfko)
- [charfreq-google](https://github.com/csmclaren/charfreq-google) (the [default tables](#default-tables))
- [charfreq-linux](https://github.com/csmclaren/charfreq-linux)
- [charfreq-shakespeare](https://github.com/csmclaren/charfreq-shakespeare)

These are described in their respective repositories.

#### Specifying tables

To override the default unigram, bigram, or trigram tables, specify the following arguments, respectively:

- `--unigram-table <FPATH>` or `-u <FPATH>`
- `--bigram-table <FPATH>` or `-b <FPATH>`
- `--trigram-table <FPATH>` or `-t <FPATH>`

For example, to override all three tables to the (included) uppercase tables in charfreq-linux (representing n‑gram frequency in the Linux source code):

``` sh
perky \
  -l example.lt.json \
  -k example-introduction.kt.json \
  -u resources/charfreq-linux/1-grams-uc.tsv \
  -b resources/charfreq-linux/2-grams-uc.tsv \
  -t resources/charfreq-linux/3-grams-uc.tsv
```

#### Format

Each n‑gram table is stored as a TSV (tab-separated values) file with one entry per line.

Each line must contain at least two columns:

- The first column must be an n‑gram, represented as a string of 1, 2, or 3 Unicode characters, depending on the kind of table. Strings may contain escape sequences as follows:

  - `\0` → NUL
  - `\\` → `\`
  - `\n` → LF
  - `\r` → CR
  - `\t` → HT
  - `\x##` → ASCII `##`, where `##` is two hexadecimal digits.

- The second column must be an unsigned integer (with a maximum value of 2^64 - 1) representing the number of times that n‑gram occurred in a corpus.

- Any additional columns are ignored.

> Note: Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved. All other n‑grams will be ignored.

#### Examples

A unigram table containing three 1-grams and their number of occurences in the corpus:

``` tsv
E   445155370175
T   330535289102
A   286527429118
```

A bigram table containing three 2-grams and their number of occurences in the corpus:

``` tsv
TH  100272945963
HE  86697336727
IN  68595215308
```

A trigram table containing three 3-grams and their number of occurences in the corpus:

``` tsv
THE 69221160871
AND 26468697834
ING 21289988294
```

### Scoring

Scoring is formalized by set of "metrics", each of which measure the ergonomic and statistical performance of a keyboard layout against certain quantitative criteria. Given a layout table, a key table, and one or more n‑gram tables, Perky can produce a score for each metric. These scores, represented as numerical values and percentages, can be used to compare, optimize, and iterate on keyboard layouts.

#### Metrics

The following tables describe the metrics used by Perky to score a keyboard layout.

Some metrics reflect effort or strain (for these, lower values are generally considered better), while others measure efficiency (for these, higher values are generally considered better). In the tables below, this is called the direction: "↓" indicates that lower is generally considered better and "↑" indicates that higher is generally considered better.

##### Unigram metrics

| Metric | Direction | Description                                |
|--------|:---------:|--------------------------------------------|
| Lt     |     ↑     | Left thumb unigram                         |
| Li     |     ↑     | Left index unigram                         |
| Lm     |     ↑     | Left middle unigram                        |
| Lr     |     ↓     | Left ring unigram                          |
| Lp     |     ↓     | Left pinky unigram                         |
| Lh     |     ↑     | Left hand unigram (sum of the above five)  |
| Rt     |     ↑     | Right thumb unigram                        |
| Ri     |     ↑     | Right index unigram                        |
| Rm     |     ↑     | Right middle unigram                       |
| Rr     |     ↓     | Right ring unigram                         |
| Rp     |     ↓     | Right pinky unigram                        |
| Rh     |     ↑     | Right hand unigram (sum of the above five) |

Each of these metrics are self-explanatory from the description: they measure the number of occurences of a single character pressed by a particular digit or hand.

##### Bigram metrics

| Metric | Direction | Description            |
|--------|:---------:|------------------------|
| Fsb    |     ↓     | Full scissor bigram    |
| Hsb    |     ↓     | Half scissor bigram    |
| Irb    |     ↑     | Inward roll bigram     |
| Lsb    |     ↓     | Lateral stretch bigram |
| Orb    |     ↑     | Outward roll bigram    |
| Sfb    |     ↓     | Same finger bigram     |

###### Fsb - Full Scissor Bigram

- Both keys are pressed by the same hand
- The keys are least *one* column apart and at least *two* rows apart from each other
- One key is pressed by a middle or ring finger and closer to the bottom of the keyboard than the other key

###### Hsb - Half Scissor Bigram

- Both keys are pressed by the same hand
- The keys are least *one* column apart and exactly *one* row apart from each other
- One key is pressed by a middle or ring finger and closer to the bottom of the keyboard than the other key

###### Irb - Inward Roll Bigram

- Both keys are pressed by the same hand
- The keys are *one* column apart from and on the same row as each other
- The digit movement proceeds inward: pinky → ring, ring → middle, or middle → index

###### Lsb - Lateral Stretch Bigram

- Both keys are pressed by the same hand
- The keys are least *two* columns apart from each other
- The keys are pressed by the index and middle fingers, in either order

###### Orb - Outward Roll Bigram

- Both keys are pressed by the same hand
- The keys are *one* column apart from and on the same row as each other
- The digit movement proceeds outward: index → middle, middle → ring, or ring → pinky

###### Sfb - Same Finger Bigram

- Both keys are pressed by the same hand
- Both keys are pressed by the same digit

##### Trigram metrics

| Metric | Direction | Description         |
|--------|:---------:|---------------------|
| Alt    |     ↓     | Alternating trigram |
| One    |     ↓     | One handed trigram  |
| Red    |     ↓     | Redirect trigram    |
| Rol    |     ↓     | Roll trigram        |

###### Alt - Alternating Trigram

- The first and third key are pressed by the same hand
- The second key is pressed by the other hand

###### One - One-Handed Trigram

- All three keys are pressed by the same hand but by different digits
- The columns of all three keys are different and strictly increase or decrease

###### Red - Redirect Trigram

- All three keys are pressed by the same hand but by different digits
- The columns of all three keys are different but do not strictly increase or decrease

###### Rol - Roll Trigram

- Two adjacent keys are pressed by the same hand but by different digits
- The other key is pressed by the other hand

#### Calculation

A metric is scored as follows:

- All possible 1, 2, or 3 physical key combinations (depending on the type of metric: unigram, bigram, and trigram, respectively) are determined by consulting the layout table.
- These key combinations are filtered to exclude those that do not meet the criteria of the metric.
- For each combination remaining, the key assignments for the keys in the combination are determined by consulting the key table. This determines the n‑gram.
- For each n‑gram, the number of occurences is determined by consulting the appropriate n‑gram table.
- The number of occurences are summed to provide the raw frequency-based score for that metric.
- The key combinations are also used to produce an effort factor. This is a function of distance between keys in the layout table and is used for weighting key combinations based on the relative physical effort. The raw frequency-based score is multiplied by the effort factor to produce the effort-weighted score.
- Both of the raw frequency-based and effort-weighted scores are then represented as a percentage of the sum of n‑gram occurences and the effort-weighted sum of n‑gram occurences, respectfully.

All metrics are deterministic given the same inputs.

#### Summary report

A summary report is table showing the scores for each metric. The scores are grouped by the type of metric - unigram, bigram, and trigram - and the header of each group is `Unigram summaries`, `Bigram summaries`, and `Trigram summaries`, respectively. Each row contains the following columns:

- Metric name
- Metric direction
- Sum of occurences of all n‑grams within the metric
- Percentage representation of all n‑grams within the metric
- Sum of occurences of all n‑grams within the metric, effort-weighted
- Percentage representation of all n‑grams within the metric, effort-weighted

By default, summary reports are printed (equivalent to `--print-summaries true`). To suppress summary reports, specify `--print-summaries false`

##### Examples

Here is the summary report from the example in the [Introduction](#introduction) section:

    Unigram summaries:
    Lt ↑: 0, 0.000%, 0, 0.000%
    Li ↑: 796991792591, 22.365%, 796991792591, 22.365%
    Lm ↑: 700328467558, 19.653%, 700328467558, 19.653%
    Lr ↓: 300164341769, 8.423%, 300164341769, 8.423%
    Lp ↓: 294025744233, 8.251%, 294025744233, 8.251%
    Lh ↑: 2091510346151, 58.692%, 2091510346151, 58.692%
    Rt ↑: 0, 0.000%, 0, 0.000%
    Ri ↑: 689614872654, 19.352%, 689614872654, 19.352%
    Rm ↑: 288992871918, 8.110%, 288992871918, 8.110%
    Rr ↓: 417275087248, 11.710%, 417275087248, 11.710%
    Rp ↓: 76112599849, 2.136%, 76112599849, 2.136%
    Rh ↑: 1471995431669, 41.308%, 1471995431669, 41.308%

    Bigram summaries:
    Fsb ↓: 29561207813, 1.048%, 76155339763, 1.779%
    Hsb ↓: 223473410253, 7.926%, 449964114055, 10.512%
    Irb ↑: 107185097537, 3.801%, 107185097537, 2.504%
    Lsb ↓: 195655811929, 6.939%, 490785251171, 11.465%
    Orb ↑: 98315980469, 3.487%, 98315980469, 2.297%
    Sfb ↓: 195686888871, 6.940%, 270316925501, 6.315%

    Trigram summaries:
    Alt ↓: 547770117520, 26.108%, 547770117520, 11.662%
    One ↓: 58813867491, 2.803%, 164484663333, 3.502%
    Red ↓: 138042336404, 6.579%, 798736382372, 17.004%
    Rol ↓: 783296865779, 37.333%, 1866401805671, 39.734%

#### Detail reports

A detail report is table showing how each individual n‑gram scored within a given metric. The header of the table is the name of the metric and its direction. Each row contains the following columns:

- N‑gram
- Number of occurences
- Number of occurences, cumulative
- Percentage representation within the metric
- Percentage representation within the metric, cumulative
- Percentage representation
- Percentage representation, cumulative
- Number of occurences, effort-weighted
- Number of occurences, effort-weighted and cumulative
- Percentage representation within the metric, effort-weighted
- Percentage representation within the metric, effort-weighted and cumulative
- Percentage representation, effort-weighted
- Percentage representation, effort-weighted and cumulative

By default, detail reports are not printed. To print a detail report, specify the `--print-details [<METRIC>...]` argument. This argument accepts one or more metric names (e.g., `li`, `orb`, `alt`, etc.)

Multiple detail reports are printed in the order defined in [Metrics](#metrics). Detail reports are printed before the summary report, if any.

##### Examples

``` sh
perky \
  -l example.lt.json \
  -k example-introduction.kt.json \
  --print-details irb \
  --print-summaries false
```

    Q W E R T Y U I O P [ ] \
    A S D F G H J K L ; '
    Z X C V B N M , . /

    Irb ↑:
    ER, 57754162106, 57754162106, 53.883%, 53.883%, 2.101%, 2.101%, 57754162106, 57754162106, 53.883%, 53.883%, 1.372%, 1.372%
    AS, 24561944198, 82316106304, 22.915%, 76.798%, 0.894%, 2.994%, 24561944198, 82316106304, 22.915%, 76.798%, 0.583%, 1.955%
    PO, 10189505383, 92505611687, 9.506%, 86.305%, 0.371%, 3.365%, 10189505383, 92505611687, 9.506%, 86.305%, 0.242%, 2.197%
    WE, 10176141608, 102681753295, 9.494%, 95.799%, 0.370%, 3.735%, 10176141608, 102681753295, 9.494%, 95.799%, 0.242%, 2.439%
    OI, 2474275212, 105156028507, 2.308%, 98.107%, 0.090%, 3.825%, 2474275212, 105156028507, 2.308%, 98.107%, 0.059%, 2.498%
    XC, 746076293, 105902104800, 0.696%, 98.803%, 0.027%, 3.852%, 746076293, 105902104800, 0.696%, 98.803%, 0.018%, 2.516%
    LK, 555883002, 106457987802, 0.519%, 99.322%, 0.020%, 3.873%, 555883002, 106457987802, 0.519%, 99.322%, 0.013%, 2.529%
    IU, 490874936, 106948862738, 0.458%, 99.780%, 0.018%, 3.891%, 490874936, 106948862738, 0.458%, 99.780%, 0.012%, 2.540%
    SD, 148275222, 107097137960, 0.138%, 99.918%, 0.005%, 3.896%, 148275222, 107097137960, 0.138%, 99.918%, 0.004%, 2.544%
    DF, 78347492, 107175485452, 0.073%, 99.991%, 0.003%, 3.899%, 78347492, 107175485452, 0.073%, 99.991%, 0.002%, 2.546%
    CV, 5869407, 107181354859, 0.005%, 99.997%, 0.000%, 3.899%, 5869407, 107181354859, 0.005%, 99.997%, 0.000%, 2.546%
    KJ, 3471162, 107184826021, 0.003%, 100.000%, 0.000%, 3.899%, 3471162, 107184826021, 0.003%, 100.000%, 0.000%, 2.546%
    QW, 163292, 107184989313, 0.000%, 100.000%, 0.000%, 3.899%, 163292, 107184989313, 0.000%, 100.000%, 0.000%, 2.546%
    ZX, 108224, 107185097537, 0.000%, 100.000%, 0.000%, 3.899%, 108224, 107185097537, 0.000%, 100.000%, 0.000%, 2.546%

### Permuting

Key tables can include up to three *permutation regions*. A permutation region is a set of keys that are assigned by permuting every possibility of a particular set of characters. The characters must be provided by the user.

This process creates a new key table for each permutation. Each key table is then scored against a chosen metric and only key tables with the best scores are retained. This enables an exhaustive search of variations to identify those with the most favourable ergonomic or statistical properties.

Permutation regions are particularly useful when designing or refining keyboard layouts where some parts are fixed (e.g., numbers or punctuation) and others are open to optimization (e.g., letters).

To permute region 1, 2, or 3, you must provide on the command line a set of characters for that region using `--region1` (or `-1`), `--region2` (or `-2`), or `--region3` (or `-3`), respectively. The option arguments for each must be a set of characters of the same size as the number of placeholders in that region (a region can not be partially permuted). Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved.

You can choose to permute all, some, or no regions. A region is only permuted if a set of characters is provided for that region, otherwise it is left unpermuted (with its placeholders intact). This allows you to optimize regions in sequence.

There is a difference between permuting two or more regions in sequence (running Perky two or more times) vs simultaneously (running Perky once). Permuting a region of size 9 (i.e., 9 placeholders) produces 9! (read as "9 [factorial](https://en.wikipedia.org/wiki/Factorial)") (or 362,880) key tables. Permuting a region of size 5 (i.e., 5 placeholders) produces 5! (or 120) key tables. If the region of size 9 is permuted first, then the best key table is chosen, then the region of size 5 is permuted (on that key table only), the total number of permutations is 9! + 5!, or 362,800 + 120 = 363,000 key tables. If the regions of size 9 and size 5 are permuted simultaneously, however, the total number of permutations is 9! \* 5!, or 362,800 \* 120 = 43,536,000.

Simultaneous permutation can take much longer to execute, but unlike sequential permutation it guarantees that the best possible key table will be found. This is because it is possible that a high-scoring but not the best key table from the first region will combine with a permutation in the second region in such a way as to produce a better score overall.

#### Metric

When permuting, Perky scores every candidate layout according to the specified [metric](#metrics). The default metric is `sfb` (Same finger bigrams), but any metric can be specified using `--metric <METRIC>` (or `-m <METRIC>`).

For example, to score against `hsb` (Half scissor bigrams), specify `--metric hsb`.

#### Goal

When permuting, Perky scores the specified [metric](#metrics) by its direction, with the goal of finding the key tables generally considered better for that metric. To explicitly choose the goal for the metric, specify `--goal <GOAL>` (or `-g <GOAL>`), where `<GOAL>` is `max` or `min`. In this way, you can choose to find key tables generally considered *worse* for that metric.

For example, one can score against `fsb` (Full scissor bigrams) to find the worse performing key tables by specifying `--metric fsb` and `--goal max`.

#### Weight

When permuting, Perky retains the records with the best raw scores for the specified [metric](#metrics). To specify that Perky should retain the records with the best effort-weighted scores, specify `--weight effort` (or `-w effort`).

#### Progress and metadata reporting

While permutations are being scored, Perky prints a progress indicator letting you know how many permutations have been completed, how many remain, the time elapsed, and the estimated time remaining.

Perky will then output [metadata](#metadata) about its run, including the total number of permutations and the elapsed duration.

#### Examples

In our QWERTY example from the [Introduction](#introduction) section, the same finger bigram score is 6.315%. Same finger bigrams are generally considered undesirable. Can we reduce this number by making a few changes? Let's permute the nine letters of the home row ("A", "S", "D", "F", "G", "H", "J", "K", and "L") by assigning them to permutation region 1.

[example-permuting.kt.json](/examples/docs/example-permuting.kt.json):

``` json
{
  "data": [
    ["Q",  "W",  "E",  "R",  "T",  "Y",  "U",  "I",  "O",  "P",  "[",  "]",  "\\"],
    [ 1,    1,    1,    1,    1,    1,    1,    1,    1,   ";",  "'"],
    ["Z",  "X",  "C",  "V",  "B",  "N",  "M",  ",",  ".",  "/"]
  ],
  "version": 1
}
```

Then, let's ask Perky to permute this region by running it against a particular set of characters. Permutation requires that the number of characters provided exactly match the number of placeholders in the corresponding region.

We also choose the scoring metric here to be `sfb` or "same finger bigrams" and to use effort-weighted scores.

``` sh
perky \
  -l example.lt.json \
  -k example-permuting.kt.json \
  -1 "ASDFGHJKL" \
  -m sfb \
  -w effort
```

Perky will permute all 362,880, and in this case it has found a single record better than all others, one that reduces same finger bigrams from 6.315% to 4.590%.

On the test machine, the permutation took about 206ms to run at a rate of about 567ns/permutation. Ironically, it finished so quickly that it wasn't able to reach its top speed (which often can be lower than 10ns/permutation).

    [████████████████████]  100.0%  362880 / 362880  0.2s  (~ 0.0s remaining)

    unigram table sum:      3563505777820
    bigram table sum:       2819662855499
    trigram table sum:      2098121156991
    goal:                   ↓
    metric:                 sfb
    weight:                 effort
    total permutations:     362880
    elapsed duration:       205.881375ms
    efficiency:             567ns / permutation
    score:                  197912380083
    total records:          1
    total unique records:   1
    total selected records: 1

    Q W E R T Y U I O P [ ] \
    S L J D K H F G A ; '
    Z X C V B N M , . /

    Unigram summaries:
    Lt ↑: 0, 0.000%, 0, 0.000%
    Li ↑: 800019793948, 22.450%, 800019793948, 22.450%
    Lm ↑: 569968849603, 15.995%, 569968849603, 15.995%
    Lr ↓: 213080081925, 5.980%, 213080081925, 5.980%
    Lp ↓: 239581127870, 6.723%, 239581127870, 6.723%
    Lh ↑: 1822649853346, 51.148%, 1822649853346, 51.148%
    Rt ↑: 0, 0.000%, 0, 0.000%
    Ri ↑: 769592402453, 21.596%, 769592402453, 21.596%
    Rm ↑: 336346958717, 9.439%, 336346958717, 9.439%
    Rr ↓: 558803963455, 15.681%, 558803963455, 15.681%
    Rp ↓: 76112599849, 2.136%, 76112599849, 2.136%
    Rh ↑: 1740855924474, 48.852%, 1740855924474, 48.852%
    TOTALS: 3563505777820, 3563505777820

    Bigram summaries:
    Fsb ↓: 29561207813, 1.075%, 76155339763, 1.796%
    Hsb ↓: 119201219669, 4.336%, 206698488517, 4.874%
    Irb ↑: 89225122898, 3.246%, 89225122898, 2.104%
    Lsb ↓: 223417437565, 8.127%, 551594885948, 13.007%
    Orb ↑: 96612684929, 3.515%, 96612684929, 2.278%
    Sfb ↓: 125722436580, 4.573%, 197912380083, 4.667%
    TOTALS: 2748955989650, 4240671685029

    Trigram summaries:
    Alt ↓: 543584137693, 28.700%, 543584137693, 12.618%
    One ↓: 57629888036, 3.043%, 170482717331, 3.957%
    Red ↓: 99165522192, 5.236%, 531069391569, 12.328%
    Rol ↓: 877169028524, 46.313%, 2007502245228, 46.600%
    TOTALS: 1893998506314, 4307952135713

#### Parallelization

Permutation is computationally expensive: even small regions can result in billions of combinations.

To manage this efficiently, Perky's code is highly-optimized and designed for parallel execution across multiple logical cores.

On the test machine, for example, the efficiency (i.e., effective throughput) is in the sub-10ns range per permutation. For example, a permutation region of size 10 will have 10! (read as "10 [factorial](https://en.wikipedia.org/wiki/Factorial)") or 3,628,800 possible permutations. At ~10ns/permutation, Perky will require only 36 ms to score all permutations.

In its quest to brute-force all possibilities, Perky will happily consume 100% of your CPU across all logical cores. You can ask it to chill out a bit (without reducing the total number of permutations) in two ways:

- Have each thread yield by a given number of nanoseconds using the argument `-—sleep-ns <SLEEP_NS>`
- Limit the number of threads using `-—threads <THREADS>`

Modern CPUs have very sophisticated power and thermal management subsystems, so it's difficult to assess which option is better, but the former may help distribute the thermal load more evenly across your CPU's die. How much time should you yield? On the test machine, yielding for 50,000 nanoseconds (`--sleep-ns 50000`) is sufficient to drop the CPU usage to about two-thirds, which is typically sufficient to keep those annoying fans off.

##### Examples

To cause each thread to yield for 50,000ns between permutation batches, and limit execution to 8 logical cores, specify `--sleep-ns 50000 --threads 8`.

#### Practical limits

Assuming you are using a fast machine and leveraging all its logical cores, you should be able to achieve an efficiency of less than 10ns/permutation. That said, it is important to design your keyboard layout carefully, possibly in multiple steps using partial permutation; or in one step using multiple, smaller permutation regions simultaneously; as the number of permutations explodes as the size of a permutation region grows.

Here are the number of permutations and estimates of the time required (assuming an efficiency of 10ns/permutation) to permute regions of size 8 through 18 (regions of size 7 or less are effectively instantaneous and regions of size 19 or greater take far too long):

| \#  | Permutations                | Estimated time |
|:---:|-----------------------------|----------------|
|  8  | 8! (40,320)                 | 403.2 μs       |
|  9  | 9! (362,880)                | 3.6 ms         |
| 10  | 10! (3,628,800)             | 36 ms          |
| 11  | 11! (39,916,800)            | 400 ms         |
| 12  | 12! (479,001,600)           | 4.79 s         |
| 13  | 13! (6,227,020,800)         | 62.27 s        |
| 14  | 14! (87,178,291,200)        | 14.53 minutes  |
| 15  | 15! (1,307,674,368,000)     | 3.63 hours     |
| 16  | 16! (20,922,789,888,000)    | 2.42 days      |
| 17  | 17! (355,687,428,096,000)   | 41.17 days     |
| 18  | 18! (6,402,373,705,728,000) | 2.03 years     |

At this rate, to permute all 26 characters of the English alphabet in a single permutation region would require about 128 billion years (and our sun is [expected to die](https://www.space.com/14732-sun-burns-star-death.html) in only 5 billion years!).

It is clear that the practical limit for a permutation region is about 15, and for interactive analysis, about 13.

#### Truncating

The permutation process will retain up to 10,000 records *with identical scores* at any given time.

> Note: This limit does not affect the number of permutations considered, only the number retained *simultaneously* (because their scores are identical).

This limit helps prevent running out of memory or slowing down the sorting and filtering steps (explained in subsequent sections) in pathological cases where too many records have identical scores for a given metric.

The default limit should be more than sufficient for most analyses, but you can increase (or decrease) it using `--truncate <N>`.

##### Examples

To increase the truncation limit to 25,000, specify `--truncate 25000`.

#### Deduplicating

After truncation, any duplicate records are discarded. Duplicate records will occur if (and only if) Perky is given a set of possible characters for a permutation region that contains duplicates.

#### Sorting

Sorting allows you to order the deduplicated set of records to present them in a consistent and meaningful way.

Sorting is by raw or effort-weighted score, depending on the value of `--weight`.

##### Syntax

Specify `--sort-asc [<METRIC>...]` or `--sort-desc [<METRIC>...]` to sort by the value of the given metric or metrics in ascending or descending order, respectively.

You may use each option multiple times and include multiple metrics per argument. Records are sorted based on the argument order.

##### Examples

To sort the records first by the highest-scoring outward roll bigrams and then by the lowest-scoring full scissor bigrams, specify `--sort-desc orb --sort-asc fsb`

#### Filtering

Filtering allows you to narrow the sorted set of records to those that satisfy certain conditions.

##### Syntax

Specify `--filter [<EXPRESSION>...]` to narrow records based on a filtering expression.

Filtering expressions are written in a mini-language based on standard mathematical operations.

The filter expression language supports:

- Metric names: `lh`, `ri`, `orb`, `sfb`, etc.
- Numeric values: `42`, `3.14`, etc.
- Negation operator: `-`
- Arithmetic operators: `+`, `-`, `*`, `/`
- Comparison operators: `==`, `!=`, `<`, `<=`, `>`, and `>=`
- Logical operators: `&` (AND), `|` (OR), and `!` (NOT)
- Parentheses (to control order of evaluation): e.g., `(irb > 5 | orb > 5) & sfb < 2`

A metric name evaluates to its raw or effort-weighted score, depending on the value of `--weight`, expressed as a percentage.

You may specify `--filter` multiple times, in which case all must evaluate to true for a record to be retained.

###### Operators by precedence

| Precedence    | Operators            |
|---------------|----------------------|
| 1 Unary       | `-` (negation), `!`  |
| 2 Factor      | `*`, `/`             |
| 3 Term        | `+`, `-`             |
| 4 Relational  | `<`, `<=`, `>`, `>=` |
| 5 Equality    | `==`, `!=`           |
| 6 Logical AND | `&`                  |
| 7 Logical OR  | <code>\|</code>      |

##### Examples

To retain only those records where the left and right hand efforts are within 5% of even, specify `--filter "lh >= 45 & lh <= 55"`

#### Selecting

Selection allows you to extract specific records from the filtered set. By default, all records are selected.

##### Syntax

Specify `--max-records <N>` to limit the records to the first *n*. `N` must fall within the bounds of the available records.

Specify `--index <N>` to select the record at the *nth* index. `N` must fall within the bounds of the available records.

If both `--index` and `--max-records` are used, `--max-records` is applied first.

Indices are 0-based, i.e., `--index 0` selects the first record. Negative indices count from the end, i.e., `--index=-1` selects the last record.

> Note: You should specify negative option arguments using the "=" syntax, such that your shell does not confuse a negative option argument with an option

##### Examples

To show only the first record, specify `--index 0`.

To show only the first 10 records, specify `--max-records 10`.

To show only the last record of the first 10, specify `--max-records 10 --index=-1`

### Printing

After Perky loads its input files; permutes the key table (if requested); and scores, filters, sorts, and selects its records; it will print:

- Metadata (if permuting), followed by;
- Selected records (in sorted order).

#### Metadata

If permuting, Perky will print the following metadata:

- N‑gram table sums

  - Unigram table sum
  - Bigram table sum
  - Trigram table sum

- Scoring options

  - Goal
  - Metric
  - Weight

- Permutation-specific metadata

  - Total permutations
  - Elapsed duration
  - Efficiency
  - Score
  - Total records
  - Total unique records
  - Total selected records

  Efficiency is the elapsed duration divided by the total permutations.

To force printing the metadata (even when not permuting), specify `--print-metadata true`. To suppress printing the metadata (even when permuting), specify `--print-metadata false`.

#### Selected records

For each selected record, Perky will print the following:

- A key table
- Any detail reports requested via `--print-details [<METRIC>...]`, printed in order by [metric](#metrics), followed by;
- A summary report, unless suppressed using `--print-summaries false`

By default, all scores include both raw and percentage representations (equivalent to `--print-perc true`). To suppress the percentage representations always, specify `--print-perc false`.

#### Format

By default, Perky will output text, which is easy to read. This is equivalent to specifying `--format text`.

Perky can also output [JSON Lines](https://jsonlines.org), which is easy to analyze programmatically. For JSON Lines output, specify `--format json`.

For the text format, output can be styled using colours and text effects to improve readability by specifying `--style <STYLE>`. By default, output will be styled when printed to a terminal but not when piped or redirected (equivalent to `--style auto`). To always style text (including when the output is piped or redirected), specify `--style always`. To never style text, specify `--style never`.

When styles are enabled, key tables will be printed in colour, representing the relative unigram frequency for that key. Bright red indicates the highest frequency and darker, desaturated red represents the frequency.

For JSON Lines format, `--style <STYLE>` is ignored.

With the exception of the colouring of the key tables in text format, both formats output the same information.

#### Examples

Using the example from the [Permuting](#permuting) section, but specifying `--print-metadata false`:

``` sh
perky \
  -l example.lt.json \
  -k example-permuting.kt.json \
  -1 "ASDFGHJKL" \
  -m sfb \
  -w effort \
  --print-metadata false
```

    [████████████████████]  100.0%  362880 / 362880  0.2s  (~ 0.0s remaining)

    Q W E R T Y U I O P [ ] \
    S L J D K H F G A ; '
    Z X C V B N M , . /

    Unigram summaries:
    Lt ↑: 0, 0.000%, 0, 0.000%
    Li ↑: 800019793948, 22.450%, 800019793948, 22.450%
    Lm ↑: 569968849603, 15.995%, 569968849603, 15.995%
    Lr ↓: 213080081925, 5.980%, 213080081925, 5.980%
    Lp ↓: 239581127870, 6.723%, 239581127870, 6.723%
    Lh ↑: 1822649853346, 51.148%, 1822649853346, 51.148%
    Rt ↑: 0, 0.000%, 0, 0.000%
    Ri ↑: 769592402453, 21.596%, 769592402453, 21.596%
    Rm ↑: 336346958717, 9.439%, 336346958717, 9.439%
    Rr ↓: 558803963455, 15.681%, 558803963455, 15.681%
    Rp ↓: 76112599849, 2.136%, 76112599849, 2.136%
    Rh ↑: 1740855924474, 48.852%, 1740855924474, 48.852%
    TOTALS: 3563505777820, 3563505777820

    Bigram summaries:
    Fsb ↓: 29561207813, 1.075%, 76155339763, 1.796%
    Hsb ↓: 119201219669, 4.336%, 206698488517, 4.874%
    Irb ↑: 89225122898, 3.246%, 89225122898, 2.104%
    Lsb ↓: 223417437565, 8.127%, 551594885948, 13.007%
    Orb ↑: 96612684929, 3.515%, 96612684929, 2.278%
    Sfb ↓: 125722436580, 4.573%, 197912380083, 4.667%
    TOTALS: 2748955989650, 4240671685029

    Trigram summaries:
    Alt ↓: 543584137693, 28.700%, 543584137693, 12.618%
    One ↓: 57629888036, 3.043%, 170482717331, 3.957%
    Red ↓: 99165522192, 5.236%, 531069391569, 12.328%
    Rol ↓: 877169028524, 46.313%, 2007502245228, 46.600%
    TOTALS: 1893998506314, 4307952135713

Using the example from the [Introduction](#introduction) section but specifying `--print-perc false`:

``` sh
perky \
  -l example.lt.json \
  -k example-introduction.kt.json \
  --print-perc false
```

    Q W E R T Y U I O P [ ] \
    A S D F G H J K L ; '
    Z X C V B N M , . /

    Unigram summaries:
    Lt ↑: 0, 0
    Li ↑: 796991792591, 796991792591
    Lm ↑: 700328467558, 700328467558
    Lr ↓: 300164341769, 300164341769
    Lp ↓: 294025744233, 294025744233
    Lh ↑: 2091510346151, 2091510346151
    Rt ↑: 0, 0
    Ri ↑: 689614872654, 689614872654
    Rm ↑: 288992871918, 288992871918
    Rr ↓: 417275087248, 417275087248
    Rp ↓: 76112599849, 76112599849
    Rh ↑: 1471995431669, 1471995431669
    TOTALS: 3563505777820, 3563505777820

    Bigram summaries:
    Fsb ↓: 29561207813, 76155339763
    Hsb ↓: 223473410253, 449964114055
    Irb ↑: 107185097537, 107185097537
    Lsb ↓: 195655811929, 490785251171
    Orb ↑: 98315980469, 98315980469
    Sfb ↓: 195686888871, 270316925501
    TOTALS: 2748955989650, 4209833679358

    Trigram summaries:
    Alt ↓: 547770117520, 547770117520
    One ↓: 58813867491, 164484663333
    Red ↓: 138042336404, 798736382372
    Rol ↓: 783296865779, 1866401805671
    TOTALS: 1893998506314, 4493113390239

Using the example in the [Introduction](#introduction) section but with `--format json` and `--print-metadata true`:

``` sh
perky \
  -l example.lt.json \
  -k example-introduction.kt.json \
  --format json \
  --print-metadata true
```

``` json
{
  "unigram_table_sum": 3563505777820,
  "bigram_table_sum": 2819662855499,
  "trigram_table_sum": 2098121156991,
  "goal": "↓",
  "metric": "sfb",
  "weight": "raw",
  "total_permutations": 1,
  "elapsed_duration": "280.583µs",
  "efficiency": "280.583µs",
  "score": 195686888871,
  "total_records": 1,
  "total_unique_records": 1,
  "total_selected_records": 1
}
{
  "index": 1,
  "key_table": [
    ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]", "\\"],
    ["A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'"],
    ["Z", "X", "C", "V", "B", "N", "M", ",", ".", "/"]
  ],
  "measurements": {
    "unigram": {
      "details": null,
      "summaries": {
        "Lh": [
          [2091510346151, 58.69249207252574],
          [2091510346151, 58.69249207252574]
        ],
        "Li": [
          [796991792591, 22.365385165127062],
          [796991792591, 22.365385165127062]
        ],
        "Lm": [
          [700328467558, 19.652794501330398],
          [700328467558, 19.652794501330398]
        ],
        "Lp": [
          [294025744233, 8.251024765080423],
          [294025744233, 8.251024765080423]
        ],
        "Lr": [
          [300164341769, 8.42328764098785],
          [300164341769, 8.42328764098785]
        ],
        "Lt": [
          [0, 0.0],
          [0, 0.0]
        ],
        "Rh": [
          [1471995431669, 41.30750792747427],
          [1471995431669, 41.30750792747427]
        ],
        "Ri": [
          [689614872654, 19.35214689271184],
          [689614872654, 19.35214689271184]
        ],
        "Rm": [
          [288992871918, 8.10979102985469],
          [288992871918, 8.10979102985469]
        ],
        "Rp": [
          [76112599849, 2.1358910184105953],
          [76112599849, 2.1358910184105953]
        ],
        "Rr": [
          [417275087248, 11.709678986497138],
          [417275087248, 11.709678986497138]
        ],
        "Rt": [
          [0, 0.0],
          [0, 0.0]
        ],
        "TOTALS": [3563505777820, 3563505777820]
      }
    },
    "bigram": {
      "details": null,
      "summaries": {
        "Fsb": [
          [29561207813, 1.0753612616680621],
          [76155339763, 1.8089868998010794]
        ],
        "Hsb": [
          [223473410253, 8.129392070822234],
          [449964114055, 10.688405963905433]
        ],
        "Irb": [
          [107185097537, 3.8991201729150613],
          [107185097537, 2.5460648971136015]
        ],
        "Lsb": [
          [195655811929, 7.117458870409603],
          [490785251171, 11.658067480847482]
        ],
        "Orb": [
          [98315980469, 3.57648433947892],
          [98315980469, 2.335388710273067]
        ],
        "Sfb": [
          [195686888871, 7.118589370210874],
          [270316925501, 6.421083256244541]
        ],
        "TOTALS": [2748955989650, 4209833679358]
      }
    },
    "trigram": {
      "details": null,
      "summaries": {
        "Alt": [
          [547770117520, 28.921359530849966],
          [547770117520, 12.191326368704502]
        ],
        "One": [
          [58813867491, 3.105275283741403],
          [164484663333, 3.6608171004616166]
        ],
        "Red": [
          [138042336404, 7.288407881200007],
          [798736382372, 17.77690240596205]
        ],
        "Rol": [
          [783296865779, 41.35678371275018],
          [1866401805671, 41.53916546432231]
        ],
        "TOTALS": [1893998506314, 4493113390239]
      }
    }
  }
}
```

<!--
## Contributing
&#10;Pull requests are welcome. Please open an issue first to discuss what you would like to change.
-->

## Author and copyright

This project was written and copyrighted in 2025 by Chris McLaren ([@csmclaren](https://www.github.com/csmclaren)).

## License

Unless otherwise noted, all files in this project are licensed under the [MIT License](https://choosealicense.com/licenses/mit/). See the [LICENSE](/LICENSE.txt) file for details.

This document was produced on August 22, 2025.
