# Nostrss
Nostrss is a CLI program that provides a bridge beetween RSS feeds and [Nostr protocol](https://nostr.com/).


## Disclaimer 

This project is in early stage and provides very limited features at the moment. 

The program in its current state does not support channel publishing yet nor tags.

If you get any trouble running it, feel free to open an issue.

## Download

You can download the application through the [releases page](https://github.com/Asone/nostrss/releases).

Note that there is no official release for official Mac OS nor Windows due to some specific configs required. Yet you can
download the sources and compile it on your own if necessary.

## Run 

To run the program you will have to provide two arguments : 

> nostrss --relays <path/to/relays> --feeds <path/to/feeds> --profiles <path/to/profiles>

Both provided files can be either `yaml` or `json` files. 
You will find examples of the files structure in the [fixtures](./src/fixtures/) folder.

## Configuration objects
### RSS feeds

| Key       | Type          | Required | Description
|-----------|---------------|----------|------------------------------------------------------------|
| id        | String        | Yes      | The identifier of the feed                                 |
| name      | String        | Yes      | The name of the feed to be displayed in the nostr message  |
| url       | String        | Yes      | The URL of the feed                                        |
| schedule  | Cron pattern  | Yes      | The Cronjob rule                                           |
| profile   | String        | No       | The profile to be used for this rss feed                   |

##### Examples : 
- [json file example](./src/fixtures/rss.json)       
- [yaml file example](./src/fixtures/rss.yaml)
### Relays

| Key       | Type          | Required | Description
|-----------|---------------|----------|------------------------------------------------------------|
| name      | String        | Yes      | The relay name                                             |
| target    | String        | Yes      | The url to the relay, must be a websocket service          |
| active    | String        | Yes      | Not used yet, will be used to skip using relays            |
| proxy     | Cron pattern  | No       | An optional proxy to connect through                       |

##### Examples : 
- [json file example](./src/fixtures/relays.json)       
- [yaml file example](./src/fixtures/relays.yaml)

### Profiles

| Key           | Type          | Required | Description                                                |
|---------------|---------------|----------|------------------------------------------------------------|
| id            | String        | Yes      | The relay name                                             |
| private_key   | String        | Yes      | The url to the relay, must be a websocket service          |
| about         | String        | No       | Not used yet, will be used to skip using relays            |
| name          | Cron pattern  | No       | The handle name                                            |
| display_name  | Cron pattern  | No       | The name to be displayed                                   |
| description   | Cron pattern  | No       |                                                            |
| picture       | Cron pattern  | No       | A valid URL to an image for picture                        |
| banner        | Cron pattern  | No       | A valid URL to an image for banner                         |
| lud16         | Cron pattern  | No       |                                                            |

##### Examples : 
- [json file example](./src/fixtures/profiles.json)       
- [yaml file example](./src/fixtures/profiles.yaml)

## Nostr identity

You must configure your Nostr identity through the environment variables. 

You can use a .env file for that. Refer to  the [.env.dist](./.env.dist) as example.

If no private key is provided, a random one will be generated. 

If you have no private key already, you can go on [astral.ninja](https://astral.ninja/) to generate one. 

## RSS broadcasting 

Cronjob rules are defined in the [feeds config file](./src/fixtures/rss.json) following the [cron crate rules](https://crates.io/crates/cron).

For each tick the remote feed will be matched with a local fingerprint, for which, any unmatching entry against of the feed will be broadcasted to relays. 


## Build from sources

````
git clone 
cd nostrss
cargo build
cargo run
````

##  Licence

MIT Licence.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

The Software is provided “as is”, without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose and noninfringement. In no event shall the authors or copyright holders be liable for any claim, damages or other liability, whether in an action of contract, tort or otherwise, arising from, out of or in connection with the software or the use or other dealings in the Software.
