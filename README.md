# Notes on implementation

1. ( Tokenizer ) For each line, break it into a list of tokens that the parser can easily read -- DONE
2. ( Parser ) Take the list of tokens, and analyze the types to remove comments + newline, and determine the commandtype and arguments -- DONE
3. ( Writer ) Pass the parsed commands to the writer and translate them into Hack Assembly -- DONE
4. ( Frontend ) Open and read/write files -- DONE

# Missing Features:
## Phase One
- [X] Comparison functions
    - [X] not
    - [X] greater_than
    - [X] less_than
    - [X] equal
- [X] Pointer references
## Phase Two
- [X] Multi-file support (pass a directory and process all .vm files inside)
- [X] Program Control Flow 
    - [X] label
    - [X] goto-label
    - [X] if-goto label
- [X] Function calls
    - [X] function declaration
    - [X] call function
    - [X] return function
- [X] Bootstrapping
- [ ] Static vars
## Phase Three
- [X] Better Error Handling (w/ line number)
- [ ] DRY up writer and other sections