# Hierarchical Deterministic index finder

This CLI utility provides a simple way to check if a provided address belongs to a Bitcoin HD Wallet.
It takes a mnemonic passphrase which will be used to find a match with the provided address.

Addresses supported formats are for now : p2pkh, p2wpkh & p2shwpkh.

As this tool uses parallel thread to maximize processing time, you can provide custom parameters for the parallel execution. 

````
USAGE:
    hdifinder [OPTIONS] [ARGS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --chunksize <chunksize>      The chuncksize for index search threads
    -e, --end <end>                  The end index for key index search
    -p, --passphrase <passphrase>    The mnemonic passphrase
    -s, --start <start>              The start index for key index search

ARGS:
    <mnemonic>    A 24 words seed (without passphrase)
    <address>     The address to be found
````
