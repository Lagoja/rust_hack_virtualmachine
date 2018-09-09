# Notes on implementation

1. ( Tokenizer ) For each line, break it into a list of tokens that the parser can easily read
2. ( Parser ) Take the list of tokens, and analyze the types to remove comments + newline, and determine the commandtype and arguments
3. ( Writer ) Pass the parsed commands to the writer and translate them into Hack Assembly
