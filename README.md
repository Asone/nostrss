# Nostrss
Nostrss is a program that provides a bridge beetween rss feeds and [Nostr protocol](https://nostr.com/).


## Disclaimer 

This project is in early stage and provides very limited features at the moment. 

The program in its current state does not support mutliple identities or channel publishing yet.

If you get any trouble running it, feel free to open an issue.

## Run 

To run the program you will have to provide two arguments : 

> nostrss --relays <path/to/relays> --feeds <path/to/feeds>

Both provided files can be either `yaml` or `json` files. 
You will find examples of the files structure in the [fixtures](./src/fixtures/) folder.

## Nostr identity

You must configure your Nostr identity through the environment variables. 

You can use a .env file for that. Refer to  the [.env.dist](./.env.dist) as example.

If no private key is provided, a random one will be generated. 

If you have no private key already, you can go on [astral.ninja](https://astral.ninja/) to generate one. 

## RSS broadcasting 

Cronjob rules are defined in the [feeds config file](./src/fixtures/rss.json) following the [cron crate](https://crates.io/crates/cron).

For each tick the remote feed will be matched with a local fingerprint, for which, any unmatching entry against of the feed will be broadcasted to
relays. 



## Build from sources

````
git clone 
cd nostrss
cargo build
````

##  Licence

MIT Licence.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

The Software is provided “as is”, without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose and noninfringement. In no event shall the authors or copyright holders be liable for any claim, damages or other liability, whether in an action of contract, tort or otherwise, arising from, out of or in connection with the software or the use or other dealings in the Software.
