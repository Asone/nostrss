# nostrss (Core)

## Run 

To run the program you will have to provide two arguments : 

> nostrss --relays <path/to/relays> --feeds <path/to/feeds> --profiles <path/to/profiles> --update <boolean>

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
| profile   | Array of strings | No       | The profiles to be used for this rss feed                   |
| tags   | Array of strings        | No       | A list of tags to be used for messages                   |
| template | String | No | An optional path to a template to use for feed publishing. |
| cache_size | Integer | No | The snapshot size made in job. If no value is provided and no default value is set through env, cache will have no limit. |

##### Examples : 
- [json file example](./src/fixtures/rss.json)       
- [yaml file example](./src/fixtures/rss.yaml)
### Relays

| Key       | Type          | Required | Description
|-----------|---------------|----------|------------------------------------------------------------|
| name      | String        | Yes      | The relay name                                             |
| target    | String        | Yes      | The url to the relay, must be a websocket service          |
| active    | Boolean        | Yes      | Not used yet, will be used to skip using relays            |
| proxy     | Cron pattern  | No       | An optional proxy to connect through                       |

##### Examples : 
- [json file example](./src/fixtures/relays.json)       
- [yaml file example](./src/fixtures/relays.yaml)

### Profiles
#### Default

You must configure your default Nostr identity through the environment variables. 

You can use a .env file for that. Refer to  the [.env.dist](./.env.dist) as example.

If no private key is provided, a random one will be generated. 

If you have no private key already, you can go on [astral.ninja](https://astral.ninja/) to generate one. 

#### Profile Values

| Key           | Type          | Required | Description                                                |
|---------------|---------------|----------|------------------------------------------------------------|
| id            | String        | Yes      |                                             |
| private_key   | String        | Yes      |           |
| about         | String        | No       | 
| name          | String        | No       | The handle name                                            |
| display_name  | String        | No       | The name to be displayed                                   |
| description   | String        | No       |                                                            |
| picture       | String        | No       | A valid URL to an image for picture                        |
| banner        | String        | No       | A valid URL to an image for banner                         |
| nip05 | String| No | Identity certificatioon
| lud16         | String        |No       | LN Wallet |
| pow_level         | String        |No       | The pow difficulty to use for publishing under the current profile |
| recommended_relays         | Array of relays ids        |No       | The relays that should be recommended to clients for the published notes |

##### Examples : 
- [json file example](./src/fixtures/profiles.json)       
- [yaml file example](./src/fixtures/profiles.yaml)

### Templating

Nostrss allows you to customize the message sent for each feed. Custom templates are optional. 
See [RSS Feeds](#rss-feeds) section to see how to provide custom templates for feeds. 

If no custom template path is provided, Nostrss will automatically fallback on the default template provided in [.env.dist](./.env.dist) config file.

If no default template is either provided, the feed threaded-job will panic, but the application will keep running.
This avoids a global panic and keeps all sane jobs running.

If provided path for custom template is non-existant, the job will raise an error and publishing will be skipped.

Below are the variables you can use for templating : 

| Variable     | Description                       |
| ------------ |---------------------------------- |
| name         | The `feed` given name             |
| title        | The `entry` title                 |
| content      | The `entry` content. Usually a description of the item |
| url          | The URL to the `entry`            |
| tags         | The tags of the `feed`            |

An example template is provided in the [fixtures](./src/fixtures/default.template)

## RSS broadcasting 

> [!WARNING]
> If you're using nostrss release up to v1.0.3, use [cron crate rules](https://crates.io/crates/cron) pattern instead. For any upper version you can rely on the information below. 

Cronjob rules are defined in the [feeds config file](./nostrss-core/src/fixtures/rss.json) following the [croner-rust crate rules](https://docs.rs/croner/latest/croner/#pattern).

For each tick the remote feed will be matched with a local fingerprint, for which, any unmatching entry against of the feed will be broadcasted to relays. 

### Dry run mode

You can run the program in a `dry-run` mode, so the program will run the whole processes as usual but will avoid broadcasting the final result onto the network. 

When activating the `dry-run` mode, the programm will log the json that would have been broadcasted into the `STDOUT`. 

To run the `dry-run` mode use the `--dry-run` flag when instanciating `nostrss` : 

> nostrss --relays <path/to/relays> --feeds <path/to/feeds> --profiles <path/to/profiles> --update <boolean> --dry-run