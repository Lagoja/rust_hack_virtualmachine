# Notes on implementation

1. ( Tokenizer ) For each line, break it into a list of tokens that the parser can easily read -- DONE
2. ( Parser ) Take the list of tokens, and analyze the types to remove comments + newline, and determine the commandtype and arguments -- DONE
3. ( Writer ) Pass the parsed commands to the writer and translate them into Hack Assembly -- DONE
4. ( Frontend ) Open and read/write files -- DONE

# Missing Features:
- [X] Comparison functions
    - [X] not
    - [X] greater_than
    - [X] less_than
    - [X] equal
- [ ] Pointer references