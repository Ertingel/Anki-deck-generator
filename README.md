# Anki deck generator

## Info

Personal code to generate a anki deck for learning the japanese language.

The code is intended for personal use and is not for others.

## Dictionaries

The code uses the following dictionaries in the following directories.

-   In `input/dictionaries`
    -   [Jmdict](https://github.com/yomidevs/jmdict-yomitan) for words and glossary data.
    -   [KANJIDIC](https://github.com/yomidevs/jmdict-yomitan) for kanji.
    -   [Yomitan-jlpt-vocab](https://github.com/stephenmk/yomitan-jlpt-vocab) for jlpt level.
-   In `input/examples`
    -   [Jitendex](https://github.com/stephenmk/Jitendex?tab=readme-ov-file) for example sentences.

## Audio

The code uses the following audio sources.

-   [AnkiConnect](https://github.com/amikey/anki-connect) built in api.

## Sentences

The code uses the following sentences api.

-   [Tatoeba](https://tatoeba.org/en) for additional example sentences.

## Run order

The intended run order.

1. `dictionary.rs`
2. `add.rs`
3. `order.rs`
4. `audio.rs`
5. `example.rs`
